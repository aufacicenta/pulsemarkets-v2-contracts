#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::Utc;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::PublicKey;
    use near_sdk::{testing_env, PromiseResult};
    use serde_json::json;

    fn setup_contract() -> (VMContextBuilder, EscrowFactory) {
        let mut context = VMContextBuilder::new();
        let pk: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
            .parse()
            .unwrap();
        testing_env!(context
            .signer_account_pk(pk)
            .current_account_id(alice())
            .build());
        let factory = EscrowFactory::new();
        (context, factory)
    }

    #[test]
    fn test_create_conditional_escrow() {
        let (mut context, mut factory) = setup_contract();

        let now = Utc::now().timestamp_nanos();
        let args = json!({ "expires_at": now, "funding_amount_limit": 1_000_000_000, "dao_factory_account_id": "daofactory.testnet", "ft_factory_account_id": "ftfactory.testnet", "metadata_url": "metadata_url.json" })
            .to_string()
            .into_bytes().to_vec().into();

        factory.create_conditional_escrow("conditional-escrow".parse().unwrap(), args);

        testing_env!(
            context.predecessor_account_id(alice()).build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );

        factory.on_create_conditional_escrow(
            format!("conditional-escrow.{}", alice()).parse().unwrap(),
            U128(0),
            alice(),
        );

        assert_eq!(
            factory.get_conditional_escrow_contracts_list(),
            vec![format!("conditional-escrow.{}", alice()).parse().unwrap()]
        );

        assert_eq!(
            factory.get_conditional_escrow_contracts(0, 100),
            vec![format!("conditional-escrow.{}", alice()).parse().unwrap()]
        );

        assert_eq!(factory.get_conditional_escrow_contracts_count(), 1);
    }

    #[test]
    #[should_panic(expected = "ERR_CREATE_CONDITIONAL_ESCROW_UNSUCCESSFUL")]
    fn test_create_conditional_escrow_fails() {
        let (mut context, mut factory) = setup_contract();

        let now = Utc::now().timestamp_nanos();
        let args = json!({ "expires_at": now, "funding_amount_limit": 1_000_000_000, "dao_factory_account_id": "daofactory.testnet", "ft_factory_account_id": "ftfactory.testnet", "metadata_url": "metadata_url.json" })
            .to_string()
            .into_bytes().to_vec().into();

        factory.create_conditional_escrow("conditional-escrow".parse().unwrap(), args);

        testing_env!(
            context.predecessor_account_id(alice()).build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Failed],
        );

        factory.on_create_conditional_escrow(
            format!("conditional-escrow.{}", alice()).parse().unwrap(),
            U128(0),
            alice(),
        );
    }
}
