use near_sdk::{env, near_bindgen, AccountId, Balance};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::storage::*;

#[near_bindgen]
impl Market {
    pub fn deposits_of(&self, payee: &AccountId, proposal_id: &ProposalId) -> Balance {
        match self.deposits_by_proposal.get(payee) {
            Some(entry) => match entry.get(proposal_id) {
                Some(balance) => *balance,
                None => 0,
            },
            None => 0,
        }
    }

    pub fn get_market_data(&self) -> MarketData {
        self.data.clone()
    }

    pub fn get_proposals(&self) -> Vec<String> {
        self.proposals.clone()
    }

    pub fn is_published(&self) -> bool {
        self.published
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub fn is_market_expired(&self) -> bool {
        self.data.expiration_date < env::block_timestamp().try_into().unwrap()
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.data.expiration_date + self.data.resolution_window
            < env::block_timestamp().try_into().unwrap()
    }

    pub fn get_random_proposal_id(&self) -> ProposalId {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        return rand_string;
    }
}
