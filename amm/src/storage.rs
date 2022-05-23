use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    json_types::{U128, U64},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, Balance, BorshStorageKey,
};
use std::fmt;

pub type OutcomeId = u64;
pub type Timestamp = u64;
pub type LiquidityProvider = AccountId;
pub type Price = f64;
pub type PriceRatio = f64;
pub type WrappedTimestamp = U64;
pub type WrappedBalance = U128;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub description: String,
    pub info: String,
    pub category: Option<String>,
    pub options: Vec<String>,
    // Datetime nanos: the market is open
    pub starts_at: Timestamp,
    // Datetime nanos: the market is closed
    pub ends_at: Timestamp,
    pub resolution_window: Timestamp,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub market: MarketData,
    pub dao_account_id: AccountId,
    pub collateral_token_account_id: AccountId,
    pub status: MarketStatus,
    // Keeps track of Outcomes prices and balances
    pub outcome_tokens: LookupMap<OutcomeId, OutcomeToken>,
    // Decimal fee to charge upon a bet
    pub lp_fee: f64,
    // Decimal to increase or decrease upon purchases, bets or drops.
    // May start at 0.1, but as the price gets closer to 1, we should reduce the ratio so that it never reaches 1
    pub price_ratio: PriceRatio,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub enum MarketStatus {
    Pending,
    Published,
    Resolved,
}

impl std::fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MarketStatus::Pending => write!(f, "Pending"),
            MarketStatus::Published => write!(f, "Published"),
            MarketStatus::Resolved => write!(f, "Resolved"),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OutcomeToken {
    // map `AccountId` to corresponding `Balance` in the market
    pub balances: LookupMap<AccountId, Balance>,
    // keep track of LP balances on mint and burn
    pub lp_balances: UnorderedMap<AccountId, Balance>,
    // total supply of this outcome_token
    pub total_supply: Balance,
    // the outcome this token represents, used for storage pointers
    pub outcome_id: OutcomeId,
    // a value between 0 & 1
    pub price: f64,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    OutcomeTokens,
}

#[derive(Serialize, Deserialize)]
pub struct AddLiquidityArgs {
    // id of the outcome to add liquidity to
    pub outcome_id: OutcomeId,
}

#[derive(Serialize, Deserialize)]
pub struct BuyArgs {
    // id of the outcome that shares are to be purchased from
    pub outcome_id: OutcomeId,
    // the minimum amount of share tokens the user expects out, this is to prevent slippage
    // pub min_shares_out: WrappedBalance,
}

#[derive(Serialize, Deserialize)]
pub enum Payload {
    BuyArgs(BuyArgs),
    AddLiquidityArgs(AddLiquidityArgs),
}
