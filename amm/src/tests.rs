#[cfg(test)]
mod tests {
    use crate::storage::*;
    use crate::FungibleTokenReceiver;
    use chrono::Utc;
    use near_sdk::test_utils::test_env::{alice, bob};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{serde_json, testing_env, AccountId, Balance};

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

    #[test]
    fn publish_market_with_3_outcomes() {
        let starts_at = add_expires_at_nanos(100);
        let ends_at = add_expires_at_nanos(1000);
        let resolution_window = 3000;

        let market_data: MarketData = create_market_data(
            "a market description".to_string(),
            3,
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
        let outcome_token_2: OutcomeToken = contract.get_outcome_token(2);

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

        assert_eq!(
            outcome_token_2.total_supply(),
            0,
            "Initial supply must be 0"
        );

        assert_eq!(
            outcome_token_0.get_price(),
            0.3333333333333333,
            "Price must be 0.3333333333333333"
        );
        assert_eq!(
            outcome_token_1.get_price(),
            0.3333333333333333,
            "Price must be 0.3333333333333333"
        );
        assert_eq!(
            outcome_token_2.get_price(),
            0.3333333333333333,
            "Price must be 0.3333333333333333"
        );
    }

    #[test]
    fn add_liquidity() {
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

        let msg = serde_json::json!({
            "AddLiquidityArgs": {
                "outcome_id": 0,
            }
        });

        contract.ft_on_transfer(alice(), 10.into(), msg.to_string());

        let mut outcome_token_0: OutcomeToken = contract.get_outcome_token(0);
        let mut outcome_token_1: OutcomeToken = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.total_supply(), 10, "Supply must be 10");
        assert_eq!(outcome_token_1.total_supply(), 0, "Supply must be 0");

        assert_eq!(
            outcome_token_0.get_lp_balance(&alice()),
            10,
            "Balance must be 10"
        );

        outcome_token_0 = contract.get_outcome_token(0);
        outcome_token_1 = contract.get_outcome_token(1);

        assert_eq!(outcome_token_0.get_price(), 0.6, "Price must be 0.6");
        assert_eq!(outcome_token_1.get_price(), 0.4, "Price must be 0.4");

        contract.ft_on_transfer(bob(), 10.into(), msg.to_string());

        outcome_token_0 = contract.get_outcome_token(0);
        outcome_token_1 = contract.get_outcome_token(1);

        assert_eq!(
            outcome_token_0.get_lp_balance(&bob()),
            10,
            "Balance must be 10"
        );

        assert_eq!(outcome_token_0.get_price(), 0.7, "Price must be 0.7");

        assert_eq!(
            outcome_token_1.get_price(),
            0.30000000000000004,
            "Price must be 0.30000000000000004"
        );
    }
}
