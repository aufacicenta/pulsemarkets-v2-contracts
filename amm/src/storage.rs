use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, Balance, BorshStorageKey};

pub type OutcomeId = u64;
pub type LiquidityProvider = AccountId;
pub type Price = f64;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub market: MarketData,
    pub dao_account_id: AccountId,
    pub collateral_token_account_id: AccountId,
    pub status: MarketStatus,
    // Keeps track of Outcomes prices and balances
    pub outcome_tokens: LookupMap<OutcomeId, OutcomeToken>,

    // Keeps track of LP pools balance by OutcomeId
    pub lp_pools_balances: LookupMap<OutcomeId, Balance>,
    // Keeps track of LPs balance by AccountId
    pub lp_balances: LookupMap<LiquidityProvider, Balance>,
    // Decimal fee to charge upon a bet
    pub lp_fee: f64,
    /**
     * Decimal to increase or decrease upon purchases, bets or drops.
     * May start at 0.1, but as the price gets closer to 1, we should reduce the ratio so that it never reaches 1
     */
    pub price_ratio: f64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum MarketStatus {
    Pending,
    Published,
    Open,
    Paused,
    Closed,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OutcomeToken {
    pub accounts: LookupMap<AccountId, Balance>, // map `AccountId` to corresponding `Balance` in the market
    pub total_supply: Balance,                   // total supply of this outcome_token
    pub outcome_id: OutcomeId, // the outcome this token represents, used for storage pointers
    pub price: f64,            // a value between 0 & 1
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    OutcomeTokens,
    LiquidityProviderBalances,
    LiquidityProviderPoolsBalances,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub description: String,
    pub info: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub options: Vec<String>,
    pub expiration_date: u64,
    pub resolution_window: u64,
}
