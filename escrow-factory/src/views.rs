use near_sdk::{near_bindgen, AccountId};

use crate::storage::*;

#[near_bindgen]
impl EscrowFactory {
    pub fn get_conditional_escrow_contracts_list(&self) -> Vec<AccountId> {
        self.conditional_escrow_contracts.to_vec()
    }

    /// Get number of created Conditional Escrow Contracts.
    pub fn get_conditional_escrow_contracts_count(&self) -> u64 {
        self.conditional_escrow_contracts.len()
    }

    /// Get Conditional Escrow Contracts in paginated view.
    pub fn get_conditional_escrow_contracts(&self, from_index: u64, limit: u64) -> Vec<AccountId> {
        let elements = self.conditional_escrow_contracts.as_vector();

        (from_index..std::cmp::min(from_index + limit, elements.len()))
            .filter_map(|index| elements.get(index))
            .collect()
    }
}
