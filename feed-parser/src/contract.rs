use near_sdk::{env, ext_contract, near_bindgen, serde_json, Promise, ONE_YOCTO};

use sbv2_near::SWITCHBOARD_PROGRAM_ID;

use crate::consts::*;
use crate::storage::*;

#[ext_contract(ext_self)]
trait Callbacks {
    fn on_aggregator_read_callback(&mut self);
}

#[near_bindgen]
impl SwitchboardFeedParser {
    #[payable]
    pub fn aggregator_read(&mut self, ix: Ix) -> Promise {
        let aggregator_read_promise = Promise::new(SWITCHBOARD_PROGRAM_ID.parse().unwrap())
            .function_call(
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
            );

        let on_aggregator_read_callback_promise = ext_self::ext(env::current_account_id())
            .with_static_gas(GAS_AGGREGATOR_READ_CALLBACK)
            .on_aggregator_read_callback();

        aggregator_read_promise.then(on_aggregator_read_callback_promise)
    }
}
