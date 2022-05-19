use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, AccountId, Balance, BorshStorageKey};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ConditionalTokens {
    pub tokens: LookupMap<u64, LookupMap<AccountId, Balance>>,
    pub total_balances: LookupMap<u64, Balance>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    ConditionalTokensBalances,
    ConditionalTokensTotalBalances,
    SubConditionalTokensAccounts { account_hash: Vec<u8> },
}

impl ConditionalTokens {

    pub fn mint(&mut self, token_idx: u64, account: AccountId, amount: Balance) { 
        let mut token = self.get_token(&token_idx);
        
        // Update Account Balance
        let mut balance_by_account = match token.get(&account) {
            Some(balance) => balance,
            None => 0,
        };
        balance_by_account = balance_by_account.wrapping_add(amount);
        token.insert(&account, &balance_by_account);
        self.tokens.insert(&token_idx, &token);

        // Update Token Balance
        let mut balance_by_token = match self.total_balances.get(&token_idx) {
            Some(balance) => balance,
            None => 0,
        };
        balance_by_token = balance_by_token.wrapping_add(amount);
        self.total_balances.insert(&token_idx, &balance_by_token);
    }

    pub fn transfer(&mut self, token_idx: u64, from: AccountId, to: AccountId, amount: Balance) { 
        let mut token = self.get_token(&token_idx);

        // Get Tokens From
        let from_balance = match token.get(&from) {
            Some(balance) => balance,
            None => 0,
        };

        // Get Tokens To
        let to_balance = match token.get(&to) {
            Some(balance) => balance,
            None => 0,
        };

        //@TODO Check Balances
        // Update Balances
        token.insert(&from, &from_balance.wrapping_sub(amount));
        token.insert(&to, &to_balance.wrapping_add(amount));

        self.tokens.insert(&token_idx, &token);
    }

    pub fn transfer_batch(&mut self, from: AccountId, to: AccountId, token_idx: Vec<u64>, amount: Vec<Balance>) {
        if token_idx.len() != amount.len(){
            env::panic_str("ERR_MARKET_IS_NOT_RUNNING");
        }

        let idxs = token_idx.len();
        for idx in 0 .. idxs {
            self.transfer(idx as u64, from.clone(), to.clone(), amount[idx]);
        }
    }

    fn get_token(&self, token_idx: &u64) -> LookupMap<AccountId, Balance> {
        let token = self
            .tokens
            .get(token_idx)
            .unwrap_or_else(|| {
                LookupMap::new(StorageKeys::SubConditionalTokensAccounts {
                    account_hash: env::sha256(&token_idx.to_be_bytes()),
                })
            });
        token
    }

    pub fn get_balance_by_token_idx(&self, token_idx: &u64) -> Balance {
        match self.total_balances.get(token_idx) {
            Some(balance) => balance,
            None => 0,
        }
    }
}
