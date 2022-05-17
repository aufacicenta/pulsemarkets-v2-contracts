use near_sdk::collections::LookupMap;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Promise};
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
            losing_balance: 0,
            winning_balance: 0,
            total_funds: 0,
            winning_options_idx: 0,
            totals_by_options_idx: LookupMap::new(StorageKeys::Totals),
            deposits_by_options_idx: LookupMap::new(StorageKeys::Deposits),
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

        let mut promise: Promise = Promise::new(self.dao_account_id.clone());
        let mut market_options_idx = 0;

        for market_option in &self.market.options {
            let args = Base64VecU8(
                json!({ "options_idx": market_options_idx })
                    .to_string()
                    .into_bytes(),
            );

            promise = promise.function_call(
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
                })
                .to_string()
                .into_bytes(),
                BALANCE_PROPOSAL_BOND,
                GAS_CREATE_DAO_PROPOSAL,
            );

            market_options_idx += 1;
        }

        let callback = Promise::new(env::current_account_id()).function_call(
            "on_create_proposals_callback".to_string(),
            json!({}).to_string().into_bytes(),
            0,
            GAS_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        promise.then(callback)
    }

    fn get_options_by_account(&self, account_id: &AccountId) -> LookupMap<u64, Balance> {
        let options = self
            .deposits_by_options_idx
            .get(account_id)
            .unwrap_or_else(|| {
                LookupMap::new(StorageKeys::SubUserOptions {
                    account_hash: env::sha256(account_id.as_bytes()),
                })
            });
        options
    }

    #[payable]
    pub fn bet(&mut self, options_idx: u64) {
        if !self.published {
            env::panic_str("ERR_MARKET_IS_NOT_PUBLISHED");
        }

        if self.is_market_expired() {
            env::panic_str("ERR_MARKET_EXPIRED");
        }

        if env::attached_deposit() == 0 {
            env::panic_str("ERR_DEPOSIT_SHOULD_NOT_BE_0");
        }

        if options_idx >= self.market.options.len() as u64 {
            env::panic_str("ERR_OPTION_INDEX");
        }

        // @TODO attached_deposit could also be an NEP141 Collateral Token
        let amount = env::attached_deposit();

        // Update amount by account
        let payee = env::signer_account_id();
        let current_balance_by_account = self.deposits_by_account(&payee, &options_idx);
        let new_balance_by_account = &(current_balance_by_account.wrapping_add(amount));

        let mut options_by_account = self.get_options_by_account(&payee);
        options_by_account.insert(&options_idx, new_balance_by_account);

        self.deposits_by_options_idx
            .insert(&payee, &options_by_account);

        // Update amount totals by option
        let current_balance_by_option = self.deposits_by_option(&options_idx);
        let new_balance_by_option = &(current_balance_by_option.wrapping_add(amount));

        self.totals_by_options_idx
            .insert(&options_idx, new_balance_by_option);

        // Update grand total
        self.total_funds = self.total_funds.wrapping_add(amount);
    }

    pub fn resolve(&mut self, options_idx: u64) {
        log!("options_idx: {}", options_idx);

        if self.resolved {
            env::panic_str("ERR_MARKET_ALREADY_RESOLVED");
        }

        if self.is_resolution_window_expired() {
            env::panic_str("ERR_RESOLUTION_WINDOW_EXPIRED");
        }

        if options_idx >= self.market.options.len() as u64 {
            env::panic_str("ERR_OPTION_INDEX");
        }

        if env::signer_account_id() != self.dao_account_id {
            env::panic_str("ERR_DAO_ACCOUNT");
        }

        // Calculate final balances
        for idx in 0 .. self.market.options.len() {
            let amount = self.deposits_by_option(&(idx as u64));

            if idx as u64 == self.winning_options_idx {
                self.winning_balance = amount;
            } else {
                self.losing_balance = self.losing_balance.wrapping_add(amount);
            }
        }

        self.winning_options_idx = options_idx;
        self.resolved = true;
    }

    pub fn withdraw(&mut self) {
        if !self.is_resolution_window_expired() {
            env::panic_str("ERR_RESOLUTION_WINDOW_STILL_OPEN");
        }

        if !self.resolved {
            env::panic_str("ERR_MARKET_NOT_RESOLVED");
        }

        if self
            .deposits_by_options_idx
            .get(&env::signer_account_id())
            .is_none()
        {
            env::panic_str("ERR_ACCOUNT_DID_NOT_BET");
        }

        let payee = env::signer_account_id();

        let mut options_by_account = self.get_options_by_account(&payee);
        let bet = match options_by_account.get(&self.winning_options_idx) {
            Some(bet) => bet,
            None => 0,
        };

        if bet == 0 {
            env::panic_str("ERR_WITHDRAW_ACCOUNT_BET_IS_0");
        }

        let payment = self.losing_balance * bet / self.winning_balance + bet;

        // @CHECK subtract the deposits of the player so that they can't withdraw again
        // Should we keep a record of withdrawals? If so, we need a new struct to track it.
        options_by_account.insert(&self.winning_options_idx, &0);

        Promise::new(payee.clone()).transfer(payment);

        self.total_funds = self.total_funds.wrapping_sub(payment);
    }
}
