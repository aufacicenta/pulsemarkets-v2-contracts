use near_sdk::{env, near_bindgen};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_status(&self) -> String {
        self.status.to_string()
    }

    pub fn get_outcome_token(&self, outcome_id: OutcomeId) -> OutcomeToken {
        match self.outcome_tokens.get(&outcome_id) {
            Some(token) => token,
            None => env::panic_str("ERR_INVALID_OUTCOME_ID"),
        }
    }

    pub fn is_published(&self) -> bool {
        matches!(self.status, MarketStatus::Published)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, MarketStatus::Pending)
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self.status, MarketStatus::Resolved)
    }

    pub fn is_open(&self) -> bool {
        self.market.starts_at < env::block_timestamp()
            && self.market.ends_at >= env::block_timestamp()
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        env::block_timestamp() > (self.market.ends_at + self.market.resolution_window)
    }
}
