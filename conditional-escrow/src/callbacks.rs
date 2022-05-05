use near_sdk::{env, near_bindgen, PromiseResult};

use crate::storage::*;

#[near_bindgen]
impl ConditionalEscrow {
    #[private]
    pub fn on_delegate_callback(&mut self) -> bool {
        if env::promise_results_count() != 2 {
            env::panic_str("ERR_CALLBACK_METHOD");
        }

        return true;
    }
}
