use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, Balance, BorshStorageKey};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub market: MarketData,
    pub dao_account_id: AccountId,
    pub resolved: bool,
    pub published: bool,
    pub losing_balance: Balance,
    pub winning_balance: Balance,
    pub total_funds: Balance,
    pub winning_options_idx: u64,
    pub totals_by_options_idx: LookupMap<u64, Balance>,
    pub deposits_by_options_idx: LookupMap<AccountId, LookupMap<u64, Balance>>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Totals,
    Deposits,
    SubUserOptions { account_hash: Vec<u8> },
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
