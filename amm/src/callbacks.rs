use near_sdk::{env, log, near_bindgen, serde_json, AccountId, PromiseResult};
use sbv2_near::AggregatorRound;

use crate::storage::*;

#[near_bindgen]
impl Market {
    #[private]
    pub fn on_ft_transfer_callback(
        &mut self,
        amount: WrappedBalance,
        payee: AccountId,
        outcome_id: OutcomeId,
        amount_payable: WrappedBalance,
    ) -> String {
        match env::promise_result(0) {
            PromiseResult::Successful(_result) => {
                log!("on_ft_transfer_callback.amount_payable: {}", amount_payable);

                let mut outcome_token = self.get_outcome_token(outcome_id);
                outcome_token.burn(&payee, amount);

                self.update_ct_balance(-amount_payable);
                self.outcome_tokens.insert(&outcome_id, &outcome_token);

                if !self.is_over() {
                    self.update_prices(outcome_id, SetPriceOptions::Decrease);
                }

                return amount_payable.to_string();
            }
            _ => env::panic_str("ERR_ON_FT_TRANSFER_CALLBACK"),
        }
    }

    #[private]
    pub fn on_create_proposals_callback(&mut self) -> bool {
        match env::promise_result(0) {
            PromiseResult::Successful(_res) => {
                self.published_at = Some(self.get_block_timestamp());
                // add 3 days after published_at
                self.resolution_window = Some(self.get_block_timestamp() + 259200 * 1_000_000_000);
                // add 30 days after resolution_window
                self.fees.claiming_window =
                    Some(self.resolution_window() + 2592000 * 1_000_000_000);

                self.market_publisher_account_id = Some(env::signer_account_id());

                true
            }
            _ => env::panic_str("ERR_CREATE_PROPOSALS_UNSUCCESSFUL"),
        }
    }

    /**
     * Make sure that the market option proposal was created
     * Then create an NEP141 token, MOT
     */
    #[private]
    pub fn on_create_proposal_callback(&mut self, _market_options_idx: u64) {
        match env::promise_result(0) {
            PromiseResult::Successful(_res) => {}
            _ => env::panic_str("ERR_CREATE_PROPOSAL_UNSUCCESSFUL"),
        }
    }

    #[private]
    pub fn on_aggregator_read_callback(&mut self) {
        match env::promise_result(0) {
            PromiseResult::Successful(serialized_round) => {
                let round: AggregatorRound = serde_json::from_slice(&serialized_round).unwrap();

                // @TODO some aggregator values may not be f64?
                let result: f64 = round.result.try_into().unwrap();

                log!("aggregator_read value: {:?}", result);

                // @TODO this logic will only work for yes/no markets where value IS GREATER than,
                // eg. will Bitcoin be above 20,000.00 in Sept 28?
                // In the future, we may create markets by using different factories or by using a MarketType enum
                // NOTE: self.market.options MUST always start with YES then NO
                if self.market.price > result {
                    self.burn_the_losers(1);
                } else {
                    self.burn_the_losers(0);
                }

                // @TODO once a market is resolved, and the claiming window is over, we can reset the same contract to be a new market
                self.resolved_at = Some(self.get_block_timestamp());
            }
            _ => env::panic_str("ERR_ON_AGGREGATOR_READ_CALLBACK"),
        }
    }
}
