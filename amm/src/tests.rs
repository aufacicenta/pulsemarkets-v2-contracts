#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::{Duration, Utc};
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{serde_json, testing_env, AccountId, Balance, PromiseResult};
    use rand::seq::SliceRandom;
    use sbv2_near::{AggregatorRound, SwitchboardDecimal, Uuid};

    const _ATTACHED_DEPOSIT: Balance = 1_000_000_000_000_000_000_000_000; // 1 Near

    const LP_FEE: WrappedBalance = 0.02;

    const IX_ADDRESS: Uuid = [
        173, 62, 255, 125, 45, 251, 162, 167, 128, 129, 25, 33, 146, 248, 118, 134, 118, 192, 215,
        84, 225, 222, 198, 48, 70, 49, 212, 195, 84, 136, 96, 56,
    ];

    const DEFAULT_PRICE: f64 = 20000.00;

    fn daniel() -> AccountId {
        AccountId::new_unchecked("daniel.near".to_string())
    }

    fn emily() -> AccountId {
        AccountId::new_unchecked("emily.near".to_string())
    }

    fn frank() -> AccountId {
        AccountId::new_unchecked("frank.near".to_string())
    }

    fn gus() -> AccountId {
        AccountId::new_unchecked("gus.near".to_string())
    }

    fn dao_account_id() -> AccountId {
        AccountId::new_unchecked("dao_account_id.near".to_string())
    }

    fn collateral_token_id() -> AccountId {
        AccountId::new_unchecked("collateral_token_id.near".to_string())
    }

    fn market_creator_account_id() -> AccountId {
        AccountId::new_unchecked("market_creator_account_id.near".to_string())
    }

    fn market_publisher_account_id() -> AccountId {
        AccountId::new_unchecked("market_publisher_account_id.near".to_string())
    }

    fn date(date: chrono::DateTime<chrono::Utc>) -> i64 {
        date.timestamp_nanos().try_into().unwrap()
    }

    fn block_timestamp(date: chrono::DateTime<chrono::Utc>) -> u64 {
        date.timestamp_nanos().try_into().unwrap()
    }

    fn setup_context() -> VMContextBuilder {
        let mut context = VMContextBuilder::new();
        let now = Utc::now().timestamp_subsec_nanos();
        testing_env!(context
            .predecessor_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());

        context
    }

    fn setup_contract(market: MarketData) -> Market {
        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let contract = Market::new(
            market,
            dao_account_id(),
            collateral_token_id(),
            market_creator_account_id(),
            LP_FEE,
            6,
            Resolution { ix },
        );

        contract
    }

    fn buy(
        c: &mut Market,
        collateral_token_balance: &mut WrappedBalance,
        account_id: AccountId,
        amount: WrappedBalance,
        outcome_id: u64,
    ) -> WrappedBalance {
        *collateral_token_balance += amount;
        c.buy(account_id, amount, BuyArgs { outcome_id })
    }

    fn sell(
        c: &mut Market,
        payee: AccountId,
        amount: WrappedBalance,
        outcome_id: u64,
        context: &VMContextBuilder,
    ) -> WrappedBalance {
        let amount_sold = c.sell(outcome_id, amount);

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(
                amount_sold.to_string().into_bytes()
            )],
        );

        c.on_ft_transfer_callback(amount, payee, outcome_id, amount_sold);

        return amount;
    }

    fn resolve(c: &mut Market, collateral_token_balance: &mut WrappedBalance) {
        c.resolve();

        let balance = *collateral_token_balance;
        *collateral_token_balance -= balance * c.get_fee_ratio();
    }

    fn create_outcome_tokens(c: &mut Market) {
        c.create_outcome_tokens();
    }

    fn publish(c: &mut Market, context: &VMContextBuilder) {
        c.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );

        c.on_create_proposals_callback();
    }

    fn create_market_data(
        description: String,
        options: u8,
        starts_at: i64,
        ends_at: i64,
    ) -> MarketData {
        MarketData {
            description,
            info: "".to_string(),
            price: DEFAULT_PRICE,
            category: None,
            options: (0..options).map(|s| s.to_string()).collect(),
            starts_at,
            ends_at,
            utc_offset: -6,
        }
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

    #[test]
    fn test_publish_binary_market() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        let now = ends_at + Duration::hours(1);
        testing_env!(context
            .block_timestamp(block_timestamp(now))
            .signer_account_id(market_publisher_account_id())
            .build());
        publish(&mut contract, &context);

        assert_eq!(contract.is_published(), true);
        assert_eq!(contract.is_claiming_window_expired(), false);
        assert_eq!(contract.is_resolution_window_expired(), false);
        assert_eq!(
            contract.resolution_window(),
            (block_timestamp(now) + 259200 * 1_000_000_000) as i64
        );
        assert_eq!(
            contract.claiming_window(),
            (contract.resolution_window() + 2592000 * 1_000_000_000) as i64
        );
        assert_eq!(
            contract.get_market_publisher_account_id(),
            market_publisher_account_id()
        );

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.total_supply(), 0.0);
        assert_eq!(outcome_token_1.total_supply(), 0.0);
        assert_eq!(outcome_token_0.get_price(), 0.5);
        assert_eq!(outcome_token_1.get_price(), 0.5);
    }

    #[test]
    fn test_create_outcome_tokens() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.total_supply(), 0.0);
        assert_eq!(outcome_token_1.total_supply(), 0.0);
        assert_eq!(outcome_token_0.get_price(), 0.5);
        assert_eq!(outcome_token_1.get_price(), 0.5);
    }

    #[test]
    fn test_create_market_with_3_outcomes() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            3,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);

        create_outcome_tokens(&mut contract);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);
        let outcome_token_2: OutcomeToken = contract.get_outcome_token(2);

        assert_eq!(outcome_token_0.total_supply(), 0.0);
        assert_eq!(outcome_token_1.total_supply(), 0.0);
        assert_eq!(outcome_token_2.total_supply(), 0.0);

        assert_eq!(outcome_token_0.get_price(), 0.3333333333333333);
        assert_eq!(outcome_token_1.get_price(), 0.3333333333333333);
        assert_eq!(outcome_token_2.get_price(), 0.3333333333333333);
    }

    #[test]
    fn test_create_market_with_4_outcomes() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            4,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);

        create_outcome_tokens(&mut contract);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);
        let outcome_token_2: OutcomeToken = contract.get_outcome_token(2);
        let outcome_token_3: OutcomeToken = contract.get_outcome_token(3);

        assert_eq!(outcome_token_0.total_supply(), 0.0);
        assert_eq!(outcome_token_1.total_supply(), 0.0);
        assert_eq!(outcome_token_2.total_supply(), 0.0);
        assert_eq!(outcome_token_3.total_supply(), 0.0);

        assert_eq!(outcome_token_0.get_price(), 0.25);
        assert_eq!(outcome_token_1.get_price(), 0.25);
        assert_eq!(outcome_token_2.get_price(), 0.25);
        assert_eq!(outcome_token_3.get_price(), 0.25);
    }

    #[test]
    fn test_binary_market() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;
        let no = 1;

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        testing_env!(context
            .block_timestamp(
                (starts_at - Duration::days(4))
                    .timestamp_nanos()
                    .try_into()
                    .unwrap()
            )
            .build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400.0,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            emily(),
            100.0,
            no,
        );

        testing_env!(context
            .block_timestamp(
                (starts_at - Duration::days(3))
                    .timestamp_nanos()
                    .try_into()
                    .unwrap()
            )
            .build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            bob(),
            300.0,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            frank(),
            100.0,
            no,
        );

        testing_env!(context
            .block_timestamp(
                (starts_at - Duration::days(2))
                    .timestamp_nanos()
                    .try_into()
                    .unwrap()
            )
            .build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            carol(),
            200.0,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            gus(),
            100.0,
            no,
        );

        testing_env!(context
            .block_timestamp(
                (starts_at - Duration::days(1))
                    .timestamp_nanos()
                    .try_into()
                    .unwrap()
            )
            .build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            daniel(),
            100.0,
            yes,
        );

        assert_eq!(contract.get_collateral_token_metadata().balance, 1300.0);

        // @TODO publish may also be made by a CT ft_on_transfer
        // Publish after the market is over
        let now = ends_at + Duration::days(1);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        publish(&mut contract, &context);

        // Resolve the market: Burn the losers
        testing_env!(context.predecessor_account_id(dao_account_id()).build());
        resolve(&mut contract, &mut collateral_token_balance);
        let outcome_token_no = contract.get_outcome_token(no);
        assert_eq!(outcome_token_no.is_active(), false);
        assert_eq!(outcome_token_no.total_supply(), 0.0);

        // Resolution window is over
        let now = now + Duration::days(4);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());

        // alice sells all her OT balance after the market is resolved
        let alice_balance = contract.balance_of(yes, alice());
        testing_env!(context.signer_account_id(alice()).build());
        sell(&mut contract, alice(), alice_balance, yes, &context);
        let alice_balance = contract.balance_of(yes, alice());
        assert_eq!(alice_balance, 0.0);

        // bob sells his OT balance after the market is resolved. Claim earnings!!
        let bob_balance = contract.balance_of(yes, bob());
        testing_env!(context.signer_account_id(bob()).build());
        sell(&mut contract, bob(), bob_balance, yes, &context);
        let bob_balance = contract.balance_of(yes, bob());
        assert_eq!(bob_balance, 0.0);

        let carol_balance = contract.balance_of(yes, carol());
        testing_env!(context.signer_account_id(carol()).build());
        sell(&mut contract, carol(), carol_balance, yes, &context);
        let carol_balance = contract.balance_of(yes, carol());
        assert_eq!(carol_balance, 0.0);

        let daniel_balance = contract.balance_of(yes, daniel());
        testing_env!(context.signer_account_id(daniel()).build());
        sell(&mut contract, daniel(), daniel_balance, yes, &context);
        let daniel_balance = contract.balance_of(yes, daniel());
        assert_eq!(daniel_balance, 0.0);

        let outcome_token_yes = contract.get_outcome_token(yes);
        assert_eq!(outcome_token_yes.total_supply().ceil(), 0.0);

        assert_eq!(
            contract.get_collateral_token_metadata().balance,
            contract.collateral_token.fee_balance
        );
    }

    #[test]
    fn test_market_with_4_outcomes() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let outcome_1 = 0;
        let outcome_2 = 1;
        let outcome_3 = 2;
        let outcome_4 = 3;

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            4,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);

        create_outcome_tokens(&mut contract);

        let amounts = vec![100.0, 200.0, 300.0, 50.0, 10.0, 20.0, 500.0];
        let buyers = vec![alice(), bob(), carol(), daniel(), emily(), frank(), gus()];
        let outcomes = vec![outcome_1, outcome_2, outcome_3, outcome_4];

        for _n in 1..20 {
            let buyer = buyers.choose(&mut rand::thread_rng()).unwrap();
            let amount = amounts.choose(&mut rand::thread_rng()).unwrap();
            let outcome = outcomes.choose(&mut rand::thread_rng()).unwrap();

            buy(
                &mut contract,
                &mut collateral_token_balance,
                buyer.clone(),
                amount.clone(),
                outcome.clone(),
            );
        }
    }

    #[test]
    fn test_sell_unresolved_market() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400.0,
            yes,
        );

        let now = ends_at + Duration::hours(1);
        testing_env!(context
            .block_timestamp(block_timestamp(now))
            .signer_account_id(alice())
            .build());
        publish(&mut contract, &context);

        testing_env!(context
            .block_timestamp(block_timestamp(now + Duration::days(4)))
            .signer_account_id(alice())
            .build());
        let alice_balance = contract.balance_of(yes, alice());
        sell(&mut contract, alice(), alice_balance, yes, &context);

        assert_eq!(contract.balance_of(yes, alice()), 0.0);
    }

    #[test]
    #[should_panic(expected = "ERR_CANT_SELL_A_LOSING_OUTCOME")]
    fn test_binary_market_errors_when_selling_losing_outcomes() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;
        let no = 1;

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        testing_env!(context
            .block_timestamp(
                (starts_at - Duration::days(4))
                    .timestamp_nanos()
                    .try_into()
                    .unwrap()
            )
            .build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400.0,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            emily(),
            100.0,
            no,
        );

        // Publish after the market is over
        let now = ends_at + Duration::days(1);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        publish(&mut contract, &context);

        // Resolve the market: YES, price is above average
        let mut aggregator_round = build_aggregator_round();
        aggregator_round.result = SwitchboardDecimal::from_f64(DEFAULT_PRICE + 1.0);

        let aggregator_round_bytes = serde_json::to_string(&aggregator_round).unwrap();

        testing_env!(
            context.predecessor_account_id(dao_account_id()).build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(
                aggregator_round_bytes.as_bytes().to_vec()
            )],
        );

        resolve(&mut contract, &mut collateral_token_balance);
        contract.on_aggregator_read_callback();

        // Resolution window is over
        let now = now + Duration::days(4);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());

        // emily tries to sell a losing outcome token
        let emily_balance = contract.balance_of(no, emily());
        testing_env!(context.signer_account_id(emily()).build());
        sell(&mut contract, emily(), emily_balance, no, &context);
    }

    #[test]
    #[should_panic(expected = "ERR_MARKET_IS_CLOSED")]
    fn test_buy_error_if_event_is_ongoing() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        testing_env!(context
            .block_timestamp(
                (starts_at + Duration::minutes(20))
                    .timestamp_nanos()
                    .try_into()
                    .unwrap()
            )
            .build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400.0,
            yes,
        );
    }

    #[test]
    #[should_panic(expected = "ERR_CREATE_OUTCOME_TOKENS_OUTCOMES_EXIST")]
    fn test_create_outcome_tokens_error() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);
        create_outcome_tokens(&mut contract);
    }

    #[test]
    #[should_panic(expected = "ERR_MARKET_IS_UNDER_RESOLUTION")]
    fn test_sell_error_if_market_is_under_resolution() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400.0,
            yes,
        );

        let now = ends_at + Duration::hours(1);
        testing_env!(context
            .block_timestamp(block_timestamp(now))
            .signer_account_id(alice())
            .build());
        publish(&mut contract, &context);

        let alice_balance = contract.balance_of(yes, alice());
        sell(&mut contract, alice(), alice_balance, yes, &context);
    }

    #[test]
    fn test_on_claim_staking_fees_resolved_callback() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            date(starts_at),
            date(ends_at),
        );

        let mut contract: Market = setup_contract(market_data);
        create_outcome_tokens(&mut contract);

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            // 50%
            vec![
                // ft_balance_of
                PromiseResult::Successful("5000000".to_string().into_bytes()),
                // ft_total_supply
                PromiseResult::Successful("10000000".to_string().into_bytes())
            ],
        );

        let amount_payable = contract.on_claim_staking_fees_resolved_callback(221.0, alice());

        // (221 = 85% of total fees for $PULSE stakers) * (0.5 = alice.near weight of $PULSE total_supply) = 110.5,
        // then convert to Collateral Token decimals precision
        assert_eq!(amount_payable, "1105000000");
        assert_eq!(contract.get_claimed_staking_fees(alice()), "1105000000");
    }
}
