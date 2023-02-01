#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::Utc;
    use near_sdk::serde_json::json;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, PromiseResult};

    const PRICE: Price = 23456.01;

    const IX_ADDRESS: [u8; 32] = [
        173, 62, 255, 125, 45, 251, 162, 167, 128, 129, 25, 33, 146, 248, 118, 134, 118, 192, 215,
        84, 225, 222, 198, 48, 70, 49, 212, 195, 84, 136, 96, 56,
    ];

    fn predecessor_account_id() -> AccountId {
        AccountId::new_unchecked("predecessor_account_id.near".to_string())
    }

    fn setup_context() -> VMContextBuilder {
        let mut context = VMContextBuilder::new();
        let now = Utc::now().timestamp_subsec_nanos();
        testing_env!(context
            .predecessor_account_id(predecessor_account_id())
            .block_timestamp(now.try_into().unwrap())
            .build());

        context
    }

    #[test]
    fn price_is_greater_than_aggregator_read_result() {
        setup_context();

        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let msg = json!({
        "AggregatorReadArgs": {
            "ix": ix,
            "market_options": vec!["yes", "no"],
            "market_outcome_ids": vec![0, 1],
            "price": 24000.0,
        }});

        let contract = SwitchboardFeedParser::default();

        // @TODO set the env to result in an AggregatorRound with the final price
        // testing_env!(
        //     context.build(),
        //     near_sdk::VMConfig::test(),
        //     near_sdk::RuntimeFeesConfig::test(),
        //     Default::default(),
        //     vec![PromiseResult::Successful(vec![PRICE])],
        // );

        contract.aggregator_read(msg.to_string());

        // @TODO execute the callback with the payload, including predecessor_account_id
        // contract.on_aggregator_read_callback(payload);

        // assert_eq!(outcome_token_1.total_supply(), 0);
    }
}
