use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen, require, serde_json, AccountId, Promise, PromiseResult};

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
        self.assert_is_claiming_window_open();

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

                let ft_total_supply_promise = env::promise_create(
                    self.staking_token_account_id.clone(),
                    "ft_total_supply",
                    json!({}).to_string().as_bytes(),
                    0,
                    GAS_FT_TOTAL_SUPPLY,
                );

                let promises = env::promise_and(&[ft_balance_of_promise, ft_total_supply_promise]);

                let amount = self.collateral_token.fee_balance * 0.15;

                let callback = env::promise_then(
                    promises,
                    env::current_account_id(),
                    "on_claim_staking_fees_resolved_callback",
                    json!({
                        "amount": amount,
                        "payee": env::signer_account_id(),
                    })
                    .to_string()
                    .as_bytes(),
                    0,
                    GAS_FT_TOTAL_SUPPLY_CALLBACK,
                );

                env::promise_return(callback)
            }
        };

        // @TODO fees for market publisher: 5%
        // @TODO check if signer is market publisher, then transfer
    }

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

        if payee != self.market_creator_account_id {
            env::panic_str("ERR_CLAIM_MARKET_CREATOR_FEES_RESOLVED_ACCOUNT_ID_MISTMATCH");
        }

        match self.fees.market_creator_fees.get(&payee) {
            Some(_) => env::panic_str("ERR_CLAIM_MARKET_CREATOR_FEES_RESOLVED_NO_FEES_TO_CLAIM"),
            None => {
                let amount = self.collateral_token.fee_balance * 0.80;
                let precision = self.get_precision();
                let amount_payable = &(amount * precision.parse::<WrappedBalance>().unwrap());

                let ft_transfer_promise = Promise::new(self.collateral_token.id.clone())
                    .function_call(
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

                let ft_transfer_callback_promise = Promise::new(env::current_account_id())
                    .function_call(
                        "on_claim_market_creator_fees_resolved_callback".to_string(),
                        json!({
                            "payee": payee,
                        })
                        .to_string()
                        .into_bytes(),
                        0,
                        GAS_FT_TRANSFER_CALLBACK,
                    );

                ft_transfer_promise.then(ft_transfer_callback_promise);
            }
        };
    }

    /**
     * Lets fee payees claim their balance
     *
     * @notice only after market is resolved
     *
     * @returns WrappedBalance of fee proportion paid
     */
    pub fn claim_market_publisher_fees_resolved(&mut self) {
        self.assert_is_resolved();
        self.assert_is_claiming_window_open();

        let payee = env::signer_account_id();

        match &self.market_publisher_account_id {
            Some(account_id) => {
                if payee != *account_id {
                    env::panic_str("ERR_CLAIM_MARKET_PUBLISHER_FEES_RESOLVED_ACCOUNT_ID_MISTMATCH");
                }

                match self.fees.market_publisher_fees.get(&payee) {
                    Some(_) => {
                        env::panic_str("ERR_CLAIM_MARKET_PUBLISHER_FEES_RESOLVED_NO_FEES_TO_CLAIM")
                    }
                    None => {
                        let amount = self.collateral_token.fee_balance * 0.05;
                        let precision = self.get_precision();
                        let amount_payable =
                            &(amount * precision.parse::<WrappedBalance>().unwrap());

                        let ft_transfer_promise = Promise::new(self.collateral_token.id.clone())
                            .function_call(
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

                        let ft_transfer_callback_promise = Promise::new(env::current_account_id())
                            .function_call(
                                "on_claim_market_publisher_fees_resolved_callback".to_string(),
                                json!({
                                    "payee": payee,
                                })
                                .to_string()
                                .into_bytes(),
                                0,
                                GAS_FT_TRANSFER_CALLBACK,
                            );

                        ft_transfer_promise.then(ft_transfer_callback_promise);
                    }
                };
            }
            None => env::panic_str("ERR_CLAIM_MARKET_PUBLISHER_FEES_ACCOUNT_ID_NOT_SET"),
        }
    }

    /**
     * Lets users claim proportional fees of an unresolved market
     *
     * @notice only if market was not resolved after resolution window
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

    /**
     * Sends the remaining unclaimed fees to DAO
     *
     * @notice only if market is resolved and fees claiming window expired
     *
     * @returns WrappedBalance of fee proportion paid
     */
    #[payable]
    pub fn claim_fees_unclaimed(&mut self) -> WrappedBalance {
        if !self.is_claiming_window_expired() || !self.is_resolved() {
            env::panic_str("ERR_CANNOT_CLAIM_FEES_OF_RESOLVED_MARKET_BEFORE_WINDOW_EXPIRATION");
        }

        // @TODO get remaining fees balance and send them to DAO

        0.0
    }

    #[private]
    pub fn on_claim_staking_fees_resolved_callback(
        &mut self,
        amount: WrappedBalance,
        payee: AccountId,
    ) -> String {
        require!(env::promise_results_count() == 2);

        let ft_balance_of_result = match env::promise_result(0) {
            PromiseResult::Successful(result) => result,
            _ => env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK_0"),
        };

        let ft_balance_of: WrappedBalance = serde_json::from_slice(&ft_balance_of_result)
            .expect("ERR_ON_FT_BALANCE_OF_CALLBACK_RESULT_0");

        let ft_total_supply_result = match env::promise_result(1) {
            PromiseResult::Successful(result) => result,
            _ => env::panic_str("ERR_ON_FT_BALANCE_OF_CALLBACK_0"),
        };

        let ft_total_supply: WrappedBalance = serde_json::from_slice(&ft_total_supply_result)
            .expect("ERR_ON_FT_BALANCE_OF_CALLBACK_RESULT_1");

        log!(
            "on_claim_staking_fees_resolved_callback ft_balance_of: {}, ft_total_supply: {}, amount: {}, payee: {}",
            ft_balance_of,
            ft_total_supply,
            amount,
            payee,
        );

        let weight = ft_balance_of / ft_total_supply;
        let precision = self.get_precision();
        let amount_payable = &((amount * weight) * precision.parse::<WrappedBalance>().unwrap());

        log!(
            "on_claim_staking_fees_resolved_callback weight: {}, precision: {}, amount_payable: {}",
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

        self.fees
            .staking_fees
            .insert(&payee, &amount_payable.to_string());

        return amount_payable.to_string();
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

        self.fees
            .market_creator_fees
            .insert(&payee, &amount_payable.to_string());

        return amount_payable.to_string();
    }

    #[private]
    pub fn on_claim_market_publisher_fees_resolved_callback(&mut self, payee: AccountId) -> String {
        let ft_transfer_result = match env::promise_result(0) {
            PromiseResult::Successful(result) => result,
            // On error, the funds were transfered back to the sender
            _ => env::panic_str("ERR_ON_FT_TRANSFER_CALLBACK"),
        };

        let amount_payable: WrappedBalance =
            serde_json::from_slice(&ft_transfer_result).expect("ERR_ON_FT_TRANSFER");

        self.fees
            .market_creator_fees
            .insert(&payee, &amount_payable.to_string());

        return amount_payable.to_string();
    }

    pub fn get_claimed_staking_fees(&self, account_id: AccountId) -> String {
        match self.fees.staking_fees.get(&account_id) {
            Some(amount) => amount,
            None => "0".to_string(),
        }
    }

    pub fn is_claiming_window_expired(&self) -> bool {
        match self.fees.claiming_window {
            Some(timestamp) => self.get_block_timestamp() > timestamp,
            None => false,
        }
    }

    pub fn claiming_window(&self) -> Timestamp {
        match self.fees.claiming_window {
            Some(timestamp) => timestamp,
            None => env::panic_str("ERR_CLAIMING_WINDOW_NOT_SET"),
        }
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
