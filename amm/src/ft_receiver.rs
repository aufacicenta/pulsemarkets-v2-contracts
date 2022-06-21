use near_sdk::{near_bindgen, serde_json, AccountId};

use crate::*;

pub trait FungibleTokenReceiver {
    // @returns amount of unused tokens
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: String,
        msg: String,
    ) -> WrappedBalance;
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
        amount: String,
        msg: String,
    ) -> WrappedBalance {
        self.assert_is_published();

        let amount: WrappedBalance = amount.parse::<WrappedBalance>().unwrap();
        assert!(amount > 0.0, "ERR_ZERO_AMOUNT");

        let payload: Payload = serde_json::from_str(&msg).expect("ERR_INVALID_PAYLOAD");

        match payload {
            Payload::BuyArgs(payload) => self.buy(sender_id, amount, payload),
        }
    }
}
