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
    /**
     * @notice a callback function only callable by the collateral token for this market
     * @param sender_id the sender of the original transaction
     * @param amount of tokens attached to this callback call
     * @param msg can be a string of any type, in this case we expect a stringified json object
     * @returns the amount of tokens that were not spent
     */
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
        };

        // All the collateral was used, so we should issue no refund on ft_resolve_transfer
        return PromiseOrValue::Value(U128::from(0));
    }
}
