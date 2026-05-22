#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, vec, Env};

#[test]
fn test_register_and_freeze() {
    let env = Env::default();
    env.mock_all_auths();

    let business = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let token = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone()];

    let contract_id = env.register(ComplianceGuard, (&));
    let client = ComplianceGuardClient::new(&env, &contract_id);

    client.register_account(&business, &token, &signers, &2u32);
    assert!(!client.is_frozen(&business));

    client.freeze_account(&signer1, &business);
    assert!(client.is_frozen(&business));
}

#[test]
fn test_key_recovery() {
    let env = Env::default();
    env.mock_all_auths();

    let business = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let new_signer = Address::generate(&env);
    let token = Address::generate(&env);
    let signers = vec![&env, signer1.clone(), signer2.clone()];

    let contract_id = env.register(ComplianceGuard, (&));
    let client = ComplianceGuardClient::new(&env, &contract_id);

    client.register_account(&business, &token, &signers, &2u32);
    client.request_key_recovery(&signer1, &business, &new_signer);
    client.approve_recovery(&signer2, &business);

    let guard = client.get_guard(&business);
    assert_eq!(guard.authorized_signers.len(), 1);
    assert_eq!(guard.authorized_signers.get(0).unwrap(), new_signer);
}
