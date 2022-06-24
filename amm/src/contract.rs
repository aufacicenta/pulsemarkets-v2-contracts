use near_sdk::collections::LookupMap;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen, AccountId, Promise};
use std::default::Default;

use crate::consts::*;
use crate::storage::*;

impl Default for Market {
    fn default() -> Self {
        env::panic_str("ERR_MARKET_NOT_INITIALIZED")
    }
}

/**
 * GLOSSARY
 *
 * Collateral Token, CT
 * Liquidity Provider, LP
 * Liquidity Provider Tokens, LPT
 * Outcome Token, OT
 *
 */
#[near_bindgen]
impl Market {
    #[init]
    pub fn new(
        market: MarketData,
        dao_account_id: AccountId,
        collateral_token_account_id: AccountId,
        fee_ratio: WrappedBalance,
        resolution_window: Timestamp,
    ) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        // @TODO assert at least 2 options

        Self {
            market,
            collateral_token: CollateralToken {
                id: collateral_token_account_id,
                balance: 0.0,
            },
            dao_account_id,
            outcome_tokens: LookupMap::new(StorageKeys::OutcomeTokens),
            fee_ratio,
            resolution_window,
            published_at: None,
            resolved_at: None,
        }
    }

    /**
     * Creates market options Sputnik2 DAO proposals
     * Creates an OT per each proposal
     *
     * The units of each OT is 0 until each is minted on the presale
     * The initial price of each unit is set to: 1 / self.market.options.len()
     *
     * @notice called by the MarketFactory contract only and only once
     * @notice publishes the market, does not mean it is open
     * @notice a market is open during the start_date and end_date period
     * @returns
     */
    #[payable]
    pub fn publish(&mut self) {
        self.assert_is_not_published();
        self.assert_in_stand_by();

        let mut outcome_id = 0;
        let options = &self.market.options.clone();

        for outcome in options {
            self.create_outcome_proposal(env::current_account_id(), outcome_id, &outcome);
            self.create_outcome_token(outcome_id);
            outcome_id += 1;
        }

        self.assert_price_constant();
        self.published_at = Some(self.get_block_timestamp());
    }

    /**
     * Lets accounts purchase OTs from the OT LP pools balances in exchange of CT
     * The price is calculated at the time of betting
     *
     * Increments the price of the selected OT by the predefined percentage
     * Decrements the price of the other OTs by the predefined percentage
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Increments the balance of OT in the buyer's balance
     * Decrements the balance of OT in the OT LP pool balance
     *
     * Charges lp_fee from the CT and transfers it to the OT LP Pool fee balance
     * LPs can withdraw the fees at any time using their LPTs
     *
     * @notice only while the market is open
     *
     * @returns amount of OT bought
     */
    #[payable]
    #[private]
    pub fn buy(
        &mut self,
        sender_id: AccountId,
        amount: WrappedBalance,
        payload: BuyArgs,
    ) -> WrappedBalance {
        self.assert_is_published();
        self.assert_is_not_over();
        self.assert_is_not_resolved();

        let mut outcome_token = self.get_outcome_token(payload.outcome_id);

        let price = outcome_token.get_price();
        let fee = amount * self.get_fee_ratio();
        let exchange_rate = (amount - fee) * (1.0 - price);
        let balance_boost = self.get_balance_boost_ratio();
        let amount_mintable = exchange_rate * balance_boost;

        log!("BUY amount: {}, fee_ratio: {}, fee_result: {}, outcome_id: {}, account_id: {}, supply: {}, price: {}, exchange_rate: {}, balance_boost: {}, amount_mintable: {}",
            amount,
            self.get_fee_ratio(),
            fee,
            outcome_token.outcome_id,
            sender_id,
            outcome_token.total_supply(),
            price,
            exchange_rate,
            balance_boost,
            amount_mintable,
        );

        outcome_token.mint(&sender_id, amount_mintable);
        self.update_ct_balance(amount);

        self.outcome_tokens
            .insert(&payload.outcome_id, &outcome_token);

        self.update_prices(payload.outcome_id, SetPriceOptions::Increase);

        return amount_mintable;
    }

    /**
     * An account may sell their OTs and get their CT back
     * No lp_fee is charged on this transaction
     *
     * Transfers CT amount to the account if their OT amount <= balance
     *
     * Decrements the price of the selected OT by the predefined ratio
     * Increments the price of the other OTs by the predefined ratio
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Decrements the balance of OT in the account's balance
     *
     * OT holders may always sell. The price is what changes.
     *
     * @notice only while the market is closed
     *
     * @param outcome_id, the id of the desired OT balance to sell
     * @param balance, how much of OTs balance to sell
     *
     * @returns amount of CT sold
     */
    #[payable]
    pub fn sell(&mut self, outcome_id: OutcomeId, amount: WrappedBalance) -> WrappedBalance {
        self.assert_is_published();

        if !self.is_resolved() {
            self.assert_is_not_under_resolution();
        }

        let outcome_token = self.get_outcome_token(outcome_id);

        let payee = env::signer_account_id();
        let mut weight = self.get_cumulative_weight(amount);
        let mut amount_payable = self.collateral_token.balance * weight;

        if self.is_resolved() {
            weight = amount / outcome_token.total_supply();
            amount_payable = self.collateral_token.balance * weight;
        }

        log!(
            "SELL amount: {}, outcome_id: {}, account_id: {}, ot_balance: {}, supply: {}, is_resolved: {}, ct_balance: {},  weight: {}, amount_payable: {}",
            amount,
            outcome_id,
            payee,
            outcome_token.get_balance(&payee),
            outcome_token.total_supply(),
            self.is_resolved(),
            self.collateral_token.balance,
            weight,
            amount_payable,
        );

        let ft_transfer_promise = Promise::new(self.collateral_token.id.clone()).function_call(
            "ft_transfer".to_string(),
            // @TODO amount_payable should not be float, but set to CT precision decimals
            json!({ "amount": amount_payable.floor().to_string(), "receiver_id": payee })
                .to_string()
                .into_bytes(),
            FT_TRANSFER_BOND,
            GAS_FT_TRANSFER,
        );

        let ft_transfer_callback_promise = Promise::new(env::current_account_id())
            .function_call(
                "on_ft_transfer_callback".to_string(),
                json!({"amount": amount, "payee": payee, "outcome_id": outcome_id, "amount_payable": amount_payable})
                    .to_string()
                    .into_bytes(),
                0,
                GAS_FT_TRANSFER_CALLBACK,
            );

        ft_transfer_promise.then(ft_transfer_callback_promise);

        return amount_payable;
    }

    /**
     * Resolves the market
     * Sets the winning OT price to 1
     * Sets the losing OT prices to 0
     *
     * NOTE: this method could be called by ft_on_transfer by $PULSE owners only
     *
     * @notice only after the market start_date and end_date period is over
     * @notice only by a Sputnik2 DAO Function Call Proposal!!
     *
     * @returns
     */
    #[payable]
    pub fn resolve(&mut self, outcome_id: OutcomeId) {
        self.assert_only_owner();
        self.assert_is_published();
        self.assert_is_not_resolved();
        self.assert_is_valid_outcome(outcome_id);

        // @TODO what happens if the resolution window expires?
        // Redeem is no longer possible? — Redeem is possible, but prices stay at their latest value
        self.assert_is_resolution_window_open();

        // @TODO distribute fee. Only when market is resolved?
        // - 95% goes to $PULSE stakers
        // - 5% goes to the user who created the market

        self.burn_the_losers(outcome_id);

        self.resolved_at = Some(self.get_block_timestamp());
    }

    #[private]
    pub fn update_prices(&mut self, outcome_id: OutcomeId, set_price_option: SetPriceOptions) {
        let price_ratio = self.get_price_ratio(outcome_id);
        let mut k: Price = 0.0;

        for id in 0..self.market.options.len() {
            let mut outcome_token = self.get_outcome_token(id as OutcomeId);

            // @TODO self.price_ratio may be updated so that it doesn't reach 1
            if outcome_token.outcome_id == outcome_id {
                match set_price_option {
                    SetPriceOptions::Increase => {
                        outcome_token.increase_price(price_ratio);
                    }
                    SetPriceOptions::Decrease => {
                        outcome_token.decrease_price(price_ratio);
                    }
                }
            } else {
                match set_price_option {
                    SetPriceOptions::Increase => {
                        outcome_token.decrease_price(price_ratio);
                    }
                    SetPriceOptions::Decrease => {
                        outcome_token.increase_price(price_ratio);
                    }
                }
            }

            self.outcome_tokens
                .insert(&(id as OutcomeId), &outcome_token);

            k += outcome_token.get_price();
        }

        assert_eq!(k, 1.0, "ERR_PRICE_CONSTANT_SHOULD_EQ_1");
    }

    #[private]
    pub fn update_ct_balance(&mut self, amount: WrappedBalance) -> WrappedBalance {
        self.collateral_token.balance += amount;
        self.collateral_token.balance
    }
}

