#![cfg(test)]
use crate::{BountyEscrowContract, BountyEscrowContractClient};
use soroban_sdk::testutils::Events;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token, vec, Address, Env,
};

use crate::{BountyEscrowContract, BountyEscrowContractClient};

fn create_test_env() -> (Env, BountyEscrowContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyEscrowContract);
    let client = BountyEscrowContractClient::new(&env, &contract_id);

    (env, client, contract_id)
}

fn create_token_contract<'a>(
    e: &'a Env,
    admin: &Address,
) -> (Address, token::Client<'a>, token::StellarAssetClient<'a>) {
    let token_id = e.register_stellar_asset_contract_v2(admin.clone());
    let token = token_id.address();
    let token_client = token::Client::new(e, &token);
    let token_admin_client = token::StellarAssetClient::new(e, &token);
    (token, token_client, token_admin_client)
}

// Release schedule helper function commented out - functionality not implemented
/*
fn setup_bounty_with_schedule(
    env: &Env,
    client: &BountyEscrowContractClient<'static>,
    contract_id: &Address,
    admin: &Address,
    token: &Address,
    bounty_id: u64,
    amount: i128,
    contributor: &Address,
    release_timestamp: u64,
) {
    // Initialize contract
    client.init(admin, token);

    // Create and fund token
    let (_, token_client, token_admin) = create_token_contract(env, admin);
    token_admin.mint(&admin, &1000_0000000);

    // Lock funds for bounty
    token_client.approve(admin, contract_id, &amount, &1000);
    client.lock_funds(&contributor.clone(), &bounty_id, &amount, &1000000000);

    // Create release schedule
    client.create_release_schedule(
        &bounty_id,
        &amount,
        &release_timestamp,
        &contributor.clone(),
    );
}
*/

// ========================================================================
// Release Schedule Tests
// NOTE: These tests are for functionality that doesn't exist in the contract.
// Commented out until release schedule functionality is implemented.
// ========================================================================

// Release schedule tests commented out - functionality not implemented
/*
#[test]
fn test_single_release_schedule() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contributor = Address::generate(&env);

    // Create token and escrow contracts
    let (token_address, token, token_admin) = create_token_contract(&env, &admin);
    let escrow = create_escrow_contract(&env);

    // Initialize escrow
    escrow.init(&admin, &token_address);

    // Mint tokens to admin
    token_admin.mint(&admin, &1000_0000000);

    let bounty_id = 1;
    let amount = 100_0000000;
    let deadline = env.ledger().timestamp() + 1000000000;

    // Lock funds
    escrow.lock_funds(&admin, &bounty_id, &amount, &deadline);

    // Create release schedule
    let release_timestamp = 1000;
    escrow.create_release_schedule(
        &bounty_id,
        &amount,
        &release_timestamp,
        &contributor.clone(),
    );

    // Verify schedule was created
    let schedule = escrow.get_release_schedule(&bounty_id, &1);
    assert_eq!(schedule.schedule_id, 1);
    assert_eq!(schedule.amount, amount);
    assert_eq!(schedule.release_timestamp, release_timestamp);
    assert_eq!(schedule.recipient, contributor);
    assert!(!schedule.released);

    // Check pending schedules
    let pending = escrow.get_pending_schedules(&bounty_id);
    assert_eq!(pending.len(), 1);

    // Event verification can be added later - focusing on core functionality
}
*/

fn create_escrow_contract<'a>(e: &Env) -> BountyEscrowContractClient<'a> {
    let contract_id = e.register_contract(None, BountyEscrowContract);
    BountyEscrowContractClient::new(e, &contract_id)
}

