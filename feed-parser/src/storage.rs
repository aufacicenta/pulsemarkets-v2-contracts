use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

pub type Price = f64;
pub type OutcomeId = u64;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
// #[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct SwitchboardFeedParser {}

#[derive(Deserialize, Serialize, Clone)]
pub struct Ix {
    pub address: [u8; 32],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PriceFeedArgs {
    pub ix: Ix,
    pub market_options: Vec<String>,
    pub market_outcome_ids: Vec<OutcomeId>,
    pub price: Price,
    pub predecessor_account_id: Option<AccountId>,
}

#[derive(Serialize, Deserialize)]
pub enum Payload {
    AggregatorReadArgs(PriceFeedArgs),
}
