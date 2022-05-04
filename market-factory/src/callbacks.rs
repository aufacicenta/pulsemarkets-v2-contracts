use near_sdk::{env, near_bindgen, PromiseResult};

#[near_bindgen]
impl MarketFactory {
    #[private]
    pub fn on_create_market_callback(&mut self, market_account_id: AccountId) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                self.markets.push(market_account_id);
                true
            }
            _ => env::panic_str("ERR_CREATE_MARKET_UNSUCCESSFUL"),
        }
    }
}
