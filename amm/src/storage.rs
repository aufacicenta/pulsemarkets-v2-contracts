use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, BorshStorageKey,
};
use shared::{OutcomeId, Price};

pub type Timestamp = i64;
pub type WrappedBalance = u128;

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
    // Market metadata
    pub market: MarketData,
    // NEP141 token metadata
    pub collateral_token: CollateralToken,
    // Market fees metadata
    pub fees: Fees,
    // MArket resolution metadata
    pub resolution: Resolution,
    // Market management account ids metadata
    pub management: Management,
    // Keeps track of Outcomes prices and balances
    pub outcome_tokens: LookupMap<OutcomeId, OutcomeToken>,
    // If self.price is set, this is a binary yes/no price market — used on self.aggregator_read
    pub price: Option<Pricing>,
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
    // keep the number of accounts with positive balance. Use for calculating the price_ratio
    pub accounts_length: u64,
    // total supply of this outcome_token
    pub total_supply: WrappedBalance,
    // the outcome this token represents, used for storage pointers
    pub outcome_id: OutcomeId,
    // can mint more tokens
    pub is_active: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone)]
pub struct CollateralToken {
    pub id: AccountId,
    pub balance: WrappedBalance,
    pub decimals: u8,
    pub fee_balance: WrappedBalance,
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub struct Fees {
    #[serde(skip_serializing, skip_deserializing)]
    pub staking_fees: Option<LookupMap<AccountId, String>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub market_creator_fees: Option<LookupMap<AccountId, String>>,
    pub claiming_window: Option<Timestamp>,
    // Decimal fee to charge upon a bet
    pub fee_ratio: WrappedBalance,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct Resolution {
    // Time to free up the market
    pub window: Timestamp,
    // When the market is resolved, set only by fn resolve
    pub resolved_at: Option<Timestamp>,
    // Unit8ByteArray with the immutable Aggregator address, this is the "is_owner" condition to resolve the market
    pub ix: Ix,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Management {
    // Gets sent fees when claiming window is open
    pub dao_account_id: AccountId,
    // Gets fees for creating a market
    pub market_creator_account_id: AccountId,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct Pricing {
    pub value: Price,
    pub base_currency_symbol: String,
    pub target_currency_symbol: String,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    OutcomeTokens,
    StakingFees,
    MarketCreatorFees,
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

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct Ix {
    pub address: [u8; 32],
}
