use near_sdk::{env, near_bindgen};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_price_ratio(&self, outcome_id: OutcomeId) -> PriceRatio {
        let outcome_token = self.get_outcome_token(outcome_id);
        let accounts_length = outcome_token.get_accounts_length();

        // @TODO 10 can become an arg of market initialization. Creators may play with this param
        let price_ratio = if accounts_length <= 10 {
            0.01
        } else {
            (1.0 / accounts_length as f64) / (accounts_length as f64 / 2.0)
        };

        println!(
            "GET_PRICE_RATIO accounts_length: {}, price_ratio: {}",
            accounts_length, price_ratio
        );

        price_ratio
    }

    pub fn get_boost_ratio(&self) -> WrappedBalance {
        println!(
            "GET_BOOST_RATIO ends_at: {}, block_timestamp: {}",
            self.market.ends_at,
            env::block_timestamp()
        );

        let start_at = if self.is_open() {
            self.market.ends_at
        } else {
            self.market.starts_at
        };

        start_at as f64 / env::block_timestamp() as f64
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

    pub fn is_resolution_window_expired(&self) -> bool {
        env::block_timestamp() > (self.market.ends_at + self.resolution_window)
    }
}
