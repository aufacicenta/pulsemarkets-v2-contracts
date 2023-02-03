use near_sdk::{env, ext_contract, log, near_bindgen, serde_json, PromiseResult::Successful};
use sbv2_near::AggregatorRound;
use shared::{OutcomeId, Price};

use crate::storage::*;

#[ext_contract(ext_market)]
trait Market {
    fn resolve(&mut self, outcome_id: u64, ix: Ix);
}

#[near_bindgen]
impl SwitchboardFeedParser {
    #[private]
    pub fn on_internal_above_price_feed_read_callback(
        &self,
        payload: AbovePriceFeedArgs,
    ) -> OutcomeId {
        let maybe_round = env::promise_result(0);

        if let Successful(serialized_round) = maybe_round {
            let round: AggregatorRound = serde_json::from_slice(&serialized_round).unwrap();
            let result: Price = round.result.try_into().unwrap();

            log!(
                "on_internal_above_price_feed_read_callback.result: {:?}",
                result
            );

            let predecessor_account_id = payload
                .predecessor_account_id
                .expect("ERR_PREDECESSOR_ACCOUNT_ID_NOT_SET");

            log!(
                "on_internal_above_price_feed_read_callback.predecessor_account_id: {}",
                predecessor_account_id
            );

            let mut winning_outcome_id = payload.market_outcome_ids[0];

            if payload.price < result {
                winning_outcome_id = payload.market_outcome_ids[1];
            }

            // @TODO add a callback for this promise in case it errors
            ext_market::ext(predecessor_account_id).resolve(winning_outcome_id, payload.ix);

            return winning_outcome_id;
        }

        env::panic_str("ERROR_ON_AGGREGATOR_READ_CALLBACK");
    }
}
