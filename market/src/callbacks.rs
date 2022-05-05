use near_sdk::{env, near_bindgen, PromiseResult};

use crate::storage::*;

#[near_bindgen]
impl Market {
    #[private]
    pub fn on_create_proposal_callback(&mut self) {
        let market_options_len = self.market.options.len() as u64;

        if env::promise_results_count() != market_options_len + 1 {
            env::panic_str("ERR_CREATE_PROPOSALS_RESPONSES");
        }

        for n in 1..market_options_len + 1 {
            match env::promise_result(n) {
                PromiseResult::Successful(res) => {
                    let proposal_id: u64 = near_sdk::serde_json::from_slice(&res).unwrap();
                    self.proposals.push(proposal_id);
                    self.published = true;
                }
                _ => env::panic_str("ERR_CREATE_PROPOSALS_UNSUCCESSFUL"),
            }
        }
    }
}
