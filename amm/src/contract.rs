use near_sdk::collections::LookupMap;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen, AccountId, Promise};
use std::default::Default;

use crate::consts::*;
use crate::storage::*;

impl Default for Market {
    fn default() -> Self {
        env::panic_str("Market should be initialized before usage")
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
        lp_fee: f64,
        price_ratio: PriceRatio,
    ) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        // @TODO assert at least 2 options

        Self {
            market,
            dao_account_id,
            collateral_token_account_id,
            status: MarketStatus::Pending,
            outcome_tokens: LookupMap::new(StorageKeys::OutcomeTokens),
            price_ratio,
            lp_fee,
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
        self.assert_is_pending();
        self.assert_is_closed();

        let mut outcome_id = 0;
        let options = &self.market.options.clone();

        for outcome in options {
            self.create_outcome_proposal(env::current_account_id(), outcome_id, &outcome);
            self.create_outcome_token(outcome_id);
            outcome_id += 1;
        }

        self.assert_aggregated_price_is_1();

        self.status = MarketStatus::Published;
    }

    /**
     * Mints units of OT in exchange of the CT
     * CT balance belongs to the contract (the LP transfers the CT to the contract)
     *
     * OT balance is incremented in the corresponding LP pool, there's an LP pool per OT
     * Each purchase increments the price of the selected OT by a predefined ratio, and
     * decrements the price of the other OTs, SUM of PRICES MUST EQUAL 1!!
     *
     * Keep balance of the CT that the LP deposited
     * Transfer LPTs to the buyer account_id as a reward
     *
     * @notice only after market is published, and
     * @notice while the market is open
     * @notice outcome_id must be between the length of outcomes
     *
     * @param outcome_id matches an Outcome created on publish
     *
     * @returns
     */
    #[payable]
    #[private]
    pub fn add_liquidity(
        &mut self,
        sender_id: AccountId,
        amount: WrappedBalance,
        payload: AddLiquidityArgs,
    ) -> WrappedBalance {
        self.assert_is_published();
        self.assert_is_closed();
        self.assert_is_valid_outcome(payload.outcome_id);

        match self.outcome_tokens.get(&payload.outcome_id) {
            Some(token) => {
                let mut outcome_token = token;

                let priced_amount = (1.0 - outcome_token.get_price()) * amount;
                // @TODO add LP rate boost to incentivise buying early, eg. buy 1, get 2
                outcome_token.mint(&sender_id, priced_amount);

                self.update_outcome_token(&outcome_token);
                self.update_outcome_tokens_prices(payload.outcome_id, true);

                return outcome_token.total_supply();
            }
            None => {
                env::panic_str("ERR_WRONG_OUTCOME_ID");
            }
        }
    }

    /**
     * An account may sell their liquidity and get their CT back
     * No lp_fee is charged on this transaction
     *
     * Transfers CT amount to the account if their OT amount <= balance
     *
     * Decrements the price of the selected OT by the predefined ratio
     * Increments the price of the other OTs by the predefined ratio
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Decrements the balance of OT in the account's balance
     * Increments the balance of OT in the OT LP pool balance
     *
     * @notice only while the market is closed
     *
     * @returns
     */
    #[payable]
    pub fn remove_liquidity(
        &mut self,
        outcome_id: OutcomeId,
        amount: WrappedBalance,
    ) -> WrappedBalance {
        self.assert_is_published();
        self.assert_is_closed();
        self.assert_is_valid_outcome(outcome_id);

        match self.outcome_tokens.get(&outcome_id) {
            Some(token) => {
                let mut outcome_token = token;

                outcome_token.burn(&env::signer_account_id(), amount);

                let priced_amount = outcome_token.get_price() * amount;
                Promise::new(self.collateral_token_account_id.clone()).function_call(
                    "ft_transfer".to_string(),
                    json!({ "amount": priced_amount, "receiver_id": env::signer_account_id() })
                        .to_string()
                        .into_bytes(),
                    FT_TRANSFER_BOND,
                    GAS_FT_TRANSFER,
                );

                // @TODO create ft_transfer callback to verify that CT funds went through

                self.update_outcome_token(&outcome_token);
                self.update_outcome_tokens_prices(outcome_id, false);

                return outcome_token.total_supply();
            }
            None => 0.0,
        }
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
     * @returns
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
        self.assert_is_open();
        self.assert_is_valid_outcome(payload.outcome_id);

        match self.outcome_tokens.get(&payload.outcome_id) {
            Some(token) => {
                let mut outcome_token = token;

                // @TODO distribute rewards
                // Maybe something like a 2.5% market fee
                // - 0.5% goes to $PULSE stakers
                // - 1.5% goes to LPs
                // - 0.5% goes to the user who created the market
                let amount_minus_fee = amount * (1.0 - self.lp_fee);
                let priced_amount = outcome_token.get_price() * amount_minus_fee;
                outcome_token.lp_pool_buy(&sender_id, priced_amount);

                self.update_outcome_token(&outcome_token);
                self.update_outcome_tokens_prices(payload.outcome_id, true);

                return outcome_token.total_supply();
            }
            None => 0.0,
        }
    }

    /**
     * Closes the market
     * Sets the winning OT price to 1
     * Sets the losing OT prices to 0
     *
     * @notice only after the market start_date and end_date period is over
     * @notice only by a Sputnik2 DAO Function Call Proposal!!
     *
     * @returns
     */
    #[payable]
    pub fn resolve(&mut self, _outcome_id: OutcomeId) {}

    /**
     * Lets LPs and accounts redeem their CTs
     *
     * Transfers CT to the account if > 0
     * Decrements the balance of OT in the account's balance
     *
     * Transfers CT to the LP account if > 0
     * Decrements the balance of OT in the OT LP pool balance
     *
     * Will transfer the proportional CT to the account because the price is 1, so
     * make a calculation of the account CT balance and the closing price and transfer the difference, eg 1 - closing price
     *
     * @notice only after the market start_date and end_date period is over, eg. self.is_closed
     *
     * @returns
     */
    #[payable]
    pub fn redeem(&mut self) {}
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
                    "description": format!("{}:\n{}\nR: {}$$$$$$$$ProposeCustomFunctionCall",
                        receiver_id,
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

    fn update_outcome_tokens_prices(&mut self, outcome_id: OutcomeId, increase: bool) {
        let mut k: Price = 0.0;

        for id in 0..self.market.options.len() {
            self.assert_is_valid_outcome(id as OutcomeId);

            match self.outcome_tokens.get(&(id as OutcomeId)) {
                Some(token) => {
                    let mut outcome_token = token;

                    if outcome_token.outcome_id == outcome_id {
                        if increase {
                            outcome_token.increase_price(self.price_ratio);
                        } else {
                            outcome_token.decrease_price(self.price_ratio);
                        }
                    } else {
                        if increase {
                            outcome_token.decrease_price(self.price_ratio);
                        } else {
                            outcome_token.increase_price(self.price_ratio);
                        }
                    }

                    self.update_outcome_token(&outcome_token);

                    k += outcome_token.get_price();
                }
                None => {}
            }
        }

        assert_eq!(k, 1.0, "ERR_PRICE_CONSTANT_SHOULD_EQ_1");
    }

    fn update_outcome_token(&mut self, outcome_token: &OutcomeToken) {
        self.outcome_tokens
            .insert(&outcome_token.outcome_id, outcome_token);
    }
}
