use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

#[derive(BorshDeserialize, BorshSerialize)]
struct CreateMarketArgs {
    market_options: Vec<String>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MarketFactory {
    markets: UnorderedMap<String, Vec<String>>,
}

impl Default for MarketFactory {
    fn default() -> Self {
        env::panic_str("MarketFactory should be initialized before usage")
    }
}

#[near_bindgen]
impl MarketFactory {
    #[init]
    pub fn new() -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        Self {
            markets: UnorderedMap::new(b"s".to_vec()),
        }
    }

    pub fn create_market(&mut self, market_options: Vec<String>) -> String {
        let market_id = self.get_random_market_id();

        self.markets.insert(&market_id, &market_options);

        return market_id;
    }
}

impl MarketFactory {
    fn get_random_market_id(&self) -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        return rand_string;
    }
}
