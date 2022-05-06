use near_sdk::{env, near_bindgen, AccountId, PromiseResult};

use crate::storage::*;

#[near_bindgen]
impl MarketFactory {
    #[private]
    pub fn on_create_market_callback(&mut self, market_account_id: AccountId) -> bool {
        let create_response: bool = match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                self.markets.push(market_account_id);
                true
            }
            _ => false,
        };

        let publish_response: bool = match env::promise_result(1) {
            PromiseResult::Successful(_result) => true,
            _ => false,
        };

        create_response && publish_response
    }
}
