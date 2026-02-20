#![no_std]
//! # Program Escrow Smart Contract
//!
//! A secure escrow system for managing hackathon and program prize pools on Stellar.
//! This contract enables organizers to lock funds and distribute prizes to multiple
//! winners through secure, auditable batch payouts.
//!
//! ## Overview
//!
//! The Program Escrow contract manages the complete lifecycle of hackathon/program prizes:
//! 1. **Initialization**: Set up program with authorized payout controller
//! 2. **Fund Locking**: Lock prize pool funds in escrow
//! 3. **Batch Payouts**: Distribute prizes to multiple winners simultaneously
//! 4. **Single Payouts**: Distribute individual prizes
//! 5. **Tracking**: Maintain complete payout history and balance tracking
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │              Program Escrow Architecture                         │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌──────────────┐                                               │
//! │  │  Organizer   │                                               │
//! │  └──────┬───────┘                                               │
//! │         │                                                        │
//! │         │ 1. init_program()                                     │
//! │         ▼                                                        │
//! │  ┌──────────────────┐                                           │
//! │  │  Program Created │                                           │
//! │  └────────┬─────────┘                                           │
//! │           │                                                      │
//! │           │ 2. lock_program_funds()                             │
//! │           ▼                                                      │
//! │  ┌──────────────────┐                                           │
//! │  │  Funds Locked    │                                           │
//! │  │  (Prize Pool)    │                                           │
//! │  └────────┬─────────┘                                           │
//! │           │                                                      │
//! │           │ 3. Hackathon happens...                             │
//! │           │                                                      │
//! │  ┌────────▼─────────┐                                           │
//! │  │ Authorized       │                                           │
//! │  │ Payout Key       │                                           │
//! │  └────────┬─────────┘                                           │
//! │           │                                                      │
//! │    ┌──────┴───────┐                                             │
//! │    │              │                                             │
//! │    ▼              ▼                                             │
//! │ batch_payout() single_payout()                                  │
//! │    │              │                                             │
//! │    ▼              ▼                                             │
//! │ ┌─────────────────────────┐                                    │
//! │ │   Winner 1, 2, 3, ...   │                                    │
//! │ └─────────────────────────┘                                    │
//! │                                                                  │
//! │  Storage:                                                        │
//! │  ┌──────────────────────────────────────────┐                  │
//! │  │ ProgramData:                             │                  │
//! │  │  - program_id                            │                  │
//! │  │  - total_funds                           │                  │
//! │  │  - remaining_balance                     │                  │
//! │  │  - authorized_payout_key                 │                  │
//! │  │  - payout_history: [PayoutRecord]        │                  │
//! │  │  - token_address                         │                  │
//! │  └──────────────────────────────────────────┘                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Model
//!
//! ### Trust Assumptions
//! - **Authorized Payout Key**: Trusted backend service that triggers payouts
//! - **Organizer**: Trusted to lock appropriate prize amounts
//! - **Token Contract**: Standard Stellar Asset Contract (SAC)
//! - **Contract**: Trustless; operates according to programmed rules
//!
//! ### Key Security Features
//! 1. **Single Initialization**: Prevents program re-configuration
//! 2. **Authorization Checks**: Only authorized key can trigger payouts
//! 3. **Balance Validation**: Prevents overdrafts
//! 4. **Atomic Transfers**: All-or-nothing batch operations
//! 5. **Complete Audit Trail**: Full payout history tracking
//! 6. **Overflow Protection**: Safe arithmetic for all calculations
//!
//! ## Usage Example
//!
//! ```rust
//! use soroban_sdk::{Address, Env, String, vec};
//!
//! // 1. Initialize program (one-time setup)
//! let program_id = String::from_str(&env, "Hackathon2024");
//! let backend = Address::from_string("GBACKEND...");
//! let usdc_token = Address::from_string("CUSDC...");
//!
//! let program = escrow_client.init_program(
//!     &program_id,
//!     &backend,
//!     &usdc_token
//! );
//!
//! // 2. Lock prize pool (10,000 USDC)
//! let prize_pool = 10_000_0000000; // 10,000 USDC (7 decimals)
//! escrow_client.lock_program_funds(&prize_pool);
//!
//! // 3. After hackathon, distribute prizes
//! let winners = vec![
//!     &env,
//!     Address::from_string("GWINNER1..."),
//!     Address::from_string("GWINNER2..."),
//!     Address::from_string("GWINNER3..."),
//! ];
//!
//! let prizes = vec![
//!     &env,
//!     5_000_0000000,  // 1st place: 5,000 USDC
//!     3_000_0000000,  // 2nd place: 3,000 USDC
//!     2_000_0000000,  // 3rd place: 2,000 USDC
//! ];
//!
//! escrow_client.batch_payout(&winners, &prizes);
//! ```
//!
//! ## Event System
//!
//! The contract emits events for all major operations:
//! - `ProgramInit`: Program initialization
//! - `FundsLocked`: Prize funds locked
//! - `BatchPayout`: Multiple prizes distributed
//! - `Payout`: Single prize distributed
//!
//! ## Best Practices
//!
//! 1. **Verify Winners**: Confirm winner addresses off-chain before payout
//! 2. **Test Payouts**: Use testnet for testing prize distributions
//! 3. **Secure Backend**: Protect authorized payout key with HSM/multi-sig
//! 4. **Audit History**: Review payout history before each distribution
//! 5. **Balance Checks**: Verify remaining balance matches expectations
//! 6. **Token Approval**: Ensure contract has token allowance before locking funds



// ── Step 1: Add module declarations near the top of lib.rs ──────────────
// (after `mod anti_abuse;` and before the contract struct)

mod error_recovery;

#[cfg(test)]
mod error_recovery_tests;

// ── Step 2: Add these public contract functions to the ProgramEscrowContract
//    impl block (alongside the existing admin functions) ──────────────────

    // ========================================================================
    // Circuit Breaker Management
    // ========================================================================

    /// Register the circuit breaker admin. Can only be set once, or changed
    /// by the existing admin.
    ///
    /// # Arguments
    /// * `new_admin` - Address to register as circuit breaker admin
    /// * `caller`    - Existing admin (None if setting for the first time)
    pub fn set_circuit_admin(env: Env, new_admin: Address, caller: Option<Address>) {
        error_recovery::set_circuit_admin(&env, new_admin, caller);
    }

    /// Returns the registered circuit breaker admin, if any.
    pub fn get_circuit_admin(env: Env) -> Option<Address> {
        error_recovery::get_circuit_admin(&env)
    }

    /// Returns the full circuit breaker status snapshot.
    ///
    /// # Returns
    /// * `CircuitBreakerStatus` with state, failure/success counts, timestamps
    pub fn get_circuit_status(env: Env) -> error_recovery::CircuitBreakerStatus {
        error_recovery::get_status(&env)
    }

    /// Admin resets the circuit breaker.
    ///
    /// Transitions:
    /// - Open     → HalfOpen  (probe mode)
    /// - HalfOpen → Closed    (hard reset)
    /// - Closed   → Closed    (no-op reset)
    ///
    /// # Panics
    /// * If caller is not the registered circuit breaker admin
    pub fn reset_circuit_breaker(env: Env, admin: Address) {
        error_recovery::reset_circuit_breaker(&env, &admin);
    }

    /// Updates the circuit breaker configuration. Admin only.
    ///
    /// # Arguments
    /// * `failure_threshold` - Consecutive failures needed to open circuit
    /// * `success_threshold` - Consecutive successes in HalfOpen to close it
    /// * `max_error_log`     - Maximum error log entries to retain
    pub fn configure_circuit_breaker(
        env: Env,
        admin: Address,
        failure_threshold: u32,
        success_threshold: u32,
        max_error_log: u32,
    ) {
        let stored = error_recovery::get_circuit_admin(&env);
        match stored {
            Some(ref a) if a == &admin => {
                admin.require_auth();
            }
            _ => panic!("Unauthorized: only circuit breaker admin can configure"),
        }
        error_recovery::set_config(
            &env,
            error_recovery::CircuitBreakerConfig {
                failure_threshold,
                success_threshold,
                max_error_log,
            },
        );
    }

    /// Returns the error log (last N failures recorded by the circuit breaker).
    pub fn get_circuit_error_log(env: Env) -> soroban_sdk::Vec<error_recovery::ErrorEntry> {
        error_recovery::get_error_log(&env)
    }

    /// Directly open the circuit (emergency lockout). Admin only.
    pub fn emergency_open_circuit(env: Env, admin: Address) {
        let stored = error_recovery::get_circuit_admin(&env);
        match stored {
            Some(ref a) if a == &admin => {
                admin.require_auth();
            }
            _ => panic!("Unauthorized"),
        }
        error_recovery::open_circuit(&env);
    }

