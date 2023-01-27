use near_sdk::{collections::UnorderedMap, log, AccountId};

use crate::storage::{OutcomeId, OutcomeToken, WrappedBalance};

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
    pub fn new(outcome_id: OutcomeId, initial_supply: WrappedBalance) -> Self {
        Self {
            total_supply: initial_supply,
            balances: UnorderedMap::new(format!("OT:{}", outcome_id).as_bytes().to_vec()),
            accounts_length: 0,
            outcome_id,
            is_active: true,
        }
    }

    /**
     * @notice mint specific amount of tokens for an account
     * @param account_id the account_id to mint tokens for
     * @param amount the amount of tokens to mint
     */
    pub fn mint(&mut self, account_id: &AccountId, amount: WrappedBalance) {
        assert_eq!(self.is_active, true, "ERR_MINT_INACTIVE");
        assert!(amount > 0, "ERR_MINT_AMOUNT_LOWER_THAN_0");

        let balance = self.balances.get(account_id).unwrap_or(0);
        let new_balance = balance + amount;
        self.balances.insert(account_id, &new_balance);
        self.total_supply += amount;
        self.accounts_length += if balance == 0 { 1 } else { 0 };

        log!(
            "Minted {} of outcome_token [{}] for {}. Supply: {}",
            amount,
            self.outcome_id,
            account_id,
            self.total_supply()
        );
    }

    /**
     * @notice burn specific amount of tokens for an account
     * @param account_id, the account_id to burn tokens for
     * @param amount, the amount of tokens to burn
     */
    pub fn burn(&mut self, account_id: &AccountId, amount: WrappedBalance) {
        assert_eq!(self.is_active, true, "ERR_BURN_INACTIVE");

        let balance = self.balances.get(&account_id).unwrap_or(0);

        assert!(balance >= amount, "ERR_BURN_INSUFFICIENT_BALANCE");

        let new_balance = balance - amount;
        self.balances.insert(account_id, &new_balance);
        self.total_supply -= amount;
        self.accounts_length -= if new_balance == 0 && self.accounts_length != 0 {
            1
        } else {
            0
        };

        log!(
            "Burned {} of outcome_token [{}] for {}. Supply: {}",
            amount,
            self.outcome_id,
            account_id,
            self.total_supply()
        );
    }

    /**
     * @notice burn all the tokens
     * @param account_id, the account_id to burn tokens for
     */
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.total_supply = 0;
    }

    /**
     * @notice returns account's balance
     * @param account_id is the account_id to return the balance of
     * @returns `accoun_id`s balance
     */
    pub fn get_balance(&self, account_id: &AccountId) -> WrappedBalance {
        self.balances.get(account_id).unwrap_or(0)
    }

    /**
     *
     */
    pub fn get_accounts_length(&self) -> u64 {
        self.accounts_length
    }

    /**
     * @returns token's total supply
     */
    pub fn total_supply(&self) -> WrappedBalance {
        self.total_supply
    }

    /**
     * @returns token is active
     */
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl OutcomeToken {
    /**
     * @notice deposit tokens into an account
     * @param account_id the account_id to deposit into
     * @param amount the amount of tokens to deposit
     */
    fn _deposit(&mut self, receiver_id: &AccountId, amount: WrappedBalance) {
        assert_eq!(self.is_active, true, "ERR_DEPOSIT_INACTIVE");
        assert!(amount > 0, "ERR_DEPOSIT_AMOUNT_LOWER_THAN_0");

        let receiver_balance = self.balances.get(&receiver_id).unwrap_or(0);
        let new_balance = receiver_balance + amount;

        self.balances.insert(&receiver_id, &new_balance);
    }

    /**
     * @notice withdraw token from an account
     * @param account_id to withdraw from
     * @param amount of tokens to withdraw
     */
    fn _withdraw(&mut self, sender_id: &AccountId, amount: WrappedBalance) {
        assert_eq!(self.is_active, true, "ERR_WITHDRAW_INACTIVE");
        let sender_balance = self.balances.get(&sender_id).unwrap_or(0);

        assert!(amount > 0, "ERR_WITHDRAW_AMOUNT_LOWER_THAN_0");
        assert!(sender_balance >= amount, "ERR_BALANCE_LOWER_THAN_AMOUNT");

        let new_balance = sender_balance - amount;
        self.balances.insert(&sender_id, &new_balance);
    }
}
