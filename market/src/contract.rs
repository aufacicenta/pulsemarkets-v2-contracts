use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen};
use near_sdk::{AccountId, Promise, PromiseOrValue};
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
    pub fn new(market: MarketData, dao_account_id: AccountId) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            market,
            dao_account_id,
            resolved: false,
            published: false,
            proposals: Vec::new(),
        }
    }

    pub fn get_data_data(&self) -> MarketData{
        self.market.clone()
    }

    pub fn is_published(&self) -> bool{
        self.published
    }

    pub fn is_resolved(&self) -> bool{
        self.resolved
    }

    pub fn publish_market(&mut self) -> Promise {
        let mut promises: Promise = Promise::new(self.dao_account_id.clone());

        for market_option in &self.market.options {
            let new_proposal = Promise::new(self.dao_account_id.clone()).function_call(
                "add_proposal".to_string(),
                json!({
                    "proposal": {
                        // @TODO interpolate the proposal description as "[market_id]: [market_option from user input]"
                        "description": "hola$$$$https://www.google.com.gt/$$$$ProposeCustomFunctionCall",
                        "kind": {
                            "FunctionCall": {
                                // @TODO a ConditionalEscrow must exist before adding the proposal
                                "receiver_id": "pulse.testnet",
                                "actions": [{
                                    // @TODO delegate_funds should be called only by the Sputnik2 DAO contract
                                    // @TODO delegate_funds should be called only after the proposal expires or it's resoluted
                                    "method_name": "delegate_funds",
                                    "args": {},
                                    "deposit": "0", // @TODO
                                    "gas": "150000000000000", // @TODO
                                }]
                            }
                        }
                    }
                }).to_string().into_bytes(),
                0,
                GAS_FOR_CREATE_DAO_PROPOSAL,
            );
            
            promises = promises.and(new_proposal);
            
            /*if res == false {
                promises = promises.and(new_proposal);
            }else{
                promises = new_proposal;
                res = true;
            }*/
        }

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_proposal_callback".to_string(),
            json!({})
                .to_string()
                .into_bytes(),
            0,
            GAS_FOR_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        promises.then(callback)
    }
}
