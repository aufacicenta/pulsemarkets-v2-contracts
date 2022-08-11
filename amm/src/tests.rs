#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::{Duration, Utc};
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance, PromiseResult};
    use rand::seq::SliceRandom;

    const _ATTACHED_DEPOSIT: Balance = 1_000_000_000_000_000_000_000_000; // 1 Near

    const LP_FEE: WrappedBalance = 0.02;

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

    fn staking_token_account_id() -> AccountId {
        AccountId::new_unchecked("staking_token_account_id.near".to_string())
    }

    fn market_creator_account_id() -> AccountId {
        AccountId::new_unchecked("market_creator_account_id.near".to_string())
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

    fn setup_contract(
        market: MarketData,
        resolution_window: Timestamp,
        claiming_window: Timestamp,
    ) -> Market {
        let contract = Market::new(
            market,
            dao_account_id(),
            collateral_token_id(),
            staking_token_account_id(),
            market_creator_account_id(),
            LP_FEE,
            resolution_window,
            claiming_window,
            6,
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

    fn resolve(c: &mut Market, collateral_token_balance: &mut WrappedBalance, outcome_id: u64) {
        c.resolve(outcome_id);
        let balance = *collateral_token_balance;
        *collateral_token_balance -= balance * c.get_fee_ratio();
    }

    fn publish(c: &mut Market, _context: &VMContextBuilder) {
        c.publish();
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
            category: None,
            options: (0..options).map(|s| s.to_string()).collect(),
            starts_at,
            ends_at,
            utc_offset: -6,
        }
    }

    #[test]
    fn test_publish_binary_market() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);
        let resolution_window = ends_at + Duration::hours(3);
        let claiming_window = resolution_window + Duration::hours(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.total_supply(), 0.0);
        assert_eq!(outcome_token_1.total_supply(), 0.0);
        assert_eq!(outcome_token_0.get_price(), 0.5);
        assert_eq!(outcome_token_1.get_price(), 0.5);
    }

    #[test]
    fn test_publish_market_with_3_outcomes() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);
        let resolution_window = ends_at + Duration::hours(3);
        let claiming_window = resolution_window + Duration::hours(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            3,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);

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
    fn test_publish_market_with_4_outcomes() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);
        let resolution_window = ends_at + Duration::days(3);
        let claiming_window = resolution_window + Duration::hours(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            4,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);

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
    fn test_single_trade_binary_unresolved() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);
        let resolution_window = ends_at + Duration::days(3);
        let claiming_window = resolution_window + Duration::hours(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        // @TODO publish may also be made by a CT ft_on_transfer
        publish(&mut contract, &context);

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
            100.0,
            yes,
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
            alice(),
            100.0,
            yes,
        );

        assert_eq!(collateral_token_balance, 200.0);

        // alice sells all her OT balance while the event is ongoing
        let mut alice_balance = contract.balance_of(yes, alice());
        testing_env!(context.signer_account_id(alice()).build());
        sell(&mut contract, alice(), alice_balance, yes, &context);

        let outcome_token_yes = contract.get_outcome_token(yes);
        alice_balance = contract.balance_of(yes, alice());
        assert_eq!(alice_balance, 0.0);

        assert_eq!(outcome_token_yes.total_supply(), 0.0);
        assert_eq!(contract.get_collateral_token_metadata().balance, 0.0);
    }

    #[test]
    fn test_multiple_trade_binary_unresolved() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;
        let no = 1;

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);
        let resolution_window = ends_at + Duration::days(3);
        let claiming_window = resolution_window + Duration::hours(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        // @TODO publish may also be made by a CT ft_on_transfer
        publish(&mut contract, &context);

        // boost the balance
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
            100.0,
            yes,
        );

        // boost the balance
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
            100.0,
            no,
        );

        // boost the balance
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
            alice(),
            100.0,
            yes,
        );

        assert_eq!(collateral_token_balance, 300.0);

        // alice sells all her OT balance while the event is ongoing
        let alice_balance = contract.balance_of(yes, alice());
        testing_env!(context.signer_account_id(alice()).build());
        sell(&mut contract, alice(), alice_balance, yes, &context);

        let outcome_token_yes = contract.get_outcome_token(yes);
        let alice_balance = contract.balance_of(yes, alice());
        assert_eq!(alice_balance, 0.0);
        assert_eq!(outcome_token_yes.total_supply(), 0.0);

        // bob sells all her OT balance while the event is ongoing
        let bob_balance = contract.balance_of(no, bob());
        testing_env!(context.signer_account_id(bob()).build());
        sell(&mut contract, bob(), bob_balance, no, &context);

        let outcome_token_no = contract.get_outcome_token(no);
        let bob_balance = contract.balance_of(no, bob());
        assert_eq!(bob_balance, 0.0);
        assert_eq!(outcome_token_no.total_supply(), 0.0);

        assert_eq!(contract.get_collateral_token_metadata().balance, 0.0);
    }

    #[test]
    fn test_binary_market() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;
        let no = 1;

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);
        let resolution_window = ends_at + Duration::days(3);
        let claiming_window = resolution_window + Duration::days(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        // @TODO publish may also be made by a CT ft_on_transfer
        publish(&mut contract, &context);

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

        // alice sells all her OT balance while the event is ongoing
        let alice_balance = contract.balance_of(yes, alice());
        testing_env!(context.signer_account_id(alice()).build());
        sell(&mut contract, alice(), alice_balance, yes, &context);
        let alice_balance = contract.balance_of(yes, alice());
        assert_eq!(alice_balance, 0.0);

        // gus sells his full OT balance while the event is ongoing
        let gus_balance = contract.balance_of(no, gus());
        testing_env!(context.signer_account_id(gus()).build());
        sell(&mut contract, gus(), gus_balance, no, &context);
        let gus_balance = contract.balance_of(no, gus());
        assert_eq!(gus_balance, 0.0);

        // Event is over. Resolution window is open
        let now = resolution_window - Duration::days(1);
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());

        // Resolve the market: Burn the losers
        testing_env!(context.signer_account_id(dao_account_id()).build());
        resolve(&mut contract, &mut collateral_token_balance, yes);
        let outcome_token_no = contract.get_outcome_token(no);
        assert_eq!(outcome_token_no.is_active(), false);

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

        assert_eq!(contract.get_collateral_token_metadata().balance, 0.0);
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
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);
        let resolution_window = ends_at + Duration::days(3);
        let claiming_window = resolution_window + Duration::days(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            4,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        // @TODO publish may also be made by a CT ft_on_transfer
        publish(&mut contract, &context);

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
    #[should_panic(expected = "ERR_EVENT_HAS_STARTED")]
    fn test_publish_error_after_event_starts() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now - Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);
        let resolution_window = ends_at + Duration::hours(3);
        let claiming_window = resolution_window + Duration::days(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);
    }

    #[test]
    #[should_panic(expected = "ERR_MARKET_IS_CLOSED")]
    fn test_buy_error_if_event_is_ongoing() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);
        let resolution_window = ends_at + Duration::hours(3);
        let claiming_window = resolution_window + Duration::days(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);

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
    #[should_panic(expected = "ERR_SELL_MARKET_IS_CLOSED_FOR_SELLS_UNTIL_RESOLUTION")]
    fn test_sell_error_if_event_is_ongoing() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::hours(1);
        let ends_at = starts_at + Duration::hours(1);
        let resolution_window = ends_at + Duration::hours(3);
        let claiming_window = resolution_window + Duration::days(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);

        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400.0,
            yes,
        );

        testing_env!(context
            .block_timestamp(
                (starts_at + Duration::minutes(20))
                    .timestamp_nanos()
                    .try_into()
                    .unwrap()
            )
            .signer_account_id(alice())
            .build());
        let alice_balance = contract.balance_of(yes, alice());
        sell(&mut contract, alice(), alice_balance, yes, &context);
    }

    #[test]
    fn test_on_claim_staking_fees_resolved_callback() {
        let mut context = setup_context();

        let now = Utc::now();
        testing_env!(context
            .block_timestamp(now.timestamp_nanos().try_into().unwrap())
            .build());
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);
        let resolution_window = ends_at + Duration::days(3);
        let claiming_window = resolution_window + Duration::days(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
            claiming_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);

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
