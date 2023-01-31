use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId};
use num_format::ToFormattedString;
use std::default::Default;

use near_contract_standards::fungible_token::core::ext_ft_core;

use crate::consts::*;
use crate::storage::*;

#[ext_contract(ext_self)]
trait Callbacks {
    fn on_ft_transfer_callback(
        &mut self,
        amount: WrappedBalance,
        payee: AccountId,
        outcome_id: OutcomeId,
        amount_payable: WrappedBalance,
    ) -> String;
}

impl Default for Market {
    fn default() -> Self {
        env::panic_str("ERR_MARKET_NOT_INITIALIZED")
    }
}

#[near_bindgen]
impl Market {
    #[init]
    pub fn new(
        market: MarketData,
        resolution: Resolution,
        management: Management,
        collateral_token: CollateralToken,
        fees: Fees,
    ) -> Self {
        if env::state_exists() {
            env::panic_str("ERR_ALREADY_INITIALIZED");
        }

        if market.options.len() < 2 {
            env::panic_str("ERR_NEW_INSUFFICIENT_MARKET_OPTIONS");
        }

        let resolution_window = resolution.window;

        Self {
            market,
            collateral_token: CollateralToken {
                balance: 0,
                fee_balance: 0,
                // @TODO collateral_token_decimals should be set by a cross-contract call to ft_metadata, otherwise the system can be tamed
                ..collateral_token
            },
            outcome_tokens: LookupMap::new(StorageKeys::OutcomeTokens),
            resolution,
            management: Management {
                staking_token_account_id: Some(AccountId::new_unchecked(
                    STAKING_TOKEN_ACCOUNT_ID.to_string(),
                )),
                ..management
            },
            fees: Fees {
                staking_fees: Some(LookupMap::new(StorageKeys::StakingFees)),
                market_creator_fees: Some(LookupMap::new(StorageKeys::MarketCreatorFees)),
                // @TODO set to less time, currently 30 days after resolution window
                claiming_window: Some(resolution_window + 2592000 * 1_000_000_000),
                ..fees
            },
        }
    }

    pub fn create_outcome_tokens(&mut self) -> usize {
        match self.outcome_tokens.get(&0) {
            Some(_token) => env::panic_str("ERR_CREATE_OUTCOME_TOKENS_OUTCOMES_EXIST"),
            None => {
                for outcome_id in 0..self.market.options.len() {
                    self.create_outcome_token(outcome_id as u64);
                }

                self.market.options.len()
            }
        }
    }

    /**
     * Lets accounts purchase OTs from the OT LP pools balances in exchange of CT
     * The price is calculated at the time of betting
     *
     * Increments the price of the selected OT by the predefined percentage
     * Decrements the price of the other OTs by the predefined percentage
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Increments the balance of OT in the buyer's balance
     * Decrements the balance of OT in the OT LP pool balance
     *
     * Charges lp_fee from the CT and transfers it to the OT LP Pool fee balance
     * LPs can withdraw the fees at any time using their LPTs
     *
     * @notice only while the market is open
     *
     * @returns amount of OT bought
     */
    #[payable]
    #[private]
    pub fn buy(
        &mut self,
        sender_id: AccountId,
        amount: WrappedBalance,
        payload: BuyArgs,
    ) -> WrappedBalance {
        self.assert_is_open();
        self.assert_is_not_resolved();

        let mut outcome_token = self.get_outcome_token(payload.outcome_id);

        let (amount_mintable, fee) = self.get_amount_mintable(amount);

        log!("BUY amount: {}, fee_ratio: {}, fee_result: {}, outcome_id: {}, account_id: {}, supply: {}, amount_mintable: {}, fee_balance: {}",
            amount.to_formatted_string(&FORMATTED_STRING_LOCALE),
            self.fees.fee_ratio.to_formatted_string(&FORMATTED_STRING_LOCALE),
            fee.to_formatted_string(&FORMATTED_STRING_LOCALE),
            outcome_token.outcome_id,
            sender_id,
            outcome_token.total_supply().to_formatted_string(&FORMATTED_STRING_LOCALE),
            amount_mintable.to_formatted_string(&FORMATTED_STRING_LOCALE),
            self.collateral_token.fee_balance.to_formatted_string(&FORMATTED_STRING_LOCALE),
        );

        outcome_token.mint(&sender_id, amount_mintable);
        self.update_ct_balance(self.collateral_token.balance + amount);
        self.update_ct_fee_balance(fee);

        self.outcome_tokens
            .insert(&payload.outcome_id, &outcome_token);

        return amount_mintable;
    }

