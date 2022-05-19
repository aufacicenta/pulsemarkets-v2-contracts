use near_sdk::{env, near_bindgen, AccountId, Balance};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn is_published(&self) -> bool {
        matches!(self.status, MarketStatus::Published)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, MarketStatus::Pending)
    }

    pub fn is_open(&self) -> bool {
        self.market.start_datetime < env::block_timestamp().try_into().unwrap()
            && self.market.end_datetime >= env::block_timestamp().try_into().unwrap()
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self.status, MarketStatus::Resolved)
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.market.end_datetime + self.market.resolution_window
            < env::block_timestamp().try_into().unwrap()
    }
}
