use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    json_types::U64,
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, BorshStorageKey,
};

pub type OutcomeId = u64;
pub type Timestamp = u64;
pub type LiquidityProvider = AccountId;
pub type Price = f32;
pub type PriceRatio = f32;
pub type WrappedTimestamp = U64;
pub type WrappedBalance = f32;
pub type Weight = f32;

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
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub market: MarketData,
    pub dao_account_id: AccountId,
    pub collateral_token_account_id: AccountId,
    // Keeps track of Outcomes prices and balances
    pub outcome_tokens: LookupMap<OutcomeId, OutcomeToken>,
    // Decimal fee to charge upon a bet
    pub fee_ratio: WrappedBalance,
    // When the market is published
    pub published_at: Option<Timestamp>,
    // When the market is published
    pub resolved_at: Option<Timestamp>,
    // Time to free up the market
    pub resolution_window: Timestamp,
}

pub enum SetPriceOptions {
    Increase,
    Decrease,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
pub struct OutcomeToken {
    // map `AccountId` to corresponding `Balance` in the market
    #[serde(skip_serializing)]
    pub balances: UnorderedMap<AccountId, WrappedBalance>,
    // keep the number of accounts with positive balance. Use for calculating the price_ratio
    pub accounts_length: u64,
    // total supply of this outcome_token
    pub total_supply: WrappedBalance,
    // the outcome this token represents, used for storage pointers
    pub outcome_id: OutcomeId,
    // a value between 0 & 1
    pub price: WrappedBalance,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    OutcomeTokens,
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
}
