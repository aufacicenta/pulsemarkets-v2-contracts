use near_sdk::{env, near_bindgen, PromiseResult};

use crate::storage::*;

#[near_bindgen]
impl Market {
    #[private]
    pub fn on_create_proposals_callback(&mut self) {
        match env::promise_result(0) {
            PromiseResult::Successful(_res) => {}
            _ => env::panic_str("ERR_CREATE_PROPOSALS_UNSUCCESSFUL"),
        }

        self.published = true;
    }
}
