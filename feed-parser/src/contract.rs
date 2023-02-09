use near_sdk::{env, ext_contract, near_bindgen, serde_json, Promise, ONE_YOCTO};

use sbv2_near::SWITCHBOARD_PROGRAM_ID;

use crate::consts::*;
use crate::storage::*;

#[ext_contract(ext_self)]
trait Callbacks {
    fn on_internal_above_price_feed_read_callback(&self, payload: AbovePriceFeedArgs);
}

#[near_bindgen]
impl SwitchboardFeedParser {
    pub fn aggregator_read(&self, msg: String) -> Promise {
        let payload: Payload =
            serde_json::from_str(&msg).expect("ERR_AGGREGATOR_READ_INVALID_PAYLOAD");

        match payload {
            Payload::AggregatorReadArgs(payload) => {
                self.internal_above_price_feed_read(AbovePriceFeedArgs {
                    predecessor_account_id: Some(env::predecessor_account_id()),
                    ..payload
                })
            }
        }
    }
}

impl SwitchboardFeedParser {
    fn internal_aggregator_read(&self, ix: &Ix) -> Promise {
        Promise::new(SWITCHBOARD_PROGRAM_ID.parse().unwrap()).function_call(
            "aggregator_read".into(),
            serde_json::json!({
                "ix": {
                    "address": ix.address,
                    "payer": ix.address,
                }
            })
            .to_string()
            .as_bytes()
            .to_vec(),
            ONE_YOCTO,
            GAS_AGGREGATOR_READ,
        )
    }

    fn internal_above_price_feed_read(&self, payload: AbovePriceFeedArgs) -> Promise {
        if payload.market_options[0] != "yes" {
            env::panic_str("ERR_INVALID_MARKET_OPTIONS");
        }

        if payload.market_outcome_ids.len() != 2 {
            env::panic_str("ERR_INVALID_MARKET_OPTIONS_LENGTH");
        }

        let ix = &payload.ix;

        let aggregator_read_promise = self.internal_aggregator_read(ix);

        let on_aggregator_read_callback_promise = ext_self::ext(env::current_account_id())
            .with_static_gas(GAS_AGGREGATOR_READ_CALLBACK)
            .on_internal_above_price_feed_read_callback(payload);

        aggregator_read_promise.then(on_aggregator_read_callback_promise)
    }
}
