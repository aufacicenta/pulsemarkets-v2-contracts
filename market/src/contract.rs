use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen};
use near_sdk::{AccountId, Promise};
use near_sdk::json_types::{Base64VecU8};
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
            closed: false,
            proposals: Vec::new(),
        }
    }

    pub fn get_market_data(&self) -> MarketData {
        self.market.clone()
    }

    pub fn get_proposals(&self) -> Vec<u64> {
        self.proposals.clone()
    }

    pub fn is_published(&self) -> bool {
        self.published
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn is_market_expired(&self) -> bool {
        self.market.expiration_date < env::block_timestamp().try_into().unwrap()
    }

    pub fn is_resolution_window_expired(&self) -> bool {
        self.market.expiration_date + self.market.resolution_window < env::block_timestamp().try_into().unwrap()
    }

    #[payable]
    pub fn publish_market(&mut self) -> Promise {
        if self.published {
            env::panic_str("ERR_MARKET_ALREADY_PUBLISHED");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        //@TODO Research for an alternative to not create an empty Promise
        let mut promises: Promise = Promise::new(self.dao_account_id.clone());
        let mut count = 0;

        for market_option in &self.market.options {
            let args = Base64VecU8(json!({"response": count}).to_string().into_bytes());
            let new_proposal = Promise::new(self.dao_account_id.clone()).function_call(
                "add_proposal".to_string(),
                json!({
                    "proposal": {
                        "description": format!("{}:\n{}\nR: {}$$$$$$$$ProposeCustomFunctionCall",
                            env::current_account_id().to_string(),
                            self.market.description,
                            market_option),
                        "kind": {
                            "FunctionCall": {
                                "receiver_id": env::current_account_id().to_string(),
                                "actions": [{
                                    "args": args,
                                    "deposit": "0", // @TODO
                                    "gas": "150000000000000", // @TODO
                                    "method_name": "resolve",
                                }]
                            }
                        }
                    }
                }).to_string().into_bytes(),
                BALANCE_PROPOSAL_BOND,
                GAS_CREATE_DAO_PROPOSAL,
            );
            
            promises = promises.and(new_proposal);
            count = count + 1;
        }

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_proposal_callback".to_string(),
            json!({})
                .to_string()
                .into_bytes(),
            0,
            GAS_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        promises.then(callback)
    }

    #[payable]
    pub fn bet(&mut self) {
        if !self.published {
            env::panic_str("ERR_MARKET_IS_NOT_PUBLISHED");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        //@TODO A user makes a bet. Deposit funds
    }

    pub fn resolve(&mut self, response: u64) {
        log!("response {}",
            response
        );

        if self.resolved {
            env::panic_str("ERR_MAKERT_ALREADY_RESOLVED");
        }

        if env::signer_account_id() != self.dao_account_id {
            env::panic_str("ERR_DAO_ACCOUNT");
        }

        if response >= self.market.options.len() as u64 {
            env::panic_str("ERR_RESPONSE_INDEX");
        }

        if self.is_resolution_window_expired() {
            env::panic_str("ERR_RESOLUTION_WINDOW_EXPIRED");
        }

        //@TODO Resolve Marter. Delegate Funds
        self.resolved = true;
    }

    pub fn close(&mut self) {
        if self.closed {
            env::panic_str("ERR_MAKERT_ALREADY_CLOSED");
        }

        if !self.is_resolution_window_expired() {
            env::panic_str("ERR_RESOLUTION_WINDOW_SHOULD_BE_EXPIRED");
        }

        //@TODO Close market when no solution is found. Return funds
        self.closed = true;
    }
}
