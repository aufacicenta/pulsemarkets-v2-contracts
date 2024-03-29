use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::ext_contract;
use near_sdk::{
    env, json_types::U128, log, near_bindgen, serde_json, AccountId, Promise, PromiseResult,
};

use crate::consts::*;
use crate::storage::*;

#[ext_contract(ext_self)]
trait Callbacks {
    fn on_claim_market_creator_fees_resolved_callback(&mut self, payee: AccountId) -> String;
    fn on_ft_balance_of_market_callback(&mut self) -> Promise;
    fn on_ft_transfer_to_dao_callback(&mut self);
}

#[near_bindgen]
impl Market {
    /**
     * Lets fee payees claim their balance
     *
     * @notice only after market is resolved
     *
     * @returns WrappedBalance of fee proportion paid
     */
    pub fn claim_market_creator_fees_resolved(&mut self) {
        self.assert_is_resolved();
        self.assert_is_claiming_window_open();

        let payee = env::signer_account_id();

        if payee != self.management.market_creator_account_id {
            env::panic_str("ERR_CLAIM_MARKET_CREATOR_FEES_RESOLVED_ACCOUNT_ID_MISTMATCH");
        }

        if let Some(market_creator_fees) = &self.fees.market_creator_fees {
            match market_creator_fees.get(&payee) {
                Some(_) => {
                    env::panic_str("ERR_CLAIM_MARKET_CREATOR_FEES_RESOLVED_NO_FEES_TO_CLAIM")
                }
                None => {
                    let amount_payable = (self.collateral_token.fee_balance * 85) / 100;

                    let ft_transfer_promise = ext_ft_core::ext(self.collateral_token.id.clone())
                        .with_attached_deposit(FT_TRANSFER_BOND)
                        .with_static_gas(GAS_FT_TRANSFER)
                        .ft_transfer(payee.clone(), U128::from(amount_payable), None);

                    let ft_transfer_callback_promise = ext_self::ext(env::current_account_id())
                        .with_attached_deposit(0)
                        .with_static_gas(GAS_FT_TRANSFER_CALLBACK)
                        .on_claim_market_creator_fees_resolved_callback(payee);

                    ft_transfer_promise.then(ft_transfer_callback_promise);
                }
            };
        } else {
            env::panic_str("ERR_CLAIM_MARKET_CREATOR_FEES_RESOLVED_NOT_SET");
        }
    }

    /**
     * Sends the remaining unclaimed collateral token balance to the DAO
     *
     * @notice only if market is resolved and fees claiming window expired
     *
     * @returns Promise
     */
    #[payable]
    pub fn claim_fees_unclaimed(&mut self) -> Promise {
        if !self.is_claiming_window_expired() {
            env::panic_str("ERR_CANNOT_CLAIM_FEES_OF_RESOLVED_MARKET_BEFORE_WINDOW_EXPIRATION");
        }

        let ft_balance_of_promise = ext_ft_core::ext(self.collateral_token.id.clone())
            .with_attached_deposit(0)
            .with_static_gas(GAS_FT_BALANCE_OF)
            .ft_balance_of(env::current_account_id());

        let ft_balance_of_callback_promise = ext_self::ext(env::current_account_id())
            .with_attached_deposit(0)
            .with_static_gas(GAS_FT_BALANCE_OF_CALLBACK)
            .on_ft_balance_of_market_callback();

        ft_balance_of_promise.then(ft_balance_of_callback_promise)
    }

    #[private]
    pub fn on_claim_market_creator_fees_resolved_callback(&mut self, payee: AccountId) -> String {
        let ft_transfer_result = match env::promise_result(0) {
            PromiseResult::Successful(result) => result,
            // On error, the funds were transfered back to the sender
            _ => env::panic_str("ERR_ON_FT_TRANSFER_CALLBACK"),
        };

        let amount_payable: WrappedBalance =
            serde_json::from_slice(&ft_transfer_result).expect("ERR_ON_FT_TRANSFER");

        if let Some(market_creator_fees) = &mut self.fees.market_creator_fees {
            market_creator_fees.insert(&payee, &amount_payable.to_string());
        }

        return amount_payable.to_string();
    }

    /**
     * Sends the remaining unclaimed fees to DAO
     *
     * @notice only if market is resolved and fees claiming window expired
     *
     * @returns Promise
     */
    #[private]
    pub fn on_ft_balance_of_market_callback(&mut self) -> Promise {
        let ft_balance_of_result: U128 = match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                serde_json::from_slice(&result).expect("ERR_ON_FT_BALANCE_OF")
            }
            _ => env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK"),
        };

        if ft_balance_of_result == U128(0) {
            env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK_BALANCE_IS_0");
        }

        let ft_transfer_promise = ext_ft_core::ext(self.collateral_token.id.clone())
            .with_attached_deposit(FT_TRANSFER_BOND)
            .with_static_gas(GAS_FT_TRANSFER)
            .ft_transfer(self.dao_account_id(), ft_balance_of_result, None);

        let ft_transfer_callback_promise = ext_self::ext(env::current_account_id())
            .with_attached_deposit(0)
            .with_static_gas(GAS_FT_TRANSFER_CALLBACK)
            .on_ft_transfer_to_dao_callback();

        ft_transfer_promise.then(ft_transfer_callback_promise)
    }

    /**
     * Logs successful ft_transfer callback
     */
    #[private]
    pub fn on_ft_transfer_to_dao_callback(&mut self) {
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let amount: u128 = serde_json::from_slice(&result).expect("ERR_ON_FT_TRANSFER");
                log!("on_ft_transfer_callback: {}", amount);
            }
            _ => env::panic_str("ERR_ON_FT_TRANFER_CALLBACK"),
        };
    }

    pub fn get_claimed_staking_fees(&self, account_id: AccountId) -> String {
        if let Some(staking_fees) = &self.fees.staking_fees {
            match staking_fees.get(&account_id) {
                Some(amount) => amount,
                None => "0".to_string(),
            }
        } else {
            "0".to_string()
        }
    }

    pub fn is_claiming_window_expired(&self) -> bool {
        if let Some(claiming_window) = self.fees.claiming_window {
            return self.get_block_timestamp() > claiming_window;
        }

        return false;
    }

    pub fn claiming_window(&self) -> Timestamp {
        if let Some(claiming_window) = self.fees.claiming_window {
            return claiming_window;
        }

        env::panic_str("ERR_CLAIMING_WINDOW_NOT_SET");
    }
}
