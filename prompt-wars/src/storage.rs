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
    // The IPFS reference-image hash of the expected prompts
    pub image_uri: String,
    // Datetime nanos: the market is open
    pub starts_at: Timestamp,
    // Datetime nanos: the market is closed
    pub ends_at: Timestamp,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Market {
    // Market metadata
    pub market: MarketData,
    // NEP141 token metadata
    pub collateral_token: CollateralToken,
    // MArket resolution metadata
    pub resolution: Resolution,
    // Market management account ids metadata
    pub management: Management,
    // Keeps track of Outcomes prices and balances
    pub outcome_tokens: LookupMap<AccountId, OutcomeToken>,
    // Market fees metadata
    pub fees: Fees,
}

#[derive(Serialize, Deserialize)]
pub enum SetPriceOptions {
    Increase,
    Decrease,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
pub struct OutcomeToken {
    // the account id of the outcome_token
    pub player_id: AccountId,
    // the outcome value, in this case, the prompt submitted to the competition
    pub value: String,
    // map `AccountId` to corresponding `Balance` in the market
    #[serde(skip_serializing)]
    pub balances: UnorderedMap<AccountId, WrappedBalance>,
    // keep the number of accounts with positive balance. Use for calculating the price_ratio
    pub accounts_length: u64,
    // total supply of this outcome_token
    pub total_supply: WrappedBalance,
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

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct Resolution {
    // Time to free up the market
    pub window: Timestamp,
    // When the market is resolved, set only by fn resolve
    pub resolved_at: Option<Timestamp>,
    // Unit8ByteArray with the immutable Aggregator address, this is the "is_owner" condition to resolve the market
    pub ix: Ix,
    // The value to compare the outcomes tokens value with
    pub source: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Management {
    // Gets sent fees when claiming window is open
    pub dao_account_id: AccountId,
    // Gets back the storage deposit upon self-destruction
    pub market_creator_account_id: AccountId,
    // Set at initialization, the market may be destroyed by the creator to claim the storage deposit
    pub self_destruct_window: Timestamp,
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub struct Fees {
    // Price to charge when creating an outcome token
    pub price: WrappedBalance,
    // Decimal fee to charge upon a bet
    pub fee_ratio: WrappedBalance,
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
    pub player_id: AccountId,
}

#[derive(Serialize, Deserialize)]
pub struct CreateOutcomeTokenArgs {
    // the account id of the outcome_token
    pub player_id: AccountId,
    // the outcome value, in this case, the prompt submitted to the competition
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub enum Payload {
    BuyArgs(BuyArgs),
    CreateOutcomeTokenArgs(CreateOutcomeTokenArgs),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct Ix {
    pub address: [u8; 32],
}
