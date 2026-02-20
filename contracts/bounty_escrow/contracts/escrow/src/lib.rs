#![no_std]
mod events;
mod test_bounty_escrow;

use events::{
    emit_bounty_initialized, emit_funds_locked, emit_funds_refunded, emit_funds_released,
    BountyEscrowInitialized, FundsLocked, FundsRefunded, FundsReleased,
};
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    BountyExists = 3,
    BountyNotFound = 4,
    FundsNotLocked = 5,
    DeadlineNotPassed = 6,
    Unauthorized = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Locked,
    Released,
    Refunded,
}
pub enum RefundMode {
    Full,
    Partial,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundRecord {
    pub amount: i128,
    pub recipient: Address,
    pub mode: RefundMode,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundApproval {
    pub bounty_id: u64,
    pub amount: i128,
    pub recipient: Address,
    pub mode: RefundMode,
    pub approved_by: Address,
    pub approved_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimRecord {
    pub bounty_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub expires_at: u64,
    pub claimed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimStatus {
    Pending,
    Claimed,
    Cancelled,
    Expired,
}

/// Complete escrow record for a bounty.
///
/// # Fields
/// * `depositor` - Address that locked the funds (receives refunds)
/// * `amount` - Token amount held in escrow (in smallest denomination)
/// * `status` - Current state of the escrow (Locked/Released/Refunded)
/// * `deadline` - Unix timestamp after which refunds are allowed
///
/// # Storage
/// Stored in persistent storage with key `DataKey::Escrow(bounty_id)`.
/// TTL is automatically extended on access.
///
/// # Example
/// ```rust
/// let escrow = Escrow {
///     depositor: depositor_address,
///     amount: 1000_0000000, // 1000 tokens
///     status: EscrowStatus::Locked,
///     deadline: current_time + 2592000, // 30 days
/// };
/// ```
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub depositor: Address,
    pub amount: i128,
    pub status: EscrowStatus,
    pub deadline: u64,
    pub refund_history: Vec<RefundRecord>,
    pub remaining_amount: i128,
}

/// Storage keys for contract data.
///
/// # Keys
/// * `Admin` - Stores the admin address (instance storage)
/// * `Token` - Stores the token contract address (instance storage)
/// * `Escrow(u64)` - Stores escrow data indexed by bounty_id (persistent storage)
///
/// # Storage Types
/// - **Instance Storage**: Admin and Token (never expires, tied to contract)
/// - **Persistent Storage**: Individual escrow records (extended TTL on access)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockFundsItem {
    pub bounty_id: u64,
    pub depositor: Address,
    pub amount: i128,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseFundsItem {
    pub bounty_id: u64,
    pub contributor: Address,
}

// Maximum batch size to prevent gas limit issues
const MAX_BATCH_SIZE: u32 = 100;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    pub lock_fee_rate: i128, // Fee rate for lock operations (basis points, e.g., 100 = 1%)
    pub release_fee_rate: i128, // Fee rate for release operations (basis points)
    pub fee_recipient: Address, // Address to receive fees
    pub fee_enabled: bool,   // Global fee enable/disable flag
}

// Fee rate is stored in basis points (1 basis point = 0.01%)
// Example: 100 basis points = 1%, 1000 basis points = 10%
const BASIS_POINTS: i128 = 10_000;
const MAX_FEE_RATE: i128 = 1_000; // Maximum 10% fee

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultisigConfig {
    pub threshold_amount: i128,
    pub signers: Vec<Address>,
    pub required_signatures: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseApproval {
    pub bounty_id: u64,
    pub contributor: Address,
    pub approvals: Vec<Address>,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Token,
    Escrow(u64),         // bounty_id
    FeeConfig,           // Fee configuration
    RefundApproval(u64), // bounty_id -> RefundApproval
    ReentrancyGuard,
    MultisigConfig,
    ReleaseApproval(u64), // bounty_id -> ReleaseApproval
    PendingClaim(u64),    // bounty_id -> ClaimRecord
    ClaimWindow,          // u64 seconds (global config)
}

// ============================================================================
// Contract Implementation
// ============================================================================

#[contract]
pub struct BountyEscrowContract;

#[contractimpl]
impl BountyEscrowContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initializes the Bounty Escrow contract with admin and token addresses.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - Address authorized to release funds
    /// * `token` - Token contract address for escrow payments (e.g., XLM, USDC)
    ///
    /// # Returns
    /// * `Ok(())` - Contract successfully initialized
    /// * `Err(Error::AlreadyInitialized)` - Contract already initialized
    ///
    /// # State Changes
    /// - Sets Admin address in instance storage
    /// - Sets Token address in instance storage
    /// - Emits BountyEscrowInitialized event
    ///
    /// # Security Considerations
    /// - Can only be called once (prevents admin takeover)
    /// - Admin should be a secure backend service address
    /// - Token must be a valid Stellar Asset Contract
    /// - No authorization required (first-caller initialization)
    ///
    /// # Events
    /// Emits: `BountyEscrowInitialized { admin, token, timestamp }`
    ///
    /// # Example
    /// ```rust
    /// let admin = Address::from_string("GADMIN...");
    /// let usdc_token = Address::from_string("CUSDC...");
    /// escrow_client.init(&admin, &usdc_token)?;
    /// ```
    ///
    /// # Gas Cost
    /// Low - Only two storage writes
    pub fn init(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        // Apply rate limiting
        anti_abuse::check_rate_limit(&env, admin.clone());

        let start = env.ledger().timestamp();
        let caller = admin.clone();

        // Prevent re-initialization
        if env.storage().instance().has(&DataKey::Admin) {
            monitoring::track_operation(&env, symbol_short!("init"), caller, false);
            return Err(Error::AlreadyInitialized);
        }

        // Store configuration
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);

        // Initialize fee config with zero fees (disabled by default)
        let fee_config = FeeConfig {
            lock_fee_rate: 0,
            release_fee_rate: 0,
            fee_recipient: admin.clone(),
            fee_enabled: false,
        };
        env.storage()
            .instance()
            .set(&DataKey::FeeConfig, &fee_config);

        // Initialize multisig config (disabled by default)
        let multisig_config = MultisigConfig {
            threshold_amount: i128::MAX,
            signers: vec![&env],
            required_signatures: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::MultisigConfig, &multisig_config);

        // Emit initialization event
        emit_bounty_initialized(
            &env,
            BountyEscrowInitialized {
                admin: admin.clone(),
                token,
                timestamp: env.ledger().timestamp(),
            },
        );

        // Track successful operation
        monitoring::track_operation(&env, symbol_short!("init"), caller, true);

        // Track performance
        let duration = env.ledger().timestamp().saturating_sub(start);
        monitoring::emit_performance(&env, symbol_short!("init"), duration);

        Ok(())
    }

    /// Calculate fee amount based on rate (in basis points)
    fn calculate_fee(amount: i128, fee_rate: i128) -> i128 {
        if fee_rate == 0 {
            return 0;
        }
        // Fee = (amount * fee_rate) / BASIS_POINTS
        // Using checked arithmetic to prevent overflow
        amount
            .checked_mul(fee_rate)
            .and_then(|x| x.checked_div(BASIS_POINTS))
            .unwrap_or(0)
    }

    /// Get fee configuration (internal helper)
    fn get_fee_config_internal(env: &Env) -> FeeConfig {
        env.storage()
            .instance()
            .get(&DataKey::FeeConfig)
            .unwrap_or_else(|| FeeConfig {
                lock_fee_rate: 0,
                release_fee_rate: 0,
                fee_recipient: env.storage().instance().get(&DataKey::Admin).unwrap(),
                fee_enabled: false,
            })
    }

    /// Update fee configuration (admin only)
    pub fn update_fee_config(
        env: Env,
        lock_fee_rate: Option<i128>,
        release_fee_rate: Option<i128>,
        fee_recipient: Option<Address>,
        fee_enabled: Option<bool>,
    ) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut fee_config = Self::get_fee_config_internal(&env);

        if let Some(rate) = lock_fee_rate {
            if !(0..=MAX_FEE_RATE).contains(&rate) {
                return Err(Error::InvalidFeeRate);
            }
            fee_config.lock_fee_rate = rate;
        }

        if let Some(rate) = release_fee_rate {
            if !(0..=MAX_FEE_RATE).contains(&rate) {
                return Err(Error::InvalidFeeRate);
            }
            fee_config.release_fee_rate = rate;
        }

        if let Some(recipient) = fee_recipient {
            fee_config.fee_recipient = recipient;
        }

        if let Some(enabled) = fee_enabled {
            fee_config.fee_enabled = enabled;
        }

        env.storage()
            .instance()
            .set(&DataKey::FeeConfig, &fee_config);

        events::emit_fee_config_updated(
            &env,
            events::FeeConfigUpdated {
                lock_fee_rate: fee_config.lock_fee_rate,
                release_fee_rate: fee_config.release_fee_rate,
                fee_recipient: fee_config.fee_recipient.clone(),
                fee_enabled: fee_config.fee_enabled,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get current fee configuration (view function)
    pub fn get_fee_config(env: Env) -> FeeConfig {
        Self::get_fee_config_internal(&env)
    }

    /// Update multisig configuration (admin only)
    pub fn update_multisig_config(
        env: Env,
        threshold_amount: i128,
        signers: Vec<Address>,
        required_signatures: u32,
    ) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if required_signatures > signers.len() {
            return Err(Error::InvalidAmount);
        }

        let config = MultisigConfig {
            threshold_amount,
            signers,
            required_signatures,
        };

        env.storage()
            .instance()
            .set(&DataKey::MultisigConfig, &config);

        Ok(())
    }

    /// Get multisig configuration
    pub fn get_multisig_config(env: Env) -> MultisigConfig {
        env.storage()
            .instance()
            .get(&DataKey::MultisigConfig)
            .unwrap_or(MultisigConfig {
                threshold_amount: i128::MAX,
                signers: vec![&env],
                required_signatures: 0,
            })
    }

    /// Approve release for large amount (requires multisig)
    pub fn approve_large_release(
        env: Env,
        bounty_id: u64,
        contributor: Address,
        approver: Address,
    ) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }

        let multisig_config: MultisigConfig = Self::get_multisig_config(env.clone());

        let mut is_signer = false;
        for signer in multisig_config.signers.iter() {
            if signer == approver {
                is_signer = true;
                break;
            }
        }

        if !is_signer {
            return Err(Error::Unauthorized);
        }

        approver.require_auth();

        let approval_key = DataKey::ReleaseApproval(bounty_id);
        let mut approval: ReleaseApproval = env
            .storage()
            .persistent()
            .get(&approval_key)
            .unwrap_or(ReleaseApproval {
                bounty_id,
                contributor: contributor.clone(),
                approvals: vec![&env],
            });

        for existing in approval.approvals.iter() {
            if existing == approver {
                return Ok(());
            }
        }

        approval.approvals.push_back(approver.clone());
        env.storage().persistent().set(&approval_key, &approval);

        events::emit_approval_added(
            &env,
            events::ApprovalAdded {
                bounty_id,
                contributor: contributor.clone(),
                approver,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    // ========================================================================
    // Core Escrow Functions
    // ========================================================================

    /// Locks funds in escrow for a specific bounty.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `depositor` - Address depositing the funds (must authorize)
    /// * `bounty_id` - Unique identifier for this bounty
    /// * `amount` - Token amount to lock (in smallest denomination)
    /// * `deadline` - Unix timestamp after which refund is allowed
    ///
    /// # Returns
    /// * `Ok(())` - Funds successfully locked
    /// * `Err(Error::NotInitialized)` - Contract not initialized
    /// * `Err(Error::BountyExists)` - Bounty ID already in use
    ///
    /// # State Changes
    /// - Transfers `amount` tokens from depositor to contract
    /// - Creates Escrow record in persistent storage
    /// - Emits FundsLocked event
    ///
    /// # Authorization
    /// - Depositor must authorize the transaction
    /// - Depositor must have sufficient token balance
    /// - Depositor must have approved contract for token transfer
    ///
    /// # Security Considerations
    /// - Bounty ID must be unique (prevents overwrites)
    /// - Amount must be positive (enforced by token contract)
    /// - Deadline should be reasonable (recommended: 7-90 days)
    /// - Token transfer is atomic with state update
    ///
    /// # Events
    /// Emits: `FundsLocked { bounty_id, amount, depositor, deadline }`
    ///
    /// # Example
    /// ```rust
    /// let depositor = Address::from_string("GDEPOSIT...");
    /// let amount = 1000_0000000; // 1000 USDC
    /// let deadline = env.ledger().timestamp() + (30 * 24 * 60 * 60); // 30 days
    ///
    /// escrow_client.lock_funds(&depositor, &42, &amount, &deadline)?;
    /// // Funds are now locked and can be released or refunded
    /// ```
    ///
    /// # Gas Cost
    /// Medium - Token transfer + storage write + event emission
    ///
    /// # Common Pitfalls
    /// - Forgetting to approve token contract before calling
    /// - Using a bounty ID that already exists
    /// - Setting deadline in the past or too far in the future
    pub fn lock_funds(
        env: Env,
        depositor: Address,
        bounty_id: u64,
        amount: i128,
        deadline: u64,
    ) -> Result<(), Error> {
        // Apply rate limiting
        anti_abuse::check_rate_limit(&env, depositor.clone());

        let start = env.ledger().timestamp();
        let caller = depositor.clone();

        // Verify depositor authorization
        depositor.require_auth();

        // Ensure contract is initialized
        if env.storage().instance().has(&DataKey::ReentrancyGuard) {
            panic!("Reentrancy detected");
        }
        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &true);

        if amount <= 0 {
            monitoring::track_operation(&env, symbol_short!("lock"), caller, false);
            env.storage().instance().remove(&DataKey::ReentrancyGuard);
            return Err(Error::InvalidAmount);
        }

        if deadline <= env.ledger().timestamp() {
            monitoring::track_operation(&env, symbol_short!("lock"), caller, false);
            env.storage().instance().remove(&DataKey::ReentrancyGuard);
            return Err(Error::InvalidDeadline);
        }
        if !env.storage().instance().has(&DataKey::Admin) {
            monitoring::track_operation(&env, symbol_short!("lock"), caller, false);
            env.storage().instance().remove(&DataKey::ReentrancyGuard);
            return Err(Error::NotInitialized);
        }

        // Prevent duplicate bounty IDs
        if env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            monitoring::track_operation(&env, symbol_short!("lock"), caller, false);
            env.storage().instance().remove(&DataKey::ReentrancyGuard);
            return Err(Error::BountyExists);
        }

        // Get token contract and transfer funds
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);

        // Calculate and collect fee if enabled
        let fee_config = Self::get_fee_config_internal(&env);
        let fee_amount = if fee_config.fee_enabled && fee_config.lock_fee_rate > 0 {
            Self::calculate_fee(amount, fee_config.lock_fee_rate)
        } else {
            0
        };
        let net_amount = amount - fee_amount;

        // Transfer net amount from depositor to contract
        client.transfer(&depositor, &env.current_contract_address(), &net_amount);

        // Transfer fee to fee recipient if applicable
        if fee_amount > 0 {
            client.transfer(&depositor, &fee_config.fee_recipient, &fee_amount);
            events::emit_fee_collected(
                &env,
                events::FeeCollected {
                    operation_type: events::FeeOperationType::Lock,
                    amount: fee_amount,
                    fee_rate: fee_config.lock_fee_rate,
                    recipient: fee_config.fee_recipient.clone(),
                    timestamp: env.ledger().timestamp(),
                },
            );
        }

        // Create escrow record
        let escrow = Escrow {
            depositor: depositor.clone(),
            amount: net_amount, // Store net amount (after fee)
            status: EscrowStatus::Locked,
            deadline,
            refund_history: vec![&env],
            remaining_amount: amount,
        };

        // Store in persistent storage with extended TTL
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(bounty_id), &escrow);

