use near_sdk::{env, near_bindgen, PromiseResult};

use crate::storage::*;

#[near_bindgen]
impl Market {
    #[private]
    pub fn on_create_proposal_callback(&mut self) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(_res) => {
                //let proposal_id: u64 = near_sdk::serde_json::from_slice(&res).unwrap();
                //self.market_options.push(proposal_id);

                return true;
            }
            _ => env::panic_str("ERR_CREATE_DAO_PROPOSAL_UNSUCCESSFUL"),
        }
    }
}
