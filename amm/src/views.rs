use near_sdk::{env, near_bindgen};

use crate::market::*;

#[near_bindgen]
impl Market {
    /*pub fn deposits_by_account(&self, payee: &AccountId, options_idx: &u64) -> Balance {
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
    }*/

    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_status(&self) -> MarketStatus {
        self.status.clone()
    }

    pub fn is_market_expired(&self) -> bool {
        self.market.expiration_date < env::block_timestamp().try_into().unwrap()
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.market.expiration_date + self.market.resolution_window
            < env::block_timestamp().try_into().unwrap()
    }
}
