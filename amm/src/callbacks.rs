use near_sdk::serde_json::json;
use near_sdk::{
    env, log, near_bindgen, require, serde_json, AccountId, Balance, Promise, PromiseResult,
};

use crate::consts::{FT_TRANSFER_BOND, GAS_FT_TRANSFER};
use crate::storage::*;

#[near_bindgen]
impl Market {
    #[private]
    pub fn on_ft_balance_of_callback(
        &mut self,
        amount: WrappedBalance,
        payee: AccountId,
    ) -> String {
        require!(env::promise_results_count() == 2);

        let ft_balance_of: Balance = match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                return serde_json::from_slice(&result)
                    .expect("ERR_ON_FT_BALANCE_OF_CALLBACK_RESULT_0");
            }
            _ => env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK"),
        };

        let supply: Balance = match env::promise_result(1) {
            PromiseResult::Successful(result) => {
                return serde_json::from_slice(&result)
                    .expect("ERR_ON_FT_BALANCE_OF_CALLBACK_RESULT_1");
            }
            _ => env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK"),
        };

        let weight = ft_balance_of / supply;
        let amount_payable = amount * weight as f32;

        let ft_transfer_promise = Promise::new(self.collateral_token.id.clone()).function_call(
            "ft_transfer".to_string(),
            json!({
                "amount": amount_payable.to_string(),
                "receiver_id": payee
            })
            .to_string()
            .into_bytes(),
            FT_TRANSFER_BOND,
            GAS_FT_TRANSFER,
        );
    }

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
    pub fn on_create_proposals_callback(&mut self) {
        match env::promise_result(0) {
            PromiseResult::Successful(_res) => {}
            _ => env::panic_str("ERR_CREATE_PROPOSALS_UNSUCCESSFUL"),
        }
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
