use near_sdk::{env, near_bindgen, serde_json::json, AccountId, Promise, PromiseResult};

use crate::consts::*;
use crate::storage::*;

#[near_bindgen]
impl MarketFactory {
    #[private]
    pub fn on_create_market_callback(&mut self, market_account_id: AccountId) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                let create_outcome_tokens_promise = Promise::new(market_account_id.clone())
                    .function_call(
                        "create_outcome_tokens".to_string(),
                        json!({}).to_string().into_bytes(),
                        0,
                        GAS_FOR_CREATE_OUTCOME_TOKENS,
                    );

                let create_outcome_tokens_promise_callback =
                    Promise::new(env::current_account_id()).function_call(
                        "on_create_outcome_tokens_callback".to_string(),
                        json!({
                            "market_account_id": market_account_id,
                        })
                        .to_string()
                        .into_bytes(),
                        0,
                        GAS_FOR_CREATE_OUTCOME_TOKENS_CALLBACK,
                    );

                create_outcome_tokens_promise.then(create_outcome_tokens_promise_callback);

                true
            }
            // @TODO return the attached deposit to the user
            _ => false,
        }
    }

    #[private]
    pub fn on_create_outcome_tokens_callback(&mut self, market_account_id: AccountId) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                self.markets.insert(&market_account_id);

                if self.ft_storage_deposit_called {
                    return true;
                }

                let ft_storage_deposit_promise = Promise::new(market_account_id.clone())
                    .function_call(
                        "ft_storage_deposit".to_string(),
                        json!({}).to_string().into_bytes(),
                        0,
                        GAS_FOR_FT_STORAGE_DEPOSIT,
                    );

                let ft_storage_deposit_promise_callback = Promise::new(env::current_account_id())
                    .function_call(
                        "on_ft_storage_deposit_callback".to_string(),
                        json!({}).to_string().into_bytes(),
                        0,
                        GAS_FOR_FT_STORAGE_DEPOSIT_CALLBACK,
                    );

                ft_storage_deposit_promise.then(ft_storage_deposit_promise_callback);

                true
            }
            _ => false,
        }
    }

    #[private]
    pub fn on_ft_storage_deposit_callback(&mut self) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                self.ft_storage_deposit_called = true;

                true
            }
            _ => false,
        }
    }
}
