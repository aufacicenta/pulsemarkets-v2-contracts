#[cfg(test)]
mod tests {
    use crate::storage::*;
    use chrono::Utc;
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance};

    const _ATTACHED_DEPOSIT: Balance = 1_000_000_000_000_000_000_000_000; // 1 Near

    const LP_FEE: f64 = 0.02;
    const PRICE_RATIO: f64 = 0.1;

    fn dao_account_id() -> AccountId {
        AccountId::new_unchecked("dao_account_id.near".to_string())
    }

    fn collateral_token_id() -> AccountId {
        AccountId::new_unchecked("collateral_token_id.near".to_string())
    }

    fn _setup_context() -> VMContextBuilder {
        let mut context = VMContextBuilder::new();
        let now = Utc::now().timestamp_subsec_nanos();
        testing_env!(context
            .predecessor_account_id(alice())
            .block_timestamp(now.try_into().unwrap())
            .build());

        context
    }

    fn setup_contract(market: MarketData) -> Market {
        let contract = Market::new(
            market,
            dao_account_id(),
            collateral_token_id(),
            LP_FEE,
            PRICE_RATIO,
        );

        contract
    }

    fn create_market_data(
        description: String,
        options: u8,
        starts_at: u64,
        ends_at: u64,
        resolution_window: u64,
    ) -> MarketData {
        MarketData {
            description,
            info: "".to_string(),
            category: None,
            options: (0..options).map(|s| s.to_string()).collect(),
            starts_at,
            ends_at,
            resolution_window,
        }
    }

    fn add_expires_at_nanos(offset: u32) -> u64 {
        let now = Utc::now().timestamp_subsec_nanos();
        (now + offset).into()
    }

    #[test]
    fn publish_binary_market() {
        let starts_at = add_expires_at_nanos(100);
        let ends_at = add_expires_at_nanos(1000);
        let resolution_window = 3000;

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            2,
            starts_at,
            ends_at,
            resolution_window,
        );

        let mut contract: Market = setup_contract(market_data);

        contract.publish();

        assert_eq!(
            contract.status.to_string(),
            MarketStatus::Published.to_string(),
            "Market status must be published"
        );

        let outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(
            outcome_token_0.total_supply(),
            0,
            "Initial supply must be 0"
        );

        assert_eq!(
            outcome_token_1.total_supply(),
            0,
            "Initial supply must be 0"
        );

        assert_eq!(outcome_token_0.get_price(), 0.5, "Price must be 0.5");
        assert_eq!(outcome_token_1.get_price(), 0.5, "Price must be 0.5");
    }
}
