use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, Balance, BorshStorageKey};

pub type MarketOptionIndex = u64;
pub type LiquidityProvider = AccountId;
pub type Price = f64;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub market: MarketData,
    pub dao_account_id: AccountId,
    pub collateral_token_account_id: AccountId,
    pub resolved: bool,
    pub published: bool,
    // Keeps track of Market Options prices
    pub options_prices: LookupMap<MarketOptionIndex, Price>,
    // Keeps track of LP pools balance by MarketOptionIndex
    pub lp_pools_balances: LookupMap<MarketOptionIndex, Balance>,
    // Keeps track of LPs balance by MarketOptionIndex
    pub lp_balances: LookupMap<LiquidityProvider, Balance>,
    // Decimal fee to charge upon a bet
    pub lp_fee: f64,
    // Decimal to increase or decrease upon purchases, bets or drops
    pub price_ratio: f64,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    MarketOptionsPrices,
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
