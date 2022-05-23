use near_sdk::{
    collections::{LookupMap, UnorderedMap},
    env, AccountId, Balance,
};

use crate::storage::{OutcomeId, OutcomeToken, Price, PriceRatio};

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
    pub fn new(outcome_id: OutcomeId, initial_supply: Balance, price: Price) -> Self {
        Self {
            total_supply: initial_supply,
            balances: LookupMap::new(format!("OT:{}", outcome_id).as_bytes().to_vec()),
            lp_balances: UnorderedMap::new(format!("LP:{}", outcome_id).as_bytes().to_vec()),
            outcome_id,
            price,
        }
    }

    /**
     * @notice mint specific amount of tokens for an account
     * @param account_id the account_id to mint tokens for
     * @param amount the amount of tokens to mint
     */
    pub fn mint(&mut self, account_id: &AccountId, amount: Balance) {
        self.total_supply += amount;
        let account_balance = self.lp_balances.get(account_id).unwrap_or(0);
        let new_balance = account_balance + amount;
        self.lp_balances.insert(account_id, &new_balance);
    }

    /**
     * @notice burn specific amount of tokens for an account
     * @param account_id the account_id to burn tokens for
     * @param amount the amount of tokens to burn
     */
    pub fn burn(&mut self, account_id: &AccountId, amount: Balance) {
        let mut balance = self.lp_balances.get(&account_id).unwrap_or(0);

        assert!(balance >= amount, "ERR_INSUFFICIENT_BALANCE");

        balance -= amount;
        self.lp_balances.insert(account_id, &balance);
        self.total_supply -= amount;
    }

    /**
     * @notice transfer tokens from one account to another
     * @param receiver_id is the account that should receive the tokens
     * @param amount of tokens to transfer from predecessor to receiver
     */
    pub fn transfer(&mut self, receiver_id: &AccountId, amount: Balance) {
        self.withdraw(&env::predecessor_account_id(), amount);
        self.deposit(receiver_id, amount);
    }

    /**
     * @notice transfer tokens from one account to another
     * @param sender is the account that's sending the tokens
     * @param receiver_id is the account that should receive the tokens
     * @param amount of tokens to transfer from sender to receiver
     */
    pub fn safe_transfer_internal(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
    ) {
        self.withdraw(sender_id, amount);
        self.deposit(receiver_id, amount);
    }

    /**
     * @notice mint specific amount of tokens for an account
     * @param account_id the account_id to mint tokens for
     * @param amount the amount of tokens to mint
     */
    pub fn increase_price(&mut self, price_ratio: PriceRatio) {
        // @TODO self.price_ratio may be updated so that it doesn't reach 1
        self.price = self.price + price_ratio;
    }

    /**
     * @notice mint specific amount of tokens for an account
     * @param account_id the account_id to mint tokens for
     * @param amount the amount of tokens to mint
     */
    pub fn decrease_price(&mut self, price_ratio: PriceRatio) {
        // @TODO self.price_ratio may be updated so that it doesn't reach 1
        self.price = self.price - price_ratio;
    }

    /**
     * @notice returns account's balance
     * @param account_id is the account_id to return the balance of
     * @returns `accoun_id`s balance
     */
    pub fn get_balance(&self, account_id: &AccountId) -> Balance {
        self.balances.get(account_id).unwrap_or(0)
    }

    /**
     * @notice returns LP account's balance
     * @param account_id is the account_id to return the balance of
     * @returns `accoun_id`s balance
     */
    pub fn get_lp_balance(&self, account_id: &AccountId) -> Balance {
        self.lp_balances.get(account_id).unwrap_or(0)
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
    pub fn total_supply(&self) -> Balance {
        self.total_supply
    }

    /**
     * @returns the next LP account_id with balance > 0
     */
    pub fn get_lp_account(&self) -> Option<AccountId> {
        let mut lp_account_id: Option<AccountId> = None;

        for account_id in self.lp_balances.keys() {
            match self.lp_balances.get(&account_id.clone()) {
                Some(balance) => {
                    if balance > 0 {
                        lp_account_id = Some(account_id);
                        break;
                    }
                }
                None => env::panic_str("ERR_INVALID_LP_ACCOUNT_ID"),
            }
        }

        return lp_account_id;
    }
}

impl OutcomeToken {
    /**
     * @notice deposit tokens into an account
     * @param account_id the account_id to deposit into
     * @param amount the amount of tokens to deposit
     */
    fn deposit(&mut self, receiver_id: &AccountId, amount: Balance) {
        assert!(amount > 0, "Cannot deposit 0 or lower");

        let receiver_balance = self.balances.get(&receiver_id).unwrap_or(0);
        let new_balance = receiver_balance + amount;

        self.balances.insert(&receiver_id, &new_balance);
    }

    /**
     * @notice withdraw token from an account
     * @param account_id to withdraw from
     * @param amount of tokens to withdraw
     */
    fn withdraw(&mut self, sender_id: &AccountId, amount: Balance) {
        let sender_balance = self.balances.get(&sender_id).unwrap_or(0);

        assert!(amount > 0, "Cannot withdraw 0 or lower");
        assert!(sender_balance >= amount, "Not enough balance");

        let new_balance = sender_balance - amount;
        self.balances.insert(&sender_id, &new_balance);
    }
}