/* Release schedule tests commented out - functionality not implemented
#[test]
fn test_multiple_release_schedules() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);

    // Create token and escrow contracts
    let (token_address, _token, token_admin) = create_token_contract(&env, &admin);
    let escrow = create_escrow_contract(&env);

    // Initialize escrow
    escrow.init(&admin, &token_address);

    // Mint tokens to admin
    token_admin.mint(&admin, &1000_0000000);

    let bounty_id = 1;
    let amount1 = 60_0000000;
    let amount2 = 40_0000000;
    let total_amount = amount1 + amount2;
    let deadline = env.ledger().timestamp() + 1000000000;

    // Lock funds
    escrow.lock_funds(&admin, &bounty_id, &total_amount, &deadline);

    // Create first release schedule
    escrow.create_release_schedule(&bounty_id, &amount1, &1000, &contributor1.clone());

    // Create second release schedule
    escrow.create_release_schedule(&bounty_id, &amount2, &2000, &contributor2.clone());

    // Verify both schedules exist
    let all_schedules = escrow.get_all_release_schedules(&bounty_id);
    assert_eq!(all_schedules.len(), 2);

    // Verify schedule IDs
    let schedule1 = escrow.get_release_schedule(&bounty_id, &1);
    let schedule2 = escrow.get_release_schedule(&bounty_id, &2);
    assert_eq!(schedule1.schedule_id, 1);
    assert_eq!(schedule2.schedule_id, 2);

    // Verify amounts
    assert_eq!(schedule1.amount, amount1);
    assert_eq!(schedule2.amount, amount2);

    // Verify recipients
    assert_eq!(schedule1.recipient, contributor1);
    assert_eq!(schedule2.recipient, contributor2);

    // Check pending schedules
    let pending = escrow.get_pending_schedules(&bounty_id);
    assert_eq!(pending.len(), 2);

    // Event verification can be added later - focusing on core functionality
}

}
*/

// All release schedule tests commented out - functionality not implemented
// These tests call methods that don't exist: create_release_schedule, get_release_schedule,
// get_pending_schedules, release_schedule_manual, release_schedule_automatic, etc.

#[test]
fn test_init_event() {
    let (env, client, _contract_id) = create_test_env();
    let _employee = Address::generate(&env);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let _depositor = Address::generate(&env);
    let _bounty_id = 1;

    env.mock_all_auths();

    // Initialize
    client.init(&admin.clone(), &token.clone());

    // Get all events emitted
    let events = env.events().all();

    // Verify the event was emitted (1 init event + 2 monitoring events)
    assert_eq!(events.len(), 3);
}

#[test]
fn test_lock_fund() {
    let (env, client, _contract_id) = create_test_env();
    let _employee = Address::generate(&env);

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let bounty_id = 1;
    let amount = 1000;
    let deadline = 10;

    env.mock_all_auths();

    // Setup token
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    // Initialize
    client.init(&admin.clone(), &token.clone());

    token_admin_client.mint(&depositor, &amount);

    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);

    // Get all events emitted
    let events = env.events().all();

    // Verify the event was emitted (5 original events + 4 monitoring events from init & lock_funds)
    assert_eq!(events.len(), 9);
}

#[test]
fn test_release_fund() {
    let (env, client, _contract_id) = create_test_env();

    let admin = Address::generate(&env);
    // let token = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor = Address::generate(&env);
    let bounty_id = 1;
    let amount = 1000;
    let deadline = 10;

    env.mock_all_auths();

    // Setup token
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    // Initialize
    client.init(&admin.clone(), &token.clone());

    token_admin_client.mint(&depositor, &amount);

    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);

    client.release_funds(&bounty_id, &contributor);

    // Get all events emitted
    let events = env.events().all();

    // Verify the event was emitted (7 original events + 6 monitoring events from init, lock_funds & release_funds)
    assert_eq!(events.len(), 13);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_lock_fund_invalid_amount() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let bounty_id = 1;
    let amount = 0; // Invalid amount
    let deadline = 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, _token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin.clone(), &token.clone());

    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_lock_fund_invalid_deadline() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let bounty_id = 1;
    let amount = 1000;
    let deadline = 0; // Past deadline (default timestamp is 0, so 0 <= 0)

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin.clone(), &token.clone());
    token_admin_client.mint(&depositor, &amount);

    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);
}

// ============================================================================
// Integration Tests: Batch Operations
// ============================================================================

