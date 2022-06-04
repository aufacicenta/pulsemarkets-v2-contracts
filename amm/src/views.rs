use near_sdk::{env, near_bindgen};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_fee_ratio(&self) -> WrappedBalance {
        self.fee_ratio
    }

    pub fn get_price_ratio(&self, outcome_id: OutcomeId) -> PriceRatio {
        let outcome_token = self.get_outcome_token(outcome_id);
        let accounts_length = outcome_token.get_accounts_length() + 1;

        let price_ratio = (1.0 - (1.0 / accounts_length as PriceRatio)) / 100.0;

        println!(
            "GET_PRICE_RATIO accounts_length: {}, price_ratio: {}\n",
            accounts_length, price_ratio
        );

        price_ratio
    }

    pub fn get_balance_boost_ratio(&self) -> WrappedBalance {
        println!(
            "GET_BALANCE_BOOST_RATIO from_timestamp: {}, block_timestamp: {}",
            self.market.ends_at,
            env::block_timestamp()
        );

        1.0 + (self.market.ends_at - env::block_timestamp()) as WrappedBalance / 1000.0
    }

    pub fn get_outcome_token(&self, outcome_id: OutcomeId) -> OutcomeToken {
        match self.outcome_tokens.get(&outcome_id) {
            Some(token) => token,
            None => env::panic_str("ERR_INVALID_OUTCOME_ID"),
        }
    }

    pub fn is_published(&self) -> bool {
        match self.published_at {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_resolved(&self) -> bool {
        match self.resolved_at {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_open(&self) -> bool {
        self.market.starts_at < env::block_timestamp()
            && self.market.ends_at >= env::block_timestamp()
    }

    pub fn is_over(&self) -> bool {
        env::block_timestamp() > self.market.ends_at
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        env::block_timestamp() > (self.market.ends_at + self.resolution_window)
    }
}
