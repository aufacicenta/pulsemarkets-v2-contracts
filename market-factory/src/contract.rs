use near_sdk::{
    collections::UnorderedSet,
    env,
    json_types::Base64VecU8,
    near_bindgen, serde_json,
    serde_json::{json, Value},
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
            ft_storage_deposit_called: false,
        }
    }

    #[payable]
    pub fn create_market(&mut self, args: Base64VecU8) -> Promise {
        let name = format!("market_{}", self.markets.len() + 1);
        let market_account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .parse()
            .unwrap();

        let mut init_args: Value = serde_json::from_slice(&args.0.as_slice()).unwrap();

        init_args.as_object_mut().unwrap().insert(
            "market_creator_account_id".to_string(),
            Value::String(env::signer_account_id().to_string()),
        );

        let create_market_promise = Promise::new(market_account_id.clone())
            .create_account()
            .deploy_contract(MARKET_CODE.to_vec())
            .transfer(env::attached_deposit())
            .function_call(
                "new".to_string(),
                init_args.to_string().into_bytes(),
                0,
                GAS_FOR_CREATE_MARKET,
            );

        // @TODO if this promise fails, the funds (attached_deposit) are not returned to the signer

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
