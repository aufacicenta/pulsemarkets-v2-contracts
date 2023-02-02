#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::Utc;
    use near_sdk::serde_json::json;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{serde_json, testing_env, AccountId, PromiseResult};
    use sbv2_near::{AggregatorRound, SwitchboardDecimal};

    const PRICE: Price = 23456.01;

    const IX_ADDRESS: [u8; 32] = [
        173, 62, 255, 125, 45, 251, 162, 167, 128, 129, 25, 33, 146, 248, 118, 134, 118, 192, 215,
        84, 225, 222, 198, 48, 70, 49, 212, 195, 84, 136, 96, 56,
    ];

    fn predecessor_account_id() -> AccountId {
        AccountId::new_unchecked("predecessor_account_id.near".to_string())
    }

    fn build_aggregator_round() -> AggregatorRound {
        let aggregator_round = AggregatorRound {
            id: 123,
            num_success: 1,
            num_error: 2,
            is_closed: false,
            round_open_slot: 1,
            round_open_timestamp: 1,
            result: SwitchboardDecimal::from_f64(20000.01),
            std_deviation: SwitchboardDecimal::from_f64(20000.01),
            min_response: SwitchboardDecimal::from_f64(20000.01),
            max_response: SwitchboardDecimal::from_f64(20000.01),
            oracles: vec![IX_ADDRESS],
            medians_data: vec![SwitchboardDecimal::from_f64(20000.01)],
            current_payout: vec![123],
            medians_fulfilled: vec![true],
            errors_fulfilled: vec![true],
            _ebuf: vec![1],
            features: vec![1],
        };

        aggregator_round
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
    fn above_price_feed_read_yes() {
        let context = setup_context();

        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let msg = json!({
            "ix": ix,
            "market_options": vec!["yes", "no"],
            "market_outcome_ids": vec![0, 1],
            "price": 24000.0,
            "predecessor_account_id": predecessor_account_id()
        });

        let contract = SwitchboardFeedParser::default();

        let mut aggregator_round = build_aggregator_round();
        aggregator_round.result = SwitchboardDecimal::from_f64(PRICE);

        let aggregator_round_bytes = serde_json::to_string(&aggregator_round).unwrap();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(
                aggregator_round_bytes.as_bytes().to_vec()
            )],
        );

        let payload: AbovePriceFeedArgs = serde_json::from_str(&msg.to_string()).unwrap();

        let winning_outcome_id = contract.on_internal_above_price_feed_read_callback(payload);

        assert_eq!(winning_outcome_id, 0);
    }

    #[test]
    fn above_price_feed_read_no() {
        let context = setup_context();

        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let msg = json!({
            "ix": ix,
            "market_options": vec!["yes", "no"],
            "market_outcome_ids": vec![0, 1],
            "price": PRICE - 1.0,
            "predecessor_account_id": predecessor_account_id()
        });

        let contract = SwitchboardFeedParser::default();

        let mut aggregator_round = build_aggregator_round();
        aggregator_round.result = SwitchboardDecimal::from_f64(PRICE);

        let aggregator_round_bytes = serde_json::to_string(&aggregator_round).unwrap();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(
                aggregator_round_bytes.as_bytes().to_vec()
            )],
        );

        let payload: AbovePriceFeedArgs = serde_json::from_str(&msg.to_string()).unwrap();

        let winning_outcome_id = contract.on_internal_above_price_feed_read_callback(payload);

        assert_eq!(winning_outcome_id, 1);
    }

    #[test]
    #[should_panic(expected = "ERR_PREDECESSOR_ACCOUNT_ID_NOT_SET")]
    fn error_above_price_feed_read_predecessor_not_set() {
        let context = setup_context();

        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let msg = json!({
            "ix": ix,
            "market_options": vec!["yes", "no"],
            "market_outcome_ids": vec![0, 1],
            "price": 24000.0,
        });

        let contract = SwitchboardFeedParser::default();

        let mut aggregator_round = build_aggregator_round();
        aggregator_round.result = SwitchboardDecimal::from_f64(PRICE);

        let aggregator_round_bytes = serde_json::to_string(&aggregator_round).unwrap();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(
                aggregator_round_bytes.as_bytes().to_vec()
            )],
        );

        let payload: AbovePriceFeedArgs = serde_json::from_str(&msg.to_string()).unwrap();

        let winning_outcome_id = contract.on_internal_above_price_feed_read_callback(payload);

        assert_eq!(winning_outcome_id, 0);
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_MARKET_OPTIONS")]
    fn error_above_price_feed_read_invalid_market_options() {
        setup_context();

        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let msg = json!({
        "AggregatorReadArgs": {
            "ix": ix,
            "market_options": vec!["whatever", "no"],
            "market_outcome_ids": vec![0, 1],
            "price": 24000.0,
        }});

        let contract = SwitchboardFeedParser::default();

        contract.aggregator_read(msg.to_string());
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_MARKET_OPTIONS_LENGTH")]
    fn error_above_price_feed_read_invalid_length() {
        setup_context();

        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let msg = json!({
        "AggregatorReadArgs": {
            "ix": ix,
            "market_options": vec!["yes", "no"],
            "market_outcome_ids": vec![0, 1, 2],
            "price": 24000.0,
        }});

        let contract = SwitchboardFeedParser::default();

        contract.aggregator_read(msg.to_string());
    }
}
