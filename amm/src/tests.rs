#[cfg(test)]
mod tests {
    use crate::storage::*;
    use crate::FungibleTokenReceiver;
    use chrono::Utc;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{serde_json, testing_env, AccountId, Balance};

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
        let amount_sold = c.sell(outcome_id, amount);
        *collateral_token_balance -= amount_sold;
        amount
    }

    fn resolve(c: &mut Market, collateral_token_balance: &mut WrappedBalance, outcome_id: u64) {
        c.resolve(outcome_id);
        let balance = *collateral_token_balance;
        *collateral_token_balance -= balance * c.get_fee();
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

        let mut collateral_token_balance: WrappedBalance = 0.0;

        let yes = 0;
        let no = 1;

        let now = add_expires_at_nanos(0);
        testing_env!(context.block_timestamp(now).build());
        let starts_at = now + 40;
        let ends_at = starts_at + 40;
        let resolution_window = 40;

        let market_data: MarketData =
            create_market_data("a market description".to_string(), 2, starts_at, ends_at);

        let mut contract: Market = setup_contract(market_data, resolution_window);

        // @TODO publish may also be made by a CT ft_on_transfer
        contract.publish();

        testing_env!(context.block_timestamp(starts_at - 40).build());
        let mut alice_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            100.0,
            yes,
        );

        testing_env!(context.block_timestamp(starts_at - 30).build());
        let mut bob_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            bob(),
            100.0,
            yes,
        );

        testing_env!(context.block_timestamp(starts_at - 20).build());
        let mut carol_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            carol(),
            100.0,
            yes,
        );

        testing_env!(context.block_timestamp(starts_at - 10).build());
        let mut daniel_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            daniel(),
            100.0,
            yes,
        );

        // let mut outcome_token_yes: OutcomeToken = contract.get_outcome_token(yes);

        // assert_eq!(
        //     outcome_token_yes.total_supply(),
        //     alice_balance + bob_balance + carol_balance + daniel_balance
        // );

        // Buy NO token
        testing_env!(context.block_timestamp(starts_at - 40).build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            emily(),
            100.0,
            no,
        );

        testing_env!(context.block_timestamp(starts_at - 30).build());
        buy(
            &mut contract,
            &mut collateral_token_balance,
            frank(),
            100.0,
            no,
        );

        testing_env!(context.block_timestamp(starts_at - 20).build());
        let mut gus_balance = buy(
            &mut contract,
            &mut collateral_token_balance,
            gus(),
            100.0,
            no,
        );

        assert_eq!(collateral_token_balance, 700.0);

        // alice sells half her OT balance while the event is ongoing
        testing_env!(context.signer_account_id(alice()).build());
        // let amount = alice_balance / 2.0;
        alice_balance -= sell(
            &mut contract,
            &mut collateral_token_balance,
            alice_balance,
            yes,
        );
        assert_eq!(alice_balance, 0.0);

        // gus sells his full OT balance while the event is ongoing
        testing_env!(context.signer_account_id(gus()).build());
        gus_balance -= sell(
            &mut contract,
            &mut collateral_token_balance,
            gus_balance,
            no,
        );
        assert_eq!(gus_balance, 0.0);

        // Event is over. Resolution window is open
        let now = (ends_at + resolution_window - 20) as u32;
        testing_env!(context.block_timestamp(now.into()).build());

        // Resolve the market
        testing_env!(context.signer_account_id(dao_account_id()).build());
        resolve(&mut contract, &mut collateral_token_balance, yes);

        // Burn the losers
        let outcome_token_no = contract.get_outcome_token(no);
        let emily_balance = outcome_token_no.get_balance(&emily());
        assert_eq!(emily_balance, 0.0);
        let frank_balance = outcome_token_no.get_balance(&emily());
        assert_eq!(frank_balance, 0.0);

        // bob sells his OT balance after the market is resolved. Claim earnings!!
        testing_env!(context.signer_account_id(bob()).build());
        bob_balance -= sell(
            &mut contract,
            &mut collateral_token_balance,
            bob_balance,
            yes,
        );
        assert_eq!(bob_balance, 0.0);

        testing_env!(context.signer_account_id(carol()).build());
        carol_balance -= sell(
            &mut contract,
            &mut collateral_token_balance,
            carol_balance,
            yes,
        );
        assert_eq!(carol_balance, 0.0);

        testing_env!(context.signer_account_id(daniel()).build());
        daniel_balance -= sell(
            &mut contract,
            &mut collateral_token_balance,
            daniel_balance,
            yes,
        );
        assert_eq!(daniel_balance, 0.0);

        let outcome_token_yes = contract.get_outcome_token(yes);
        // Convert to u64 because a f64 0 is not precise
        assert_eq!(outcome_token_yes.total_supply() as u64, 0);

        // Convert to u64 because a f64 0 is not precise
        assert_eq!(collateral_token_balance as u64, 0);
    }
}
