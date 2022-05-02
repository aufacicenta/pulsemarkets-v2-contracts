use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, Balance, Gas, Promise, PromiseResult};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::default::Default;

const GAS_FOR_CREATE_DAO_PROPOSAL: Gas = Gas(90_000_000_000_000);
const GAS_FOR_CREATE_DAO_PROPOSAL_CALLBACK: Gas = Gas(2_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize)]
struct CreateMarketArgs {
    market_options: Vector<String>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MarketFactory {
    markets: LookupMap<String, Vec<String>>,
    dao_account_id: AccountId,
}

impl Default for MarketFactory {
    fn default() -> Self {
        env::panic_str("MarketFactory should be initialized before usage")
    }
}

#[near_bindgen]
impl MarketFactory {
    #[init]
    pub fn new(dao_account_id: AccountId) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            markets: LookupMap::new(b"s".to_vec()),
            dao_account_id,
        }
    }

    pub fn create_market(&mut self, market_options: Vec<String>) -> String {
        let market_id = self.get_random_market_id();

        self.markets.insert(&market_id, &market_options);
        self.process_market_options(&market_id, market_options);

        return market_id;
    }

    fn process_market_options(&self, market_id: &String, market_options: Vec<String>) -> bool {
        for market_option in market_options {
            self.create_proposal(&market_id, market_option);
        }

        return true;
    }

    fn create_proposal(&self, market_id: &String, market_option: String) -> bool {
        let dao_proposal_promise = Promise::new(self.dao_account_id.clone()).function_call(
            "create_proposal".to_string(),
            json!({ "title": market_option }).to_string().into_bytes(),
            0,
            GAS_FOR_CREATE_DAO_PROPOSAL,
        );

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_proposal_callback".to_string(),
            json!({}).to_string().into_bytes(),
            0,
            GAS_FOR_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        dao_proposal_promise.then(callback);

        return true;
    }

    fn get_random_market_id(&self) -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        return rand_string;
    }

    #[private]
    pub fn on_create_proposal_callback(&mut self) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let res: bool = near_sdk::serde_json::from_slice(&result).unwrap();
                return res;
            }
            _ => env::panic_str("ERR_CREATE_DAO_PROPOSAL_UNSUCCESSFUL"),
        }
    }
}