// ── Step 3: Wrap batch_payout and single_payout with circuit breaker ────
//
// In the existing batch_payout function, add at the very top (after getting
// program_data but before the auth check):
//
//   use crate::error_recovery;
//   if let Err(_) = error_recovery::check_and_allow(&env) {
//       panic!("Circuit breaker is open: payout operations are temporarily disabled");
//   }
//
// After a successful transfer loop, add:
//   error_recovery::record_success(&env);
//
// If a transfer panics/fails, the circuit breaker failure should be recorded
// via record_failure() before re-panicking.
//
// For a clean integration, wrap the token transfer call like this:
//
//   let transfer_ok = std::panic::catch_unwind(|| {
//       token_client.transfer(&contract_address, &recipient.clone(), &net_amount);
//   });
//   match transfer_ok {
//       Ok(_) => error_recovery::record_success(&env),
//       Err(_) => {
//           error_recovery::record_failure(
//               &env,
//               program_id.clone(),
//               soroban_sdk::symbol_short!("batch_pay"),
//               error_recovery::ERR_TRANSFER_FAILED,
//           );
//           panic!("Token transfer failed");
//       }
//   }
//
// Note: Soroban's environment panics abort the transaction, so in practice
// you record the failure and re-panic. The circuit breaker state is committed
// because Soroban persists storage writes made before the panic in tests
// (but not in production transactions that abort). For full production
// integration, use the `try_*` variants of client calls where available.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, vec, Address, Env, String, Symbol,
    Vec,
};

// Event types
const PROGRAM_INITIALIZED: Symbol = symbol_short!("PrgInit");
const FUNDS_LOCKED: Symbol = symbol_short!("FndsLock");
const BATCH_PAYOUT: Symbol = symbol_short!("BatchPay");
const PAYOUT: Symbol = symbol_short!("Payout");

