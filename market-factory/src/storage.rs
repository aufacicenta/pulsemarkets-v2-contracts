use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedSet,
    near_bindgen, AccountId,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MarketFactory {
    pub markets: UnorderedSet<AccountId>,
    pub ft_storage_deposit_called: bool,
}
