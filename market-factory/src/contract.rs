use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, Promise};
use std::default::Default;

use crate::consts::*;
use crate::storage::*;

impl Default for MarketFactory {
    fn default() -> Self {
        env::panic_str("MarketFactory should be initialized before usage")
    }
}

#[near_bindgen]
impl MarketFactory {
    #[init]
    pub fn new() -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            markets: Vec::new(),
        }
    }

    #[payable]
    pub fn create_market(&mut self, description: String, market_options: Vec<String>) -> Promise {
        let name = format!("market_{}", self.markets.len() + 1);
        let market_account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .parse()
            .unwrap();

        let promise = Promise::new(market_account_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(env::attached_deposit())
            .deploy_contract(MARKET_CODE.to_vec())
            .function_call(
                "new".to_string(),
                json!({ "description": description, "market_options": market_options })
                    .to_string()
                    .into_bytes(),
                0,
                GAS_FOR_CREATE_MARKET,
            );

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_market_callback".to_string(),
            json!({}).to_string().into_bytes(),
            0,
            GAS_FOR_CREATE_MARKET_CALLBACK,
        );

        promise.then(callback)
    }
}
