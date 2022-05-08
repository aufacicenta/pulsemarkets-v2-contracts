use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, Balance};
use std::collections::HashMap;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub data: MarketData,
    pub dao_account_id: AccountId,
    pub resolved: bool,
    pub published: bool,
    pub total_funds: Balance,
    pub proposals: Vec<u64>,
    pub winning_options_idx: Option<u64>,
    pub deposits_by_options_idx: LookupMap<AccountId, HashMap<u64, Balance>>,
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
