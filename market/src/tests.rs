#[cfg(test)]
mod tests {
    use crate::storage::Market;
    use chrono::Utc;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance, PromiseResult};
    use crate::storage::*;

    const ATTACHED_DEPOSIT: Balance = 1_000_000_000_000_000_000_000_000; // 1 Near

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
        description: String,
        options: u8,
        expiration_date: u64,
        resolution_window: u64,
    ) -> MarketData {
        MarketData {
            description,
            info: "".to_string(),
            category: None,
            subcategory: None,
            options: (0 .. options).map(|s| s.to_string()).collect(),
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
    fn test_deposits_by_account_with_no_balance() {
        let contract = Market::new(
            create_marketdata("".to_string(), 1, 1, 1),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );

        assert_eq!(
            0,
            contract.deposits_by_account(&alice(), &1),
            "Account deposits by Alice with option idx 1 should be 0"
        );
    }

    #[test]
    fn test_deposits_by_account_with_balance() {
        let mut context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata("".to_string(), 1, expires_at, 100),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );

        contract.publish_market();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![
                PromiseResult::Successful(vec![])
            ],
        );

        contract.on_create_proposals_callback();

        testing_env!(context
            .signer_account_id(bob())
            .attached_deposit(ATTACHED_DEPOSIT)
            .build());

        contract.bet(0);

        assert_eq!(
            ATTACHED_DEPOSIT,
            contract.deposits_by_account(&bob(), &0),
            "Account deposits by Bob with option 0 should be 1 Near"
        );

        assert_eq!(
            0,
            contract.deposits_by_account(&bob(), &1),
            "Account deposits by Bob with option 1 should be 0"
        );
    }

    #[test]
    fn test_deposits_by_option_with_no_balance() {
        let contract = Market::new(
            create_marketdata("".to_string(), 1, 1, 1),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );

        assert_eq!(
            0,
            contract.deposits_by_option(&1),
            "Account deposits with option idx 1 should be 0"
        );
    }

    #[test]
    fn test_deposits_by_option_with_balance() {
        let mut context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );

        contract.publish_market();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![
                PromiseResult::Successful(vec![])
            ],
        );

        contract.on_create_proposals_callback();

        testing_env!(context
            .signer_account_id(bob())
            .attached_deposit(ATTACHED_DEPOSIT)
            .build());

        contract.bet(0);

        assert_eq!(
            ATTACHED_DEPOSIT,
            contract.deposits_by_option(&0),
            "Account deposits with option idx 0 should be 1 Near"
        );

        testing_env!(context
            .signer_account_id(carol())
            .attached_deposit(ATTACHED_DEPOSIT / 2)
            .build());

        contract.bet(1);

        assert_eq!(
            ATTACHED_DEPOSIT / 2,
            contract.deposits_by_option(&1),
            "Account deposits with option idx 1 should be 0.5 Near"
        );

        assert_eq!(
            0,
            contract.deposits_by_option(&2),
            "Account deposits with option idx 2 should be 0"
        );
    }

    #[test]
    fn test_get_market_data() {
        let contract = Market::new(
            create_marketdata("".to_string(), 1, 1, 1),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );

        assert_eq!(
            create_marketdata("".to_string(), 1, 1, 1),
            contract.get_market_data()
        );
    }

    #[test]
    fn test_is_published_true() {
        let context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );

        contract.publish_market();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![
                PromiseResult::Successful(vec![])
            ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            true,
            contract.is_published()
        );
    }

    #[test]
    #[should_panic(expected = "ERR_CREATE_PROPOSALS_UNSUCCESSFUL")]
    fn test_is_published_false() {
        let context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );

        contract.publish_market();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![ PromiseResult::Failed ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            false,
            contract.is_published()
        );
    }

    #[test]
    fn test_is_resolved_true() {
        let mut context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
        );

        contract.publish_market();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![
                PromiseResult::Successful(vec![])
            ],
        );

        contract.on_create_proposals_callback();

        testing_env!(context
            .signer_account_id(alice())
            .build());

        contract.resolve(0);

        assert_eq!(
            true,
            contract.is_resolved()
        );
    }

    #[test]
    fn test_is_resolved_false() {
        let context = setup_context();

        let expires_at = add_expires_at_nanos(100);

        let mut contract = Market::new(
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
        );

        assert_eq!(
            false,
            contract.is_resolved()
        );

        contract.publish_market();

        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![
                PromiseResult::Successful(vec![])
            ],
        );

        contract.on_create_proposals_callback();

        assert_eq!(
            false,
            contract.is_resolved()
        );
    }

    #[test]
    fn test_is_market_expired_false() {
        let expires_at = add_expires_at_nanos(100);

        let contract = Market::new(
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
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
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
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
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
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
            create_marketdata("".to_string(), 2, expires_at, 100),
            AccountId::new_unchecked(alice().to_string()),
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
}
