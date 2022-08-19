use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen, AccountId, Promise, PromiseResult};

use crate::consts::*;
use crate::storage::*;

#[near_bindgen]
impl Market {
    #[private]
    pub fn on_ft_transfer_callback(
        &mut self,
        amount: WrappedBalance,
        payee: AccountId,
        outcome_id: OutcomeId,
        amount_payable: WrappedBalance,
    ) -> String {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                log!("on_ft_transfer_callback.amount_payable: {}", amount_payable);

                let mut outcome_token = self.get_outcome_token(outcome_id);
                outcome_token.burn(&payee, amount);

                self.update_ct_balance(-amount_payable);
                self.outcome_tokens.insert(&outcome_id, &outcome_token);

                if !self.is_over() {
                    self.update_prices(outcome_id, SetPriceOptions::Decrease);
                }

                return amount_payable.to_string();
            }
            _ => env::panic_str("ERR_ON_FT_TRANSFER_CALLBACK"),
        }
    }

    #[private]
    pub fn on_storage_deposit_callback(&mut self) {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                log!("on_storage_deposit_callback: success");
            }
            _ => env::panic_str("ERR_ON_STORAGE_DEPOSIT_CALLBACK"),
        }
    }

    #[private]
    pub fn on_create_proposals_callback(&mut self) -> Promise {
        match env::promise_result(0) {
            PromiseResult::Successful(_res) => {}
            _ => env::panic_str("ERR_CREATE_PROPOSALS_UNSUCCESSFUL"),
        }

        for outcome_id in 0 .. self.market.options.len() {
            self.create_outcome_token(outcome_id as u64);
        }

        self.assert_price_constant();
        self.published_at = Some(self.get_block_timestamp());
        self.market_publisher_account_id = Some(env::signer_account_id());

        let storage_deposit_promise = Promise::new(self.collateral_token.id.clone()).function_call(
            "storage_deposit".to_string(),
            json!({ "account_id": env::current_account_id() })
                .to_string()
                .into_bytes(),
            STORAGE_DEPOSIT_BOND,
            GAS_STORAGE_DEPOSIT,
        );

        let storage_deposit_callback_promise = Promise::new(env::current_account_id())
            .function_call(
                "on_storage_deposit_callback".to_string(),
                json!({}).to_string().into_bytes(),
                0,
                GAS_STORAGE_DEPOSIT_CALLBACK,
            );

        storage_deposit_promise.then(storage_deposit_callback_promise)
    }

    /**
     * Make sure that the market option proposal was created
     * Then create an NEP141 token, MOT
     */
    #[private]
    pub fn on_create_proposal_callback(&mut self, _market_options_idx: u64) {
        match env::promise_result(0) {
            PromiseResult::Successful(_res) => {}
            _ => env::panic_str("ERR_CREATE_PROPOSAL_UNSUCCESSFUL"),
        }
    }

    fn create_outcome_token(&mut self, outcome_id: OutcomeId) {
        let price = self.get_initial_outcome_token_price();
        let outcome_token = OutcomeToken::new(outcome_id, 0.0, price);
        self.outcome_tokens.insert(&outcome_id, &outcome_token);
    }

    fn get_initial_outcome_token_price(&self) -> Price {
        1 as Price / self.market.options.len() as Price
    }
}
