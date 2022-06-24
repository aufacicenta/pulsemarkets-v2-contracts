#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::{Duration, Utc};
    use near_sdk::serde_json::json;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance, PromiseResult};

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

    fn setup_context() -> VMContextBuilder {
        let mut context = VMContextBuilder::new();
        let now = Utc::now().timestamp_subsec_nanos();
        testing_env!(context
            .predecessor_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());

        context
    }

    fn setup_contract(market: MarketData, resolution_window: Timestamp) -> Market {
        let contract = Market::new(
            market,
            dao_account_id(),
            collateral_token_id(),
            LP_FEE,
            resolution_window,
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

    fn publish(c: &mut Market, context: &VMContextBuilder) {
        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(
                json!({
                    "spec": "1.0.0".to_string(),
                    "name": "usdt.fakes.testnet".to_string(),
                    "symbol": "USDT".to_string(),
                    "decimals": 6,
                })
                .to_string()
                .into_bytes()
            )],
        );

        c.publish();
        c.on_ft_metadata_callback();
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
        }
    }

    fn add_expires_at_nanos(offset: i64) -> i64 {
        let now = Utc::now().timestamp_nanos();
        (now + offset).into()
    }

    #[test]
    fn test_publish_binary_market() {
        let context = setup_context();

        let starts_at = add_expires_at_nanos(100);
        let ends_at = add_expires_at_nanos(1000);
        let resolution_window = 3000;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 2, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

        publish(&mut contract, &context);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.total_supply(), 0.0);
        assert_eq!(outcome_token_1.total_supply(), 0.0);
        assert_eq!(outcome_token_0.get_price(), 0.5);
        assert_eq!(outcome_token_1.get_price(), 0.5);
    }

    #[test]
    fn test_collateral_token_precision() {
        let context = setup_context();

        let now = Utc::now();
        let starts_at = now + Duration::days(5);
        let ends_at = starts_at + Duration::days(10);
        let resolution_window = ends_at + Duration::days(3);

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
        );

        publish(&mut contract, &context);

        assert_eq!(contract.get_precision(), "10000000");
        assert_eq!(
            contract.get_collateral_token_metadata().decimals.unwrap(),
            6
        );
    }

    #[test]
    fn test_publish_market_with_3_outcomes() {
        let context = setup_context();

        let starts_at = add_expires_at_nanos(100);
        let ends_at = add_expires_at_nanos(1000);
        let resolution_window = 3000;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 3, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

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
    fn test_single_trade_binary_unresolved() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;

        let now = add_expires_at_nanos(0);
        testing_env!(context.block_timestamp(now.try_into().unwrap()).build());
        let starts_at = now * 2;
        let ends_at = now * 3;
        let resolution_window = now * 4;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 2, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

        // @TODO publish may also be made by a CT ft_on_transfer
        publish(&mut contract, &context);

        testing_env!(context
            .block_timestamp((starts_at as f32 - (starts_at as f32 * 0.25)) as u64)
            .build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            100.0,
            yes,
        );

        testing_env!(context
            .block_timestamp((starts_at as f32 - (starts_at as f32 * 0.5)) as u64)
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

        let now = add_expires_at_nanos(0);
        testing_env!(context.block_timestamp(now.try_into().unwrap()).build());
        let starts_at = now * 2;
        let ends_at = now * 3;
        let resolution_window = now * 4;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 2, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

        // @TODO publish may also be made by a CT ft_on_transfer
        publish(&mut contract, &context);

        // boost the balance
        testing_env!(context
            .block_timestamp((starts_at as f32 - (starts_at as f32 * 0.25)) as u64)
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
            .block_timestamp((starts_at as f32 - (starts_at as f32 * 0.30)) as u64)
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
            .block_timestamp((starts_at as f32 - (starts_at as f32 * 0.5)) as u64)
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

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at.timestamp_nanos().try_into().unwrap(),
            ends_at.timestamp_nanos().try_into().unwrap(),
        );

        let mut contract: Market = setup_contract(
            market_data,
            resolution_window.timestamp_nanos().try_into().unwrap(),
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
        let emily_balance = contract.balance_of(no, emily());
        assert_eq!(emily_balance, 0.0);
        let frank_balance = contract.balance_of(no, frank());
        assert_eq!(frank_balance, 0.0);
        let outcome_token_no = contract.get_outcome_token(no);
        assert_eq!(outcome_token_no.total_supply() < 0.0, true);
        assert_eq!(outcome_token_no.total_supply() > -1.0, true);

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
        assert_eq!(outcome_token_yes.total_supply() > 0.0, true);
        assert_eq!(outcome_token_yes.total_supply() < 1.0, true);

        assert_eq!(contract.get_collateral_token_metadata().balance > 0.0, true);
        assert_eq!(contract.get_collateral_token_metadata().balance < 1.0, true);
    }
}
