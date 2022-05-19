use near_sdk::{collections::LookupMap, env, AccountId, Balance};

use crate::storage::{OutcomeId, OutcomeToken, Price};

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
        let mut accounts: LookupMap<AccountId, Balance> =
            LookupMap::new(format!("OT:{}", outcome_id).as_bytes().to_vec());

        accounts.insert(&env::current_account_id(), &initial_supply);

        Self {
            total_supply: initial_supply,
            accounts,
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
        let account_balance = self.accounts.get(account_id).unwrap_or(0);
        let new_balance = account_balance + amount;
        self.accounts.insert(account_id, &new_balance);
    }

    /**
     * @notice burn specific amount of tokens for an account
     * @param account_id the account_id to burn tokens for
     * @param amount the amount of tokens to burn
     */
    pub fn burn(&mut self, account_id: &AccountId, amount: Balance) {
        let mut balance = self.accounts.get(&account_id).unwrap_or(0);

        assert!(balance >= amount, "ERR_INSUFFICIENT_BALANCE");

        balance -= amount;
        self.accounts.insert(account_id, &balance);
        self.total_supply -= amount;
    }

    /**
     * @notice returns account's balance
     * @param account_id is the account_id to return the balance of
     * @returns `accoun_id`s balance
     */
    pub fn get_balance(&self, account_id: &AccountId) -> Balance {
        self.accounts.get(account_id).unwrap_or(0)
    }

    /**
     * @returns token's total supply
     */
    pub fn total_supply(&self) -> Balance {
        self.total_supply
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
        sender: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
    ) {
        self.withdraw(sender, amount);
        self.deposit(receiver_id, amount);
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

        let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);
        let new_balance = receiver_balance + amount;

        self.accounts.insert(&receiver_id, &new_balance);
    }

    /**
     * @notice withdraw token from an account
     * @param account_id to withdraw from
     * @param amount of tokens to withdraw
     */
    fn withdraw(&mut self, sender_id: &AccountId, amount: Balance) {
        let sender_balance = self.accounts.get(&sender_id).unwrap_or(0);

        assert!(amount > 0, "Cannot withdraw 0 or lower");
        assert!(sender_balance >= amount, "Not enough balance");

        let new_balance = sender_balance - amount;
        self.accounts.insert(&sender_id, &new_balance);
    }
}
