use near_sdk::{env, near_bindgen, AccountId, Balance};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn deposits_of(&self, proposal_id: &u64, payee: &AccountId) -> Balance {
        match self.deposits.get(proposal_id) {
            Some(entry) => match entry.get(payee) {
                Some(balance) => balance,
                None => 0,
            },
            None => 0,
        }
    }

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