    /**
     * An account may sell their OTs and get their CT back
     * No lp_fee is charged on this transaction
     *
     * Transfers CT amount to the account if their OT amount <= balance
     *
     * Decrements the price of the selected OT by the predefined ratio
     * Increments the price of the other OTs by the predefined ratio
     * SUM of PRICES MUST EQUAL 1!!
     *
     * Decrements the balance of OT in the account's balance
     *
     * OT holders may always sell. The price is what changes.
     *
     * @notice only while the market is closed
     *
     * @param outcome_id, the id of the desired OT balance to sell
     * @param balance, how much of OTs balance to sell
     *
     * @returns amount of CT sold
     */
    #[payable]
    pub fn sell(&mut self, outcome_id: OutcomeId, amount: WrappedBalance) -> WrappedBalance {
        // @TODO if there are participants only in 1 outcome, allow to claim funds after resolution, otherwise funds will be locked
        if self.is_resolution_window_expired() && !self.is_resolved() {
            return self.internal_sell(outcome_id, amount);
        }

        self.assert_is_not_under_resolution();
        self.assert_is_resolved();

        return self.internal_sell(outcome_id, amount);
    }

    /**
     * Resolves the market
     * Sets the winning OT price to 1
     * Sets the losing OT prices to 0
     *
     * NOTE: this method could be called by ft_on_transfer by $PULSE owners only
     *
     * @notice only after the market start_date and end_date period is over
     * @notice only by a Sputnik2 DAO Function Call Proposal!!
     *
     * @returns
     */
    #[payable]
    pub fn resolve(&mut self, outcome_id: OutcomeId, ix: Ix) {
        self.assert_only_owner(ix);
        self.assert_is_not_resolved();
        self.assert_is_valid_outcome(outcome_id);

        self.assert_is_resolution_window_open();

        self.burn_the_losers(outcome_id);

        self.resolution.resolved_at = Some(self.get_block_timestamp());
    }

    #[private]
    pub fn update_ct_balance(&mut self, amount: WrappedBalance) -> WrappedBalance {
        log!(
            "update_ct_balance: {}",
            amount.to_formatted_string(&FORMATTED_STRING_LOCALE)
        );

        self.collateral_token.balance = amount;
        self.collateral_token.balance
    }

    #[private]
    pub fn update_ct_fee_balance(&mut self, amount: WrappedBalance) -> WrappedBalance {
        self.collateral_token.fee_balance += amount;
        self.collateral_token.fee_balance
    }
}

impl Market {
    fn burn_the_losers(&mut self, outcome_id: OutcomeId) {
        for id in 0..self.market.options.len() {
            let mut outcome_token = self.get_outcome_token(id as OutcomeId);
            if outcome_token.outcome_id != outcome_id {
                outcome_token.deactivate();
                self.outcome_tokens
                    .insert(&(id as OutcomeId), &outcome_token);
            }
        }
    }

    fn create_outcome_token(&mut self, outcome_id: OutcomeId) {
        let outcome_token = OutcomeToken::new(outcome_id, 0);
        self.outcome_tokens.insert(&outcome_id, &outcome_token);
    }

    fn internal_sell(&mut self, outcome_id: OutcomeId, amount: WrappedBalance) -> WrappedBalance {
        if amount > self.balance_of(outcome_id, env::signer_account_id()) {
            env::panic_str("ERR_SELL_AMOUNT_GREATER_THAN_BALANCE");
        }

        let outcome_token = self.get_outcome_token(outcome_id);

        let payee = env::signer_account_id();
        let (amount_payable, weight) = self.get_amount_payable(amount, outcome_id);

        if amount_payable <= 0 {
            env::panic_str("ERR_CANT_SELL_A_LOSING_OUTCOME");
        }

        log!(
            "SELL amount: {}, outcome_id: {}, account_id: {}, ot_balance: {}, supply: {}, is_resolved: {}, ct_balance: {},  weight: {}, amount_payable: {}",
            amount.to_formatted_string(&FORMATTED_STRING_LOCALE),
            outcome_id,
            payee,
            outcome_token.get_balance(&payee).to_formatted_string(&FORMATTED_STRING_LOCALE),
            outcome_token.total_supply().to_formatted_string(&FORMATTED_STRING_LOCALE),
            self.is_resolved(),
            self.collateral_token.balance.to_formatted_string(&FORMATTED_STRING_LOCALE),
            weight.to_formatted_string(&FORMATTED_STRING_LOCALE),
            amount_payable.to_formatted_string(&FORMATTED_STRING_LOCALE),
        );

        let ft_transfer_promise = ext_ft_core::ext(self.collateral_token.id.clone())
            .with_attached_deposit(FT_TRANSFER_BOND)
            .with_static_gas(GAS_FT_TRANSFER)
            .ft_transfer(payee.clone(), U128::from(amount_payable), None);

        let ft_transfer_callback_promise = ext_self::ext(env::current_account_id())
            .with_attached_deposit(0)
            .with_static_gas(GAS_FT_TRANSFER_CALLBACK)
            .on_ft_transfer_callback(amount, payee, outcome_id, amount_payable);

        ft_transfer_promise.then(ft_transfer_callback_promise);

        amount_payable
    }
}
