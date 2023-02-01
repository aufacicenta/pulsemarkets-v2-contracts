use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen,
    serde::{Deserialize, Serialize},
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct SwitchboardFeedParser {}

#[derive(Deserialize, Serialize)]
pub struct Ix {
    pub address: [u8; 32],
}
