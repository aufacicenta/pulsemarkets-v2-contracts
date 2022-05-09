use near_sdk::collections::LookupMap;
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

    pub fn get_options_by_account(&self, account_id: &AccountId) -> LookupMap<u64, Balance> {
        let options = self
            .deposits_by_options_idx
            .get(account_id)
            .unwrap_or_else(|| {
                LookupMap::new(StorageKeys::SubUserOptions {
                    account_hash: env::sha256(account_id.as_bytes()),
                })
            });
        options
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
        self.published
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub fn is_market_expired(&self) -> bool {
        self.market.expiration_date < env::block_timestamp().try_into().unwrap()
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.market.expiration_date + self.market.resolution_window
            < env::block_timestamp().try_into().unwrap()
    }
}
