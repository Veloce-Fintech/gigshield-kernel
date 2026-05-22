#![cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, vec, Env};

#[test]
fn test_full_escrow_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let client = Address::generate(&env);
    let freelancer = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register(EscrowVault, (&client, &freelancer, &arbitrator, &token, &1000i128));
    let escrow_client = EscrowVaultClient::new(&env, &contract_id);

    escrow_client.add_milestone(&client, &symbol_short!("setup"), &500i128, &1000u64);
    escrow_client.add_milestone(&client, &symbol_short!("deploy"), &500i128, &2000u64);

    escrow_client.complete_milestone(&freelancer, &0u32);
    escrow_client.approve_milestone(&client, &0u32);

    let milestones = escrow_client.get_milestones();
    assert_eq!(milestones.len(), 2);
    assert!(milestones.get(0).unwrap().approved);
}

#[test]
fn test_dispute_and_resolve() {
    let env = Env::default();
    env.mock_all_auths();

    let client = Address::generate(&env);
    let freelancer = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register(EscrowVault, (&client, &freelancer, &arbitrator, &token, &1000i128));
    let ec = EscrowVaultClient::new(&env, &contract_id);

    ec.add_milestone(&client, &symbol_short!("phase1"), &1000i128, &5000u64);
    ec.raise_dispute(&freelancer, &symbol_short!("quality"), &500i128);
    ec.resolve_dispute(&arbitrator, &300i128, &700i128);

    let escrow = ec.get_escrow();
    assert_eq!(escrow.released, 1000);
}