impl Market {
    fn create_outcome_proposal(
        &self,
        receiver_id: AccountId,
        outcome_id: OutcomeId,
        outcome: &String,
    ) {
        let args = Base64VecU8(json!({ "outcome_id": outcome_id }).to_string().into_bytes());

        Promise::new(self.dao_account_id.clone()).function_call(
            "add_proposal".to_string(),
            json!({
                "proposal": {
                    "description": format!("{}\nOutcome: {}$$$$$$$$ProposeCustomFunctionCall",
                        self.market.description,
                        outcome),
                    "kind": {
                        "FunctionCall": {
                            "receiver_id": receiver_id,
                            "actions": [{
                                "args": args,
                                "deposit": "0", // @TODO
                                "gas": "150000000000000", // @TODO
                                "method_name": "resolve",
                            }]
                        }
                    }
                }
            })
            .to_string()
            .into_bytes(),
            BALANCE_PROPOSAL_BOND,
            GAS_CREATE_DAO_PROPOSAL,
        );
    }

    fn create_outcome_token(&mut self, outcome_id: OutcomeId) {
        let price = self.get_initial_outcome_token_price();
        let outcome_token = OutcomeToken::new(outcome_id, 0.0, price);
        self.outcome_tokens.insert(&outcome_id, &outcome_token);
    }

    fn get_initial_outcome_token_price(&self) -> Price {
        1 as Price / self.market.options.len() as Price
    }

    fn burn_the_losers(&mut self, outcome_id: OutcomeId) {
        for id in 0..self.market.options.len() {
            let mut outcome_token = self.get_outcome_token(id as OutcomeId);
            if outcome_token.outcome_id != outcome_id {
                outcome_token.burn_all();
                self.outcome_tokens
                    .insert(&(id as OutcomeId), &outcome_token);
            }
        }
    }

    fn get_cumulative_weight(&mut self, amount: WrappedBalance) -> WrappedBalance {
        let mut supply = 0.0;

        for id in 0..self.market.options.len() {
            let outcome_token = self.get_outcome_token(id as OutcomeId);
            supply += outcome_token.total_supply();
        }

        amount / supply
    }
}
