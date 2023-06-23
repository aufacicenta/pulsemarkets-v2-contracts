use near_sdk::{json_types::U128, near_bindgen, serde_json, AccountId, PromiseOrValue};

use crate::*;

pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl FungibleTokenReceiver for Market {
    #[payable]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let amount: WrappedBalance = amount.try_into().unwrap();
        assert!(amount > 0, "ERR_ZERO_AMOUNT");

        let payload: Payload = serde_json::from_str(&msg).expect("ERR_INVALID_PAYLOAD");

        match payload {
            Payload::BuyArgs(payload) => self.buy(sender_id, amount, payload),
            Payload::CreateOutcomeTokenArgs(payload) => {
                self.create_outcome_token(sender_id, amount, payload)
            }
        };

        // All the collateral was used, so we should issue no refund on ft_resolve_transfer
        return PromiseOrValue::Value(U128::from(0));
    }
}
