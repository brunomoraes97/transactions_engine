use crate::{accounts::{Account}, transactions::Transaction};
use std::collections::HashMap;

#[derive(Clone)]
pub struct State {
    accounts: HashMap<u16, Account>,
    transactions: HashMap<u32, Transaction>,
}

impl State {

    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }

    }

    pub fn get_or_create_account(&mut self, &account_id:&u16) -> Account {

       let account = self.accounts
        .entry(account_id)
        .or_insert_with(|| Account::new(account_id));

        return *account;
    }

    pub fn get_transaction(&mut self, transaction_id: u32) -> Transaction {
        self.transactions[&transaction_id]
    }

    pub fn save_transaction(&mut self, transaction: &Transaction) -> &mut Transaction{
        
        let saved_transaction = self.transactions
            .entry(*transaction.tx())
            .or_insert_with(|| Transaction::new(
                *transaction.transaction_type(),
                *transaction.client_id(),
                *transaction.tx(),
                *transaction.amount(),
            ));

        return saved_transaction;
    }

    pub fn get_all_accounts_in_csv(self) {
        
        let mut stdout_writer = csv::Writer::from_writer(std::io::stdout());

        for (_, account) in self.accounts {
            stdout_writer.serialize(account).unwrap();
        }

        stdout_writer.flush().unwrap();

    }

}