// Storage keys
const PROGRAM_DATA: Symbol = symbol_short!("ProgData");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayoutRecord {
    pub recipient: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramData {
    pub program_id: String,
    pub total_funds: i128,
    pub remaining_balance: i128,
    pub authorized_payout_key: Address,
    pub payout_history: Vec<PayoutRecord>,
    pub token_address: Address, // Token contract address for transfers
}

#[contract]
pub struct ProgramEscrowContract;

#[contractimpl]
impl ProgramEscrowContract {
    /// Initialize a new program escrow
    ///
    /// # Arguments
    /// * `program_id` - Unique identifier for the program/hackathon
    /// * `authorized_payout_key` - Address authorized to trigger payouts (backend)
    /// * `token_address` - Address of the token contract to use for transfers
    ///
    /// # Returns
    /// The initialized ProgramData
    pub fn init_program(
        env: Env,
        program_id: String,
        authorized_payout_key: Address,
        token_address: Address,
    ) -> ProgramData {
        // Apply rate limiting
        anti_abuse::check_rate_limit(&env, authorized_payout_key.clone());

        let start = env.ledger().timestamp();
        let caller = authorized_payout_key.clone();

        // Validate program_id
        if program_id.is_empty() {
            monitoring::track_operation(&env, symbol_short!("init_prg"), caller, false);
            panic!("Program ID cannot be empty");
        }

        // Check if program already exists
        if env.storage().instance().has(&PROGRAM_DATA) {
            panic!("Program already initialized");
        }

        let program_data = ProgramData {
            program_id: program_id.clone(),
            total_funds: 0,
            remaining_balance: 0,
            authorized_payout_key: authorized_payout_key.clone(),
            payout_history: vec![&env],
            token_address: token_address.clone(),
        };

        // Store program data
        env.storage().instance().set(&PROGRAM_DATA, &program_data);

        // Emit ProgramInitialized event
        env.events().publish(
            (PROGRAM_INITIALIZED,),
            (program_id, authorized_payout_key, token_address, 0i128),
        );

        program_data
    }

    /// Lock initial funds into the program escrow
    ///
    /// # Arguments
    /// * `amount` - Amount of funds to lock (in native token units)
    ///
    /// # Returns
    /// * `ProgramData` - Updated program data with new balance
    ///
    /// # Panics
    /// * If amount is zero or negative
    /// * If program is not initialized
    ///
    /// # State Changes
    /// - Increases `total_funds` by amount
    /// - Increases `remaining_balance` by amount
    /// - Emits FundsLocked event
    ///
    /// # Prerequisites
    /// Before calling this function:
    /// 1. Caller must have sufficient token balance
    /// 2. Caller must approve contract for token transfer
    /// 3. Tokens must actually be transferred to contract
    ///
    /// # Security Considerations
    /// - Amount must be positive
    /// - This function doesn't perform the actual token transfer
    /// - Caller is responsible for transferring tokens to contract
    /// - Consider verifying contract balance matches recorded amount
    /// - Multiple lock operations are additive (cumulative)
    ///
    /// # Events
    /// Emits: `FundsLocked(program_id, amount, new_remaining_balance)`
    ///
    /// # Example
    /// ```rust
    /// use soroban_sdk::token;
    ///
    /// // 1. Transfer tokens to contract
    /// let amount = 10_000_0000000; // 10,000 USDC
    /// token_client.transfer(
    ///     &organizer,
    ///     &contract_address,
    ///     &amount
    /// );
    ///
    /// // 2. Record the locked funds
    /// let updated = escrow_client.lock_program_funds(&amount);
    /// println!("Locked: {} USDC", amount / 10_000_000);
    /// println!("Remaining: {}", updated.remaining_balance);
    /// ```
    ///
    /// # Production Usage
    /// ```bash
    /// # 1. Transfer USDC to contract
    /// stellar contract invoke \
    ///   --id USDC_TOKEN_ID \
    ///   --source ORGANIZER_KEY \
    ///   -- transfer \
    ///   --from ORGANIZER_ADDRESS \
    ///   --to CONTRACT_ADDRESS \
    ///   --amount 10000000000
    ///
    /// # 2. Record locked funds
    /// stellar contract invoke \
    ///   --id CONTRACT_ID \
    ///   --source ORGANIZER_KEY \
    ///   -- lock_program_funds \
    ///   --amount 10000000000
    /// ```
    ///
    /// # Gas Cost
    /// Low - Storage update + event emission
    ///
    /// # Common Pitfalls
    /// - Forgetting to transfer tokens before calling
    /// -  Locking amount that exceeds actual contract balance
    /// -  Not verifying contract received the tokens

    pub fn lock_program_funds(env: Env, program_id: String, amount: i128) -> ProgramData {
        // Apply rate limiting
        anti_abuse::check_rate_limit(&env, env.current_contract_address());

        let _start = env.ledger().timestamp();
        let caller = env.current_contract_address();

        // Validate amount
        if amount <= 0 {
            panic!("Amount must be greater than zero");
        }

        let mut program_data: ProgramData = env
            .storage()
            .instance()
            .get(&PROGRAM_DATA)
            .unwrap_or_else(|| panic!("Program not initialized"));

        // Update balances
        program_data.total_funds += amount;
        program_data.remaining_balance += amount;

        // Store updated data
        env.storage().instance().set(&PROGRAM_DATA, &program_data);

        // Emit FundsLocked event
        env.events().publish(
            (FUNDS_LOCKED,),
            (
                program_data.program_id.clone(),
                amount,
                program_data.remaining_balance,
            ),
        );

        program_data
    }

    /// Execute batch payouts to multiple recipients
    ///
    /// # Arguments
    /// * `recipients` - Vector of recipient addresses
    /// * `amounts` - Vector of amounts (must match recipients length)
    ///
    /// # Returns
    /// Updated ProgramData after payouts
    pub fn batch_payout(env: Env, recipients: Vec<Address>, amounts: Vec<i128>) -> ProgramData {
        // Verify authorization
        let program_data: ProgramData = env
            .storage()
            .instance()
            .get(&PROGRAM_DATA)
            .unwrap_or_else(|| panic!("Program not initialized"));

        program_data.authorized_payout_key.require_auth();

        // Validate input lengths match
        if recipients.len() != amounts.len() {
            panic!("Recipients and amounts vectors must have the same length");
        }

        if recipients.len() == 0 {
            panic!("Cannot process empty batch");
        }

        // Calculate total payout amount
        let mut total_payout: i128 = 0;
        for amount in amounts.iter() {
            if amount <= 0 {
                panic!("All amounts must be greater than zero");
            }
            total_payout = total_payout
                .checked_add(amount)
                .unwrap_or_else(|| panic!("Payout amount overflow"));
        }

        // Validate sufficient balance
        if total_payout > program_data.remaining_balance {
            panic!("Insufficient balance");
        }

        // Execute transfers
        let mut updated_history = program_data.payout_history.clone();
        let timestamp = env.ledger().timestamp();
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(&env, &program_data.token_address);

        for i in 0..recipients.len() {
            let recipient = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            // Transfer funds from contract to recipient
            token_client.transfer(&contract_address, &recipient, &amount);

            // Record payout
            let payout_record = PayoutRecord {
                recipient,
                amount,
                timestamp,
            };
            updated_history.push_back(payout_record);
        }

        // Update program data
        let mut updated_data = program_data.clone();
        updated_data.remaining_balance -= total_payout;
        updated_data.payout_history = updated_history;

        // Store updated data
        env.storage().instance().set(&PROGRAM_DATA, &updated_data);

        // Emit BatchPayout event
        env.events().publish(
            (BATCH_PAYOUT,),
            (
                updated_data.program_id.clone(),
                recipients.len() as u32,
                program_id,
                recipients.len(),
                total_payout,
                updated_data.remaining_balance,
            ),
        );

        updated_data
    }

    /// Execute a single payout to one recipient
    ///
    /// # Arguments
    /// * `recipient` - Address of the recipient
    /// * `amount` - Amount to transfer
    ///
    /// # Returns
    /// Updated ProgramData after payout
    pub fn single_payout(env: Env, recipient: Address, amount: i128) -> ProgramData {
        // Verify authorization
        let program_data: ProgramData = env
            .storage()
            .instance()
            .get(&PROGRAM_DATA)
            .unwrap_or_else(|| panic!("Program not initialized"));

        program_data.authorized_payout_key.require_auth();

        // Validate amount
        if amount <= 0 {
            panic!("Amount must be greater than zero");
        }

        // Validate sufficient balance
        if amount > program_data.remaining_balance {
            panic!("Insufficient balance");
        }

        // Transfer funds from contract to recipient
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(&env, &program_data.token_address);
        token_client.transfer(&contract_address, &recipient, &amount);

        // Record payout
        let timestamp = env.ledger().timestamp();
        let payout_record = PayoutRecord {
            recipient: recipient.clone(),
            amount,
            timestamp,
        };

        let mut updated_history = program_data.payout_history.clone();
        updated_history.push_back(payout_record);

        // Update program data
        let mut updated_data = program_data.clone();
        updated_data.remaining_balance -= amount;
        updated_data.payout_history = updated_history;

        // Store updated data
        env.storage().instance().set(&PROGRAM_DATA, &updated_data);

        // Emit Payout event
        env.events().publish(
            (PAYOUT,),
            (
                updated_data.program_id.clone(),
                recipient,
                amount,
                updated_data.remaining_balance,
            ),
        );

        updated_data
    }

    /// Get program information
    ///
    /// # Returns
    /// ProgramData containing all program information
    pub fn get_program_info(env: Env) -> ProgramData {
        env.storage()
            .instance()
            .get(&PROGRAM_DATA)
            .unwrap_or_else(|| panic!("Program not initialized"))
    }

    /// Get remaining balance
    ///
    /// # Returns
    /// Current remaining balance
    pub fn get_remaining_balance(env: Env) -> i128 {
        let program_data: ProgramData = env
            .storage()
            .instance()
            .get(&PROGRAM_DATA)
            .unwrap_or_else(|| panic!("Program not initialized"));

        program_data.remaining_balance
    }
}

#[cfg(test)]
mod test;

    /// Admin cancels an unclaimed (possibly expired) pending claim.
    pub fn cancel_program_claim(env: Env, program_id: String, schedule_id: u64) {
        let program_key = DataKey::Program(program_id.clone());
        let program_data: ProgramData = env
            .storage()
            .instance()
            .get(&program_key)
            .unwrap_or_else(|| panic!("Program not found"));
        program_data.authorized_payout_key.require_auth();

        if !env
            .storage()
            .persistent()
            .has(&DataKey::PendingClaim(program_id.clone(), schedule_id))
        {
            panic!("No pending claim found");
        }
        let claim: ClaimRecord = env
            .storage()
            .persistent()
            .get(&DataKey::PendingClaim(program_id.clone(), schedule_id))
            .unwrap();

        if claim.claimed {
            panic!("Claim already executed");
        }

        env.storage()
            .persistent()
            .remove(&DataKey::PendingClaim(program_id, schedule_id));

        env.events().publish(
            (symbol_short!("claim"), symbol_short!("cancel")),
            ClaimCancelled {
                bounty_id: schedule_id,
                recipient: claim.recipient,
                amount: claim.amount,
                cancelled_at: env.ledger().timestamp(),
                cancelled_by: program_data.authorized_payout_key,
            },
        );
    }

    /// View: get a pending claim for a program schedule.
    pub fn get_program_pending_claim(
        env: Env,
        program_id: String,
        schedule_id: u64,
    ) -> ClaimRecord {
        env.storage()
            .persistent()
            .get(&DataKey::PendingClaim(program_id, schedule_id))
            .unwrap_or_else(|| panic!("No pending claim found"))
    }

    // ========================================================================
    // View Functions (Read-only)
    // ========================================================================

    /// Retrieves complete program information.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    ///
    /// # Returns
    /// * `ProgramData` - Complete program state including:
    ///   - Program ID
    ///   - Total funds locked
    ///   - Remaining balance
    ///   - Authorized payout key
    ///   - Complete payout history
    ///   - Token contract address
    ///
    /// # Panics
    /// * If program is not initialized
    ///
    /// # Use Cases
    /// - Verifying program configuration
    /// - Checking balances before payouts
    /// - Auditing payout history
    /// - Displaying program status in UI
    ///
    /// # Example
    /// ```rust
    /// let info = escrow_client.get_program_info();
    /// println!("Program: {}", info.program_id);
    /// println!("Total Locked: {}", info.total_funds);
    /// println!("Remaining: {}", info.remaining_balance);
    /// println!("Payouts Made: {}", info.payout_history.len());
    /// ```
    ///
    /// # Gas Cost
    /// Very Low - Single storage read
    pub fn get_program_info(env: Env, program_id: String) -> ProgramData {
        let program_key = DataKey::Program(program_id);
        env.storage()
            .instance()
            .get(&program_key)
            .unwrap_or_else(|| panic!("Program not found"))
    }

    /// Retrieves the remaining balance for a specific program.
    ///
    /// # Arguments
    /// * `program_id` - The program ID to query
    ///
    /// # Returns
    /// * `i128` - Remaining balance
    ///
    /// # Panics
    /// * If program doesn't exist
    pub fn get_remaining_balance(env: Env, program_id: String) -> i128 {
        let program_key = DataKey::Program(program_id);
        let program_data: ProgramData = env
            .storage()
            .instance()
            .get(&program_key)
            .unwrap_or_else(|| panic!("Program not found"));

        program_data.remaining_balance
    }

    /// Update fee configuration (admin only - uses authorized_payout_key)
    ///
    /// # Arguments
    /// * `lock_fee_rate` - Optional new lock fee rate (basis points)
    /// * `payout_fee_rate` - Optional new payout fee rate (basis points)
    /// * `fee_recipient` - Optional new fee recipient address
    /// * `fee_enabled` - Optional fee enable/disable flag
    pub fn update_fee_config(
        env: Env,
        lock_fee_rate: Option<i128>,
        payout_fee_rate: Option<i128>,
        fee_recipient: Option<Address>,
        fee_enabled: Option<bool>,
    ) {
        // Verify authorization
        let program_data: ProgramData = env
            .storage()
            .instance()
            .get(&PROGRAM_DATA)
            .unwrap_or_else(|| panic!("Program not initialized"));

        // Note: In Soroban, we check authorization by requiring auth from the authorized key
        // For this function, we'll require auth from the authorized_payout_key
        program_data.authorized_payout_key.require_auth();

        let mut fee_config = Self::get_fee_config_internal(&env);

        if let Some(rate) = lock_fee_rate {
            if !(0..=MAX_FEE_RATE).contains(&rate) {
                panic!(
                    "Invalid lock fee rate: must be between 0 and {}",
                    MAX_FEE_RATE
                );
            }
            fee_config.lock_fee_rate = rate;
        }

        if let Some(rate) = payout_fee_rate {
            if !(0..=MAX_FEE_RATE).contains(&rate) {
                panic!(
                    "Invalid payout fee rate: must be between 0 and {}",
                    MAX_FEE_RATE
                );
            }
            fee_config.payout_fee_rate = rate;
        }

        if let Some(recipient) = fee_recipient {
            fee_config.fee_recipient = recipient;
        }

        if let Some(enabled) = fee_enabled {
            fee_config.fee_enabled = enabled;
        }

        env.storage().instance().set(&FEE_CONFIG, &fee_config);

        // Emit fee config updated event
        env.events().publish(
            (symbol_short!("fee_cfg"),),
            (
                fee_config.lock_fee_rate,
                fee_config.payout_fee_rate,
                fee_config.fee_recipient,
                fee_config.fee_enabled,
            ),
        );
    }

    /// Get current fee configuration (view function)
    pub fn get_fee_config(env: Env) -> FeeConfig {
        Self::get_fee_config_internal(&env)
    }

    /// Update multisig configuration for a program (authorized payout key only)
    pub fn update_multisig_config(
        env: Env,
        program_id: String,
        threshold_amount: i128,
        signers: Vec<Address>,
        required_signatures: u32,
    ) {
        let program_key = DataKey::Program(program_id.clone());
        let program_data: ProgramData = env
            .storage()
            .instance()
            .get(&program_key)
            .unwrap_or_else(|| panic!("Program not found"));

        program_data.authorized_payout_key.require_auth();

        if required_signatures > signers.len() {
            panic!("Required signatures cannot exceed number of signers");
        }

        let config = MultisigConfig {
            threshold_amount,
            signers,
            required_signatures,
        };

        env.storage()
            .persistent()
            .set(&DataKey::MultisigConfig(program_id), &config);
    }

    /// Get multisig configuration for a program
    pub fn get_multisig_config(env: Env, program_id: String) -> MultisigConfig {
        env.storage()
            .persistent()
            .get(&DataKey::MultisigConfig(program_id))
            .unwrap_or(MultisigConfig {
                threshold_amount: i128::MAX,
                signers: vec![&env],
                required_signatures: 0,
            })
    }

    /// Approve large payout (requires multisig)
    pub fn approve_large_payout(
        env: Env,
        program_id: String,
        recipient: Address,
        amount: i128,
        approver: Address,
    ) {
        let multisig_config: MultisigConfig =
            Self::get_multisig_config(env.clone(), program_id.clone());

        let mut is_signer = false;
        for signer in multisig_config.signers.iter() {
            if signer == approver {
                is_signer = true;
                break;
            }
        }

        if !is_signer {
            panic!("Caller is not an authorized signer");
        }

        approver.require_auth();

        let approval_key = DataKey::PayoutApproval(program_id.clone(), recipient.clone());
        let mut approval: PayoutApproval =
            env.storage()
                .persistent()
                .get(&approval_key)
                .unwrap_or(PayoutApproval {
                    program_id: program_id.clone(),
                    recipient: recipient.clone(),
                    amount,
                    approvals: vec![&env],
                });

        for existing in approval.approvals.iter() {
            if existing == approver {
                return;
            }
        }

        approval.approvals.push_back(approver.clone());
        env.storage().persistent().set(&approval_key, &approval);

        env.events().publish(
            (symbol_short!("approval"),),
            (program_id, recipient, amount, approver),
        );
    }

    /// Gets the total number of programs registered.
    ///
    /// # Returns
    /// * `u32` - Count of registered programs
    pub fn get_program_count(env: Env) -> u32 {
        let registry: Vec<String> = env
            .storage()
            .instance()
            .get(&PROGRAM_REGISTRY)
            .unwrap_or(vec![&env]);

        registry.len()
    }

    // ========================================================================
    // Monitoring & Analytics Functions
    // ========================================================================

    /// Health check - returns contract health status
    pub fn health_check(env: Env) -> monitoring::HealthStatus {
        monitoring::health_check(&env)
    }

    /// Get analytics - returns usage analytics
    pub fn get_analytics(env: Env) -> monitoring::Analytics {
        monitoring::get_analytics(&env)
    }

    /// Get state snapshot - returns current state
    pub fn get_state_snapshot(env: Env) -> monitoring::StateSnapshot {
        monitoring::get_state_snapshot(&env)
    }

    /// Get performance stats for a function
    pub fn get_performance_stats(env: Env, function_name: Symbol) -> monitoring::PerformanceStats {
        monitoring::get_performance_stats(&env, function_name)
    }

    // ========================================================================
    // Anti-Abuse Administrative Functions
    // ========================================================================

    /// Sets the administrative address for anti-abuse configuration.
    /// Can only be called once or by the existing admin.
    pub fn set_admin(env: Env, new_admin: Address) {
        if let Some(current_admin) = anti_abuse::get_admin(&env) {
            current_admin.require_auth();
        }
        anti_abuse::set_admin(&env, new_admin);
    }

    /// Updates the rate limit configuration.
    /// Only the admin can call this.
    pub fn update_rate_limit_config(
        env: Env,
        window_size: u64,
        max_operations: u32,
        cooldown_period: u64,
    ) {
        let admin = anti_abuse::get_admin(&env).expect("Admin not set");
        admin.require_auth();

        anti_abuse::set_config(
            &env,
            anti_abuse::AntiAbuseConfig {
                window_size,
                max_operations,
                cooldown_period,
            },
        );
    }

    /// Adds or removes an address from the whitelist.
    /// Only the admin can call this.
    pub fn set_whitelist(env: Env, address: Address, whitelisted: bool) {
        let admin = anti_abuse::get_admin(&env).expect("Admin not set");
        admin.require_auth();

        anti_abuse::set_whitelist(&env, address, whitelisted);
    }

    /// Checks if an address is whitelisted.
    pub fn is_whitelisted(env: Env, address: Address) -> bool {
        anti_abuse::is_whitelisted(&env, address)
    }

    /// Gets the current rate limit configuration.
    pub fn get_rate_limit_config(env: Env) -> anti_abuse::AntiAbuseConfig {
        anti_abuse::get_config(&env)
    }

    /// Gets the current admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        anti_abuse::get_admin(&env)
    }

    // ========================================================================
    // Schedule View Functions
    // ========================================================================

    /// Retrieves a specific program release schedule.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - The program containing the schedule
    /// * `schedule_id` - The schedule ID to retrieve
    ///
    /// # Returns
    /// * `ProgramReleaseSchedule` - The schedule details
    ///
    /// # Panics
    /// * If schedule doesn't exist
    pub fn get_program_release_schedule(
        env: Env,
        program_id: String,
        schedule_id: u64,
    ) -> ProgramReleaseSchedule {
        env.storage()
            .persistent()
            .get(&DataKey::ReleaseSchedule(program_id, schedule_id))
            .unwrap_or_else(|| panic!("Schedule not found"))
    }

    /// Retrieves all release schedules for a program.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - The program to query
    ///
    /// # Returns
    /// * `Vec<ProgramReleaseSchedule>` - All schedules for the program
    pub fn get_all_prog_release_schedules(
        env: Env,
        program_id: String,
    ) -> Vec<ProgramReleaseSchedule> {
        let mut schedules = Vec::new(&env);
        let next_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextScheduleId(program_id.clone()))
            .unwrap_or(1);

        for schedule_id in 1..next_id {
            if env
                .storage()
                .persistent()
                .has(&DataKey::ReleaseSchedule(program_id.clone(), schedule_id))
            {
                let schedule: ProgramReleaseSchedule = env
                    .storage()
                    .persistent()
                    .get(&DataKey::ReleaseSchedule(program_id.clone(), schedule_id))
                    .unwrap();
                schedules.push_back(schedule);
            }
        }

        schedules
    }

    /// Retrieves pending (unreleased) schedules for a program.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - The program to query
    ///
    /// # Returns
    /// * `Vec<ProgramReleaseSchedule>` - All pending schedules
    pub fn get_pending_program_schedules(
        env: Env,
        program_id: String,
    ) -> Vec<ProgramReleaseSchedule> {
        let all_schedules = Self::get_all_prog_release_schedules(env.clone(), program_id.clone());
        let mut pending = Vec::new(&env);

        for schedule in all_schedules.iter() {
            if !schedule.released {
                pending.push_back(schedule.clone());
            }
        }

        pending
    }

    /// Retrieves due schedules (timestamp passed but not released) for a program.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - The program to query
    ///
    /// # Returns
    /// * `Vec<ProgramReleaseSchedule>` - All due but unreleased schedules
    pub fn get_due_program_schedules(env: Env, program_id: String) -> Vec<ProgramReleaseSchedule> {
        let pending = Self::get_pending_program_schedules(env.clone(), program_id.clone());
        let mut due = Vec::new(&env);
        let now = env.ledger().timestamp();

        for schedule in pending.iter() {
            if schedule.release_timestamp <= now {
                due.push_back(schedule.clone());
            }
        }

        due
    }

    /// Retrieves release history for a program.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - The program to query
    ///
    /// # Returns
    /// * `Vec<ProgramReleaseHistory>` - Complete release history
    pub fn get_program_release_history(env: Env, program_id: String) -> Vec<ProgramReleaseHistory> {
        env.storage()
            .persistent()
            .get(&DataKey::ReleaseHistory(program_id))
            .unwrap_or(vec![&env])
    }
}

