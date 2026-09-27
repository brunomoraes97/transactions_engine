use std::collections::HashMap;

use crate::{accounts::Account, transactions::{SuccessfulTransaction, TransactionAttempt}};

pub struct State {
    accounts_registry: HashMap<u16, Account>,
    transaction_registry: HashMap<u32, SuccessfulTransaction>
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts_registry: HashMap::new(),
            transaction_registry: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn get_account(&mut self, account_id: &u16) -> Option<&mut Account> {
        self.accounts_registry.get_mut(account_id)
    }

    pub fn get_or_create_account(&mut self, account_id: &u16) -> &mut Account {

       self.accounts_registry
            .entry(*account_id)
            .or_insert_with(|| Account::new(*account_id))

    }

    pub fn save_successful_transaction(&mut self, transaction_attempt: TransactionAttempt) {

        self.transaction_registry
            .entry(*transaction_attempt.tx())
            .or_insert_with(|| SuccessfulTransaction::from(transaction_attempt));

    }

    pub fn get_successful_transaction(&self, tx: &u32) -> Option<&SuccessfulTransaction> {
        self.transaction_registry.get(tx)

    }

}
