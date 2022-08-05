use near_sdk::serde_json::json;
use near_sdk::{
    env, log, near_bindgen, require, serde_json, AccountId, Balance, Promise, PromiseResult,
};

use crate::consts::*;
use crate::storage::*;

#[near_bindgen]
impl Market {
    /**
     * Lets fee payees claim their balance
     *
     * @notice only after market is resolved
     *
     * @returns WrappedBalance of fee proportion paid
     */
    pub fn claim_staking_fees_resolved(&mut self) {
        self.assert_is_resolved();

        // @TODO fees for $PULSE stakers: 85%
        // @TODO get balance_of $PULSE for env::account_signer_id() and check its weight to the general token supply
        match self.fees.staking_fees.get(&env::signer_account_id()) {
            Some(_) => env::panic_str("ERR_CLAIM_STAKING_FEES_RESOLVED_NO_FEES_TO_CLAIM"),
            None => {
                let ft_balance_of_promise = env::promise_create(
                    self.staking_token_account_id.clone(),
                    "ft_balance_of",
                    json!({
                        "account_id": env::signer_account_id(),
                    })
                    .to_string()
                    .as_bytes(),
                    0,
                    GAS_FT_BALANCE_OF,
                );

                let ft_metadata_promise = env::promise_create(
                    self.staking_token_account_id.clone(),
                    "ft_total_supply",
                    json!({}).to_string().as_bytes(),
                    0,
                    GAS_FT_METADATA,
                );

                let promises = env::promise_and(&[ft_balance_of_promise, ft_metadata_promise]);

                let callback = env::promise_then(
                    promises,
                    env::current_account_id(),
                    "on_ft_balance_of_callback",
                    json!({
                        "amount": self.collateral_token.fee_balance,
                        "payee": env::signer_account_id(),
                    })
                    .to_string()
                    .as_bytes(),
                    0,
                    GAS_FT_METADATA_CALLBACK,
                );

                env::promise_return(callback)
            }
        };

        // @TODO fees for market creator: 10%
        // @TODO check if signer is market creator, then transfer

        // @TODO fees for market publisher: 5%
        // @TODO check if signer is market publisher, then transfer
    }

    /**
     * Lets users claim proportional fees of an unresolved market
     *
     * @notice only is market was not resolved after resolution window
     *
     * @returns WrappedBalance of fee proportion paid
     */
    #[payable]
    pub fn claim_fees_unresolved(&mut self) -> WrappedBalance {
        if !self.is_resolution_window_expired() && self.is_resolved() {
            env::panic_str("ERR_CANNOT_CLAIM_FEES_OF_RESOLVED_MARKET");
        }

        // @TODO let users claim their proportional fees
        // @TODO iterate over all outcome token supplies for the user and get their cumulative weight

        0.0
    }

    #[private]
    pub fn on_ft_balance_of_callback(&mut self, amount: WrappedBalance, payee: AccountId) {
        require!(env::promise_results_count() == 2);

        let ft_balance_of_result = match env::promise_result(0) {
            PromiseResult::Successful(result) => result,
            _ => env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK_0"),
        };

        let ft_balance_of: Balance = serde_json::from_slice(&ft_balance_of_result)
            .expect("ERR_ON_FT_BALANCE_OF_CALLBACK_RESULT_0");

        let supply_result = match env::promise_result(1) {
            PromiseResult::Successful(result) => result,
            _ => env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK_0"),
        };

        let ft_total_supply: Balance =
            serde_json::from_slice(&supply_result).expect("ERR_ON_FT_BALANCE_OF_CALLBACK_RESULT_1");

        log!(
            "on_ft_balance_of_callback ft_balance_of: {}, ft_total_supply: {}",
            ft_balance_of,
            ft_total_supply,
        );

        let weight = ft_balance_of / ft_total_supply;
        let precision = self.get_precision();
        let amount_payable =
            &((amount * weight as WrappedBalance) * precision.parse::<WrappedBalance>().unwrap());

        log!(
            "on_ft_balance_of_callback weight: {}, precision: {}, amount_payable: {}",
            weight,
            precision,
            amount_payable
        );

        // let ft_transfer_promise =
        Promise::new(self.collateral_token.id.clone()).function_call(
            "ft_transfer".to_string(),
            json!({
                "amount": amount_payable.to_string(),
                "receiver_id": payee
            })
            .to_string()
            .into_bytes(),
            FT_TRANSFER_BOND,
            GAS_FT_TRANSFER,
        );

        // @TODO callback of a failed transfer

        self.fees.staking_fees.insert(&payee, &true);
    }

    fn get_precision(&self) -> String {
        let precision = format!(
            "{:0<p$}",
            10,
            p = self.collateral_token.decimals as usize + 2
        );

        precision
    }
}