/// Helper function to calculate total scheduled amount for a program.
fn get_program_total_scheduled_amount(env: &Env, program_id: &String) -> i128 {
    let next_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::NextScheduleId(program_id.clone()))
        .unwrap_or(1);

    let mut total = 0i128;
    for schedule_id in 1..next_id {
        if env
            .storage()
            .persistent()
            .has(&DataKey::ReleaseSchedule(program_id.clone(), schedule_id))
        {
            let schedule: ProgramReleaseSchedule = env
                .storage()
                .persistent()
                .get(&DataKey::ReleaseSchedule(program_id.clone(), schedule_id))
                .unwrap();
            if !schedule.released {
                total += schedule.amount;
            }
        }
    }

    total
}

/// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        token, Address, Env, String, Vec,
    };

    // Test helper to create a mock token contract
    fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::Client<'a> {
        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token_address = token_contract.address();
        token::Client::new(env, &token_address)
    }

    // ========================================================================
    // Program Registration Tests
    // ========================================================================

    fn setup_program_with_schedule(
        env: &Env,
        client: &ProgramEscrowContractClient<'static>,
        contract_id: &Address,
        authorized_key: &Address,
        _token: &Address,
        program_id: &String,
        total_amount: i128,
        winner: &Address,
        release_timestamp: u64,
    ) {
        // // Register program
        // client.register_program(program_id, token, authorized_key);

        // // Create and fund token
        // let token_client = create_token_contract(env, authorized_key);
        // let token_admin = token::StellarAssetClient::new(env, &token_client.address);
        // token_admin.mint(authorized_key, &total_amount);

        // // Lock funds for program
        // token_client.approve(authorized_key, &env.current_contract_address(), &total_amount, &1000);
        // client.lock_funds(program_id, &total_amount);

        // Create and fund token first, then register the program with the real token address
        let token_client = create_token_contract(env, authorized_key);
        let token_admin = token::StellarAssetClient::new(env, &token_client.address);
        token_admin.mint(authorized_key, &total_amount);

        // Register program using the created token contract address
        client.initialize_program(&program_id, &authorized_key, &token_client.address);

        // Transfer tokens to contract first
        token_client.transfer(&authorized_key, contract_id, &total_amount);

        // Lock funds for program (records the amount in program state)
        client.lock_program_funds(program_id, &total_amount);

        // Create release schedule
        client.create_program_release_schedule(
            &program_id,
            &total_amount,
            &release_timestamp,
            winner,
        );
    }

    #[test]
    fn test_single_program_release_schedule() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let authorized_key = Address::generate(&env);
        let winner = Address::generate(&env);
        let token = Address::generate(&env);
        let program_id = String::from_str(&env, "Hackathon2024");
        let amount = 1000_0000000;
        let release_timestamp = 1000;

        env.mock_all_auths();

        // Setup program with schedule
        setup_program_with_schedule(
            &env,
            &client,
            &contract_id,
            &authorized_key,
            &token,
            &program_id,
            amount,
            &winner,
            release_timestamp,
        );

        // Verify schedule was created
        let schedule = client.get_program_release_schedule(&program_id, &1);
        assert_eq!(schedule.schedule_id, 1);
        assert_eq!(schedule.amount, amount);
        assert_eq!(schedule.release_timestamp, release_timestamp);
        assert_eq!(schedule.recipient, winner);
        assert!(!schedule.released);

        // Check pending schedules
        let pending = client.get_pending_program_schedules(&program_id);
        assert_eq!(pending.len(), 1);

        // Event verification can be added later - focusing on core functionality
    }

    #[test]
    fn test_multiple_program_release_schedules() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let authorized_key = Address::generate(&env);
        let winner1 = Address::generate(&env);
        let winner2 = Address::generate(&env);
        let token = Address::generate(&env);
        let program_id = String::from_str(&env, "Hackathon2024");
        let amount1 = 600_0000000;
        let amount2 = 400_0000000;
        let total_amount = amount1 + amount2;

        env.mock_all_auths();

        // Register program
        client.initialize_program(&program_id, &authorized_key, &token);

        // Create and fund token
        let token_client = create_token_contract(&env, &authorized_key);
        let token_admin = token::StellarAssetClient::new(&env, &token_client.address);
        token_admin.mint(&authorized_key, &total_amount);

        // Transfer tokens to contract first
        token_client.transfer(&authorized_key, &contract_id, &total_amount);

        // Lock funds for program
        client.lock_program_funds(&program_id, &total_amount);

        // Create first release schedule
        client.create_program_release_schedule(&program_id, &amount1, &1000, &winner1);

        // Create second release schedule
        client.create_program_release_schedule(&program_id, &amount2, &2000, &winner2);

        // Verify both schedules exist
        let all_schedules = client.get_all_prog_release_schedules(&program_id);
        assert_eq!(all_schedules.len(), 2);

        // Verify schedule IDs
        let schedule1 = client.get_program_release_schedule(&program_id, &1);
        let schedule2 = client.get_program_release_schedule(&program_id, &2);
        assert_eq!(schedule1.schedule_id, 1);
        assert_eq!(schedule2.schedule_id, 2);

        // Verify amounts
        assert_eq!(schedule1.amount, amount1);
        assert_eq!(schedule2.amount, amount2);

        // Verify recipients
        assert_eq!(schedule1.recipient, winner1);
        assert_eq!(schedule2.recipient, winner2);

        // Check pending schedules
        let pending = client.get_pending_program_schedules(&program_id);
        assert_eq!(pending.len(), 2);

        // Event verification can be added later - focusing on core functionality
    }

    #[test]
    fn test_program_automatic_release_at_timestamp() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let authorized_key = Address::generate(&env);
        let winner = Address::generate(&env);
        let token = Address::generate(&env);
        let program_id = String::from_str(&env, "Hackathon2024");
        let amount = 1000_0000000;
        let release_timestamp = 1000;

        env.mock_all_auths();

        // Setup program with schedule
        setup_program_with_schedule(
            &env,
            &client,
            &contract_id,
            &authorized_key,
            &token,
            &program_id,
            amount,
            &winner,
            release_timestamp,
        );

        // Try to release before timestamp (should fail)
        env.ledger().set_timestamp(999);
        let result = client.try_release_prog_schedule_automatic(&program_id, &1);
        assert!(result.is_err());

        // Advance time to after release timestamp
        env.ledger().set_timestamp(1001);

        // Release automatically
        client.release_prog_schedule_automatic(&program_id, &1);

        // Verify schedule was released
        let schedule = client.get_program_release_schedule(&program_id, &1);
        assert!(schedule.released);
        assert_eq!(schedule.released_at, Some(1001));

        assert_eq!(schedule.released_by, Some(contract_id.clone()));

        // Check no pending schedules
        let pending = client.get_pending_program_schedules(&program_id);
        assert_eq!(pending.len(), 0);

        // Verify release history
        let history = client.get_program_release_history(&program_id);
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap().release_type, ReleaseType::Automatic);

        // Event verification can be added later - focusing on core functionality
    }

    #[test]
    fn test_program_manual_trigger_before_after_timestamp() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let authorized_key = Address::generate(&env);
        let winner = Address::generate(&env);
        let token = Address::generate(&env);
        let program_id = String::from_str(&env, "Hackathon2024");
        let amount = 1000_0000000;
        let release_timestamp = 1000;

        env.mock_all_auths();

        // Setup program with schedule
        setup_program_with_schedule(
            &env,
            &client,
            &contract_id,
            &authorized_key,
            &token,
            &program_id,
            amount,
            &winner,
            release_timestamp,
        );

        // Manually release before timestamp (authorized key can do this)
        env.ledger().set_timestamp(999);
        client.release_program_schedule_manual(&program_id, &1);

        // Verify schedule was released
        let schedule = client.get_program_release_schedule(&program_id, &1);
        assert!(schedule.released);
        assert_eq!(schedule.released_at, Some(999));
        assert_eq!(schedule.released_by, Some(authorized_key.clone()));

        // Verify release history
        let history = client.get_program_release_history(&program_id);
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap().release_type, ReleaseType::Manual);

        // Event verification can be added later - focusing on core functionality
    }

    #[test]
    fn test_verify_program_schedule_tracking_and_history() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let authorized_key = Address::generate(&env);
        let winner1 = Address::generate(&env);
        let winner2 = Address::generate(&env);
        let program_id = String::from_str(&env, "Hackathon2024");
        let amount1 = 600_0000000;
        let amount2 = 400_0000000;
        let total_amount = amount1 + amount2;

        env.mock_all_auths();

        // Create and fund token FIRST
        let token_client = create_token_contract(&env, &authorized_key);
        let token_admin = token::StellarAssetClient::new(&env, &token_client.address);
        token_admin.mint(&authorized_key, &total_amount);

        // Register program with REAL token address
        client.initialize_program(&program_id, &authorized_key, &token_client.address);

        // Transfer tokens to contract first
        token_client.transfer(&authorized_key, &contract_id, &total_amount);

        // Lock funds for program
        client.lock_program_funds(&program_id, &total_amount);

        // Create first schedule
        client.create_program_release_schedule(&program_id, &amount1, &1000, &winner1);

        // Create second schedule
        client.create_program_release_schedule(&program_id, &amount2, &2000, &winner2);

        // Release first schedule manually
        client.release_program_schedule_manual(&program_id, &1);

        // Advance time and release second schedule automatically
        env.ledger().set_timestamp(2001);
        client.release_prog_schedule_automatic(&program_id, &2);

        // Verify complete history
        let history = client.get_program_release_history(&program_id);
        assert_eq!(history.len(), 2);

        // Check first release (manual)
        let first_release = history.get(0).unwrap();
        assert_eq!(first_release.schedule_id, 1);
        assert_eq!(first_release.amount, amount1);
        assert_eq!(first_release.recipient, winner1);
        assert_eq!(first_release.release_type, ReleaseType::Manual);

        // Check second release (automatic)
        let second_release = history.get(1).unwrap();
        assert_eq!(second_release.schedule_id, 2);
        assert_eq!(second_release.amount, amount2);
        assert_eq!(second_release.recipient, winner2);
        assert_eq!(second_release.release_type, ReleaseType::Automatic);

        // Verify no pending schedules
        let pending = client.get_pending_program_schedules(&program_id);
        assert_eq!(pending.len(), 0);

        // Verify all schedules are marked as released
        let all_schedules = client.get_all_prog_release_schedules(&program_id);
        assert_eq!(all_schedules.len(), 2);
        assert!(all_schedules.get(0).unwrap().released);
        assert!(all_schedules.get(1).unwrap().released);
    }

    #[test]
    fn test_program_overlapping_schedules() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let authorized_key = Address::generate(&env);
        let winner1 = Address::generate(&env);
        let winner2 = Address::generate(&env);
        let winner3 = Address::generate(&env);
        let program_id = String::from_str(&env, "Hackathon2024");
        let amount1 = 300_0000000;
        let amount2 = 300_0000000;
        let amount3 = 400_0000000;
        let total_amount = amount1 + amount2 + amount3;
        let base_timestamp = 1000;

        env.mock_all_auths();

        // Create and fund token FIRST
        let token_client = create_token_contract(&env, &authorized_key);
        let token_admin = token::StellarAssetClient::new(&env, &token_client.address);
        token_admin.mint(&authorized_key, &total_amount);

        // Register program with REAL token address
        client.initialize_program(&program_id, &authorized_key, &token_client.address);

        // Transfer tokens to contract first
        token_client.transfer(&authorized_key, &contract_id, &total_amount);

        // Lock funds for program
        client.lock_program_funds(&program_id, &total_amount);

        // Create overlapping schedules (all at same timestamp)
        client.create_program_release_schedule(
            &program_id,
            &amount1,
            &base_timestamp,
            &winner1.clone(),
        );

        client.create_program_release_schedule(
            &program_id,
            &amount2,
            &base_timestamp,
            &winner2.clone(),
        );

        client.create_program_release_schedule(
            &program_id,
            &amount3,
            &base_timestamp,
            &winner3.clone(),
        );

        // Advance time to after release timestamp
        env.ledger().set_timestamp(base_timestamp + 1);

        // Check due schedules (should be all 3)
        let due = client.get_due_program_schedules(&program_id);
        assert_eq!(due.len(), 3);

        // Release schedules one by one
        client.release_prog_schedule_automatic(&program_id, &1);
        client.release_prog_schedule_automatic(&program_id, &2);
        client.release_prog_schedule_automatic(&program_id, &3);

        // Verify all schedules are released
        let pending = client.get_pending_program_schedules(&program_id);
        assert_eq!(pending.len(), 0);

        // Verify complete history
        let history = client.get_program_release_history(&program_id);
        assert_eq!(history.len(), 3);

        // Verify all were automatic releases
        for release in history.iter() {
            assert_eq!(release.release_type, ReleaseType::Automatic);
        }

        // Event verification can be added later - focusing on core functionality
    }

    #[test]
    fn test_register_single_program() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let backend = Address::generate(&env);
        let token = Address::generate(&env);
        let prog_id = String::from_str(&env, "Hackathon2024");

        // Register program
        let program = client.initialize_program(&prog_id, &backend, &token);

        // Verify program data
        assert_eq!(program.program_id, prog_id);
        assert_eq!(program.authorized_payout_key, backend);
        assert_eq!(program.token_address, token);
        assert_eq!(program.total_funds, 0);
        assert_eq!(program.remaining_balance, 0);
        assert_eq!(program.payout_history.len(), 0);

        // Verify it exists
        assert!(client.program_exists(&prog_id));
        assert_eq!(client.get_program_count(), 1);
    }

    #[test]
    fn test_multiple_programs_isolation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let backend1 = Address::generate(&env);
        let backend2 = Address::generate(&env);
        let backend3 = Address::generate(&env);
        let token = Address::generate(&env);

        // Register three programs
        let prog1 = String::from_str(&env, "ETHGlobal2024");
        let prog2 = String::from_str(&env, "Stellar2024");
        let prog3 = String::from_str(&env, "BuildathonQ1");

        client.initialize_program(&prog1, &backend1, &token);
        client.initialize_program(&prog2, &backend2, &token);
        client.initialize_program(&prog3, &backend3, &token);

        // Verify all exist
        assert!(client.program_exists(&prog1));
        assert!(client.program_exists(&prog2));
        assert!(client.program_exists(&prog3));
        assert_eq!(client.get_program_count(), 3);

        // Verify complete isolation
        let info1 = client.get_program_info(&prog1);
        let info2 = client.get_program_info(&prog2);
        let info3 = client.get_program_info(&prog3);

        assert_eq!(info1.program_id, prog1);
        assert_eq!(info2.program_id, prog2);
        assert_eq!(info3.program_id, prog3);

        assert_eq!(info1.authorized_payout_key, backend1);
        assert_eq!(info2.authorized_payout_key, backend2);
        assert_eq!(info3.authorized_payout_key, backend3);

        // Verify list programs
        let programs = client.list_programs();
        assert_eq!(programs.len(), 3);
    }

    #[test]
    #[should_panic(expected = "Program already exists")]
    fn test_duplicate_program_registration() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let backend = Address::generate(&env);
        let token = Address::generate(&env);
        let prog_id = String::from_str(&env, "Hackathon2024");

        // Register once - should succeed
        client.initialize_program(&prog_id, &backend, &token);

        // Register again - should panic
        client.initialize_program(&prog_id, &backend, &token);
    }

    #[test]
    #[should_panic(expected = "Program ID cannot be empty")]
    fn test_empty_program_id() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let backend = Address::generate(&env);
        let token = Address::generate(&env);
        let empty_id = String::from_str(&env, "");

        client.initialize_program(&empty_id, &backend, &token);
    }

    #[test]
    #[should_panic(expected = "Program not found")]
    fn test_get_nonexistent_program() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let prog_id = String::from_str(&env, "DoesNotExist");
        client.get_program_info(&prog_id);
    }

    // ========================================================================
    // Fund Locking Tests
    // ========================================================================

    #[test]
    fn test_lock_funds_single_program() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);
        let token_client = create_token_contract(&env, &admin);

        let backend = Address::generate(&env);
        let prog_id = String::from_str(&env, "Hackathon2024");

        // Register program
        client.initialize_program(&prog_id, &backend, &token_client.address);

        // Lock funds
        let amount = 10_000_0000000i128; // 10,000 USDC
        let updated = client.lock_program_funds(&prog_id, &amount);

        assert_eq!(updated.total_funds, amount);
        assert_eq!(updated.remaining_balance, amount);
    }

    #[test]
    fn test_lock_funds_multiple_programs_isolation() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);
        let token_client = create_token_contract(&env, &admin);

        let backend1 = Address::generate(&env);
        let backend2 = Address::generate(&env);

        let prog1 = String::from_str(&env, "Program1");
        let prog2 = String::from_str(&env, "Program2");

        // Register programs
        client.initialize_program(&prog1, &backend1, &token_client.address);
        client.initialize_program(&prog2, &backend2, &token_client.address);

        // Lock different amounts in each program
        let amount1 = 5_000_0000000i128;
        let amount2 = 10_000_0000000i128;

        client.lock_program_funds(&prog1, &amount1);
        client.lock_program_funds(&prog2, &amount2);

        // Verify isolation - funds don't mix
        let info1 = client.get_program_info(&prog1);
        let info2 = client.get_program_info(&prog2);

        assert_eq!(info1.total_funds, amount1);
        assert_eq!(info1.remaining_balance, amount1);
        assert_eq!(info2.total_funds, amount2);
        assert_eq!(info2.remaining_balance, amount2);
    }

    #[test]
    fn test_lock_funds_cumulative() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);
        let token_client = create_token_contract(&env, &admin);

        let backend = Address::generate(&env);
        let prog_id = String::from_str(&env, "Hackathon2024");

        client.initialize_program(&prog_id, &backend, &token_client.address);

        // Lock funds multiple times
        client.lock_program_funds(&prog_id, &1_000_0000000);
        client.lock_program_funds(&prog_id, &2_000_0000000);
        client.lock_program_funds(&prog_id, &3_000_0000000);

        let info = client.get_program_info(&prog_id);
        assert_eq!(info.total_funds, 6_000_0000000);
        assert_eq!(info.remaining_balance, 6_000_0000000);
    }

    #[test]
    #[should_panic(expected = "Amount must be greater than zero")]
    fn test_lock_zero_funds() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let backend = Address::generate(&env);
        let token = Address::generate(&env);
        let prog_id = String::from_str(&env, "Hackathon2024");

        client.initialize_program(&prog_id, &backend, &token);
        client.lock_program_funds(&prog_id, &0);
    }

    // ========================================================================
    // Batch Payout Tests
    // ========================================================================

    #[test]
    #[should_panic(expected = "Recipients and amounts vectors must have the same length")]
    fn test_batch_payout_mismatched_lengths() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);
        let token_client = create_token_contract(&env, &admin);

        let backend = Address::generate(&env);
        let prog_id = String::from_str(&env, "Test");

        client.initialize_program(&prog_id, &backend, &token_client.address);
        client.lock_program_funds(&prog_id, &10_000_0000000);

        let recipients = soroban_sdk::vec![&env, Address::generate(&env), Address::generate(&env)];
        let amounts = soroban_sdk::vec![&env, 1_000_0000000i128]; // Mismatch!

        client.batch_payout(&prog_id, &recipients, &amounts);
    }

    #[test]
    #[should_panic(expected = "Insufficient balance")]
    fn test_batch_payout_insufficient_balance() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);
        let token_client = create_token_contract(&env, &admin);

        let backend = Address::generate(&env);
        let prog_id = String::from_str(&env, "Test");

        client.initialize_program(&prog_id, &backend, &token_client.address);
        client.lock_program_funds(&prog_id, &5_000_0000000);

        let recipients = soroban_sdk::vec![&env, Address::generate(&env)];
        let amounts = soroban_sdk::vec![&env, 10_000_0000000i128]; // More than available!

        client.batch_payout(&prog_id, &recipients, &amounts);
    }

    #[test]
    fn test_program_count() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        assert_eq!(client.get_program_count(), 0);

        let backend = Address::generate(&env);
        let token = Address::generate(&env);

        client.initialize_program(&String::from_str(&env, "P1"), &backend, &token);
        assert_eq!(client.get_program_count(), 1);

        client.initialize_program(&String::from_str(&env, "P2"), &backend, &token);
        assert_eq!(client.get_program_count(), 2);

        client.initialize_program(&String::from_str(&env, "P3"), &backend, &token);
        assert_eq!(client.get_program_count(), 3);
    }

    // ========================================================================
    // Anti-Abuse Tests
    // ========================================================================

    #[test]
    #[should_panic(expected = "Operation in cooldown period")]
    fn test_anti_abuse_cooldown_panic() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1000);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.set_admin(&admin);
        client.update_rate_limit_config(&3600, &10, &60);

        let backend = Address::generate(&env);
        let token = Address::generate(&env);

        client.initialize_program(&String::from_str(&env, "P1"), &backend, &token);

        // Advance time by 30s (less than 60s cooldown)
        env.ledger().with_mut(|li| li.timestamp += 30);

        client.initialize_program(&String::from_str(&env, "P2"), &backend, &token);
    }

    #[test]
    #[should_panic(expected = "Rate limit exceeded")]
    fn test_anti_abuse_limit_panic() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1000);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.set_admin(&admin);
        client.update_rate_limit_config(&3600, &2, &0); // 2 ops max, no cooldown

        let backend = Address::generate(&env);
        let token = Address::generate(&env);

        client.initialize_program(&String::from_str(&env, "P1"), &backend, &token);
        client.initialize_program(&String::from_str(&env, "P2"), &backend, &token);
        client.initialize_program(&String::from_str(&env, "P3"), &backend, &token);
        // Should panic
    }

    #[test]
    fn test_anti_abuse_whitelist() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1000);
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.set_admin(&admin);
        client.update_rate_limit_config(&3600, &1, &60); // 1 op max

        let backend = Address::generate(&env);
        let token = Address::generate(&env);

        client.set_whitelist(&backend, &true);

        client.initialize_program(&String::from_str(&env, "P1"), &backend, &token);
        client.initialize_program(&String::from_str(&env, "P2"), &backend, &token);
        // Should work because whitelisted
    }

    #[test]
    fn test_anti_abuse_config_update() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.set_admin(&admin);

        client.update_rate_limit_config(&7200, &5, &120);

        let config = client.get_rate_limit_config();
        assert_eq!(config.window_size, 7200);
        assert_eq!(config.max_operations, 5);
        assert_eq!(config.cooldown_period, 120);
    }

    #[test]
    fn test_admin_rotation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let old_admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        client.set_admin(&old_admin);
        assert_eq!(client.get_admin(), Some(old_admin.clone()));

        client.set_admin(&new_admin);
        assert_eq!(client.get_admin(), Some(new_admin));
    }

    #[test]
    fn test_new_admin_can_update_config() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        let old_admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        client.set_admin(&old_admin);
        client.set_admin(&new_admin);

        client.update_rate_limit_config(&3600, &10, &30);

        let config = client.get_rate_limit_config();
        assert_eq!(config.window_size, 3600);
        assert_eq!(config.max_operations, 10);
        assert_eq!(config.cooldown_period, 30);
    }

    #[test]
    #[should_panic(expected = "Admin not set")]
    fn test_non_admin_cannot_update_config() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, ProgramEscrowContract);
        let client = ProgramEscrowContractClient::new(&env, &contract_id);

        client.update_rate_limit_config(&3600, &10, &30);
    }
}
