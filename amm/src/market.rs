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
    pub payout_numerators: Vec<u64>,
    pub payout_denominator: u64,
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
            payout_numerators: Vec::new(),
            payout_denominator: 0,
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

    fn mint_through_all_outcomes(&mut self, amount: u128) {
        for outcome_idx in 0 .. self.market.options {
            self.conditional_tokens.mint(outcome_idx as u64, env::current_account_id(), amount);
        }
    }

    fn burn_through_all_outcomes(&mut self, amount: u128) {
        for outcome_idx in 0 .. self.market.options {
            self.conditional_tokens.burn(outcome_idx as u64, env::current_account_id(), amount);
        }
    }

    fn get_pool_balances(&self) -> Vec<u128>{
        let mut balances = Vec::new();

        for market_option in 0 .. self.market.options {
            let outcome_balance = self.conditional_tokens.get_balance_by_account(&(market_option as u64), &env::current_account_id());
            balances.push(outcome_balance);
        }
        
        balances
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
                let balance = self.conditional_tokens.get_balance_by_account(&(market_option as u64), &env::current_account_id());
                if pool_weight < balance {
                    pool_weight = balance;
                }
            }

            // Calculate LP to Mint and SendBacks
            for market_option in 0 .. self.market.options {
                let balance = self.conditional_tokens.get_balance_by_account(&(market_option as u64), &env::current_account_id());
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
        self.mint_through_all_outcomes(amount);

        // Send BackTokens
        if send_back.len() > 0 {
            self.conditional_tokens.transfer_batch(env::current_account_id(), env::signer_account_id(), vec![0; self.market.options.into()], send_back);
        }
    }

    #[payable]
    pub fn remove_liquidity(&mut self, lp_to_burn: u128) {
        let pool_balances = self.get_pool_balances();
        let lp_total_supply = self.liquidity_token.total_supply;
        let mut send_back = Vec::new();

        for i in 0 .. pool_balances.len() {
            send_back.push(math::complex_div_u128(ONE, math::complex_mul_u128(ONE, pool_balances[i], lp_to_burn), lp_total_supply));
        }

        // Burn LP Tokens
        self.liquidity_token.internal_withdraw(&env::signer_account_id(), lp_to_burn);

        // Transfer Outcomes Tokens
        self.conditional_tokens.transfer_batch(env::current_account_id(), env::signer_account_id(), vec![0; pool_balances.len()], send_back);
    }

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
        self.mint_through_all_outcomes(investment_amount);

        // Tranfer Conditional Token
        self.conditional_tokens.transfer(outcome_idx, env::current_account_id(), env::signer_account_id(), outcome_tokens_to_buy);
    }

    #[payable]
    pub fn sell(&mut self, return_amount: u128, outcome_idx: u64, max_outcome_tokens_to_sell: u128) {
        if self.status != MarketStatus::Running {
            env::panic_str("ERR_MARKET_IS_NOT_RUNNING");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        if return_amount == 0 {
            env::panic_str("ERR_DEPOSIT_SHOULD_NOT_BE_0");
        }

        let outcome_tokens_to_sell = self.calc_sell_amount(return_amount, outcome_idx);

        if outcome_tokens_to_sell > max_outcome_tokens_to_sell {
            env::panic_str("ERR_maximum_sell_amount_exceeded");
        }

        // Tranfer Conditional Token from the signer to the Pool
        self.conditional_tokens.transfer(outcome_idx, env::signer_account_id(), env::current_account_id(), outcome_tokens_to_sell);

        //@TODO Calculate fees

        // Burn Conditional Tokens from the Pool
        self.burn_through_all_outcomes(return_amount);

        // Return Collateral Token
        Promise::new(env::signer_account_id()).transfer(return_amount);
    }

    #[payable]
    pub fn resolve(&mut self, payouts: Vec<u64>) {
        if self.status == MarketStatus::Closed {
            env::panic_str("ERR_MARKET_ALREADY_CLOSED");
        }

        if self.is_resolution_window_expired() {
            env::panic_str("ERR_RESOLUTION_WINDOW_EXPIRED");
        }

        if env::signer_account_id() != self.market.oracle {
            env::panic_str("ERR_ORACLE_ACCOUNT");
        }

        if payouts.len() as u8 != self.market.options {
            env::panic_str("ERR_PAYOUTS_LEN");
        }

        let mut denominator = 0;
        for i in 0 ..  payouts.len() {
            let response = payouts[i];

            denominator += response;
            self.payout_numerators.push(response);
        }

        if denominator == 0 {
            env::panic_str("ERR_DENOMINATOR_CAN_NOT_BE_0");
        }

        self.payout_denominator = denominator;
        self.status = MarketStatus::Closed;
    }

    #[payable]
    pub fn redeem_positions(&mut self) {
        if self.status != MarketStatus::Closed {
            env::panic_str("ERR_MARKET_SHOULD_BE_CLOSED");
        }

        if self.payout_denominator == 0 || self.payout_numerators.len() == 0{
            env::panic_str("ERR_MARKET_NOT_RESOLVED");
        }

        let mut total_payout = 0;

        for i in 0 .. self.payout_numerators.len() {
            let payout_stake = self.conditional_tokens.get_balance_by_account(&(i as u64), &env::signer_account_id());
            if payout_stake > 0 {
                total_payout = total_payout + math::complex_div_u128(
                    ONE, 
                    math::complex_mul_u128(ONE, payout_stake, self.payout_numerators[i] as u128), 
                    self.payout_denominator as u128);
                
                // Burn conditional tokens
                self.conditional_tokens.burn(i as u64, env::signer_account_id(), payout_stake);
            }
        }

        if total_payout > 0 {
            // Redeem Collateral Token
            Promise::new(env::signer_account_id()).transfer(total_payout);
        }

    }
}

near_contract_standards::impl_fungible_token_core!(Market, liquidity_token);
near_contract_standards::impl_fungible_token_storage!(Market, liquidity_token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Market {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
