use near_sdk::{env, near_bindgen};

use crate::market::*;
use crate::consts::*;

#[near_bindgen]
impl Market {
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

    pub fn calc_buy_amount(&self, investment_amount: u128, outcome_idx: u64) -> u128 {
        let buy_token_pool_balance = self.conditional_tokens.get_balance_by_token_idx(&outcome_idx);
        let mut ending_outcome_balance = buy_token_pool_balance.clone();
        for i in 0 .. self.market.options {
            let balance = self.conditional_tokens.get_balance_by_token_idx(&(i as u64));

            if i as u64 != outcome_idx {
                let k = buy_token_pool_balance.wrapping_div(ONE).wrapping_mul(balance.wrapping_div(ONE));
                let new_outcome_balance = (balance + investment_amount).wrapping_div(ONE);
                ending_outcome_balance = k.wrapping_div(new_outcome_balance).wrapping_mul(ONE);
            }
        }

        buy_token_pool_balance + investment_amount - ending_outcome_balance
    }
}
