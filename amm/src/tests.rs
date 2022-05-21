#[cfg(test)]
mod tests {
    use chrono::Utc;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance, PromiseResult};
    use near_contract_standards::fungible_token::core::FungibleTokenCore;
    use crate::market::*;
    use crate::math;

    const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000; // 1 Near

    fn setup_context() -> VMContextBuilder {
        let mut context = VMContextBuilder::new();
        let now = Utc::now().timestamp_subsec_nanos();
        testing_env!(context
            .predecessor_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());

        context
    }

    fn create_marketdata(
        options: u8,
        expiration_date: u64,
        resolution_window: u64,
    ) -> MarketData {
        MarketData {
            oracle: AccountId::new_unchecked("oracle.near".to_string()),
            question_id: 1,
            options,
            expiration_date,
            resolution_window,
        }
    }

    fn add_expires_at_nanos(offset: u32) -> u64 {
        let now = Utc::now().timestamp_subsec_nanos();
        (now + offset).into()
    }

    ////////////////
    // Testing views
    ////////////////
    #[test]
    fn test_get_market_data() {
        let contract = Market::new(
            create_marketdata(2, 1, 1),
            AccountId::new_unchecked("collateral.near".to_string()),
            10
        );

        assert_eq!(
            create_marketdata(2, 1, 1),
            contract.get_market_data()
        );
    }

    #[test]
    fn test_get_status() {
        let contract = Market::new(
            create_marketdata(2, 1, 1),
            AccountId::new_unchecked("collateral.near".to_string()),
            10
        );

        assert_eq!(
            MarketStatus::Pending,
            contract.get_status()
        );
    }

    #[test]
    fn test_is_market_expired_false() {
        let expires_at = add_expires_at_nanos(100);

        let contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        assert_eq!(
            false,
            contract.is_market_expired()
        );
    }

    #[test]
    fn test_is_market_expired_true() {
        let mut context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        let now = Utc::now().timestamp_subsec_nanos() + 1_000; 

        testing_env!(context
            .signer_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());
        
        assert_eq!(
            true,
            contract.is_market_expired()
        );
    }

    #[test]
    fn test_is_resolution_window_expired_false() {
        let expires_at = add_expires_at_nanos(100);

        let contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        assert_eq!(
            false,
            contract.is_resolution_window_expired()
        );
    }

    #[test]
    fn test_is_resolution_window_expired_true() {
        let mut context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        let now = Utc::now().timestamp_subsec_nanos() + 1_000; 

        testing_env!(context
            .signer_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());
        
        assert_eq!(
            true,
            contract.is_resolution_window_expired()
        );
    }

    //////////////////
    // Testing Publish
    //////////////////
    #[test]
    fn test_publish_success() {
        let context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Successful(vec![]) ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            MarketStatus::Running,
            contract.get_status()
        );
    }

    #[test]
    #[should_panic(expected = "ERR_CREATE_PROPOSALS_UNSUCCESSFUL")]
    fn test_publish_fail() {
        let context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Failed ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            MarketStatus::Pending,
            contract.get_status()
        );
    }

    #[test]
    #[should_panic(expected = "ERR_MARKET_ALREADY_PUBLISHED")]
    fn test_publish_fail_2() {
        let context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Successful(vec![]) ],
        );

        contract.on_create_proposals_callback();

        contract.publish();
    }

    #[test]
    #[should_panic(expected = "ERR_MARKET_EXPIRED")]
    fn test_publish_fail_3() {
        let mut context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        let now = Utc::now().timestamp_subsec_nanos() + 1_000; 

        testing_env!(context
            .signer_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());

        contract.publish();

    }

    ////////////////////////
    // Testing add_liquidity 
    ////////////////////////
    #[test]
    fn test_add_liquidity_success() {
        let mut context = setup_context();
        let expires_at = add_expires_at_nanos(100);
        let contract_account = AccountId::new_unchecked("amm.near".to_string());


        testing_env!(context
            .current_account_id(contract_account.clone())
            .build());

        let mut contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Successful(vec![]) ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            MarketStatus::Running,
            contract.get_status()
        );

        // ##################
        // Bob adds liquidity
        testing_env!(context
            .signer_account_id(bob())
            .attached_deposit(ONE_NEAR)
            .build());

        contract.add_liquidity();

        // Checking Collateral Tokens Balance
        assert_eq!(ONE_NEAR, contract.conditional_tokens.get_balance_by_token_idx(&0));
        assert_eq!(ONE_NEAR, contract.conditional_tokens.get_balance_by_token_idx(&1));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_token_idx(&2));

        // Checking Collateral Tokens Accounts
        assert_eq!(ONE_NEAR, contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone()));
        assert_eq!(ONE_NEAR, contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone()));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&1, &bob()));

        // Checking Liquidity Tokens
        assert_eq!(ONE_NEAR, contract.ft_balance_of(bob()).0);
        assert_eq!(0, contract.ft_balance_of(carol()).0);
        assert_eq!(0, contract.ft_balance_of(contract_account.clone()).0);

        // ####################
        // Alice adds liquidity
        testing_env!(context
            .signer_account_id(alice())
            .attached_deposit(ONE_NEAR * 3)
            .build());

        contract.add_liquidity();

        // Checking Collateral Tokens Balance
        assert_eq!(ONE_NEAR * 4, contract.conditional_tokens.get_balance_by_token_idx(&0));
        assert_eq!(ONE_NEAR * 4, contract.conditional_tokens.get_balance_by_token_idx(&1));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&1, &bob()));

        // Checking Collateral Tokens Accounts
        assert_eq!(ONE_NEAR * 4, contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone()));
        assert_eq!(ONE_NEAR * 4, contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone()));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&1, &bob()));

        // Checking Liquidity Tokens
        assert_eq!(ONE_NEAR, contract.ft_balance_of(bob()).0);
        assert_eq!(ONE_NEAR * 3, contract.ft_balance_of(alice()).0);
        assert_eq!(0, contract.ft_balance_of(carol()).0);

        // ########################
        // Bob adds liquidity again
        testing_env!(context
            .signer_account_id(bob())
            .attached_deposit(ONE_NEAR * 4)
            .build());

        contract.add_liquidity();

        // Checking Collateral Tokens Balance
        assert_eq!(ONE_NEAR * 8, contract.conditional_tokens.get_balance_by_token_idx(&0));
        assert_eq!(ONE_NEAR * 8, contract.conditional_tokens.get_balance_by_token_idx(&1));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&1, &bob()));

        // Checking Collateral Tokens Accounts
        assert_eq!(ONE_NEAR * 8, contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone()));
        assert_eq!(ONE_NEAR * 8, contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone()));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&1, &bob()));

        // Checking Liquidity Tokens
        assert_eq!(ONE_NEAR * 5, contract.ft_balance_of(bob()).0);
        assert_eq!(ONE_NEAR * 3, contract.ft_balance_of(alice()).0);
        assert_eq!(0, contract.ft_balance_of(carol()).0);
    }

    #[test]
    #[should_panic(expected = "ERR_DEPOSIT_SHOULD_NOT_BE_0")]
    fn test_add_liquidity_fail() {
        let context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Successful(vec![]) ],
        );

        contract.on_create_proposals_callback();

        contract.add_liquidity();
    }

    #[test]
    fn test_calc_buy_amount() {
        let mut context = setup_context();
        let expires_at = add_expires_at_nanos(100);
        let contract_account = AccountId::new_unchecked("amm.near".to_string());


        testing_env!(context
            .current_account_id(contract_account.clone())
            .build());

        let mut contract = Market::new(
            create_marketdata(2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Successful(vec![]) ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            MarketStatus::Running,
            contract.get_status()
        );

        // ##################
        // Bob adds liquidity
        testing_env!(context
            .signer_account_id(bob())
            .attached_deposit(ONE_NEAR * 10)
            .build());

        contract.add_liquidity();

        // Checking Collateral Tokens Balance
        assert_eq!(ONE_NEAR * 10, contract.conditional_tokens.get_balance_by_token_idx(&0));
        assert_eq!(ONE_NEAR * 10, contract.conditional_tokens.get_balance_by_token_idx(&1));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_token_idx(&2));

        // Checking Collateral Tokens Accounts
        assert_eq!(ONE_NEAR * 10, contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone()));
        assert_eq!(ONE_NEAR * 10, contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone()));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&1, &bob()));

        // Checking Liquidity Tokens
        assert_eq!(ONE_NEAR * 10, contract.ft_balance_of(bob()).0);
        assert_eq!(0, contract.ft_balance_of(carol()).0);
        assert_eq!(0, contract.ft_balance_of(contract_account.clone()).0);

        assert_eq!(ONE_NEAR * 15, contract.calc_buy_amount(ONE_NEAR * 10, 1));
    }

    #[test]
    fn test_get_balances() {
        let mut context = setup_context();
        let expires_at = add_expires_at_nanos(100);
        let contract_account = AccountId::new_unchecked("amm.near".to_string());
        let outcomes = 2;

        testing_env!(context
            .current_account_id(contract_account.clone())
            .build());

        let mut contract = Market::new(
            create_marketdata(outcomes, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Successful(vec![]) ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            vec![0, 0],
            contract.conditional_tokens.get_balances(outcomes as u64)
        );

        // Bob adds liquidity
        testing_env!(context
            .signer_account_id(bob())
            .attached_deposit(ONE_NEAR)
            .build());

        contract.add_liquidity();

        assert_eq!(
            vec![ONE_NEAR, ONE_NEAR],
            contract.conditional_tokens.get_balances(outcomes as u64)
        );

        // Alice adds liquidity
        testing_env!(context
            .signer_account_id(alice())
            .attached_deposit(ONE_NEAR * 2)
            .build());

        contract.add_liquidity();

        assert_eq!(
            vec![ONE_NEAR * 3, ONE_NEAR * 3],
            contract.conditional_tokens.get_balances(outcomes as u64)
        );
    }

    #[test]
    fn test_buy() {
        let mut context = setup_context();
        let expires_at = add_expires_at_nanos(100);
        let contract_account = AccountId::new_unchecked("amm.near".to_string());
        let outcomes = 2;

        testing_env!(context
            .current_account_id(contract_account.clone())
            .build());

        let mut contract = Market::new(
            create_marketdata(outcomes, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
            10
        );

        contract.publish();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Successful(vec![]) ],
        );

        contract.on_create_proposals_callback();

        // Bob adds liquidity
        testing_env!(context
            .signer_account_id(bob())
            .attached_deposit(ONE_NEAR * 10)
            .build());

        contract.add_liquidity();

        assert_eq!(vec![ONE_NEAR * 10, ONE_NEAR * 10], contract.conditional_tokens.get_balances(outcomes as u64));

        let balance_outcome_0 = contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone());
        let balance_outcome_1 = contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone());
        let k = math::complex_mul_u128(ONE_NEAR, balance_outcome_0, balance_outcome_1);

        // #####
        // Alice Buys the outcome 1 with 10 as collateral
        testing_env!(context
            .signer_account_id(alice())
            .attached_deposit(ONE_NEAR * 10)
            .build());

        contract.buy(1, 0);

        assert_eq!(vec![ONE_NEAR * 20, ONE_NEAR * 20], contract.conditional_tokens.get_balances(outcomes as u64));

        assert_eq!(k / ONE_NEAR, math::complex_mul_u128(ONE_NEAR, balance_outcome_0, balance_outcome_1) / ONE_NEAR);

        // Checking Collateral Tokens Accounts
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&0, &alice()));
        assert_eq!(ONE_NEAR * 15, contract.conditional_tokens.get_balance_by_account(&1, &alice()));
        assert_eq!(0, contract.conditional_tokens.get_balance_by_account(&1, &bob()));

        assert_eq!(ONE_NEAR * 20, contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone()));
        assert_eq!(ONE_NEAR * 5, contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone()));

        // #####
        // Alice Buys the outcome 0 with 1 as collateral
        testing_env!(context
            .signer_account_id(alice())
            .attached_deposit(ONE_NEAR)
            .build());

        contract.buy(0, 0);

        let balance_outcome_0 = contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone());
        let balance_outcome_1 = contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone());

        assert_eq!(k / ONE_NEAR, math::complex_mul_u128(ONE_NEAR, balance_outcome_0, balance_outcome_1) / ONE_NEAR);

        // #####
        // Alice Buys the outcome 1 with 1 as collateral
        testing_env!(context
            .signer_account_id(alice())
            .attached_deposit(ONE_NEAR * 2)
            .build());

        contract.buy(1, 0);

        let balance_outcome_0 = contract.conditional_tokens.get_balance_by_account(&0, &contract_account.clone());
        let balance_outcome_1 = contract.conditional_tokens.get_balance_by_account(&1, &contract_account.clone());

        assert_eq!(k / ONE_NEAR, math::complex_mul_u128(ONE_NEAR, balance_outcome_0, balance_outcome_1) / ONE_NEAR);
    }
}