        // Emit event for off-chain indexing
        emit_funds_locked(
            &env,
            FundsLocked {
                bounty_id,
                amount: net_amount, // Emit net amount (after fee)
                depositor: depositor.clone(),
                deadline,
            },
        );

        env.storage().instance().remove(&DataKey::ReentrancyGuard);

        // Track successful operation
        monitoring::track_operation(&env, symbol_short!("lock"), caller, true);

        // Track performance
        let duration = env.ledger().timestamp().saturating_sub(start);
        monitoring::emit_performance(&env, symbol_short!("lock"), duration);

        Ok(())
    }

    /// Releases escrowed funds to a contributor.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `bounty_id` - The bounty to release funds for
    /// * `contributor` - Address to receive the funds
    ///
    /// # Returns
    /// * `Ok(())` - Funds successfully released
    /// * `Err(Error::NotInitialized)` - Contract not initialized
    /// * `Err(Error::Unauthorized)` - Caller is not the admin
    /// * `Err(Error::BountyNotFound)` - Bounty doesn't exist
    /// * `Err(Error::FundsNotLocked)` - Funds not in LOCKED state
    ///
    /// # State Changes
    /// - Transfers tokens from contract to contributor
    /// - Updates escrow status to Released
    /// - Emits FundsReleased event
    ///
    /// # Authorization
    /// - **CRITICAL**: Only admin can call this function
    /// - Admin address must match initialization value
    ///
    /// # Security Considerations
    /// - This is the most security-critical function
    /// - Admin should verify task completion off-chain before calling
    /// - Once released, funds cannot be retrieved
    /// - Recipient address should be verified carefully
    /// - Consider implementing multi-sig for admin
    ///
    /// # Events
    /// Emits: `FundsReleased { bounty_id, amount, recipient, timestamp }`
    ///
    /// # Example
    /// ```rust
    /// // After verifying task completion off-chain:
    /// let contributor = Address::from_string("GCONTRIB...");
    ///
    /// // Admin calls release
    /// escrow_client.release_funds(&42, &contributor)?;
    /// // Funds transferred to contributor, escrow marked as Released
    /// ```
    ///
    /// # Gas Cost
    /// Medium - Token transfer + storage update + event emission
    ///
    /// # Best Practices
    /// 1. Verify contributor identity off-chain
    /// 2. Confirm task completion before release
    /// 3. Log release decisions in backend system
    /// 4. Monitor release events for anomalies
    /// 5. Consider implementing release delays for high-value bounties
    pub fn release_funds(env: Env, bounty_id: u64, contributor: Address) -> Result<(), Error> {
        let start = env.ledger().timestamp();

        // Ensure contract is initialized
        if env.storage().instance().has(&DataKey::ReentrancyGuard) {
            panic!("Reentrancy detected");
        }
        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &true);
        if !env.storage().instance().has(&DataKey::Admin) {
            env.storage().instance().remove(&DataKey::ReentrancyGuard);
            return Err(Error::NotInitialized);
        }

        // Verify admin authorization
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

        // Apply rate limiting
        anti_abuse::check_rate_limit(&env, admin.clone());

        // Verify bounty exists
        if !env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            monitoring::track_operation(&env, symbol_short!("release"), admin.clone(), false);
            env.storage().instance().remove(&DataKey::ReentrancyGuard);
            return Err(Error::BountyNotFound);
        }

        // Get and verify escrow state
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(bounty_id))
            .unwrap();

        if escrow.status != EscrowStatus::Locked {
            monitoring::track_operation(&env, symbol_short!("release"), admin.clone(), false);
            env.storage().instance().remove(&DataKey::ReentrancyGuard);
            return Err(Error::FundsNotLocked);
        }

        // Check if multisig approval is required
        let multisig_config: MultisigConfig = Self::get_multisig_config(env.clone());

        if escrow.amount >= multisig_config.threshold_amount
            && multisig_config.required_signatures > 0
        {
            // Large release - requires multisig approval
            let approval_key = DataKey::ReleaseApproval(bounty_id);

            if !env.storage().persistent().has(&approval_key) {
                env.storage().instance().remove(&DataKey::ReentrancyGuard);
                return Err(Error::Unauthorized);
            }

            let approval: ReleaseApproval = env.storage().persistent().get(&approval_key).unwrap();

            if approval.approvals.len() < multisig_config.required_signatures {
                env.storage().instance().remove(&DataKey::ReentrancyGuard);
                return Err(Error::Unauthorized);
            }

            // Clear approval after use
            env.storage().persistent().remove(&approval_key);
        } else {
            // Small release - single admin approval
            admin.require_auth();
        }

        // Transfer funds to contributor
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);
        escrow.status = EscrowStatus::Released;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(bounty_id), &escrow);

        // Calculate and collect fee if enabled
        let fee_config = Self::get_fee_config_internal(&env);
        let fee_amount = if fee_config.fee_enabled && fee_config.release_fee_rate > 0 {
            Self::calculate_fee(escrow.amount, fee_config.release_fee_rate)
        } else {
            0
        };
        let net_amount = escrow.amount - fee_amount;

        // Transfer net amount to contributor
        client.transfer(&env.current_contract_address(), &contributor, &net_amount);

        // Transfer fee to fee recipient if applicable
        if fee_amount > 0 {
            client.transfer(
                &env.current_contract_address(),
                &fee_config.fee_recipient,
                &fee_amount,
            );
            events::emit_fee_collected(
                &env,
                events::FeeCollected {
                    operation_type: events::FeeOperationType::Release,
                    amount: fee_amount,
                    fee_rate: fee_config.release_fee_rate,
                    recipient: fee_config.fee_recipient.clone(),
                    timestamp: env.ledger().timestamp(),
                },
            );
        }

        // Update escrow state - mark as released and set remaining_amount to 0
        escrow.status = EscrowStatus::Released;
        escrow.remaining_amount = 0;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(bounty_id), &escrow);

        // Emit release event
        emit_funds_released(
            &env,
            FundsReleased {
                bounty_id,
                amount: net_amount, // Emit net amount (after fee)
                recipient: contributor.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        env.storage().instance().remove(&DataKey::ReentrancyGuard);

        // Track successful operation
        monitoring::track_operation(&env, symbol_short!("release"), admin, true);

        // Track performance
        let duration = env.ledger().timestamp().saturating_sub(start);
        monitoring::emit_performance(&env, symbol_short!("release"), duration);
        Ok(())
    }

    /// Set the claim window duration (admin only).
    /// claim_window: seconds beneficiary has to claim after release is authorized.
    pub fn set_claim_window(env: Env, claim_window: u64) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::ClaimWindow, &claim_window);
        Ok(())
    }

    /// Authorize a release as a pending claim instead of immediate transfer.
    /// Admin calls this instead of release_funds when claim period is active.
    /// Beneficiary must call claim() within the window to receive funds.
    pub fn authorize_claim(env: Env, bounty_id: u64, recipient: Address) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if !env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            return Err(Error::BountyNotFound);
        }
        let escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(bounty_id))
            .unwrap();
        if escrow.status != EscrowStatus::Locked {
            return Err(Error::FundsNotLocked);
        }

        let claim_window: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ClaimWindow)
            .unwrap_or(86400); // 24h default
        let now = env.ledger().timestamp();
        let claim = ClaimRecord {
            bounty_id,
            recipient: recipient.clone(),
            amount: escrow.amount,
            expires_at: now.saturating_add(claim_window),
            claimed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PendingClaim(bounty_id), &claim);

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub depositor: Address,
    pub amount: i128,
    pub status: EscrowStatus,
    pub deadline: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Token,
    Escrow(u64), // bounty_id
}

