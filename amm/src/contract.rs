use near_sdk::collections::LookupMap;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Promise};
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
        price_ratio: f64,
    ) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            market,
            dao_account_id,
            collateral_token_account_id,
            resolved: false,
            published: false,
            outcome_tokens: LookupMap::new(StorageKeys::OutcomeTokens),
            lp_fee,
            lp_balances: LookupMap::new(StorageKeys::LiquidityProviderBalances),
            lp_pools_balances: LookupMap::new(StorageKeys::LiquidityProviderPoolsBalances),
            price_ratio,
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
    pub fn publish_market(&mut self) {
        if self.published {
            env::panic_str("ERR_MARKET_ALREADY_PUBLISHED");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        let mut outcome_id = 0;

        for market_option in &self.market.options {
            self.create_market_option_proposal(
                env::current_account_id(),
                outcome_id,
                market_option,
            );

            self.create_outcome_token(outcome_id);

            outcome_id += 1;
        }

        self.published = true;
    }

    /**
     * Mints units of OT in exchange of the CT
     * CT balance belongs to the contract (the LP transfers the CT to the contract)
     *
     * OT balance is incremented in the corresponding LP pool, there's an LP pool per OT
     * Each purchase increments the price of the selected OT by a predefined percentage, and
     * decrements the price of the other OTs, SUM of PRICES MUST EQUAL 1!!
     *
     * Keep balance of the CT that the LP deposited
     * Transfer LPTs to the buyer account_id as a reward
     *
     * @notice only after market is published, and
     * @notice while the market is open
     * @notice outcome_id must be between the length of market_options
     *
     * @param outcome_id matches a Market Option created on publish_market
     *
     * @returns
     */
    #[payable]
    pub fn purchase_outcome_tokens(&mut self, outcome_id: OutcomeId) {}

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
    pub fn bet(&mut self, outcome_id: OutcomeId) -> Promise {}

    /**
     * An account may drop their bet and get their CT back
     * No lp_fee is charged on this transaction
     *
     * Transfers CT amount to the account if their OT amount <= balance
     *
     * Decrements the price of the selected OT by the predefined percentage
     * Increments the price of the other OTs by the predefined percentage
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Decrements the balance of OT in the account's balance
     * Increments the balance of OT in the OT LP pool balance
     *
     * @notice only while the market is open
     *
     * @returns
     */
    #[payable]
    pub fn drop(&mut self, outcome_id: OutcomeId, amount: u64) -> Promise {}

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
    pub fn resolve(&mut self, outcome_id: OutcomeId) -> Promise {}

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
    pub fn withdraw(&mut self) -> Promise {}
}

impl Market {
    fn create_market_option_proposal(
        &self,
        receiver_id: AccountId,
        outcome_id: OutcomeId,
        market_option: &String,
    ) {
        let args = Base64VecU8(json!({ "outcome_id": outcome_id }).to_string().into_bytes());

        Promise::new(self.dao_account_id.clone()).function_call(
            "add_proposal".to_string(),
            json!({
                "proposal": {
                    "description": format!("{}:\n{}\nR: {}$$$$$$$$ProposeCustomFunctionCall",
                        receiver_id,
                        self.market.description,
                        market_option),
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
        let outcome_token = OutcomeToken::new(outcome_id, 0, price);
        self.outcome_tokens.insert(&outcome_id, &outcome_token);
    }

    fn get_initial_outcome_token_price(&self) -> Price {
        1 as Price / self.market.options.len() as Price
    }
}
