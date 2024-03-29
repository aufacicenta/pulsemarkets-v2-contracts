use near_sdk::{env, near_bindgen};
use shared::OutcomeId;

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn assert_is_not_resolved(&self) {
        if self.is_resolved() {
            env::panic_str("ERR_MARKET_RESOLVED");
        }
    }

    pub fn assert_is_resolved(&self) {
        if !self.is_resolved() {
            env::panic_str("ERR_MARKET_NOT_RESOLVED");
        }
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

    pub fn assert_is_claiming_window_open(&self) {
        if self.is_claiming_window_expired() {
            env::panic_str("ERR_CLAIMING_WINDOW_EXPIRED");
        }
    }

    pub fn assert_is_not_under_resolution(&self) {
        if self.is_over() && !self.is_resolved() && !self.is_resolution_window_expired() {
            env::panic_str("ERR_MARKET_IS_UNDER_RESOLUTION");
        }
    }

    pub fn assert_only_owner(&self, ix: Ix) {
        if self.resolution.ix.address != ix.address {
            env::panic_str("ERR_SIGNER_IS_NOT_OWNER");
        }
    }

    pub fn assert_is_valid_outcome(&self, outcome_id: OutcomeId) {
        self.get_outcome_token(outcome_id);
    }
}