#[contract]
pub struct BountyEscrowContract;

#[contractimpl]
impl BountyEscrowContract {
    /// Initialize the contract with the admin address and the token address (XLM).
    pub fn init(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);

        emit_bounty_initialized(
            &env,
            BountyEscrowInitialized {
                admin,
                token,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Lock funds for a specific bounty.
    pub fn lock_funds(
        env: Env,
        depositor: Address,
        bounty_id: u64,
        amount: i128,
        deadline: u64,
    ) -> Result<(), Error> {
        depositor.require_auth();

        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }

        if env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            return Err(Error::BountyExists);
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);

        // Transfer funds from depositor to contract
        client.transfer(&depositor, &env.current_contract_address(), &amount);

        let escrow = Escrow {
            depositor: depositor.clone(),
            amount,
            status: EscrowStatus::Locked,
            deadline,
        };

        // Extend the TTL of the storage entry to ensure it lives long enough
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(bounty_id), &escrow);

        // Emit value allows for off-chain indexing
        emit_funds_locked(
            &env,
            FundsLocked {
                bounty_id,
                amount,
                depositor: depositor.clone(),
                deadline,
            },
        );

        Ok(())
    }

    /// Release funds to the contributor.
    /// Only the admin (backend) can authorize this.
    pub fn release_funds(env: Env, bounty_id: u64, contributor: Address) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if !env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            return Err(Error::BountyNotFound);
        }

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(bounty_id))
            .unwrap();

        if escrow.status != EscrowStatus::Locked {
            return Err(Error::FundsNotLocked);
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);

        // Transfer funds to contributor
        client.transfer(
            &env.current_contract_address(),
            &contributor,
            &escrow.amount,
        );

        escrow.status = EscrowStatus::Released;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(bounty_id), &escrow);

        emit_funds_released(
            &env,
            FundsReleased {
                bounty_id,
                amount: escrow.amount,
                recipient: contributor.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Refund funds to the original depositor if the deadline has passed.
    pub fn refund(env: Env, bounty_id: u64) -> Result<(), Error> {
        // We'll allow anyone to trigger the refund if conditions are met,
        // effectively making it permissionless but conditional.
        // OR we can require depositor auth. Let's make it permissionless to ensure funds aren't stuck if depositor key is lost,
        // but strictly logic bound.
        // However, usually refund is triggered by depositor. Let's stick to logic.

        if !env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            return Err(Error::BountyNotFound);
        }

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(bounty_id))
            .unwrap();

        if escrow.status != EscrowStatus::Locked {
            return Err(Error::FundsNotLocked);
        }

        let now = env.ledger().timestamp();
        if now < escrow.deadline {
            return Err(Error::DeadlineNotPassed);
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);

        // Transfer funds back to depositor
        client.transfer(
            &env.current_contract_address(),
            &escrow.depositor,
            &escrow.amount,
        );

        escrow.status = EscrowStatus::Refunded;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(bounty_id), &escrow);

        emit_funds_refunded(
            &env,
            FundsRefunded {
                bounty_id,
                amount: escrow.amount,
                refund_to: escrow.depositor,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// view function to get escrow info
    pub fn get_escrow_info(env: Env, bounty_id: u64) -> Result<Escrow, Error> {
        if !env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            return Err(Error::BountyNotFound);
        }
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(bounty_id))
            .unwrap())
    }

    /// view function to get contract balance of the token
    pub fn get_balance(env: Env) -> Result<i128, Error> {
        if !env.storage().instance().has(&DataKey::Token) {
            return Err(Error::NotInitialized);
        }
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);
        Ok(client.balance(&env.current_contract_address()))
    }

    /// Retrieves the refund history for a specific bounty.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `bounty_id` - The bounty to query
    ///
    /// # Returns
    /// * `Ok(Vec<RefundRecord>)` - The refund history
    /// * `Err(Error::BountyNotFound)` - Bounty doesn't exist
    pub fn get_refund_history(env: Env, bounty_id: u64) -> Result<Vec<RefundRecord>, Error> {
        if !env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            return Err(Error::BountyNotFound);
        }
        let escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(bounty_id))
            .unwrap();
        Ok(escrow.refund_history)
    }

    /// Gets refund eligibility information for a bounty.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `bounty_id` - The bounty to query
    ///
    /// # Returns
    /// * `Ok((bool, bool, i128, Option<RefundApproval>))` - Tuple containing:
    ///   - can_refund: Whether refund is possible
    ///   - deadline_passed: Whether the deadline has passed
    ///   - remaining: Remaining amount in escrow
    ///   - approval: Optional refund approval if exists
    /// * `Err(Error::BountyNotFound)` - Bounty doesn't exist
    pub fn get_refund_eligibility(
        env: Env,
        bounty_id: u64,
    ) -> Result<(bool, bool, i128, Option<RefundApproval>), Error> {
        if !env.storage().persistent().has(&DataKey::Escrow(bounty_id)) {
            return Err(Error::BountyNotFound);
        }
        let escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(bounty_id))
            .unwrap();

        let now = env.ledger().timestamp();
        let deadline_passed = now >= escrow.deadline;

        let approval = if env
            .storage()
            .persistent()
            .has(&DataKey::RefundApproval(bounty_id))
        {
            Some(
                env.storage()
                    .persistent()
                    .get(&DataKey::RefundApproval(bounty_id))
                    .unwrap(),
            )
        } else {
            None
        };

        // can_refund is true if:
        // 1. Status is Locked or PartiallyRefunded AND
        // 2. (deadline has passed OR there's an approval)
        let can_refund = (escrow.status == EscrowStatus::Locked
            || escrow.status == EscrowStatus::PartiallyRefunded)
            && (deadline_passed || approval.is_some());

        Ok((
            can_refund,
            deadline_passed,
            escrow.remaining_amount,
            approval,
        ))
    }

    /// Adds or removes an address from the whitelist.
    /// Only the admin can call this.
    pub fn set_whitelist(env: Env, address: Address, whitelisted: bool) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        admin.require_auth();

        anti_abuse::set_whitelist(&env, address, whitelisted);
    }

    /// Gets the current rate limit configuration.
    pub fn get_config(env: Env) -> anti_abuse::AntiAbuseConfig {
        anti_abuse::get_config(&env)
    }

    /// Batch lock funds for multiple bounties in a single transaction.
    /// This improves gas efficiency by reducing transaction overhead.
    ///
    /// # Arguments
    /// * `items` - Vector of LockFundsItem containing bounty_id, depositor, amount, and deadline
    ///
    /// # Returns
    /// Number of successfully locked bounties
    ///
    /// # Errors
    /// * InvalidBatchSize - if batch size exceeds MAX_BATCH_SIZE or is zero
    /// * BountyExists - if any bounty_id already exists
    /// * NotInitialized - if contract is not initialized
    ///
    /// # Note
    /// This operation is atomic - if any item fails, the entire transaction reverts.
    pub fn batch_lock_funds(env: Env, items: Vec<LockFundsItem>) -> Result<u32, Error> {
        // Validate batch size
        let batch_size = items.len();
        if batch_size == 0 {
            return Err(Error::InvalidBatchSize);
        }
        if batch_size > MAX_BATCH_SIZE {
            return Err(Error::InvalidBatchSize);
        }

        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);
        let contract_address = env.current_contract_address();
        let timestamp = env.ledger().timestamp();

        // Validate all items before processing (all-or-nothing approach)
        for item in items.iter() {
            // Check if bounty already exists
            if env
                .storage()
                .persistent()
                .has(&DataKey::Escrow(item.bounty_id))
            {
                return Err(Error::BountyExists);
            }

            // Validate amount
            if item.amount <= 0 {
                return Err(Error::InvalidAmount);
            }

            // Check for duplicate bounty_ids in the batch
            let mut count = 0u32;
            for other_item in items.iter() {
                if other_item.bounty_id == item.bounty_id {
                    count += 1;
                }
            }
            if count > 1 {
                return Err(Error::DuplicateBountyId);
            }
        }

        // Collect unique depositors and require auth once for each
        // This prevents "frame is already authorized" errors when same depositor appears multiple times
        let mut seen_depositors: Vec<Address> = Vec::new(&env);
        for item in items.iter() {
            let mut found = false;
            for seen in seen_depositors.iter() {
                if seen.clone() == item.depositor {
                    found = true;
                    break;
                }
            }
            if !found {
                seen_depositors.push_back(item.depositor.clone());
                item.depositor.require_auth();
            }
        }

        // Process all items (atomic - all succeed or all fail)
        let mut locked_count = 0u32;
        for item in items.iter() {
            // Transfer funds from depositor to contract
            client.transfer(&item.depositor, &contract_address, &item.amount);

            // Create escrow record
            let escrow = Escrow {
                depositor: item.depositor.clone(),
                amount: item.amount,
                status: EscrowStatus::Locked,
                deadline: item.deadline,
                refund_history: vec![&env],
                remaining_amount: item.amount,
            };

            // Store escrow
            env.storage()
                .persistent()
                .set(&DataKey::Escrow(item.bounty_id), &escrow);

            // Emit individual event for each locked bounty
            emit_funds_locked(
                &env,
                FundsLocked {
                    bounty_id: item.bounty_id,
                    amount: item.amount,
                    depositor: item.depositor.clone(),
                    deadline: item.deadline,
                },
            );

            locked_count += 1;
        }

        // Emit batch event
        emit_batch_funds_locked(
            &env,
            BatchFundsLocked {
                count: locked_count,
                total_amount: items.iter().map(|i| i.amount).sum(),
                timestamp,
            },
        );

        Ok(locked_count)
    }

    /// Batch release funds to multiple contributors in a single transaction.
    /// This improves gas efficiency by reducing transaction overhead.
    ///
    /// # Arguments
    /// * `items` - Vector of ReleaseFundsItem containing bounty_id and contributor address
    ///
    /// # Returns
    /// Number of successfully released bounties
    ///
    /// # Errors
    /// * InvalidBatchSize - if batch size exceeds MAX_BATCH_SIZE or is zero
    /// * BountyNotFound - if any bounty_id doesn't exist
    /// * FundsNotLocked - if any bounty is not in Locked status
    /// * Unauthorized - if caller is not admin
    ///
    /// # Note
    /// This operation is atomic - if any item fails, the entire transaction reverts.
    pub fn batch_release_funds(env: Env, items: Vec<ReleaseFundsItem>) -> Result<u32, Error> {
        // Validate batch size
        let batch_size = items.len();
        if batch_size == 0 {
            return Err(Error::InvalidBatchSize);
        }
        if batch_size > MAX_BATCH_SIZE {
            return Err(Error::InvalidBatchSize);
        }

        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = token::Client::new(&env, &token_addr);
        let contract_address = env.current_contract_address();
        let timestamp = env.ledger().timestamp();

        // Validate all items before processing (all-or-nothing approach)
        let mut total_amount: i128 = 0;
        for item in items.iter() {
            // Check if bounty exists
            if !env
                .storage()
                .persistent()
                .has(&DataKey::Escrow(item.bounty_id))
            {
                return Err(Error::BountyNotFound);
            }

            let escrow: Escrow = env
                .storage()
                .persistent()
                .get(&DataKey::Escrow(item.bounty_id))
                .unwrap();

            // Check if funds are locked
            if escrow.status != EscrowStatus::Locked {
                return Err(Error::FundsNotLocked);
            }

            // Check for duplicate bounty_ids in the batch
            let mut count = 0u32;
            for other_item in items.iter() {
                if other_item.bounty_id == item.bounty_id {
                    count += 1;
                }
            }
            if count > 1 {
                return Err(Error::DuplicateBountyId);
            }

            total_amount = total_amount
                .checked_add(escrow.amount)
                .ok_or(Error::InvalidAmount)?;
        }

        // Process all items (atomic - all succeed or all fail)
        let mut released_count = 0u32;
        for item in items.iter() {
            let mut escrow: Escrow = env
                .storage()
                .persistent()
                .get(&DataKey::Escrow(item.bounty_id))
                .unwrap();

            // Transfer funds to contributor
            client.transfer(&contract_address, &item.contributor, &escrow.amount);

            // Update escrow status
            escrow.status = EscrowStatus::Released;
            env.storage()
                .persistent()
                .set(&DataKey::Escrow(item.bounty_id), &escrow);

            // Emit individual event for each released bounty
            emit_funds_released(
                &env,
                FundsReleased {
                    bounty_id: item.bounty_id,
                    amount: escrow.amount,
                    recipient: item.contributor.clone(),
                    timestamp,
                },
            );

            released_count += 1;
        }

        // Emit batch event
        emit_batch_funds_released(
            &env,
            BatchFundsReleased {
                count: released_count,
                total_amount,
                timestamp,
            },
        );

        Ok(released_count)
    }
}

#[cfg(test)]
mod test;
