#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    token, vec, Address, Env, String,
};

fn setup_program(
    env: &Env,
    initial_amount: i128,
) -> (
    ProgramEscrowContractClient<'static>,
    Address,
    token::Client<'static>,
    token::StellarAssetClient<'static>,
) {
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProgramEscrowContract);
    let client = ProgramEscrowContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::Client::new(env, &token_id);
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    let program_id = String::from_str(env, "hack-2026");
    client.init_program(&program_id, &admin, &token_id);

    if initial_amount > 0 {
        token_admin_client.mint(&client.address, &initial_amount);
        client.lock_program_funds(&initial_amount);
    }

    (client, admin, token_client, token_admin_client)
}

fn next_seed(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}

#[test]
fn test_init_program_and_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProgramEscrowContract);
    let client = ProgramEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let program_id = String::from_str(&env, "hack-2026");

    let data = client.init_program(&program_id, &admin, &token_id);
    assert_eq!(data.total_funds, 0);
    assert_eq!(data.remaining_balance, 0);

    let events = env.events().all();
    assert!(events.len() >= 1);
}

#[test]
fn test_lock_program_funds_multi_step_balance() {
    let env = Env::default();
    let (client, _admin, _token, _token_admin) = setup_program(&env, 0);

    client.lock_program_funds(&10_000);
    client.lock_program_funds(&5_000);
    assert_eq!(client.get_remaining_balance(), 15_000);
    assert_eq!(client.get_program_info().total_funds, 15_000);
}

#[test]
fn test_edge_zero_initial_state() {
    let env = Env::default();
    let (client, _admin, token_client, _token_admin) = setup_program(&env, 0);

    assert_eq!(client.get_remaining_balance(), 0);
    assert_eq!(client.get_program_info().payout_history.len(), 0);
    assert_eq!(token_client.balance(&client.address), 0);
}

#[test]
fn test_edge_max_safe_lock_and_payout() {
    let env = Env::default();
    let safe_max = i64::MAX as i128;
    let (client, _admin, token_client, _token_admin) = setup_program(&env, safe_max);

    let recipient = Address::generate(&env);
    client.single_payout(&recipient, &safe_max);

    assert_eq!(client.get_remaining_balance(), 0);
    assert_eq!(token_client.balance(&recipient), safe_max);
    assert_eq!(token_client.balance(&client.address), 0);
}

#[test]
fn test_single_payout_token_transfer_integration() {
    let env = Env::default();
    let (client, _admin, token_client, _token_admin) = setup_program(&env, 100_000);

    let recipient = Address::generate(&env);
    let data = client.single_payout(&recipient, &30_000);

    assert_eq!(data.remaining_balance, 70_000);
    assert_eq!(token_client.balance(&recipient), 30_000);
    assert_eq!(token_client.balance(&client.address), 70_000);
}

#[test]
fn test_batch_payout_token_transfer_integration() {
    let env = Env::default();
    let (client, _admin, token_client, _token_admin) = setup_program(&env, 150_000);

    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);

    let recipients = vec![&env, r1.clone(), r2.clone(), r3.clone()];
    let amounts = vec![&env, 10_000, 20_000, 30_000];

    let data = client.batch_payout(&recipients, &amounts);
    assert_eq!(data.remaining_balance, 90_000);
    assert_eq!(data.payout_history.len(), 3);

    assert_eq!(token_client.balance(&r1), 10_000);
    assert_eq!(token_client.balance(&r2), 20_000);
    assert_eq!(token_client.balance(&r3), 30_000);
}

#[test]
fn test_complete_lifecycle_integration() {
    let env = Env::default();
    let (client, _admin, token_client, token_admin) = setup_program(&env, 0);

    token_admin.mint(&client.address, &300_000);
    client.lock_program_funds(&300_000);

    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);

    client.single_payout(&r1, &50_000);
    let recipients = vec![&env, r2.clone(), r3.clone()];
    let amounts = vec![&env, 70_000, 30_000];
    client.batch_payout(&recipients, &amounts);

    let info = client.get_program_info();
    assert_eq!(info.total_funds, 300_000);
    assert_eq!(info.remaining_balance, 150_000);
    assert_eq!(info.payout_history.len(), 3);
    assert_eq!(token_client.balance(&client.address), 150_000);
}

