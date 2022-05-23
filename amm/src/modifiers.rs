use near_sdk::{env, near_bindgen};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn assert_is_published(&self) {
        if !self.is_published() {
            env::panic_str("ERR_MARKET_NOT_PUBLISHED");
        }
    }

    pub fn assert_is_closed(&self) {
        if self.is_open() {
            env::panic_str("ERR_MARKET_IS_CLOSED");
        }
    }

    pub fn assert_is_open(&self) {
        if !self.is_open() {
            env::panic_str("ERR_MARKET_IS_CLOSED");
        }
    }

    pub fn assert_is_pending(&self) {
        if !self.is_pending() {
            env::panic_str("ERR_MARKET_ALREADY_PUBLISHED");
        }
    }

    pub fn assert_valid_outcome(&self, outcome_id: OutcomeId) {
        match self.outcome_tokens.get(&outcome_id) {
            Some(_) => {}
            None => env::panic_str("ERR_INVALID_OUTCOME_ID"),
        }
    }
}
