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
 * Market Option Token, MOT
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
            lp_fee,
            options_prices: LookupMap::new(StorageKeys::MarketOptionsPrices),
            lp_balances: LookupMap::new(StorageKeys::LiquidityProviderBalances),
            lp_pools_balances: LookupMap::new(StorageKeys::LiquidityProviderPoolsBalances),
            price_ratio,
        }
    }

    /**
     * Creates market options Sputnik2 DAO proposals
     * Creates a MOT per each proposal
     *
     * The units of each MOT is 0 until each is minted on the presale
     * The initial price of each unit is set to: 1 / market_options_length
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

        self.create_market_options_proposals();
    }

    /**
     * Mints units of MOT in exchange of the CT
     * CT balance belongs to the contract (the LP transfers the CT to the contract)
     *
     * MOT balance is incremented in the corresponding LP pool, there's an LP pool per MOT
     * Each purchase increments the price of the selected MOT by a predefined percentage, and
     * decrements the price of the other MOTs, SUM of PRICES MUST EQUAL 1!!
     *
     * Keep balance of the CT that the LP deposited
     * Transfer LPTs to the buyer account_id as a reward
     *
     * @notice only after market is published, and
     * @notice while the market is open
     * @notice market_option_index must be between the length of market_options
     *
     * @param market_option_index matches a Market Option created on publish_market
     *
     * @returns
     */
    #[payable]
    pub fn purchase_market_option_tokens(&mut self, market_option_index: MarketOptionIndex) {}

    /**
     * Lets accounts purchase MOTs from the MOT LP pools balances in exchange of CT
     * The price is calculated at the time of betting
     *
     * Increments the price of the selected MOT by the predefined percentage
     * Decrements the price of the other MOTs by the predefined percentage
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Increments the balance of MOT in the buyer's balance
     * Decrements the balance of MOT in the MOT LP pool balance
     *
     * Charges lp_fee from the CT and transfers it to the MOT LP Pool fee balance
     * LPs can withdraw the fees at any time using their LPTs
     *
     * @notice only while the market is open
     *
     * @returns
     */
    #[payable]
    pub fn bet(&mut self, market_option_index: MarketOptionIndex) -> Promise {}

    /**
     * An account may drop their bet and get their CT back
     * No lp_fee is charged on this transaction
     *
     * Transfers CT amount to the account if their MOT amount <= balance
     *
     * Decrements the price of the selected MOT by the predefined percentage
     * Increments the price of the other MOTs by the predefined percentage
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Decrements the balance of MOT in the account's balance
     * Increments the balance of MOT in the MOT LP pool balance
     *
     * @notice only while the market is open
     *
     * @returns
     */
    #[payable]
    pub fn drop(&mut self, market_option_index: MarketOptionIndex, amount: u64) -> Promise {}

    /**
     * Closes the market
     * Sets the winning MOT price to 1
     * Sets the losing MOT prices to 0
     *
     * @notice only after the market start_date and end_date period is over
     * @notice only by a Sputnik2 DAO Function Call Proposal!!
     *
     * @returns
     */
    #[payable]
    pub fn resolve(&mut self, market_option_index: MarketOptionIndex) -> Promise {}

    /**
     * Lets LPs and accounts redeem their CTs
     *
     * Transfers CT to the account if > 0
     * Decrements the balance of MOT in the account's balance
     *
     * Transfers CT to the LP account if > 0
     * Decrements the balance of MOT in the MOT LP pool balance
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
    fn create_market_options_proposals(&self) {
        let mut market_options_idx = 0;
        let receiver_id = env::current_account_id().to_string();

        for market_option in &self.market.options {
            let args = Base64VecU8(
                json!({ "market_options_idx": market_options_idx })
                    .to_string()
                    .into_bytes(),
            );

            let promise = Promise::new(self.dao_account_id.clone()).function_call(
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

            let callback = Promise::new(env::current_account_id()).function_call(
                "on_create_proposal_callback".to_string(),
                json!({ "market_options_idx": market_options_idx })
                    .to_string()
                    .into_bytes(),
                0,
                GAS_CREATE_DAO_PROPOSAL_CALLBACK,
            );

            promise.then(callback);

            market_options_idx += 1;
        }
    }
}
