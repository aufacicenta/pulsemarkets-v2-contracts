use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MarketFactory {
    pub markets: Vec<AccountId>,
    pub escrow_factory_account_id: AccountId,
    pub dao_account_id: AccountId,
}
