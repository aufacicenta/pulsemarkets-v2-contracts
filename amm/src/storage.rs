use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, BorshStorageKey,
};

pub type OutcomeId = u64;
pub type Timestamp = i64;
pub type LiquidityProvider = AccountId;
pub type Price = f32;
pub type PriceRatio = f32;
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
    // Keep track of the timezone
    pub utc_offset: i8,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    pub market: MarketData,
    pub collateral_token: CollateralToken,
    pub dao_account_id: AccountId,
    pub staking_token_account_id: AccountId,
    pub market_creator_account_id: AccountId,
    pub market_publisher_account_id: Option<AccountId>,
    // Keeps track of Outcomes prices and balances
    pub outcome_tokens: LookupMap<OutcomeId, OutcomeToken>,
    // Decimal fee to charge upon a bet
    pub fee_ratio: WrappedBalance,
    // When the market is published
    pub published_at: Option<Timestamp>,
    // When the market is published
    pub resolved_at: Option<Timestamp>,
    // Time to free up the market
    pub resolution_window: Option<Timestamp>,
    // Time to free up the market
    // Maps to check if fee has been paid for AccountId
    pub fees: Fees,
}

#[derive(Serialize, Deserialize)]
pub enum SetPriceOptions {
    Increase,
    Decrease,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
pub struct OutcomeToken {
    // map `AccountId` to corresponding `Balance` in the market
    #[serde(skip_serializing)]
    pub balances: UnorderedMap<AccountId, WrappedBalance>,
    // vec! of price history by block_timestamp
    pub price_history: Vec<PriceHistory>,
    // keep the number of accounts with positive balance. Use for calculating the price_ratio
    pub accounts_length: u64,
    // total supply of this outcome_token
    pub total_supply: WrappedBalance,
    // the outcome this token represents, used for storage pointers
    pub outcome_id: OutcomeId,
    // a value between 0 & 1
    pub price: WrappedBalance,
    // can mint more tokens
    pub is_active: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
pub struct CollateralToken {
    pub id: AccountId,
    pub balance: WrappedBalance,
    pub decimals: u8,
    pub fee_balance: WrappedBalance,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Fees {
    pub staking_fees: LookupMap<AccountId, String>,
    pub market_creator_fees: LookupMap<AccountId, String>,
    pub market_publisher_fees: LookupMap<AccountId, String>,
    pub claiming_window: Option<Timestamp>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
pub struct PriceHistory {
    pub timestamp: u64,
    pub price: WrappedBalance,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    OutcomeTokens,
    StakingFees,
    MarketCreatorFees,
    MarketPublisherFees,
}

#[derive(Serialize, Deserialize)]
pub struct BuyArgs {
    // id of the outcome that shares are to be purchased from
    pub outcome_id: OutcomeId,
}

#[derive(Serialize, Deserialize)]
pub enum Payload {
    BuyArgs(BuyArgs),
}
