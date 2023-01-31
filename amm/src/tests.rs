#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::{Duration, Utc};
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance, PromiseResult};
    use rand::seq::SliceRandom;

    const _ATTACHED_DEPOSIT: Balance = 1_000_000_000_000_000_000_000_000; // 1 Near

    // 0.02 or 2% for 6 decimals precision points, 0.02*1e6
    const LP_FEE: WrappedBalance = 20_000;

    const IX_ADDRESS: [u8; 32] = [
        173, 62, 255, 125, 45, 251, 162, 167, 128, 129, 25, 33, 146, 248, 118, 134, 118, 192, 215,
        84, 225, 222, 198, 48, 70, 49, 212, 195, 84, 136, 96, 56,
    ];

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

    fn setup_contract(market: MarketData, res: Option<Resolution>) -> Market {
        let ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        let mut resolution = Resolution {
            // 3 days
            window: market.ends_at + 259200 * 1_000_000_000,
            resolved_at: None,
            ix,
        };

        if let Some(res) = res {
            resolution.window = res.window;
        }

        let management = Management {
            dao_account_id: dao_account_id(),
            staking_token_account_id: None,
            market_creator_account_id: market_creator_account_id(),
        };

        let collateral_token = CollateralToken {
            id: collateral_token_id(),
            balance: 0,
            decimals: 6,
            fee_balance: 0,
        };

        let fees = Fees {
            staking_fees: None,
            market_creator_fees: None,
            claiming_window: None,
            fee_ratio: LP_FEE,
        };

        let contract = Market::new(market, resolution, management, collateral_token, fees);

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

    fn resolve(
        c: &mut Market,
        collateral_token_balance: &mut WrappedBalance,
        outcome_id: u64,
        instruction: Option<Ix>,
    ) {
        let mut ix: Ix = Ix {
            address: IX_ADDRESS,
        };

        if let Some(inst) = instruction {
            ix.address = inst.address;
        }

        c.resolve(outcome_id, ix);
        let balance = *collateral_token_balance;
        *collateral_token_balance -= c.calc_percentage(balance, c.get_fee_ratio());
    }

    fn create_outcome_tokens(c: &mut Market) {
        c.create_outcome_tokens();
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
    fn new_outcome_tokens() {
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

        let mut contract: Market = setup_contract(market_data, None);
        create_outcome_tokens(&mut contract);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.total_supply(), 0);
        assert_eq!(outcome_token_1.total_supply(), 0);
    }

    #[test]
    fn create_market_with_3_outcomes() {
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

        let mut contract: Market = setup_contract(market_data, None);

        create_outcome_tokens(&mut contract);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);
        let outcome_token_2: OutcomeToken = contract.get_outcome_token(2);

        assert_eq!(outcome_token_0.total_supply(), 0);
        assert_eq!(outcome_token_1.total_supply(), 0);
        assert_eq!(outcome_token_2.total_supply(), 0);
    }

    #[test]
    fn create_market_with_4_outcomes() {
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

        let mut contract: Market = setup_contract(market_data, None);

        create_outcome_tokens(&mut contract);

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);
        let outcome_token_2: OutcomeToken = contract.get_outcome_token(2);
        let outcome_token_3: OutcomeToken = contract.get_outcome_token(3);

        assert_eq!(outcome_token_0.total_supply(), 0);
        assert_eq!(outcome_token_1.total_supply(), 0);
        assert_eq!(outcome_token_2.total_supply(), 0);
        assert_eq!(outcome_token_3.total_supply(), 0);
    }

    #[test]
    fn binary_market_full_flow() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0;

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

        let mut contract: Market = setup_contract(market_data, None);
        let now = ends_at + Duration::days(1);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());
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
            400_000_000,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            emily(),
            100_000_000,
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
            300_000_000,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            frank(),
            100_000_000,
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
            200_000_000,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            gus(),
            100_000_000,
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
            100_000_000,
            yes,
        );

        assert_eq!(
            contract.get_collateral_token_metadata().balance,
            1_300_000_000
        );
        assert_eq!(
            contract.get_collateral_token_metadata().fee_balance,
            260_000
        );

        let outcome_token_yes = contract.get_outcome_token(yes);
        let outcome_token_no = contract.get_outcome_token(no);
        assert_eq!(outcome_token_yes.total_supply(), 999_800_000);
        assert_eq!(outcome_token_no.total_supply(), 299_940_000);

        // Resolve the market: Burn the losers
        testing_env!(context.predecessor_account_id(dao_account_id()).build());
        resolve(&mut contract, &mut collateral_token_balance, yes, None);
        let outcome_token_no = contract.get_outcome_token(no);
        assert_eq!(outcome_token_no.is_active(), false);
        assert_eq!(outcome_token_no.total_supply(), 0);

        // Resolution window is over
        let now = now + Duration::days(4);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());

        // alice sells all her OT balance after the market is resolved
        let alice_balance = contract.balance_of(yes, alice());
        testing_env!(context.signer_account_id(alice()).build());
        sell(&mut contract, alice(), alice_balance, yes, &context);
        let alice_balance = contract.balance_of(yes, alice());
        assert_eq!(alice_balance, 0);

        // bob sells his OT balance after the market is resolved. Claim earnings!!
        let bob_balance = contract.balance_of(yes, bob());
        testing_env!(context.signer_account_id(bob()).build());
        sell(&mut contract, bob(), bob_balance, yes, &context);
        let bob_balance = contract.balance_of(yes, bob());
        assert_eq!(bob_balance, 0);

        let carol_balance = contract.balance_of(yes, carol());
        testing_env!(context.signer_account_id(carol()).build());
        sell(&mut contract, carol(), carol_balance, yes, &context);
        let carol_balance = contract.balance_of(yes, carol());
        assert_eq!(carol_balance, 0);

        let daniel_balance = contract.balance_of(yes, daniel());
        testing_env!(context.signer_account_id(daniel()).build());
        sell(&mut contract, daniel(), daniel_balance, yes, &context);
        let daniel_balance = contract.balance_of(yes, daniel());
        assert_eq!(daniel_balance, 0);

        let outcome_token_yes = contract.get_outcome_token(yes);
        assert_eq!(outcome_token_yes.total_supply(), 0);

        assert_eq!(
            contract.get_collateral_token_metadata().balance,
            contract.get_collateral_token_metadata().fee_balance
        );
    }

    #[test]
    #[should_panic(expected = "ERR_CANT_SELL_A_LOSING_OUTCOME")]
    fn binary_market_errors_when_selling_losing_outcomes() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0;

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

        let mut contract: Market = setup_contract(market_data, None);
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
            400_000_000,
            yes,
        );
        buy(
            &mut contract,
            &mut collateral_token_balance,
            emily(),
            100_000_000,
            no,
        );

        // Resolve the market: Burn the losers
        testing_env!(context.predecessor_account_id(dao_account_id()).build());
        resolve(&mut contract, &mut collateral_token_balance, yes, None);

        // Resolution window is over
        let now = now + Duration::days(4);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());

        // emily tries to sell a losing outcome token
        let emily_balance = contract.balance_of(no, emily());
        testing_env!(context.signer_account_id(emily()).build());
        sell(&mut contract, emily(), emily_balance, no, &context);
    }

    #[test]
    fn market_with_4_outcomes() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0;

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

        let mut contract: Market = setup_contract(market_data, None);

        create_outcome_tokens(&mut contract);

        let amounts = vec![
            100_000_000,
            200_000_000,
            300_000_000,
            50_000_000,
            10_000_000,
            20_000_000,
            500_000_000,
        ];

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
    #[should_panic(expected = "ERR_MARKET_IS_CLOSED")]
    fn buy_error_if_event_is_ongoing() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0;

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

        let mut contract: Market = setup_contract(market_data, None);
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
            400_000_000,
            yes,
        );
    }

    #[test]
    fn sell_unresolved_market() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0;

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

        let mut contract: Market = setup_contract(market_data, None);
        create_outcome_tokens(&mut contract);

        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400_000_000,
            yes,
        );

        testing_env!(context
            .block_timestamp(block_timestamp(now + Duration::days(4)))
            .signer_account_id(alice())
            .build());
        let alice_balance = contract.balance_of(yes, alice());
        sell(&mut contract, alice(), alice_balance, yes, &context);

        assert_eq!(contract.balance_of(yes, alice()), 0);
    }

    #[test]
    #[should_panic(expected = "ERR_CREATE_OUTCOME_TOKENS_OUTCOMES_EXIST")]
    fn create_outcome_tokens_error() {
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

        let mut contract: Market = setup_contract(market_data, None);
        create_outcome_tokens(&mut contract);
        create_outcome_tokens(&mut contract);
    }

    #[test]
    #[should_panic(expected = "ERR_MARKET_IS_UNDER_RESOLUTION")]
    fn sell_error_if_market_is_under_resolution() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0;

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

        let mut contract: Market = setup_contract(market_data, None);
        create_outcome_tokens(&mut contract);

        buy(
            &mut contract,
            &mut collateral_token_balance,
            alice(),
            400_000_000,
            yes,
        );

        // Bypass the self.is_over check
        let now = ends_at + Duration::hours(1);
        testing_env!(context.block_timestamp(block_timestamp(now)).build());

        let alice_balance = contract.balance_of(yes, alice());
        sell(&mut contract, alice(), alice_balance, yes, &context);
    }

    #[test]
    fn on_claim_staking_fees_resolved_callback() {
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

        let mut contract: Market = setup_contract(market_data, None);
        create_outcome_tokens(&mut contract);

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            // These 2 amounts will get the weight % of the staking wallet: 50%
            vec![
                // ft_balance_of
                PromiseResult::Successful("5000000".to_string().into_bytes()),
                // ft_total_supply
                PromiseResult::Successful("10000000".to_string().into_bytes())
            ],
        );

        // This amount represents 85% of the total fee_balance
        let amount_payable = contract.on_claim_staking_fees_resolved_callback(221_000_000, alice());

        // (221 = 85% of total fees for $PULSE stakers) * (5 = alice.near weight of $PULSE total_supply) = 110.5,
        // then convert to Collateral Token decimals precision
        assert_eq!(amount_payable, "110500000");
        assert_eq!(contract.get_claimed_staking_fees(alice()), "110500000");
    }

    #[test]
    #[should_panic(expected = "ERR_SIGNER_IS_NOT_OWNER")]
    fn signer_is_not_owner() {
        let mut context = setup_context();

        let mut collateral_token_balance: WrappedBalance = 0;

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

        let mut contract: Market = setup_contract(market_data, None);
        create_outcome_tokens(&mut contract);

        let ix = Ix {
            address: [
                173, 62, 255, 125, 45, 251, 162, 167, 128, 129, 25, 33, 146, 248, 118, 134, 118,
                192, 215, 84, 225, 222, 198, 48, 70, 49, 212, 195, 84, 136, 96, 33,
            ],
        };

        resolve(&mut contract, &mut collateral_token_balance, yes, Some(ix));
    }
}
