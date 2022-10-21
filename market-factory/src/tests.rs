#[cfg(test)]
mod tests {
    use crate::consts::STORAGE_DEPOSIT_BOND;
    use crate::storage::MarketFactory;
    use chrono::Utc;
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{serde_json::json, testing_env, AccountId};

    fn setup_context() -> VMContextBuilder {
        let mut context = VMContextBuilder::new();
        let now = Utc::now().timestamp_subsec_nanos();
        testing_env!(context
            .predecessor_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .attached_deposit(STORAGE_DEPOSIT_BOND * 2)
            .build());

        context
    }

    fn setup_contract() -> MarketFactory {
        let contract = MarketFactory::new();
        contract
    }

    #[test]
    fn create_market() {
        setup_context();

        let mut contract = setup_contract();
        let name = AccountId::new_unchecked("480c9dbe-a5ec".to_string());
        contract.create_market(name,
            json!({
                "args": "eyJtYXJrZXQiOnsiZGVzY3JpcHRpb24iOiJGSUZBIFdvcmxkY3VwIDIwMjI6IFNlbmVnYWwgdnMgTmV0aGVybGFuZHMiLCJpbmZvIjoibWFya2V0IGluZm8iLCJvcHRpb25zIjpbIlNlbmVnYWwiLCJOZXRoZXJsYW5kcyJdLCJzdGFydHNfYXQiOjE2NjYzNzM0MDAwMDAwMDAwMDAsImVuZHNfYXQiOjE2NjYzNzQwMDAwMDAwMDAwMDAsInV0Y19vZmZzZXQiOi02fSwiZGFvX2FjY291bnRfaWQiOiJwdWxzZS1kYW8uc3B1dG5pa3YyLnRlc3RuZXQiLCJjb2xsYXRlcmFsX3Rva2VuX2FjY291bnRfaWQiOiJ1c2R0LmZha2VzLnRlc3RuZXQiLCJzdGFraW5nX3Rva2VuX2FjY291bnRfaWQiOiJwdWxzZS5mYWtlcy50ZXN0bmV0IiwiZmVlX3JhdGlvIjowLjAyLCJjb2xsYXRlcmFsX3Rva2VuX2RlY2ltYWxzIjo2fQ=="
            }).to_string().into_bytes().to_vec().into());
    }
}
