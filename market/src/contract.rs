use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, Promise};
use std::default::Default;

use crate::consts::*;
use crate::storage::*;

impl Default for Market {
    fn default() -> Self {
        env::panic_str("Market should be initialized before usage")
    }
}

#[near_bindgen]
impl Market {
    #[init]
    pub fn new(description: String, dao_account_id: AccountId) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            description,
            dao_account_id,
            market_options: Vec::new(),
        }
    }

    pub fn process_market_options(&self, market_options: Vec<String>) -> bool {
        for market_option in market_options {
            self.create_proposal(market_option);
        }

        return true;
    }

    fn create_proposal(&self, market_option: String) -> bool {
        // @TODO this add_proposal cross-contract promise should return an u64 ID of the new proposal
        // @TODO store the proposal_id in Market.market_option.proposal_id
        let dao_proposal_promise = Promise::new(self.dao_account_id.clone()).function_call(
            "add_proposal".to_string(),
            json!({
                "proposal": {
                    // @TODO interpolate the proposal description as "[market_id]: [market_option from user input]"
                    "description": market_option,
                    "kind": {
                        "FunctionCall": {
                            // @TODO a ConditionalEscrow must exist before adding the proposal
                            "receiver_id": "CONDITIONAL_ESCROW_ID",
                            "actions": [{
                                // @TODO delegate_funds should be called only by the Sputnik2 DAO contract
                                // @TODO delegate_funds should be called only after the proposal expires or it's resoluted
                                "method_name": "delegate_funds",
                                "args": {},
                                "deposit": 0, // @TODO
                                "gas": 0, // @TODO
                            }]
                        }
                    }
                }
            })
            .to_string()
            .into_bytes(),
            0,
            GAS_FOR_CREATE_DAO_PROPOSAL,
        );

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_proposal_callback".to_string(),
            json!({}).to_string().into_bytes(),
            0,
            GAS_FOR_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        dao_proposal_promise.then(callback);

        return true;
    }
}