#[test]
fn test_property_fuzz_balance_invariants() {
    let env = Env::default();
    let (client, _admin, token_client, _token_admin) = setup_program(&env, 1_000_000);

    let mut seed = 123_u64;
    let mut expected_remaining = 1_000_000_i128;

    for _ in 0..40 {
        let amount = (next_seed(&mut seed) % 4_000 + 1) as i128;
        if amount > expected_remaining {
            continue;
        }

        if next_seed(&mut seed) % 2 == 0 {
            let recipient = Address::generate(&env);
            client.single_payout(&recipient, &amount);
        } else {
            let recipient1 = Address::generate(&env);
            let recipient2 = Address::generate(&env);
            let first = amount / 2;
            let second = amount - first;
            if first == 0 || second == 0 || first + second > expected_remaining {
                continue;
            }
            let recipients = vec![&env, recipient1, recipient2];
            let amounts = vec![&env, first, second];
            client.batch_payout(&recipients, &amounts);
        }

        expected_remaining -= amount;
        assert_eq!(client.get_remaining_balance(), expected_remaining);
        assert_eq!(token_client.balance(&client.address), expected_remaining);

        if expected_remaining == 0 {
            break;
        }
    }
}

#[test]
fn test_stress_high_load_many_payouts() {
    let env = Env::default();
    let (client, _admin, token_client, _token_admin) = setup_program(&env, 1_000_000);

    for _ in 0..100 {
        let recipient = Address::generate(&env);
        client.single_payout(&recipient, &3_000);
    }

    let info = client.get_program_info();
    assert_eq!(info.payout_history.len(), 100);
    assert_eq!(info.remaining_balance, 700_000);
    assert_eq!(token_client.balance(&client.address), 700_000);
}

#[test]
fn test_gas_proxy_batch_vs_single_event_efficiency() {
    let env_single = Env::default();
    let (single_client, _single_admin, _single_token, _single_token_admin) =
        setup_program(&env_single, 200_000);

    let single_before = env_single.events().all().len();
    for _ in 0..10 {
        let recipient = Address::generate(&env_single);
        single_client.single_payout(&recipient, &1_000);
    }
    let single_events = env_single.events().all().len() - single_before;

    let env_batch = Env::default();
    let (batch_client, _batch_admin, _batch_token, _batch_token_admin) =
        setup_program(&env_batch, 200_000);

    let mut recipients = vec![&env_batch];
    let mut amounts = vec![&env_batch];
    for _ in 0..10 {
        recipients.push_back(Address::generate(&env_batch));
        amounts.push_back(1_000);
    }

    let batch_before = env_batch.events().all().len();
    batch_client.batch_payout(&recipients, &amounts);
    let batch_events = env_batch.events().all().len() - batch_before;

    assert!(batch_events <= single_events);
}

// ============================================================================
// CLAIM PERIOD TESTS FOR PROGRAM ESCROW
// ============================================================================

#[test]
fn test_set_program_claim_window_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program(&env);

    // Should not panic
    contract.set_program_claim_window(&env, program_id, 7200);
}

#[test]
fn test_authorize_program_schedule_claim_creates_record() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program_with_funds(&env, 100_000_000_000);

    let recipient = Address::generate(&env);
    let release_time = env.ledger().timestamp() + 5000;

    // Create a release schedule first
    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        10_000_000_000,
        release_time,
        recipient.clone(),
    );

    // Set claim window and authorize claim
    contract.set_program_claim_window(&env, program_id.clone(), 3600);
    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);

    // Verify pending claim exists
    let claim = contract.get_program_pending_claim(&env, program_id, 1);
    assert_eq!(claim.recipient, recipient);
    assert_eq!(claim.amount, 10_000_000_000);
    assert!(!claim.claimed);
    assert!(claim.expires_at > env.ledger().timestamp());
}

#[test]
fn test_claim_program_schedule_transfers_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program_with_funds(&env, 100_000_000_000);

    let recipient = Address::generate(&env);
    let release_time = env.ledger().timestamp() + 5000;

    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        10_000_000_000,
        release_time,
        recipient.clone(),
    );

    contract.set_program_claim_window(&env, program_id.clone(), 3600);
    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);

    // Claim within window
    contract.claim_program_schedule(&env, program_id.clone(), 1);

    // Balance should be reduced
    let remaining = contract.get_remaining_balance(&env, program_id.clone());
    assert_eq!(remaining, 90_000_000_000);

    // Schedule should be marked released
    let schedule = contract.get_program_release_schedule(&env, program_id, 1);
    assert!(schedule.released);
}

