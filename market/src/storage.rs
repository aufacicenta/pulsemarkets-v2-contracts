use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Market {
    pub description: String,
    pub dao_account_id: AccountId,
    pub market_options: Vec<u64>,
}
