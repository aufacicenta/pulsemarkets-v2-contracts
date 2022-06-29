use near_sdk::{
    collections::UnorderedSet, env, json_types::Base64VecU8, near_bindgen, serde_json::json,
    AccountId, Promise,
};
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
            markets: UnorderedSet::new(b"d".to_vec()),
        }
    }

    #[payable]
    pub fn create_market(&mut self, args: Base64VecU8) -> Promise {
        let name = format!("market_{}", self.markets.len() + 1);
        let market_account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .parse()
            .unwrap();

        let create_market_promise = Promise::new(market_account_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(BALANCE_CREATE_MARKET)
            .deploy_contract(MARKET_CODE.to_vec())
            .function_call("new".to_string(), args.into(), 0, GAS_FOR_CREATE_MARKET);

        let create_market_callback = Promise::new(env::current_account_id()).function_call(
            "on_create_market_callback".to_string(),
            json!({ "market_account_id": market_account_id })
                .to_string()
                .into_bytes(),
            0,
            GAS_FOR_CREATE_MARKET_CALLBACK,
        );

        create_market_promise.then(create_market_callback)
    }
}
