#[cfg(test)]
mod tests {
    use crate::storage::MarketFactory;
    use chrono::Utc;
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId};

    fn setup_context() -> VMContextBuilder {
        let mut context = VMContextBuilder::new();
        let now = Utc::now().timestamp_subsec_nanos();
        testing_env!(context
            .predecessor_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());

        context
    }

    fn setup_contract() -> MarketFactory {
        let contract =
            MarketFactory::new(AccountId::new_unchecked("escrowfactory.near".to_string()));
        contract
    }

    #[test]
    fn get_markets() {}

    #[test]
    fn create_market() {}

    #[test]
    #[should_panic(expected = "ERR_ALREADY_INITIALIZED")]
    fn already_initialized_failure() {
        let mut context = setup_context();
    }

    #[test]
    #[should_panic(expected = "ERR_CREATE_MARKET_UNSUCCESSFUL")]
    fn market_creation_fails() {
        let mut context = setup_context();
    }
}
