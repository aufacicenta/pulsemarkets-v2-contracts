use near_sdk::{env, near_bindgen, AccountId, Balance};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn deposits_by_account(&self, payee: &AccountId, options_idx: &u64) -> Balance {
        match self.deposits_by_options_idx.get(payee) {
            Some(entry) => match entry.get(options_idx) {
                Some(balance) => balance,
                None => 0,
            },
            None => 0,
        }
    }

    pub fn deposits_by_option(&self, options_idx: &u64) -> Balance {
        match self.totals_by_options_idx.get(options_idx) {
            Some(deposit) => deposit,
            None => 0,
        }
    }

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