#[test]
fn test_batch_lock_funds() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin, &token);

    // Mint tokens for batch operations
    let total_amount = 5000i128;
    token_admin_client.mint(&depositor, &total_amount);

    // Create batch lock items
    let mut items = vec![&env];
    items.push_back(crate::LockFundsItem {
        bounty_id: 1,
        depositor: depositor.clone(),
        amount: 1000,
        deadline: 100,
    });
    items.push_back(crate::LockFundsItem {
        bounty_id: 2,
        depositor: depositor.clone(),
        amount: 2000,
        deadline: 200,
    });
    items.push_back(crate::LockFundsItem {
        bounty_id: 3,
        depositor: depositor.clone(),
        amount: 2000,
        deadline: 300,
    });

    // Execute batch lock
    let locked_count = client.batch_lock_funds(&items);
    assert_eq!(locked_count, 3);

    // Verify all bounties are locked
    let escrow1 = client.get_escrow_info(&1);
    let escrow2 = client.get_escrow_info(&2);
    let escrow3 = client.get_escrow_info(&3);

    assert_eq!(escrow1.amount, 1000);
    assert_eq!(escrow2.amount, 2000);
    assert_eq!(escrow3.amount, 2000);
}

#[test]
fn test_batch_release_funds() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin, &token);

    // Lock funds for multiple bounties
    let amount1 = 1000i128;
    let amount2 = 2000i128;
    token_admin_client.mint(&depositor, &(amount1 + amount2));

    client.lock_funds(&depositor, &1, &amount1, &100);
    client.lock_funds(&depositor, &2, &amount2, &200);

    // Create batch release items
    let mut items = vec![&env];
    items.push_back(crate::ReleaseFundsItem {
        bounty_id: 1,
        contributor: contributor1.clone(),
    });
    items.push_back(crate::ReleaseFundsItem {
        bounty_id: 2,
        contributor: contributor2.clone(),
    });

    // Execute batch release
    let released_count = client.batch_release_funds(&items);
    assert_eq!(released_count, 2);

    // Verify funds were released
    let escrow1 = client.get_escrow_info(&1);
    let escrow2 = client.get_escrow_info(&2);

    assert_eq!(escrow1.status, crate::EscrowStatus::Released);
    assert_eq!(escrow2.status, crate::EscrowStatus::Released);
}

// ============================================================================
// Integration Tests: Error Propagation
// ============================================================================

#[test]
#[should_panic(expected = "Error(Contract, #12)")] // DuplicateBountyId
fn test_batch_lock_duplicate_bounty_id() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &5000);

    // Create batch with duplicate bounty IDs
    let mut items = vec![&env];
    items.push_back(crate::LockFundsItem {
        bounty_id: 1,
        depositor: depositor.clone(),
        amount: 1000,
        deadline: 100,
    });
    items.push_back(crate::LockFundsItem {
        bounty_id: 1, // Duplicate!
        depositor: depositor.clone(),
        amount: 2000,
        deadline: 200,
    });

    client.batch_lock_funds(&items);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_batch_lock_existing_bounty() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &5000);

    // Lock a bounty first
    client.lock_funds(&depositor, &1, &1000, &100);

    // Try to batch lock the same bounty
    let mut items = vec![&env];
    items.push_back(crate::LockFundsItem {
        bounty_id: 1, // Already exists!
        depositor: depositor.clone(),
        amount: 2000,
        deadline: 200,
    });

    client.batch_lock_funds(&items);
}

// ============================================================================
// Integration Tests: Event Emission
// ============================================================================

#[test]
fn test_batch_lock_event_emission() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &5000);

    let initial_event_count = env.events().all().len();

    // Create batch lock items
    let mut items = vec![&env];
    items.push_back(crate::LockFundsItem {
        bounty_id: 1,
        depositor: depositor.clone(),
        amount: 1000,
        deadline: 100,
    });
    items.push_back(crate::LockFundsItem {
        bounty_id: 2,
        depositor: depositor.clone(),
        amount: 2000,
        deadline: 200,
    });

    client.batch_lock_funds(&items);

    // Verify events were emitted (individual + batch events)
    let events = env.events().all();
    assert!(events.len() > initial_event_count);
}

#[test]
fn test_batch_release_event_emission() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &5000);

    // Lock funds
    client.lock_funds(&depositor, &1, &1000, &100);
    client.lock_funds(&depositor, &2, &2000, &200);

    let initial_event_count = env.events().all().len();

    // Create batch release items
    let mut items = vec![&env];
    items.push_back(crate::ReleaseFundsItem {
        bounty_id: 1,
        contributor: contributor1.clone(),
    });
    items.push_back(crate::ReleaseFundsItem {
        bounty_id: 2,
        contributor: contributor2.clone(),
    });

    client.batch_release_funds(&items);

    // Verify events were emitted
    let events = env.events().all();
    assert!(events.len() > initial_event_count);
}