#[test]
fn test_cancel_program_claim_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program_with_funds(&env, 100_000_000_000);

    let recipient = Address::generate(&env);
    let release_time = env.ledger().timestamp() + 5000;

    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        10_000_000_000,
        release_time,
        recipient.clone(),
    );

    contract.set_program_claim_window(&env, program_id.clone(), 3600);
    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);

    // Admin cancels the claim
    contract.cancel_program_claim(&env, program_id.clone(), 1);

    // Balance should be unchanged
    let remaining = contract.get_remaining_balance(&env, program_id);
    assert_eq!(remaining, 100_000_000_000);
}

#[test]
#[should_panic]
fn test_claim_program_schedule_after_window_expires() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program_with_funds(&env, 100_000_000_000);

    let recipient = Address::generate(&env);
    let release_time = env.ledger().timestamp() + 5000;

    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        10_000_000_000,
        release_time,
        recipient.clone(),
    );

    contract.set_program_claim_window(&env, program_id.clone(), 3600);
    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);

    // Advance past the claim window
    env.ledger().set_timestamp(env.ledger().timestamp() + 3601);

    // Should panic — window expired
    contract.claim_program_schedule(&env, program_id, 1);
}

#[test]
#[should_panic]
fn test_get_program_pending_claim_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, _, _, program_id) = setup_program(&env);

    // No claim was ever authorized
    contract.get_program_pending_claim(&env, program_id, 99);
}

#[test]
#[should_panic]
fn test_cancel_program_claim_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, _, _, program_id) = setup_program(&env);

    // Nothing to cancel
    contract.cancel_program_claim(&env, program_id, 1);
}

#[test]
#[should_panic]
fn test_claim_program_schedule_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program_with_funds(&env, 100_000_000_000);

    let recipient = Address::generate(&env);
    let release_time = env.ledger().timestamp() + 5000;

    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        10_000_000_000,
        release_time,
        recipient.clone(),
    );

    contract.set_program_claim_window(&env, program_id.clone(), 3600);
    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);

    contract.claim_program_schedule(&env, program_id.clone(), 1);
    // Second claim should panic
    contract.claim_program_schedule(&env, program_id, 1);
}

#[test]
fn test_cancel_then_reauthorize_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program_with_funds(&env, 100_000_000_000);

    let recipient = Address::generate(&env);
    let new_recipient = Address::generate(&env);
    let release_time = env.ledger().timestamp() + 5000;

    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        10_000_000_000,
        release_time,
        recipient.clone(),
    );

    contract.set_program_claim_window(&env, program_id.clone(), 3600);
    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);

    // Cancel original claim
    contract.cancel_program_claim(&env, program_id.clone(), 1);

    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);

    let claim = contract.get_program_pending_claim(&env, program_id, 1);
    assert!(!claim.claimed);
    assert_eq!(claim.amount, 10_000_000_000);
}

#[test]
fn test_claim_does_not_affect_other_schedules() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract, admin, token, program_id) = setup_program_with_funds(&env, 100_000_000_000);

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let release_time = env.ledger().timestamp() + 5000;

    // Create two schedules
    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        10_000_000_000,
        release_time,
        recipient1.clone(),
    );
    contract.create_program_release_schedule(
        &env,
        program_id.clone(),
        20_000_000_000,
        release_time,
        recipient2.clone(),
    );

    contract.set_program_claim_window(&env, program_id.clone(), 3600);

    // Only authorize and claim schedule 1
    contract.authorize_program_schedule_claim(&env, program_id.clone(), 1);
    contract.claim_program_schedule(&env, program_id.clone(), 1);

    // Schedule 2 should be untouched
    let schedule2 = contract.get_program_release_schedule(&env, program_id.clone(), 2);
    assert!(!schedule2.released);
    assert_eq!(schedule2.amount, 20_000_000_000);

    // Remaining balance should only be reduced by schedule 1's amount
    let remaining = contract.get_remaining_balance(&env, program_id);
    assert_eq!(remaining, 90_000_000_000);
}