use near_sdk::{
    serde::{Deserialize, Serialize},
    AccountId,
};

pub type Price = f64;
pub type OutcomeId = u64;

#[derive(Deserialize, Serialize, Clone)]
pub struct Ix {
    pub address: [u8; 32],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AbovePriceFeedArgs {
    pub ix: Ix,
    pub market_options: Vec<String>,
    pub market_outcome_ids: Vec<OutcomeId>,
    pub price: Price,
    pub predecessor_account_id: Option<AccountId>,
}

#[derive(Serialize, Deserialize)]
pub enum Payload {
    AggregatorReadArgs(AbovePriceFeedArgs),
}
