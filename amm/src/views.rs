use crate::math;
use near_sdk::{env, log, near_bindgen, AccountId};
use num_format::ToFormattedString;
use shared::OutcomeId;

use crate::{storage::*, FORMATTED_STRING_LOCALE};

#[near_bindgen]
impl Market {
    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_fee_ratio(&self) -> WrappedBalance {
        self.fees.fee_ratio
    }

    pub fn get_outcome_token(&self, outcome_id: OutcomeId) -> OutcomeToken {
        match self.outcome_tokens.get(&outcome_id) {
            Some(token) => token,
            None => env::panic_str("ERR_INVALID_OUTCOME_ID"),
        }
    }

    pub fn get_outcome_ids(&self) -> Vec<OutcomeId> {
        self.market
            .options
            .iter()
            .enumerate()
            .map(|(index, _)| index as OutcomeId)
            .collect()
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

    pub fn is_resolution_window_expired(&self) -> bool {
        self.get_block_timestamp() > self.resolution.window
    }

    pub fn balance_of(&self, outcome_id: OutcomeId, account_id: AccountId) -> WrappedBalance {
        self.get_outcome_token(outcome_id).get_balance(&account_id)
    }

    pub fn get_amount_mintable(&self, amount: WrappedBalance) -> (WrappedBalance, WrappedBalance) {
        let fee = self.calc_percentage(amount, self.get_fee_ratio());
        let amount_mintable = amount - fee;

        (amount_mintable, fee)
    }

    pub fn get_amount_payable(
        &self,
        amount: WrappedBalance,
        outcome_id: OutcomeId,
    ) -> (WrappedBalance, WrappedBalance) {
        let balance = self.collateral_token.balance - self.collateral_token.fee_balance;

        let mut weight = math::complex_div_u128(self.get_precision_decimals(), amount, balance);
        let mut amount_payable =
            math::complex_mul_u128(self.get_precision_decimals(), amount, weight);

        if self.is_resolved() {
            let outcome_token = self.get_outcome_token(outcome_id);

            if outcome_token.total_supply() <= 0 {
                env::panic_str("ERR_CANT_SELL_A_LOSING_OUTCOME");
            }

            weight = math::complex_div_u128(
                self.get_precision_decimals(),
                amount,
                outcome_token.total_supply(),
            );

            amount_payable = math::complex_mul_u128(self.get_precision_decimals(), balance, weight);

            log!(
                "get_amount_payable - RESOLVED -- selling: {}, ct_balance: {}, weight: {}, amount_payable: {}",
                amount.to_formatted_string(&FORMATTED_STRING_LOCALE),
                balance.to_formatted_string(&FORMATTED_STRING_LOCALE),
                weight.to_formatted_string(&FORMATTED_STRING_LOCALE),
                amount_payable.to_formatted_string(&FORMATTED_STRING_LOCALE)
            );
        } else {
            log!(
                "get_amount_payable - UNRESOLVED -- selling: {}, ct_balance: {}, cumulative_weight: {}, amount_payable: {}",
                amount.to_formatted_string(&FORMATTED_STRING_LOCALE),
                balance.to_formatted_string(&FORMATTED_STRING_LOCALE),
                weight.to_formatted_string(&FORMATTED_STRING_LOCALE),
                amount_payable.to_formatted_string(&FORMATTED_STRING_LOCALE)
            );
        }

        (amount_payable, weight)
    }

    pub fn get_precision_decimals(&self) -> WrappedBalance {
        let precision = format!(
            "{:0<p$}",
            10,
            p = self.collateral_token.decimals as usize + 1
        );

        precision.parse().unwrap()
    }

    pub fn calc_percentage(&self, amount: WrappedBalance, bps: WrappedBalance) -> WrappedBalance {
        math::complex_div_u128(
            self.get_precision_decimals(),
            math::complex_mul_u128(self.get_precision_decimals(), amount, bps),
            math::complex_mul_u128(1, self.get_precision_decimals(), 100),
        )
    }
}
