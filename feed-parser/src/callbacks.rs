use near_sdk::{env, log, near_bindgen, serde_json, PromiseResult::Successful};
use sbv2_near::AggregatorRound;

use crate::storage::*;

#[near_bindgen]
impl SwitchboardFeedParser {
    #[private]
    pub fn on_aggregator_read_callback(&mut self) {
        let maybe_round = env::promise_result(0);

        if let Successful(serialized_round) = maybe_round {
            let round: AggregatorRound = serde_json::from_slice(&serialized_round).unwrap();
            let val: f64 = round.result.try_into().unwrap();

            log!("Feed value: {:?}", val);
        } else {
            log!("ERROR_ON_AGGREGATOR_READ_CALLBACK");
        }
    }
}
