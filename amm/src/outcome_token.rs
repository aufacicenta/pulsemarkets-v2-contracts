use near_sdk::{
    collections::{LookupMap, UnorderedMap},
    AccountId,
};

use crate::storage::{OutcomeId, OutcomeToken, Price, PriceRatio, WrappedBalance};

impl Default for OutcomeToken {
    fn default() -> Self {
        panic!("OutcomeToken should be initialized before usage")
    }
}

impl OutcomeToken {
    /**
     * @notice create new outcome token
     * @param outcome_id the outcome this token represent within the pool
     * @param the initial supply to be minted at creation
     * @returns the newly created outcome token
     * */
    pub fn new(outcome_id: OutcomeId, initial_supply: WrappedBalance, price: Price) -> Self {
        Self {
            total_supply: initial_supply,
            balances: LookupMap::new(format!("OT:{}", outcome_id).as_bytes().to_vec()),
            accounts_length: 0,
            outcome_id,
            price,
        }
    }

    /**
     * @notice mint specific amount of tokens for an account
     * @param account_id the account_id to mint tokens for
     * @param amount the amount of tokens to mint
     */
    pub fn mint(&mut self, account_id: &AccountId, amount: WrappedBalance) {
        assert!(amount > 0.0, "ERR_MINT_AMOUNT_LOWER_THAN_0");

        let balance = self.balances.get(account_id).unwrap_or(0.0);
        let new_balance = balance + amount;
        self.balances.insert(account_id, &new_balance);
        self.total_supply += amount;
        self.accounts_length += if balance == 0.0 { 1 } else { 0 };
    }

    /**
     * @notice burn specific amount of tokens for an account
     * @param account_id, the account_id to burn tokens for
     * @param amount, the amount of tokens to burn
     */
    pub fn burn(&mut self, account_id: &AccountId, amount: WrappedBalance) {
        let balance = self.balances.get(&account_id).unwrap_or(0.0);

        assert!(balance >= amount, "ERR_BURN_INSUFFICIENT_BALANCE");

        let new_balance = balance - amount;
        self.balances.insert(account_id, &new_balance);
        self.total_supply -= amount;
        self.accounts_length -= if balance - amount == 0.0 { 1 } else { 0 };
    }

    /**
     * @notice update price
     * @param price f64
     */
    pub fn set_price(&mut self, price: WrappedBalance) {
        self.price = price;
    }

    /**
     * @notice increase price
     * @param price_ratio a number between 0 and 1. Price should always > 0 < 1
     */
    pub fn increase_price(&mut self, price_ratio: PriceRatio) {
        self.set_price(self.price + price_ratio);
    }

    /**
     * @notice decrease price
     * @param price_ratio a number between 0 and 1. Price should always > 0 < 1
     */
    pub fn decrease_price(&mut self, price_ratio: PriceRatio) {
        self.set_price(self.price - price_ratio);
    }

    /**
     * @notice returns account's balance
     * @param account_id is the account_id to return the balance of
     * @returns `accoun_id`s balance
     */
    pub fn get_balance(&self, account_id: &AccountId) -> WrappedBalance {
        self.balances.get(account_id).unwrap_or(0.0)
    }

    /**
     *
     */
    pub fn get_accounts_length(&self) -> u64 {
        self.accounts_length
    }

    /**
     * @notice returns account's balance
     * @param account_id is the account_id to return the balance of
     * @returns `accoun_id`s balance
     */
    pub fn get_price(&self) -> Price {
        self.price
    }

    /**
     * @returns token's total supply
     */
    pub fn total_supply(&self) -> WrappedBalance {
        self.total_supply
    }
}

impl OutcomeToken {
    /**
     * @notice deposit tokens into an account
     * @param account_id the account_id to deposit into
     * @param amount the amount of tokens to deposit
     */
    fn deposit(&mut self, receiver_id: &AccountId, amount: WrappedBalance) {
        assert!(amount > 0.0, "ERR_DEPOSIT_AMOUNT_LOWER_THAN_0");

        let receiver_balance = self.balances.get(&receiver_id).unwrap_or(0.0);
        let new_balance = receiver_balance + amount;

        self.balances.insert(&receiver_id, &new_balance);
    }

    /**
     * @notice withdraw token from an account
     * @param account_id to withdraw from
     * @param amount of tokens to withdraw
     */
    fn withdraw(&mut self, sender_id: &AccountId, amount: WrappedBalance) {
        let sender_balance = self.balances.get(&sender_id).unwrap_or(0.0);

        assert!(amount > 0.0, "ERR_WITHDRAW_AMOUNT_LOWER_THAN_0");
        assert!(sender_balance >= amount, "ERR_BALANCE_LOWER_THAN_AMOUNT");

        let new_balance = sender_balance - amount;
        self.balances.insert(&sender_id, &new_balance);
    }
}
