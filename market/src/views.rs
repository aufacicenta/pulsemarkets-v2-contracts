use near_sdk::{env, near_bindgen};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_proposals(&self) -> Vec<u64> {
        self.proposals.clone()
    }

    pub fn is_published(&self) -> bool {
        self.published
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn is_market_expired(&self) -> bool {
        self.market.expiration_date < env::block_timestamp().try_into().unwrap()
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.market.expiration_date + self.market.resolution_window
            < env::block_timestamp().try_into().unwrap()
    }
}