// ============================================================================
// Integration Tests: Complete Workflow
// ============================================================================

#[test]
fn test_complete_bounty_workflow_lock_release() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    // 1. Initialize contract
    client.init(&admin, &token);

    // 2. Mint tokens to depositor
    let amount = 5000i128;
    token_admin_client.mint(&depositor, &amount);

    // 3. Lock funds
    let bounty_id = 1u64;
    let deadline = 1000u64;
    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);

    // 4. Verify funds locked
    let escrow = client.get_escrow_info(&bounty_id);
    assert_eq!(escrow.amount, amount);
    assert_eq!(escrow.status, crate::EscrowStatus::Locked);

    // 5. Verify contract balance
    let contract_balance = client.get_balance();
    assert_eq!(contract_balance, amount);

    // 6. Release funds to contributor
    client.release_funds(&bounty_id, &contributor);

    // 7. Verify funds released
    let escrow_after = client.get_escrow_info(&bounty_id);
    assert_eq!(escrow_after.status, crate::EscrowStatus::Released);

    // 8. Verify contributor received funds
    let contributor_balance = token_client.balance(&contributor);
    assert_eq!(contributor_balance, amount);
}

#[test]
fn test_complete_bounty_workflow_lock_refund() {
    let (env, client, _contract_id) = create_test_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    client.init(&admin, &token);

    let amount = 5000i128;
    token_admin_client.mint(&depositor, &amount);

    let bounty_id = 1u64;
    // Use a future deadline, then advance the ledger timestamp past it
    let current_time = env.ledger().timestamp();
    let deadline = current_time + 1_000;
    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);

    // Advance time past deadline so refund is eligible
    env.ledger().set_timestamp(deadline + 1);

    // Refund funds (deadline has already passed)
    client.refund(
        &bounty_id,
        &None::<i128>,
        &None::<Address>,
        &crate::RefundMode::Full,
    );

    // Verify funds refunded
    let escrow = client.get_escrow_info(&bounty_id);
    assert_eq!(escrow.status, crate::EscrowStatus::Refunded);

    // Verify depositor received refund
    let depositor_balance = token_client.balance(&depositor);
    assert_eq!(depositor_balance, amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // AlreadyInitialized
fn test_init_rejects_reinitialization() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    env.mock_all_auths();

    client.init(&admin, &token);
    client.init(&admin, &token);
}

#[test]
fn test_lock_funds_zero_amount_edge_case() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let bounty_id = 100;
    let amount = 0;
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &1_000);

    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);

    let escrow = client.get_escrow_info(&bounty_id);
    assert_eq!(escrow.amount, 0);
    assert_eq!(escrow.status, crate::EscrowStatus::Locked);
}

#[test]
#[should_panic] // Token transfer fails due to insufficient balance, protecting against overflows/invalid accounting.
fn test_lock_funds_insufficient_balance_rejected() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let bounty_id = 101;
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &100);

    client.lock_funds(&depositor, &bounty_id, &1_000, &deadline);
}

#[test]
fn test_refund_allows_exact_deadline_boundary() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let bounty_id = 102;
    let amount = 700;
    let now = env.ledger().timestamp();
    let deadline = now + 500;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &amount);
    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);

    env.ledger().set_timestamp(deadline);
    client.refund(&bounty_id);

    let escrow = client.get_escrow_info(&bounty_id);
    assert_eq!(escrow.status, crate::EscrowStatus::Refunded);
    assert_eq!(token_client.balance(&depositor), amount);
}

#[test]
fn test_maximum_lock_and_release_path() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor = Address::generate(&env);
    let bounty_id = 103;
    let amount = i64::MAX as i128;
    let deadline = env.ledger().timestamp() + 1_000;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &amount);
    client.lock_funds(&depositor, &bounty_id, &amount, &deadline);

    assert_eq!(token_client.balance(&client.address), amount);
    client.release_funds(&bounty_id, &contributor);
    assert_eq!(token_client.balance(&client.address), 0);
    assert_eq!(token_client.balance(&contributor), amount);
}

