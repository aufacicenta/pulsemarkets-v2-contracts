use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, Balance, Gas, Promise, PromiseResult};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::default::Default;

const GAS_FOR_CREATE_DAO_PROPOSAL: Gas = Gas(90_000_000_000_000);
const GAS_FOR_CREATE_DAO_PROPOSAL_CALLBACK: Gas = Gas(2_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize)]
struct CreateMarketArgs {
    market_options: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
struct MarketOption {
    proposal_id: u8,
}

#[derive(BorshDeserialize, BorshSerialize)]
struct Market {
    description: String,
    market_options: Vec<MarketOption>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MarketFactory {
    markets: LookupMap<String, Market>,
    dao_account_id: AccountId,
}

impl Default for MarketFactory {
    fn default() -> Self {
        env::panic_str("MarketFactory should be initialized before usage")
    }
}

#[near_bindgen]
impl MarketFactory {
    #[init]
    pub fn new(dao_account_id: AccountId) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            markets: LookupMap::new(b"s".to_vec()),
            dao_account_id,
        }
    }

    pub fn create_market(&mut self, description: String, market_options: Vec<String>) -> String {
        let market_id = self.get_random_market_id();

        self.markets.insert(
            &market_id,
            &Market {
                description,
                market_options: Vec::new(),
            },
        );

        // @TODO create ConditionalEscrow using the Factory: https://github.com/aufacicenta/near.holdings/blob/master/rust-escrow/src/lib.rs#L59

        self.process_market_options(&market_id, market_options);

        return market_id;
    }

    fn process_market_options(&self, market_id: &String, market_options: Vec<String>) -> bool {
        for market_option in market_options {
            self.create_proposal(&market_id, market_option);
        }

        return true;
    }

    fn create_proposal(&self, market_id: &String, market_option: String) -> bool {
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
            json!({ "market_id": market_id }).to_string().into_bytes(),
            0,
            GAS_FOR_CREATE_DAO_PROPOSAL_CALLBACK,
        );

        dao_proposal_promise.then(callback);

        return true;
    }

    fn get_random_market_id(&self) -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        return rand_string;
    }

    #[private]
    pub fn on_create_proposal_callback(&mut self, market_id: String) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(proposal_id) => {
                // @TODO get the market by id, its market_options Vec and push the proposal_id
                self.markets
                    .get(&market_id)
                    .market_options
                    .push(proposal_id);

                return true;
            }
            _ => env::panic_str("ERR_CREATE_DAO_PROPOSAL_UNSUCCESSFUL"),
        }
    }
}
