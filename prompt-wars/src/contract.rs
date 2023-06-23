use near_sdk::{
    collections::LookupMap, env, ext_contract, json_types::U128, log, near_bindgen, serde_json,
    AccountId, Promise,
};
use num_format::ToFormattedString;
use shared::OutcomeId;
use std::default::Default;

use near_contract_standards::fungible_token::core::ext_ft_core;

use crate::consts::*;
use crate::storage::*;

#[ext_contract(ext_self)]
trait Callbacks {
    fn on_ft_transfer_callback(
        &mut self,
        amount: WrappedBalance,
        payee: AccountId,
        outcome_id: OutcomeId,
        amount_payable: WrappedBalance,
    ) -> String;
}

#[ext_contract(ext_feed_parser)]
trait SwitchboardFeedParser {
    fn aggregator_read(&self, msg: String) -> Promise;
}

impl Default for Market {
    fn default() -> Self {
        env::panic_str("ERR_MARKET_NOT_INITIALIZED")
    }
}

#[near_bindgen]
impl Market {
    #[init]
    pub fn new(
        market: MarketData,
        resolution: Resolution,
        management: Management,
        collateral_token: CollateralToken,
    ) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        let resolution_window = resolution.window;
        let self_destruct_window = resolution_window + 2_592_000 * 1_000_000_000;

        Self {
            market,
            outcome_tokens: LookupMap::new(StorageKeys::OutcomeTokens),
            resolution,
            fees: Fees {
                price: CREATE_OUTCOME_TOKEN_PRICE,
                fee_ratio: FEE_RATIO,
            },
            collateral_token: CollateralToken {
                balance: 0,
                fee_balance: 0,
                // @TODO collateral_token_decimals should be set by a cross-contract call to ft_metadata
                ..collateral_token
            },
            management: Management {
                self_destruct_window,
                ..management
            },
        }
    }

    pub fn create_outcome_token(
        &mut self,
        sender_id: AccountId,
        amount: WrappedBalance,
        payload: CreateOutcomeTokenArgs,
    ) -> WrappedBalance {
        self.assert_is_open();
        self.assert_is_not_resolved();

        let player_id = payload.player_id;
        let value = payload.value;

        match self.outcome_tokens.get(&player_id) {
            Some(_token) => env::panic_str("ERR_CREATE_OUTCOME_TOKEN_PLAYER_ID_EXIST"),
            None => {
                self.internal_create_outcome_token(player_id, value, amount);
            }
        }

        amount
    }

    #[payable]
    #[private]
    pub fn buy(
        &mut self,
        sender_id: AccountId,
        amount: WrappedBalance,
        payload: BuyArgs,
    ) -> WrappedBalance {
        self.assert_is_open();
        self.assert_is_not_resolved();

        let mut outcome_token = self.get_outcome_token(payload.player_id);

        let (amount_mintable, fee) = self.get_amount_mintable(amount);

        log!("BUY amount: {}, fee_ratio: {}, fee_result: {}, player_id: {}, sender_id: {}, supply: {}, amount_mintable: {}, fee_balance: {}",
            amount.to_formatted_string(&FORMATTED_STRING_LOCALE),
            self.fees.fee_ratio.to_formatted_string(&FORMATTED_STRING_LOCALE),
            fee.to_formatted_string(&FORMATTED_STRING_LOCALE),
            outcome_token.player_id,
            sender_id,
            outcome_token.total_supply().to_formatted_string(&FORMATTED_STRING_LOCALE),
            amount_mintable.to_formatted_string(&FORMATTED_STRING_LOCALE),
            self.collateral_token.fee_balance.to_formatted_string(&FORMATTED_STRING_LOCALE),
        );

        outcome_token.mint(&sender_id, amount_mintable);
        self.update_ct_balance(self.collateral_token.balance + amount);
        self.update_ct_fee_balance(fee);

        self.outcome_tokens
            .insert(&payload.player_id, &outcome_token);

        return amount_mintable;
    }

    #[payable]
    pub fn sell(&mut self, outcome_id: OutcomeId, amount: WrappedBalance) -> WrappedBalance {
        // @TODO if there are participants only in 1 outcome, allow to claim funds after resolution, otherwise funds will be locked
        if self.is_expired_unresolved() {
            return self.internal_sell(outcome_id, amount);
        }

        self.assert_is_not_under_resolution();
        self.assert_is_resolved();

        return self.internal_sell(outcome_id, amount);
    }

    #[payable]
    pub fn resolve(&mut self, outcome_id: OutcomeId, ix: Ix) {
        self.assert_only_owner(ix);
        self.assert_is_not_resolved();
        self.assert_is_valid_outcome(outcome_id);

        self.assert_is_resolution_window_open();

        self.burn_the_losers(outcome_id);

        self.resolution.resolved_at = Some(self.get_block_timestamp());
    }

    pub fn aggregator_read(&mut self) -> Promise {
        let ix = self.resolution.ix.clone();

        let msg = serde_json::json!({
            "AggregatorReadArgs": {
                "ix": ix,
                "market_options": self.get_market_data().options,
                "market_outcome_ids": self.get_outcome_ids(),
                "price": self.get_pricing_data().value,
            }
        });

        log!("{}", msg.to_string());

        ext_feed_parser::ext(FEED_PARSER_ACCOUNT_ID.to_string().try_into().unwrap())
            .with_static_gas(GAS_AGGREGATOR_READ)
            .aggregator_read(msg.to_string())
    }

    #[private]
    pub fn update_ct_balance(&mut self, amount: WrappedBalance) -> WrappedBalance {
        log!(
            "update_ct_balance: {}",
            amount.to_formatted_string(&FORMATTED_STRING_LOCALE)
        );

        self.collateral_token.balance = amount;
        self.collateral_token.balance
    }

    #[private]
    pub fn update_ct_fee_balance(&mut self, amount: WrappedBalance) -> WrappedBalance {
        self.collateral_token.fee_balance += amount;
        self.collateral_token.fee_balance
    }
}

