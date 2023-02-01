use near_sdk::{
    env::{self, predecessor_account_id},
    ext_contract, log, near_bindgen, serde_json,
    PromiseResult::Successful,
};
use sbv2_near::AggregatorRound;

use crate::storage::*;

#[ext_contract(ext_market)]
trait Market {
    fn resolve(&mut self, outcome_id: u64, ix: Ix);
}

#[near_bindgen]
impl SwitchboardFeedParser {
    #[private]
    pub fn on_aggregator_read_callback(&self, payload: PriceFeedArgs) {
        let maybe_round = env::promise_result(0);

        if let Successful(serialized_round) = maybe_round {
            let round: AggregatorRound = serde_json::from_slice(&serialized_round).unwrap();
            let val: Price = round.result.try_into().unwrap();

            log!("Feed value: {:?}", val);

            let predecessor_account_id = payload
                .predecessor_account_id
                .expect("ERR_PREDECESSOR_ACCOUNT_ID_NOT_SET");

            let winning_outcome_id = 0;

            let market_resolve_promise =
                ext_market::ext(predecessor_account_id).resolve(winning_outcome_id, payload.ix);
        } else {
            log!("ERROR_ON_AGGREGATOR_READ_CALLBACK");
        }
    }
}
