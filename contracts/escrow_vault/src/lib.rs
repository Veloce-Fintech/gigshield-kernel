#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub description: Symbol,
    pub amount: i128,
    pub deadline: u64,
    pub completed: bool,
    pub approved: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub client: Address,
    pub freelancer: Address,
    pub arbitrator: Address,
    pub token: Address,
    pub total_amount: i128,
    pub released: i128,
    pub disputed: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Arbitration {
    pub raised_by: Address,
    pub reason: Symbol,
    pub proposed_split: i128,
    pub resolved: bool,
}

#[contracttype]
pub enum DataKey {
    Escrow,
    Milestones,
    Arbitration,
    Closed,
}

#[contract]
pub struct EscrowVault;

#[contractimpl]
impl EscrowVault {
    pub fn initialize(
        env: Env,
        client: Address,
        freelancer: Address,
        arbitrator: Address,
        token: Address,
        total_amount: i128,
    ) {
        if env.storage().instance().has(&DataKey::Escrow) {
            panic!("already initialized");
        }
        let escrow = Escrow {
            client,
            freelancer,
            arbitrator,
            token,
            total_amount,
            released: 0,
            disputed: false,
            created_at: env.ledger().timestamp(),
        };
        env.storage().instance().set(&DataKey::Escrow, &escrow);
    }

    pub fn add_milestone(env: Env, caller: Address, description: Symbol, amount: i128, deadline: u64) {
        caller.require_auth();
        let escrow: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        if caller != escrow.client {
            panic!("only client can add milestones");
        }
        let milestone = Milestone {
            description,
            amount,
            deadline,
            completed: false,
            approved: false,
        };
        let mut milestones: Vec<Milestone> = env.storage().instance().get(&DataKey::Milestones).unwrap_or(Vec::new(&env));
        milestones.push_back(milestone);
        env.storage().instance().set(&DataKey::Milestones, &milestones);
    }

    pub fn complete_milestone(env: Env, caller: Address, index: u32) {
        caller.require_auth();
        let escrow: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        if caller != escrow.freelancer {
            panic!("only freelancer can mark complete");
        }
        let mut milestones: Vec<Milestone> = env.storage().instance().get(&DataKey::Milestones).unwrap();
        if let Some(mut ms) = milestones.get(index) {
            if ms.completed {
                panic!("milestone already completed");
            }
            ms.completed = true;
            milestones.set(index, ms);
            env.storage().instance().set(&DataKey::Milestones, &milestones);
        }
    }

    pub fn approve_milestone(env: Env, caller: Address, index: u32) {
        caller.require_auth();
        let escrow: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        if caller != escrow.client {
            panic!("only client can approve");
        }
        let mut milestones: Vec<Milestone> = env.storage().instance().get(&DataKey::Milestones).unwrap();
        if let Some(mut ms) = milestones.get(index) {
            if !ms.completed {
                panic!("milestone not completed");
            }
            if ms.approved {
                panic!("milestone already approved");
            }
            ms.approved = true;
            let amount = ms.amount;
            milestones.set(index, ms);
            env.storage().instance().set(&DataKey::Milestones, &milestones);

            let token_client = token::Client::new(&env, &escrow.token);
            token_client.transfer(&escrow.client, &escrow.freelancer, &amount);

            let mut escrow_current: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
            escrow_current.released += amount;
            env.storage().instance().set(&DataKey::Escrow, &escrow_current);
        }
    }

    pub fn raise_dispute(env: Env, caller: Address, reason: Symbol, proposed_split: i128) {
        caller.require_auth();
        let escrow: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        if caller != escrow.freelancer && caller != escrow.client {
            panic!("only client or freelancer can dispute");
        }
        let arb = Arbitration {
            raised_by: caller,
            reason,
            proposed_split,
            resolved: false,
        };
        env.storage().instance().set(&DataKey::Arbitration, &arb);
        let mut escrow_current: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        escrow_current.disputed = true;
        env.storage().instance().set(&DataKey::Escrow, &escrow_current);
    }

    pub fn resolve_dispute(env: Env, caller: Address, client_amount: i128, freelancer_amount: i128) {
        caller.require_auth();
        let escrow: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        if caller != escrow.arbitrator {
            panic!("only arbitrator can resolve");
        }
        let arb: Arbitration = env.storage().instance().get(&DataKey::Arbitration).unwrap();
        if arb.resolved {
            panic!("dispute already resolved");
        }
        let token_client = token::Client::new(&env, &escrow.token);
        if client_amount > 0 {
            token_client.transfer(&escrow.client, &escrow.client, &client_amount);
        }
        if freelancer_amount > 0 {
            token_client.transfer(&escrow.client, &escrow.freelancer, &freelancer_amount);
        }
        let mut arb_resolved = arb;
        arb_resolved.resolved = true;
        env.storage().instance().set(&DataKey::Arbitration, &arb_resolved);
        let mut escrow_current: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        escrow_current.disputed = false;
        escrow_current.released += client_amount + freelancer_amount;
        env.storage().instance().set(&DataKey::Escrow, &escrow_current);
    }

    pub fn release_deadline_locked(env: Env, caller: Address, index: u32) {
        caller.require_auth();
        let escrow: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
        let mut milestones: Vec<Milestone> = env.storage().instance().get(&DataKey::Milestones).unwrap();
        if let Some(ms) = milestones.get(index) {
            if ms.completed && !ms.approved && env.ledger().timestamp() > ms.deadline {
                if caller != escrow.arbitrator {
                    panic!("only arbitrator can release deadline-locked funds");
                }
                let token_client = token::Client::new(&env, &escrow.token);
                token_client.transfer(&escrow.client, &escrow.freelancer, &ms.amount);
                let mut ms_released = ms;
                ms_released.approved = true;
                milestones.set(index, ms_released);
                env.storage().instance().set(&DataKey::Milestones, &milestones);
                let mut escrow_current: Escrow = env.storage().instance().get(&DataKey::Escrow).unwrap();
                escrow_current.released += ms.amount;
                env.storage().instance().set(&DataKey::Escrow, &escrow_current);
            }
        }
    }

    pub fn get_escrow(env: Env) -> Escrow {
        env.storage().instance().get(&DataKey::Escrow).unwrap()
    }

    pub fn get_milestones(env: Env) -> Vec<Milestone> {
        env.storage().instance().get(&DataKey::Milestones).unwrap_or(Vec::new(&env))
    }
}

mod test;
