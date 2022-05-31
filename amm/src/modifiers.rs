use near_sdk::{env, near_bindgen};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn assert_is_published(&self) {
        if !self.is_published() {
            env::panic_str("ERR_MARKET_NOT_PUBLISHED");
        }
    }

    pub fn assert_is_resolved(&self) {
        if !self.is_resolved() {
            env::panic_str("ERR_MARKET_NOT_RESOLVED");
        }
    }

    pub fn assert_is_closed(&self) {
        if self.is_open() {
            env::panic_str("ERR_MARKET_IS_OPEN");
        }
    }

    pub fn assert_aggregated_price_is_1(&self) {
        let mut k: Price = 0.0;

        for id in 0..self.market.options.len() {
            self.assert_is_valid_outcome(id as OutcomeId);

            match self.outcome_tokens.get(&(id as OutcomeId)) {
                Some(token) => {
                    k += token.get_price();
                }
                None => {}
            }
        }

        assert_eq!(k, 1.0, "ERR_PRICE_CONSTANT_SHOULD_EQ_1");
    }

    pub fn assert_is_open(&self) {
        if !self.is_open() {
            env::panic_str("ERR_MARKET_IS_CLOSED");
        }
    }

    pub fn assert_is_resolution_window_open(&self) {
        if self.is_resolution_window_expired() {
            env::panic_str("ERR_RESOLUTION_WINDOW_EXPIRED");
        }
    }

    pub fn assert_is_pending(&self) {
        if !self.is_pending() {
            env::panic_str("ERR_MARKET_ALREADY_PUBLISHED");
        }
    }

    pub fn assert_only_owner(&self) {
        if self.dao_account_id != env::signer_account_id() {
            env::panic_str("ERR_SIGNER_IS_NOT_OWNER");
        }
    }

    pub fn assert_is_valid_outcome(&self, outcome_id: OutcomeId) {
        match self.outcome_tokens.get(&outcome_id) {
            Some(_) => {}
            None => env::panic_str("ERR_INVALID_OUTCOME_ID"),
        }
    }
}