#[test]
fn test_integration_multi_bounty_lifecycle() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor = Address::generate(&env);
    let now = env.ledger().timestamp();

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &10_000);

    client.lock_funds(&depositor, &201, &3_000, &(now + 100));
    client.lock_funds(&depositor, &202, &2_000, &(now + 200));
    client.lock_funds(&depositor, &203, &1_000, &(now + 300));
    assert_eq!(token_client.balance(&client.address), 6_000);

    client.release_funds(&201, &contributor);
    env.ledger().set_timestamp(now + 201);
    client.refund(&202);
    assert_eq!(token_client.balance(&client.address), 1_000);

    let escrow_201 = client.get_escrow_info(&201);
    let escrow_202 = client.get_escrow_info(&202);
    let escrow_203 = client.get_escrow_info(&203);
    assert_eq!(escrow_201.status, crate::EscrowStatus::Released);
    assert_eq!(escrow_202.status, crate::EscrowStatus::Refunded);
    assert_eq!(escrow_203.status, crate::EscrowStatus::Locked);
    assert_eq!(token_client.balance(&contributor), 3_000);
}

fn next_seed(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}

#[test]
fn test_property_fuzz_lock_release_refund_invariants() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor = Address::generate(&env);
    let start = env.ledger().timestamp();

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);

    let mut seed = 7_u64;
    let mut fuzz_cases: [(u64, i128, u64); 40] = [(0, 0, 0); 40];
    let mut total_locked = 0_i128;
    for i in 0..40_u64 {
        let amount = (next_seed(&mut seed) % 900 + 100) as i128;
        let deadline = start + (next_seed(&mut seed) % 500 + 10);
        fuzz_cases[i as usize] = (2_000 + i, amount, deadline);
        total_locked += amount;
    }
    token_admin_client.mint(&depositor, &total_locked);

    // Lock deterministic fuzz cases.
    for (id, amount, deadline) in fuzz_cases.iter() {
        client.lock_funds(&depositor, id, amount, deadline);
    }

    let mut expected_locked_balance = client.get_balance();
    for i in 0..40_u64 {
        let id = 2_000 + i;
        if i % 3 == 0 {
            let info = client.get_escrow_info(&id);
            client.release_funds(&id, &contributor);
            expected_locked_balance -= info.amount;
        } else if i % 3 == 1 {
            let info = client.get_escrow_info(&id);
            env.ledger().set_timestamp(info.deadline);
            client.refund(&id);
            expected_locked_balance -= info.amount;
        }
    }

    assert_eq!(client.get_balance(), expected_locked_balance);
}

#[test]
fn test_stress_high_load_bounty_operations() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor = Address::generate(&env);
    let now = env.ledger().timestamp();

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &1_000_000);

    for i in 0..120_u64 {
        let amount = 100 + (i as i128 % 10);
        let deadline = now + 30 + i;
        client.lock_funds(&depositor, &(5_000 + i), &amount, &deadline);
    }
    assert!(client.get_balance() > 0);

    for i in 0..120_u64 {
        let id = 5_000 + i;
        if i % 2 == 0 {
            client.release_funds(&id, &contributor);
        } else {
            let info = client.get_escrow_info(&id);
            env.ledger().set_timestamp(info.deadline);
            client.refund(&id);
        }
    }

    assert_eq!(client.get_balance(), 0);
    assert!(token_client.balance(&contributor) > 0);
}

#[test]
fn test_gas_proxy_event_footprint_per_operation_is_constant() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let contributor = Address::generate(&env);
    let now = env.ledger().timestamp();

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &10_000);

    let before_lock = env.events().all().len();
    for offset in 0..20_u64 {
        let id = 8_001 + offset;
        client.lock_funds(&depositor, &id, &10, &(now + 100 + offset));
    }
    let after_locks = env.events().all().len();
    let lock_event_growth = after_locks - before_lock;
    assert!(lock_event_growth <= 20);

    let before_release = env.events().all().len();
    client.release_funds(&8_001, &contributor);
    let after_release = env.events().all().len();
    assert!(after_release - before_release <= 1);
}
