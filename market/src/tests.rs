#[cfg(test)]
mod tests {
    use crate::storage::Market;
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

    fn setup_contract() -> Market {
        let contract = Market::new(
            "Market description".to_string(),
            AccountId::new_unchecked("dao_account_id.near".to_string()),
        );
        contract
    }

    #[test]
    fn get_description() {}

    #[test]
    fn get_dao_account_id() {}

    #[test]
    fn process_market_options_and_stores_proposal_ids() {}

    #[test]
    #[should_panic(expected = "ERR_ALREADY_INITIALIZED")]
    fn already_initialized_failure() {
        let mut context = setup_context();
    }

    #[test]
    #[should_panic(expected = "ERR_CREATE_DAO_PROPOSAL_UNSUCCESSFUL")]
    fn dao_creation_fails() {
        let mut context = setup_context();
    }
}
