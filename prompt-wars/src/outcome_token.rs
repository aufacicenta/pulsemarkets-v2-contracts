use near_sdk::{collections::UnorderedMap, log, AccountId};
use num_format::ToFormattedString;
use shared::OutcomeId;

use crate::{
    storage::{OutcomeToken, WrappedBalance},
    FORMATTED_STRING_LOCALE,
};

impl Default for OutcomeToken {
    fn default() -> Self {
        panic!("OutcomeToken should be initialized before usage")
    }
}

impl OutcomeToken {
    pub fn new(player_id: AccountId, value: String, initial_supply: WrappedBalance) -> Self {
        Self {
            player_id,
            value,
            total_supply: initial_supply,
            balances: UnorderedMap::new(format!("OT:{}", player_id).as_bytes().to_vec()),
            accounts_length: 0,
            is_active: true,
        }
    }

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
            amount.to_formatted_string(&FORMATTED_STRING_LOCALE),
            self.outcome_id,
            account_id,
            self.total_supply()
                .to_formatted_string(&FORMATTED_STRING_LOCALE)
        );
    }

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

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.total_supply = 0;
    }

    pub fn get_balance(&self, account_id: &AccountId) -> WrappedBalance {
        self.balances.get(account_id).unwrap_or(0)
    }

    pub fn get_accounts_length(&self) -> u64 {
        self.accounts_length
    }

    pub fn total_supply(&self) -> WrappedBalance {
        self.total_supply
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
