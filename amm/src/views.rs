use near_sdk::{env, near_bindgen};

use crate::consts::*;
use crate::market::*;
use crate::math;

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
        if outcome_idx >= self.market.options as u64 {
            env::panic_str("ERR_OUTCOME_IDX");
        }

        let buy_token_pool_balance = self.conditional_tokens.get_balance_by_account(&outcome_idx, &env::current_account_id());
        let mut ending_outcome_balance = buy_token_pool_balance.clone();
        //@TODO Calc fees
        
        for i in 0 .. self.market.options {
            let pool_balance = self.conditional_tokens.get_balance_by_account(&(i as u64), &env::current_account_id());

            if i as u64 != outcome_idx {
                let k = math::complex_mul_u128(ONE, ending_outcome_balance, pool_balance);
                ending_outcome_balance = math::complex_div_u128(ONE, k, pool_balance + investment_amount);
            }
        }

        buy_token_pool_balance + investment_amount - ending_outcome_balance
    }

    pub fn calc_sell_amount(&self, retire_amount: u128, outcome_idx: u64) -> u128 {
        if outcome_idx >= self.market.options as u64 {
            env::panic_str("ERR_OUTCOME_IDX");
        }
        
        let sell_token_pool_balance = self.conditional_tokens.get_balance_by_account(&outcome_idx, &env::current_account_id());
        let mut ending_outcome_balance = sell_token_pool_balance.clone();
        //@TODO Calc fees

        for i in 0 .. self.market.options {
            let pool_balance = self.conditional_tokens.get_balance_by_account(&(i as u64), &env::current_account_id());

            if i as u64 != outcome_idx {
                let k = math::complex_mul_u128(ONE, ending_outcome_balance, pool_balance);

                if retire_amount > pool_balance {
                    env::panic_str("ERR_NON_ZERO_POOLS");
                }

                ending_outcome_balance = math::complex_div_u128(ONE, k, pool_balance - retire_amount);
            }
        }

        if ending_outcome_balance <= 0 {
            env::panic_str("ERR_NON_ZERO_BALANCES");
        }

        retire_amount + ending_outcome_balance - sell_token_pool_balance
    }

    pub fn calc_outcome_price(&self, outcome_idx: u64) -> u128 {
        if outcome_idx >= self.market.options as u64 {
            env::panic_str("ERR_OUTCOME_IDX");
        }

        let mut total_pools_balance = 0;
        let mut outcome_balance = 0;
        for i in 0 .. self.market.options {
            let pool_balance = self.conditional_tokens.get_balance_by_account(&(i as u64), &env::current_account_id());

            total_pools_balance += pool_balance;
            if i as u64 != outcome_idx {
                outcome_balance += pool_balance;
            }
        }

        //@TODO Check decimals
        math::complex_div_u128(ONE, outcome_balance, total_pools_balance)
    }
}
