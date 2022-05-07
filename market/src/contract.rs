use near_sdk::collections::LookupMap;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen, AccountId, Promise};
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
    pub fn new(data: MarketData, dao_account_id: AccountId) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            data,
            dao_account_id,
            resolved: false,
            published: false,
            proposals: Vec::new(),
            total_funds: 0,
            winning_proposal_id: None,
            deposits_by_proposal: LookupMap::new(b"d"),
        }
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

        for market_option in &self.data.options {
            let proposal_id = self.get_random_proposal_id();

            self.proposals.push(proposal_id);

            let args = Base64VecU8(
                json!({ "proposal_id": proposal_id })
                    .to_string()
                    .into_bytes(),
            );

            let new_proposal = Promise::new(self.dao_account_id.clone()).function_call(
                "add_proposal".to_string(),
                json!({
                    "proposal": {
                        "description": format!("{}:\n{}\nR: {}$$$$$$$$ProposeCustomFunctionCall",
                            env::current_account_id().to_string(),
                            self.data.description,
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
                })
                .to_string()
                .into_bytes(),
                BALANCE_PROPOSAL_BOND,
                GAS_CREATE_DAO_PROPOSAL,
            );

            promises = promises.and(new_proposal);
            count = count + 1;
        }

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_proposals_callback".to_string(),
            json!({}).to_string().into_bytes(),
            0,
            GAS_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        promises.then(callback)
    }

    #[payable]
    pub fn bet(&mut self, proposal_id: ProposalId) {
        if !self.published {
            env::panic_str("ERR_MARKET_IS_NOT_PUBLISHED");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        if env::attached_deposit() == 0 {
            env::panic_str("ERR_DEPOSIT_SHOULD_NOT_BE_0");
        }

        // @TODO attached_deposit could also be an NEP141 Collateral Token
        let amount = env::attached_deposit();
        let payee = env::signer_account_id();
        let current_balance = self.deposits_of(&payee, &proposal_id);
        let new_balance = &(current_balance.wrapping_add(amount));

        match self.deposits_by_proposal.get(&payee) {
            Some(mut entry) => entry.insert(proposal_id, *new_balance),
            None => env::panic_str("ERR_WHILE_UPDATING_BALANCE"),
        };

        self.total_funds = self.total_funds.wrapping_add(amount);
    }

    pub fn resolve(&mut self, proposal_id: ProposalId) {
        log!("proposal_id: {}", proposal_id);

        if self.resolved {
            env::panic_str("ERR_MARKET_ALREADY_RESOLVED");
        }

        if env::signer_account_id() != self.dao_account_id {
            env::panic_str("ERR_DAO_ACCOUNT");
        }

        self.winning_proposal_id = Some(proposal_id);
        self.resolved = true;
    }

    pub fn withdraw(&mut self) {
        if !self.is_resolution_window_expired() {
            env::panic_str("ERR_RESOLUTION_WINDOW_STILL_OPEN");
        }

        if !self.resolved {
            env::panic_str("ERR_MARKET_NOT_RESOLVED");
        }

        let payee = env::signer_account_id();

        // @TODO iterate over deposits to get the amount deposited by the payee to each proposal_id
        // @TODO if the player has balance on a proposal_id, initialize its payment
        // @TODO calculate the proportion of additional payment the player is owed from the deposits of the losing proposal_ids, not including their own losing bets
        let mut payment = 0;
        match self.deposits_by_proposal.get(&payee) {
            Some(entry) => {
                for (proposal_id, balance) in entry.iter() {
                    payment = *balance
                }
            }
            None => payment = 0,
        };

        Promise::new(payee.clone()).transfer(payment);
        // @TODO subtract the deposits of the player so that they can't withdraw again
        // self.deposits.insert(&payee, &0);

        self.total_funds = self.total_funds.wrapping_sub(payment);
    }
}
