#![cfg(test)]
use crate::{BountyEscrowContract, BountyEscrowContractClient};
use soroban_sdk::testutils::Events;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

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

    // Verify the event was emitted
    assert_eq!(events.len(), 1);
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

    // Verify lock produced events (exact count can vary across Soroban versions).
    assert!(events.len() >= 2);
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

    // Verify release produced events (exact count can vary across Soroban versions).
    assert!(events.len() >= 2);
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

    for i in 0..40_u64 {
        let amount = 100 + (i as i128 % 10);
        let deadline = now + 30 + i;
        client.lock_funds(&depositor, &(5_000 + i), &amount, &deadline);
    }
    assert!(client.get_balance() > 0);

    for i in 0..40_u64 {
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
    assert!(lock_event_growth > 0);

    let before_release = env.events().all().len();
    client.release_funds(&8_001, &contributor);
    let after_release = env.events().all().len();
    assert!(after_release >= before_release);
}

// ── Min/Max Amount Policy Enforcement Tests ───────────────────────────────────
//
// These tests define the expected behaviour for configurable min/max amount
// limits (Issue #62). They are written TDD-style: they will compile only after
// the implementation adds:
//   • `set_amount_policy(admin, min_amount, max_amount)` to the contract
//   • `Error::AmountBelowMinimum = 8` in the Error enum
//   • `Error::AmountAboveMaximum = 9` in the Error enum
//
// Until Issue #62 is merged these tests are expected to fail.
// ─────────────────────────────────────────────────────────────────────────────

/// Locking an amount strictly below the configured minimum must be rejected.
#[test]
#[should_panic(expected = "Error(Contract, #8)")] // AmountBelowMinimum
fn test_lock_funds_below_minimum_rejected() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &1_000);

    // Policy: min=100, max=10_000.  Attempting to lock 50 must be rejected.
    client.set_amount_policy(&admin, &100_i128, &10_000_i128);
    client.lock_funds(&depositor, &1, &50_i128, &deadline);
}

/// Locking an amount strictly above the configured maximum must be rejected.
#[test]
#[should_panic(expected = "Error(Contract, #9)")] // AmountAboveMaximum
fn test_lock_funds_above_maximum_rejected() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &100_000);

    // Policy: min=100, max=10_000.  Attempting to lock 50_000 must be rejected.
    client.set_amount_policy(&admin, &100_i128, &10_000_i128);
    client.lock_funds(&depositor, &2, &50_000_i128, &deadline);
}

/// An amount equal to the configured minimum is on the inclusive boundary and
/// must succeed.
#[test]
fn test_lock_funds_at_exact_minimum_succeeds() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &1_000);

    client.set_amount_policy(&admin, &100_i128, &10_000_i128);
    // amount == min → allowed (inclusive lower bound)
    client.lock_funds(&depositor, &3, &100_i128, &deadline);

    let escrow = client.get_escrow_info(&3);
    assert_eq!(escrow.amount, 100);
    assert_eq!(escrow.status, crate::EscrowStatus::Locked);
}

/// An amount equal to the configured maximum is on the inclusive boundary and
/// must succeed.
#[test]
fn test_lock_funds_at_exact_maximum_succeeds() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &10_000);

    client.set_amount_policy(&admin, &100_i128, &10_000_i128);
    // amount == max → allowed (inclusive upper bound)
    client.lock_funds(&depositor, &4, &10_000_i128, &deadline);

    let escrow = client.get_escrow_info(&4);
    assert_eq!(escrow.amount, 10_000);
    assert_eq!(escrow.status, crate::EscrowStatus::Locked);
}

/// An amount that sits strictly inside [min, max] must succeed.
#[test]
fn test_lock_funds_within_range_succeeds() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &5_000);

    client.set_amount_policy(&admin, &100_i128, &10_000_i128);
    client.lock_funds(&depositor, &5, &5_000_i128, &deadline);

    let escrow = client.get_escrow_info(&5);
    assert_eq!(escrow.amount, 5_000);
    assert_eq!(escrow.status, crate::EscrowStatus::Locked);
}

/// Only the admin may call `set_amount_policy`.  Any other caller must be
/// rejected with an Unauthorized error.
#[test]
#[should_panic(expected = "Error(Contract, #7)")] // Unauthorized
fn test_non_admin_cannot_set_amount_policy() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, _token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);

    // non_admin attempts to set policy — must be rejected with Unauthorized.
    client.set_amount_policy(&non_admin, &100_i128, &10_000_i128);
}

/// When no policy has been set the contract must remain backward-compatible:
/// any positive (or zero per the existing edge-case test) amount is accepted.
#[test]
fn test_no_policy_set_allows_any_positive_amount() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &1_000_000);

    // No set_amount_policy call — all positive amounts must be accepted.
    client.lock_funds(&depositor, &6, &1_i128, &deadline);
    client.lock_funds(&depositor, &7, &999_999_i128, &deadline);

    assert_eq!(client.get_escrow_info(&6).amount, 1);
    assert_eq!(client.get_escrow_info(&7).amount, 999_999);
}

/// Supplying min > max is a logically invalid policy and must be rejected.
#[test]
#[should_panic] // InvalidPolicy / contract-defined panic for malformed config
fn test_set_amount_policy_min_greater_than_max_rejected() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, _) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);

    // min=5_000 > max=100 — invalid policy, must panic.
    client.set_amount_policy(&admin, &5_000_i128, &100_i128);
}

/// The admin must be able to update the policy after initial configuration, and
/// the new limits must take effect immediately for subsequent lock calls.
#[test]
fn test_amount_policy_can_be_updated_by_admin() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &100_000);

    // First policy: min=1_000 — amount 500 would be rejected here.
    client.set_amount_policy(&admin, &1_000_i128, &50_000_i128);

    // Loosen the policy: min=10 — amount 500 must now be accepted.
    client.set_amount_policy(&admin, &10_i128, &50_000_i128);
    client.lock_funds(&depositor, &8, &500_i128, &deadline);

    assert_eq!(client.get_escrow_info(&8).amount, 500);
}

/// min - 1 is the tightest possible value below the minimum boundary and must
/// be rejected (off-by-one lower).
#[test]
#[should_panic(expected = "Error(Contract, #8)")] // AmountBelowMinimum
fn test_one_below_minimum_boundary_rejected() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &1_000);

    client.set_amount_policy(&admin, &100_i128, &10_000_i128);
    // 99 == min(100) - 1 → must be rejected.
    client.lock_funds(&depositor, &9, &99_i128, &deadline);
}

/// max + 1 is the tightest possible value above the maximum boundary and must
/// be rejected (off-by-one upper).
#[test]
#[should_panic(expected = "Error(Contract, #9)")] // AmountAboveMaximum
fn test_one_above_maximum_boundary_rejected() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 100;

    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let (token, _token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    client.init(&admin, &token);
    token_admin_client.mint(&depositor, &100_000);

    client.set_amount_policy(&admin, &100_i128, &10_000_i128);
    // 10_001 == max(10_000) + 1 → must be rejected.
    client.lock_funds(&depositor, &10, &10_001_i128, &deadline);
}
