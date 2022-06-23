use near_sdk::{env, log, near_bindgen, AccountId};

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

        log!(
            "GET_PRICE_RATIO accounts_length: {}, price_ratio: {}\n",
            accounts_length - 1,
            price_ratio
        );

        price_ratio
    }

    pub fn get_balance_boost_ratio(&self) -> WrappedBalance {
        let boost = self.get_block_timestamp() as f32 / self.market.ends_at as f32;

        log!(
            "GET_BALANCE_BOOST_RATIO from_timestamp: {}, block_timestamp: {}, boost: {}",
            self.market.ends_at as f32,
            self.get_block_timestamp(),
            boost,
        );

        1.0 + boost as WrappedBalance
    }

    pub fn get_outcome_token(&self, outcome_id: OutcomeId) -> OutcomeToken {
        match self.outcome_tokens.get(&outcome_id) {
            Some(token) => token,
            None => env::panic_str("ERR_INVALID_OUTCOME_ID"),
        }
    }

    pub fn get_block_timestamp(&self) -> Timestamp {
        env::block_timestamp().try_into().unwrap()
    }

    pub fn dao_account_id(&self) -> AccountId {
        self.dao_account_id.clone()
    }

    pub fn collateral_token_account_id(&self) -> AccountId {
        self.collateral_token.id.clone()
    }

    pub fn published_at(&self) -> Timestamp {
        match self.published_at {
            Some(timestamp) => timestamp,
            None => env::panic_str("ERR_PUBLISHED_AT"),
        }
    }

    pub fn resolved_at(&self) -> Timestamp {
        match self.resolved_at {
            Some(timestamp) => timestamp,
            None => env::panic_str("ERR_RESOLVED_AT"),
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
        self.market.starts_at < self.get_block_timestamp()
            && self.market.ends_at >= self.get_block_timestamp()
    }

    pub fn is_over(&self) -> bool {
        self.get_block_timestamp() > self.market.ends_at
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.get_block_timestamp() > (self.market.ends_at + self.resolution_window)
    }

    pub fn balance_of(&self, outcome_id: OutcomeId, account_id: AccountId) -> WrappedBalance {
        self.get_outcome_token(outcome_id).get_balance(&account_id)
    }

    pub fn get_ct_balance(&self) -> WrappedBalance {
        self.collateral_token.balance
    }
}
