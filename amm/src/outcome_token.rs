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
            lp_pool_balance: 0,
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
        let lp_balance = self.lp_balances.get(account_id).unwrap_or(0);
        let new_balance = lp_balance + amount;
        self.lp_pool_balance += amount;
        self.lp_balances.insert(account_id, &new_balance);
    }

    /**
     * @notice burn specific amount of tokens for an account
     * @param account_id the account_id to burn tokens for
     * @param amount the amount of tokens to burn
     */
    pub fn burn(&mut self, account_id: &AccountId, amount: Balance) {
        let mut lp_balance = self.lp_balances.get(&account_id).unwrap_or(0);

        assert!(lp_balance >= amount, "ERR_INSUFFICIENT_BALANCE");

        lp_balance -= amount;
        self.lp_balances.insert(account_id, &lp_balance);
        self.lp_pool_balance -= amount;
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
     * @notice transfer tokens from LP pool to receiver account
     * @notice subtract an equal amount proportion to all LPs
     * @param receiver_id is the account that should receive the tokens
     * @param amount of tokens to transfer from LP Pool to receiver
     */
    pub fn lp_pool_transfer(&mut self, receiver_id: &AccountId, amount: Balance) {
        assert!(amount > 0, "ERR_AMOUNT_LOWER_THAN_0");

        // @TODO if lp_pool_balance is not enough for amount, then mint more tokens?
        assert!(
            self.lp_pool_balance >= amount,
            "ERR_LP_POOL_BALANCE_LOWER_THAN_AMOUNT"
        );

        let receiver_balance = self.balances.get(&receiver_id).unwrap_or(0);
        let new_balance = receiver_balance + amount;

        self.balances.insert(&receiver_id, &new_balance);

        let lp_balances = self.lp_balances.to_vec();
        for values in lp_balances.iter() {
            let lp_account_id = &values.0;
            let lp_balance = &values.1;
            let lp_weight: f64 = (lp_balance / self.lp_pool_balance) as f64;
            env::log_str(&*format!(
                "lp_pool_balance: {}, lp_account_id: {}, lp_balance: {}, lp_weight: {}",
                self.lp_pool_balance, lp_account_id, lp_balance, lp_weight
            ));
            let new_lp_balance = lp_balance - (amount as f64 * lp_weight);
            self.lp_balances.insert(&lp_account_id, &new_lp_balance);
        }

        self.lp_pool_balance -= amount;
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
     * @notice returns LP account's balance
     * @param account_id is the account_id to return the balance of
     * @returns `accoun_id`s balance
     */
    pub fn get_lp_pool_balance(&self) -> Balance {
        self.lp_pool_balance
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
