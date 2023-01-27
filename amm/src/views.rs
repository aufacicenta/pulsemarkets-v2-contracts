use near_sdk::{env, near_bindgen, AccountId};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_fee_ratio(&self) -> WrappedBalance {
        self.fees.fee_ratio
    }

    pub fn get_balance_boost_ratio(&self) -> WrappedBalance {
        1
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

    pub fn get_market_creator_account_id(&self) -> AccountId {
        self.management.market_creator_account_id.clone()
    }

    pub fn dao_account_id(&self) -> AccountId {
        self.management.dao_account_id.clone()
    }

    pub fn resolution_window(&self) -> Timestamp {
        self.resolution.window
    }

    pub fn resolved_at(&self) -> Timestamp {
        match self.resolution.resolved_at {
            Some(timestamp) => timestamp,
            None => env::panic_str("ERR_RESOLVED_AT"),
        }
    }

    pub fn is_resolved(&self) -> bool {
        match self.resolution.resolved_at {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_buy_sell_timestamp(&self) -> i64 {
        let diff = (self.market.ends_at - self.market.starts_at) as f64 * 0.75;

        self.market.ends_at - diff as i64
    }

    /**
     * A market is open (buys are enabled) 3/4 before the event ends
     * the reason being that users should not buy 1 minute before the outcome becomes evident
     */
    pub fn is_open(&self) -> bool {
        let limit = self.get_buy_sell_timestamp();

        self.get_block_timestamp() <= limit
    }

    pub fn is_closed(&self) -> bool {
        !self.is_open()
    }

    pub fn is_over(&self) -> bool {
        self.get_block_timestamp() > self.market.ends_at
    }

    pub fn has_begun(&self) -> bool {
        self.get_block_timestamp() > self.market.starts_at
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.get_block_timestamp() > self.resolution.window
    }

    pub fn balance_of(&self, outcome_id: OutcomeId, account_id: AccountId) -> WrappedBalance {
        self.get_outcome_token(outcome_id).get_balance(&account_id)
    }

    pub fn get_cumulative_weight(&self, amount: WrappedBalance) -> WrappedBalance {
        let mut supply = 0;

        for id in 0..self.market.options.len() {
            let outcome_token = self.get_outcome_token(id as OutcomeId);
            supply += outcome_token.total_supply();
        }

        if supply == 0 {
            return 1;
        }

        amount / supply
    }

    pub fn get_amount_mintable(&self, amount: WrappedBalance) -> (WrappedBalance, WrappedBalance) {
        let fee = (amount * self.get_fee_ratio()) / 100;
        let amount_mintable = amount - fee;

        (amount_mintable, fee)
    }

    pub fn get_amount_payable(
        &self,
        amount: WrappedBalance,
        outcome_id: OutcomeId,
        balance: WrappedBalance,
    ) -> (WrappedBalance, WrappedBalance) {
        let fees = self.collateral_token.fee_balance;

        let mut weight = self.get_cumulative_weight(amount);
        let mut amount_payable = balance * weight - fees * weight;

        if self.is_resolved() {
            let outcome_token = self.get_outcome_token(outcome_id);
            weight = amount / outcome_token.total_supply();
            amount_payable = balance * weight - fees * weight;
        }

        (weight, amount_payable)
    }
}
