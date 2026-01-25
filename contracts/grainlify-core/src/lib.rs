#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[contract]
pub struct GrainlifyContract;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Version,
}

const VERSION: u32 = 1;

#[contractimpl]
impl GrainlifyContract {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &VERSION);
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn get_version(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Version).unwrap_or(0)
    }
    
    // Helper to update version number after code upgrade, if needed.
    // In a real scenario, the new WASM would likely have a new VERSION constant 
    // and a migration function that updates the stored version.
    pub fn set_version(env: Env, new_version: u32) {
         let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
         admin.require_auth();
         env.storage().instance().set(&DataKey::Version, &new_version);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, Address};
    
    #[test]
    fn test_init_and_get_version() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GrainlifyContract {});
        let client = GrainlifyContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        client.init(&admin);
        
        let version = client.get_version();
        assert_eq!(version, VERSION);
    }
    
    #[test]
    fn test_set_version() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GrainlifyContract {});
        let client = GrainlifyContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        client.init(&admin);
        
        let new_version = 2;
        client.set_version(&new_version);
        
        let version = client.get_version();
        assert_eq!(version, new_version);
    }
    
    #[test]
    #[should_panic(expected = "Already initialized")]
    fn test_double_init_should_panic() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GrainlifyContract {});
        let client = GrainlifyContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        client.init(&admin);
        client.init(&admin); // This should panic
    }
}