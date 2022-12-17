use near_sdk::{env, near_bindgen, require, serde_json::json, AccountId, PromiseResult};

use crate::consts::*;
use crate::storage::*;

#[near_bindgen]
impl MarketFactory {
    #[private]
    pub fn on_create_market_callback(
        &mut self,
        market_account_id: AccountId,
        collateral_token_account_id: AccountId,
    ) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                let create_outcome_tokens_promise = env::promise_create(
                    market_account_id.clone(),
                    "create_outcome_tokens",
                    json!({}).to_string().as_bytes(),
                    0,
                    GAS_FOR_CREATE_OUTCOME_TOKENS,
                );

                let storage_deposit_promise = env::promise_create(
                    collateral_token_account_id,
                    "storage_deposit",
                    json!({ "account_id": market_account_id, "registration_only": true })
                        .to_string()
                        .as_bytes(),
                    STORAGE_DEPOSIT_BOND,
                    GAS_FOR_FT_STORAGE_DEPOSIT,
                );

                let promises =
                    env::promise_and(&[create_outcome_tokens_promise, storage_deposit_promise]);

                let callback = env::promise_then(
                    promises,
                    env::current_account_id(),
                    "on_create_outcome_tokens_ft_storage_deposit_callback",
                    json!({ "market_account_id": market_account_id })
                        .to_string()
                        .as_bytes(),
                    0,
                    GAS_FOR_CREATE_OUTCOME_TOKENS_CALLBACK,
                );

                env::promise_return(callback);

                true
            }
            // @TODO return the attached deposit to the user
            _ => false,
        }
    }

    #[private]
    pub fn on_create_outcome_tokens_ft_storage_deposit_callback(
        &mut self,
        market_account_id: AccountId,
    ) -> bool {
        require!(env::promise_results_count() == 2);

        let are_outcome_tokens_created = match env::promise_result(0) {
            PromiseResult::Successful(_result) => true,
            _ => env::panic_str("ERR_ON_CREATE_OUTCOME_TOKENS_CALLBACK_0"),
        };

        let is_storage_deposit_success = match env::promise_result(1) {
            PromiseResult::Successful(_result) => true,
            _ => env::panic_str("ERR_ON_FT_STORAGE_DEPOSIT_CALLBACK_1"),
        };

        if !are_outcome_tokens_created || !is_storage_deposit_success {
            return false;
        }

        self.markets.insert(&market_account_id);

        true
    }
}
