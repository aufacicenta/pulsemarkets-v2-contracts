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
    pub collateral_token: AccountId,
    pub status: MarketStatus,
    pub fee: u64,
    pub liquidity_token: LiquidityToken,
    pub conditional_tokens: ConditionalTokens,
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

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum MarketStatus {
    Pending,
    Running,
    Paused,
    Closed,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct LiquidityToken {
    pub balances: LookupMap<AccountId, Balance>,
    pub total_balance: Balance,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ConditionalTokens {
    pub balances: LookupMap<u64, LookupMap<AccountId, Balance>>,
    pub total_balances: LookupMap<u64, Balance>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    LiquidityTokenBalances,
    ConditionalTokensBalances,
    ConditionalTokensTotalBalances,
}
