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

    pub fn get_collateral_token_metadata(&self) -> CollateralToken {
        self.collateral_token.clone()
    }

    pub fn dao_account_id(&self) -> AccountId {
        self.dao_account_id.clone()
    }

    pub fn published_at(&self) -> Timestamp {
        match self.published_at {
            Some(timestamp) => timestamp,
            None => env::panic_str("ERR_PUBLISHED_AT"),
        }
    }

    pub fn resolution_window(&self) -> Timestamp {
        self.resolution_window
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

    pub fn get_cumulative_weight(&self, amount: WrappedBalance) -> WrappedBalance {
        let mut supply = 0.0;

        for id in 0..self.market.options.len() {
            let outcome_token = self.get_outcome_token(id as OutcomeId);
            supply += outcome_token.total_supply();
        }

        amount / supply
    }

    pub fn get_amount_mintable(
        &self,
        amount: WrappedBalance,
        outcome_id: OutcomeId,
    ) -> (
        Price,
        WrappedBalance,
        WrappedBalance,
        WrappedBalance,
        WrappedBalance,
    ) {
        let outcome_token = self.get_outcome_token(outcome_id);

        let price = outcome_token.get_price();
        let fee = amount * self.get_fee_ratio();
        let exchange_rate = (amount - fee) * (1.0 - price);
        let balance_boost = self.get_balance_boost_ratio();
        let amount_mintable = exchange_rate * balance_boost;

        (price, fee, exchange_rate, balance_boost, amount_mintable)
    }

    pub fn get_amount_payable(
        &self,
        amount: WrappedBalance,
        outcome_id: OutcomeId,
    ) -> (WrappedBalance, WrappedBalance) {
        let mut weight = self.get_cumulative_weight(amount);
        let mut amount_payable = (self.collateral_token.balance * weight).floor();

        if self.is_resolved() {
            let outcome_token = self.get_outcome_token(outcome_id);
            weight = amount / outcome_token.total_supply();
            amount_payable = (self.collateral_token.balance * weight).floor();
        }

        (weight, amount_payable)
    }
}
