use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen, AccountId, Promise, PromiseOrValue};

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use std::default::Default;

use crate::consts::*;
use crate::conditional_tokens::*;
use crate::math;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub market: MarketData,
    pub collateral_token: AccountId,
    pub status: MarketStatus,
    pub fee: u64,
    pub conditional_tokens: ConditionalTokens,
    pub liquidity_token: FungibleToken,
    pub metadata: LazyOption<FungibleTokenMetadata>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub oracle: AccountId,
    pub question_id: u64,
    pub options: u8,
    pub expiration_date: u64,
    pub resolution_window: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum MarketStatus {
    Pending,
    Running,
    Paused,
    Closed,
}

impl Default for Market {
    fn default() -> Self {
        env::panic_str("Market should be initialized before usage")
    }
}

/**
 * GLOSSARY
 *
 * Collateral Token, CT
 * Liquidity Provider Token, LP
 * Conditional Tokens, CT
 *
 */
#[near_bindgen]
impl Market {
    #[init]
    pub fn new(
        market: MarketData,
        collateral_token: AccountId,
        fee: u64,
    ) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            market,
            collateral_token,
            status: MarketStatus::Pending,
            fee,
            conditional_tokens: ConditionalTokens {
                tokens: LookupMap::new(StorageKeys::ConditionalTokensBalances),
                total_balances: LookupMap::new(StorageKeys::ConditionalTokensTotalBalances),
            },
            liquidity_token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), None),
        }
    }

    
    /**
     * Creates market options Sputnik2 DAO proposals
     * Creates an NEP141 per each MOT
     *
     * The units of each MOT is 0 until each is minted on the presale
     * The initial price of each unit is set to: 1 / market_options_length
     *
     * @notice Should called by the MarketFactory contract only and only once
     * @notice publishes the market, does not mean it is open
     * @notice a market is open during the start_date and end_date period
     * @returns
     */
    #[payable]
    pub fn publish(&mut self) -> Promise {
        if self.status != MarketStatus::Pending {
            env::panic_str("ERR_MARKET_ALREADY_PUBLISHED");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        let mut promise: Promise = Promise::new(self.market.oracle.clone());

        for market_option in 0 .. self.market.options {
            let mut options = vec![0; self.market.options.into()];
            options.insert(market_option.into(), 1);
            
            let args = Base64VecU8(
                json!(options)
                    .to_string()
                    .into_bytes(),
            );

            promise = promise.function_call(
                "add_proposal".to_string(),
                json!({
                    "proposal": {
                        "description": format!("{}:\n{}\nR: {}$$$$$$$$ProposeCustomFunctionCall",
                            env::current_account_id().to_string(),
                            self.market.question_id,
                            market_option),
                        "kind": {
                            "FunctionCall": {
                                "receiver_id": env::current_account_id().to_string(),
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

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_proposals_callback".to_string(),
            json!({}).to_string().into_bytes(),
            0,
            GAS_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        promise.then(callback)
    }

    fn add_liquidity_through_all_options(&mut self, amount: u128) {
        for market_option in 0 .. self.market.options {
            self.conditional_tokens.mint(market_option as u64, env::current_account_id(), amount);
        }
    }

    #[payable]
    pub fn add_liquidity(&mut self) {
        if self.status != MarketStatus::Running {
            env::panic_str("ERR_MARKET_IS_NOT_RUNNING");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        if env::attached_deposit() == 0 {
            env::panic_str("ERR_DEPOSIT_SHOULD_NOT_BE_0");
        }

        let amount = env::attached_deposit();
        let lp_total_supply = self.liquidity_token.total_supply;
        let mut mint_amount = amount;
        let mut send_back = Vec::new();

        if lp_total_supply > 0 {
            // Getting the max Pool Weight
            let mut pool_weight = 0;
            for market_option in 0 .. self.market.options {
                let balance = self.conditional_tokens.get_balance_by_token_idx(&(market_option as u64));
                if pool_weight < balance {
                    pool_weight = balance;
                }
            }

            // Calculate LP to Mint and SendBacks
            for market_option in 0 .. self.market.options {
                let balance = self.conditional_tokens.get_balance_by_token_idx(&(market_option as u64));
                let remaining = math::complex_div_u128(ONE, math::complex_mul_u128(ONE, balance, amount), pool_weight);
                send_back.push(amount - remaining);
            }

            mint_amount = math::complex_div_u128(ONE, math::complex_mul_u128(ONE, lp_total_supply, amount), pool_weight);
        }

        // Mint Liquidity Tokens
        if !self.liquidity_token.accounts.get(&env::signer_account_id()).is_some() {
            self.liquidity_token.internal_register_account(&env::signer_account_id());
        }
        self.liquidity_token.internal_deposit(&env::signer_account_id(), mint_amount);

        // Mint Conditional Tokens
        self.add_liquidity_through_all_options(amount);

        // Send BackTokens
        if send_back.len() > 0 {
            self.conditional_tokens.transfer_batch(env::current_account_id(), env::signer_account_id(), vec![0; self.market.options.into()], send_back);
        }
    }

    #[payable]
    pub fn remove_liquidity(&mut self) {}

    #[payable]
    pub fn buy(&mut self, outcome_idx: u64, min_outcome_tokens_to_buy: u128) {
        if self.status != MarketStatus::Running {
            env::panic_str("ERR_MARKET_IS_NOT_RUNNING");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        if env::attached_deposit() == 0 {
            env::panic_str("ERR_DEPOSIT_SHOULD_NOT_BE_0");
        }

        let investment_amount = env::attached_deposit();
        let outcome_tokens_to_buy = self.calc_buy_amount(investment_amount, outcome_idx);

        if outcome_tokens_to_buy < min_outcome_tokens_to_buy {
            env::panic_str("ERR_minimum_buy_amount_not_reached");
        }

        //@TODO Send collateral token to the market
        //@TODO Calculate fees

        // Mint Conditional Tokens
        self.add_liquidity_through_all_options(investment_amount);

        // Tranfer Conditional Token
        self.conditional_tokens.transfer(outcome_idx, env::current_account_id(), env::signer_account_id(), outcome_tokens_to_buy);
    }

    #[payable]
    pub fn sell(&mut self) {}

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
    pub fn resolve(&mut self) {}

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
    pub fn redeem(&mut self) {}
}

near_contract_standards::impl_fungible_token_core!(Market, liquidity_token);
near_contract_standards::impl_fungible_token_storage!(Market, liquidity_token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Market {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