impl Market {
    fn internal_create_outcome_token(
        &mut self,
        player_id: AccountId,
        value: String,
        amount: WrappedBalance,
    ) -> WrappedBalance {
        let (amount_mintable, fee) = self.get_amount_mintable(amount);

        let outcome_token = OutcomeToken::new(player_id, value, 0);
        self.outcome_tokens.insert(&player_id, &outcome_token);

        amount_mintable
    }

    fn burn_the_losers(&mut self, outcome_id: OutcomeId) {
        for id in 0..self.market.options.len() {
            let mut outcome_token = self.get_outcome_token(id as OutcomeId);
            if outcome_token.outcome_id != outcome_id {
                outcome_token.deactivate();
                self.outcome_tokens
                    .insert(&(id as OutcomeId), &outcome_token);
            }
        }
    }

    fn internal_sell(&mut self, outcome_id: OutcomeId, amount: WrappedBalance) -> WrappedBalance {
        if amount > self.balance_of(outcome_id, env::signer_account_id()) {
            env::panic_str("ERR_SELL_AMOUNT_GREATER_THAN_BALANCE");
        }

        let (amount_payable, weight) =
            self.get_amount_payable(amount, outcome_id, env::signer_account_id());

        if amount_payable <= 0 {
            env::panic_str("ERR_CANT_SELL_A_LOSING_OUTCOME");
        }

        let outcome_token = self.get_outcome_token(outcome_id);

        let payee = env::signer_account_id();

        log!(
            "SELL amount: {}, outcome_id: {}, account_id: {}, ot_balance: {}, supply: {}, is_resolved: {}, ct_balance: {},  weight: {}, amount_payable: {}",
            amount.to_formatted_string(&FORMATTED_STRING_LOCALE),
            outcome_id,
            payee,
            outcome_token.get_balance(&payee).to_formatted_string(&FORMATTED_STRING_LOCALE),
            outcome_token.total_supply().to_formatted_string(&FORMATTED_STRING_LOCALE),
            self.is_resolved(),
            self.collateral_token.balance.to_formatted_string(&FORMATTED_STRING_LOCALE),
            weight.to_formatted_string(&FORMATTED_STRING_LOCALE),
            amount_payable.to_formatted_string(&FORMATTED_STRING_LOCALE),
        );

        let ft_transfer_promise = ext_ft_core::ext(self.collateral_token.id.clone())
            .with_attached_deposit(FT_TRANSFER_BOND)
            .with_static_gas(GAS_FT_TRANSFER)
            .ft_transfer(payee.clone(), U128::from(amount_payable), None);

        let ft_transfer_callback_promise = ext_self::ext(env::current_account_id())
            .with_attached_deposit(0)
            .with_static_gas(GAS_FT_TRANSFER_CALLBACK)
            .on_ft_transfer_callback(amount, payee, outcome_id, amount_payable);

        ft_transfer_promise.then(ft_transfer_callback_promise);

        amount_payable
    }
}
