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

        self.published_at = Some(self.get_block_timestamp());
        // add 3 days after published_at
        self.resolution_window = Some(self.get_block_timestamp() + 259200 * 1_000_000_000);
        // add 30 days after resolution_window
        self.fees.claiming_window = Some(self.resolution_window() + 2592000 * 1_000_000_000);

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
}
