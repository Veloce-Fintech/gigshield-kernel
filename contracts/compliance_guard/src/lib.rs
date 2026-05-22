#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, BytesN, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardedAccount {
    pub business: Address,
    pub token: Address,
    pub authorized_signers: Vec<Address>,
    pub min_approvals: u32,
    pub auth_required: bool,
    pub clawback_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryRequest {
    pub requested_by: Address,
    pub new_key: Address,
    pub approved_by: Vec<Address>,
    pub executed: bool,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Guard(Address),
    Recovery(Address),
    Frozen(Address),
}

#[contract]
pub struct ComplianceGuard;

#[contractimpl]
impl ComplianceGuard {
    pub fn register_account(
        env: Env,
        business: Address,
        token: Address,
        authorized_signers: Vec<Address>,
        min_approvals: u32,
    ) {
        business.require_auth();
        let guard = GuardedAccount {
            business: business.clone(),
            token,
            authorized_signers,
            min_approvals,
            auth_required: true,
            clawback_enabled: true,
        };
        env.storage().instance().set(&DataKey::Guard(business.clone()), &guard);
        env.storage().instance().set(&DataKey::Frozen(business), &false);
    }

    pub fn is_frozen(env: Env, business: Address) -> bool {
        env.storage().instance().get(&DataKey::Frozen(business)).unwrap_or(false)
    }

    pub fn freeze_account(env: Env, caller: Address, business: Address) {
        caller.require_auth();
        let guard: GuardedAccount = env.storage().instance().get(&DataKey::Guard(business.clone())).unwrap();
        let is_signer = guard.authorized_signers.iter().any(|s| s == caller);
        if !is_signer && caller != guard.business {
            panic!("unauthorized: not a signer or business");
        }
        env.storage().instance().set(&DataKey::Frozen(business), &true);
    }

    pub fn unfreeze_account(env: Env, caller: Address, business: Address) {
        caller.require_auth();
        let guard: GuardedAccount = env.storage().instance().get(&DataKey::Guard(business.clone())).unwrap();
        if caller != guard.business {
            panic!("only business can unfreeze");
        }
        env.storage().instance().set(&DataKey::Frozen(business), &false);
    }

    pub fn clawback(
        env: Env,
        caller: Address,
        business: Address,
        from: Address,
        amount: i128,
    ) {
        caller.require_auth();
        let guard: GuardedAccount = env.storage().instance().get(&DataKey::Guard(business.clone())).unwrap();
        let is_signer = guard.authorized_signers.iter().any(|s| s == caller);
        if !is_signer {
            panic!("unauthorized: caller is not an authorized signer");
        }
        if !guard.clawback_enabled {
            panic!("clawback not enabled for this account");
        }
        let frozen: bool = env.storage().instance().get(&DataKey::Frozen(business.clone())).unwrap_or(false);
        if !frozen {
            panic!("account must be frozen before clawback");
        }
        let token_client = token::Client::new(&env, &guard.token);
        token_client.clawback(&from, &amount);
    }

    pub fn request_key_recovery(
        env: Env,
        caller: Address,
        business: Address,
        new_key: Address,
    ) {
        caller.require_auth();
        let guard: GuardedAccount = env.storage().instance().get(&DataKey::Guard(business.clone())).unwrap();
        let is_signer = guard.authorized_signers.iter().any(|s| s == caller);
        if !is_signer {
            panic!("only authorized signers can request recovery");
        }
        let req = RecoveryRequest {
            requested_by: caller,
            new_key,
            approved_by: Vec::new(&env),
            executed: false,
            created_at: env.ledger().timestamp(),
        };
        env.storage().instance().set(&DataKey::Recovery(business), &req);
    }

    pub fn approve_recovery(env: Env, caller: Address, business: Address) {
        caller.require_auth();
        let guard: GuardedAccount = env.storage().instance().get(&DataKey::Guard(business.clone())).unwrap();
        let is_signer = guard.authorized_signers.iter().any(|s| s == caller);
        if !is_signer {
            panic!("only authorized signers can approve recovery");
        }
        let mut req: RecoveryRequest = env.storage().instance().get(&DataKey::Recovery(business.clone())).unwrap();
        if req.executed {
            panic!("recovery already executed");
        }
        let already_approved = req.approved_by.iter().any(|a| a == caller);
        if already_approved {
            panic!("already approved");
        }
        req.approved_by.push_back(caller);
        if req.approved_by.len() >= guard.min_approvals as u32 {
            let new_signers = Vec::from_array(&env, [req.new_key.clone()]);
            let updated = GuardedAccount {
                authorized_signers: new_signers,
                ..guard
            };
            env.storage().instance().set(&DataKey::Guard(business.clone()), &updated);
            req.executed = true;
        }
        env.storage().instance().set(&DataKey::Recovery(business), &req);
    }

    pub fn set_auth_required(env: Env, caller: Address, business: Address, required: bool) {
        caller.require_auth();
        let mut guard: GuardedAccount = env.storage().instance().get(&DataKey::Guard(business.clone())).unwrap();
        if caller != guard.business {
            panic!("only business can toggle auth");
        }
        guard.auth_required = required;
        env.storage().instance().set(&DataKey::Guard(business), &guard);
    }

    pub fn get_guard(env: Env, business: Address) -> GuardedAccount {
        env.storage().instance().get(&DataKey::Guard(business)).unwrap()
    }
}

mod test;
