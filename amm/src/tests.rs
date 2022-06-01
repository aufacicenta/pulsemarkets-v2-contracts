#[cfg(test)]
mod tests {
    use crate::storage::*;
    use crate::FungibleTokenReceiver;
    use chrono::Utc;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{serde_json, testing_env, AccountId, Balance};

    const _ATTACHED_DEPOSIT: Balance = 1_000_000_000_000_000_000_000_000; // 1 Near

    const LP_FEE: f64 = 0.02;

    fn daniel() -> AccountId {
        AccountId::new_unchecked("daniel.near".to_string())
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
        let msg = serde_json::json!({
            "BuyArgs": {
                "outcome_id": outcome_id,
            }
        });

        *collateral_token_balance += amount;
        c.ft_on_transfer(account_id, amount, msg.to_string())
    }

    fn sell(
        c: &mut Market,
        collateral_token_balance: &mut WrappedBalance,
        amount: WrappedBalance,
        outcome_id: u64,
    ) -> WrappedBalance {
        *collateral_token_balance += amount;
        c.sell(outcome_id, amount)
    }

    fn create_market_data(
        description: String,
        options: u8,
        starts_at: u64,
        ends_at: u64,
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

    fn add_expires_at_nanos(offset: u32) -> u64 {
        let now = Utc::now().timestamp_subsec_nanos();
        (now + offset).into()
    }

    #[test]
    fn test_publish_binary_market() {
        let starts_at = add_expires_at_nanos(100);
        let ends_at = add_expires_at_nanos(1000);
        let resolution_window = 3000;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 2, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

        contract.publish();

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.total_supply(), 0.0);
        assert_eq!(outcome_token_1.total_supply(), 0.0);
        assert_eq!(outcome_token_0.get_price(), 0.5);
        assert_eq!(outcome_token_1.get_price(), 0.5);
    }

    #[test]
    fn test_publish_market_with_3_outcomes() {
        let starts_at = add_expires_at_nanos(100);
        let ends_at = add_expires_at_nanos(1000);
        let resolution_window = 3000;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 3, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

        contract.publish();

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
    fn test_binary_market() {
        let mut context = setup_context();

        let mut collateral_token_balance: f64 = 0.0;

        let yes = 0;
        let no = 1;

        let starts_at = add_expires_at_nanos(100);
        let ends_at = add_expires_at_nanos(300);
        let resolution_window = 100;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 2, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

        contract.publish();

        testing_env!(context.block_timestamp(starts_at - 99).build());
        let alice_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            100.0,
            yes,
        );

        testing_env!(context.block_timestamp(starts_at - 89).build());
        let bob_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            bob(),
            100.0,
            yes,
        );

        testing_env!(context.block_timestamp(starts_at - 79).build());
        let carol_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            carol(),
            100.0,
            yes,
        );

        testing_env!(context.block_timestamp(starts_at - 69).build());
        let daniel_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            daniel(),
            100.0,
            yes,
        );

        let mut outcome_token_yes: OutcomeToken = contract.get_outcome_token(yes);

        assert_eq!(
            outcome_token_yes.total_supply(),
            alice_balance + bob_balance + carol_balance + daniel_balance
        );

        assert_eq!(collateral_token_balance, 400.0);

        // Open the market
        // let mut context = setup_context();
        // let mut now = (starts_at + 100) as u32;
        // testing_env!(context.block_timestamp(now.into()).build());

        // buy(
        //     &mut contract,
        //     &mut collateral_token_balance,
        //     carol(),
        //     100.0,
        //     yes,
        // );

        // outcome_token_yes = contract.get_outcome_token(yes);

        // assert_eq!(
        //     outcome_token_yes.get_lp_pool_balance(),
        //     outcome_token_yes.get_lp_balance(&alice()) + outcome_token_yes.get_lp_balance(&bob())
        // );

        // let lp_balance_1 = outcome_token_yes.get_lp_balance(&alice());
        // let lp_balance_2 = outcome_token_yes.get_lp_balance(&bob());
        // let total_supply = lp_balance_1 + lp_balance_2 + outcome_token_yes.get_balance(&carol());

        // // OT total_supply is not modified upon purchase since "buy" won't mint new units
        // assert_eq!(outcome_token_yes.total_supply(), total_supply);

        // assert_eq!(collateral_token_balance, 300.0);
        // assert_eq!(outcome_token_yes.get_balance(&carol()), 50.96);

        // // Close the market
        // now = (ends_at + resolution_window - 20) as u32;
        // testing_env!(context.block_timestamp(now.into()).build());

        // // Resolve the market
        // testing_env!(context.signer_account_id(dao_account_id()).build());
        // contract.resolve(yes);

        // outcome_token_yes = contract.get_outcome_token(yes);
        // let outcome_token_no: OutcomeToken = contract.get_outcome_token(no);

        // assert_eq!(contract.get_status(), "Resolved");
        // assert_eq!(outcome_token_yes.get_price(), 1.0);
        // assert_eq!(outcome_token_no.get_price(), 0.0);

        // testing_env!(context.signer_account_id(carol()).build());
        // let prev_collateral_token_balance = collateral_token_balance;
        // let earnings = redeem(&mut contract, &mut collateral_token_balance, yes);

        // assert_eq!(outcome_token_yes.get_balance(&carol()), 0.0);
        // assert_eq!(
        //     collateral_token_balance,
        //     prev_collateral_token_balance - earnings
        // );

        // // OT total_supply is still unmodified after redeem. Supply only changes upon minting or burning. Redeeming wont burn tokens
        // assert_eq!(outcome_token_yes.total_supply(), total_supply);

        // // LP balances remain unmodified after a buyer redeems
        // assert_eq!(outcome_token_yes.get_lp_balance(&alice()), lp_balance_1);
        // assert_eq!(outcome_token_yes.get_lp_balance(&bob()), lp_balance_2);

        // testing_env!(context.signer_account_id(alice()).build());
        // lp_redeem(&mut contract, &mut collateral_token_balance, yes);

        // testing_env!(context.signer_account_id(bob()).build());
        // lp_redeem(&mut contract, &mut collateral_token_balance, yes);

        // // outcome_token_yes = contract.get_outcome_token(yes);

        // assert_eq!(outcome_token_yes.get_lp_balance(&alice()), 0.0);
        // assert_eq!(outcome_token_yes.get_lp_balance(&bob()), 0.0);
        // assert_eq!(collateral_token_balance, 0.0);
    }
}
