use crate::{accounts::{Account}, transactions::Transaction};
use std::collections::HashMap;

pub struct Engine {
    accounts: HashMap<u16, Account>,
    transactions: HashMap<u32, Transaction>,
}

impl Engine {

    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, transaction: &Transaction) {
        let account = self.get_or_create_account(transaction.client_id());
        account.deposit(transaction.amount());
    }

    pub fn withdraw(&mut self, transaction: &Transaction) {
        let account = self.get_or_create_account(transaction.client_id());
        account.withdraw(transaction.amount());
    }

    pub fn dispute(&mut self, transaction: &Transaction) {

        let (client_id, amount_disputed) = match self.transactions.get(transaction.tx()) {
            Some(disputed_transaction) => (
                *disputed_transaction.client_id(),
                disputed_transaction.amount(),
            ),
            None => return,
        };

        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

        account.dispute(amount_disputed);
    }

    pub fn resolve(&mut self, transaction: &Transaction) {

        let (client_id, amount_resolved) = match self.transactions.get(transaction.tx()) {
            Some(resolved_transaction) => (
                *resolved_transaction.client_id(),
                resolved_transaction.amount(),
            ),
            None => return,
        };

        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

        account.resolve(amount_resolved);


    }

    pub fn chargeback(&mut self, transaction: &Transaction) {

        let (client_id, chargedback_amount) = match self.transactions.get(transaction.tx()) {
            Some(chargedback_transaction) => (
                *chargedback_transaction.client_id(),
                chargedback_transaction.amount(),
            ),
            None => return,
        };

        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

        account.chargeback(chargedback_amount);


    }

    pub fn get_or_create_account(&mut self, account_id: &u16) -> &mut Account {

       self.accounts
            .entry(*account_id)
            .or_insert_with(|| Account::new(*account_id))

    }

    pub fn save_transaction(&mut self, transaction: Transaction) -> &Transaction{
        
        self.transactions
            .entry(*transaction.tx())
            .or_insert(transaction)
    }

    pub fn get_all_accounts_in_csv(self) {
        
        let mut stdout_writer = csv::Writer::from_writer(std::io::stdout());

        for (_, account) in self.accounts {
            stdout_writer.serialize(account).unwrap();
        }

        stdout_writer.flush().unwrap();

    }

}