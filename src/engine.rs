use crate::{accounts::Account, state::State, transactions::{TransactionAttempt, TransactionType}};

pub struct Engine {
    state: State
}

impl Engine {

    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub fn process(self: &mut Self, transaction_attempt: TransactionAttempt) {

        match transaction_attempt.transaction_type() {
            TransactionType::Deposit => self.deposit(transaction_attempt),
            TransactionType::Withdrawal => self.withdraw(&transaction),
            TransactionType::Dispute => self.dispute(&transaction),
            TransactionType::Resolve => self.resolve(&transaction),
            TransactionType::Chargeback => self.chargeback(&transaction),
        }

}


    pub fn deposit(&mut self, transaction_attempt: TransactionAttempt) {
        
        
        let result = {

            let account = self
                .state
                .get_or_create_account(transaction_attempt.client_id());
            
            account.deposit(transaction_attempt.amount())

        };

        match result {
            Ok(()) => {
                self.state
                    .save_successful_transaction(transaction_attempt);
            }
            Err(error) => {
                eprintln!("An error occurred: {:?}", error);
            }
        }
    }

    pub fn withdraw(&mut self, transaction: &Transaction) {
        let account = self.get_or_create_account(transaction.client_id());
        if let Err(error) = account.withdraw(transaction.amount()) {
            eprintln!(
                "An error occurred: {:?}",
                error,
            )
        };
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

        if let Err(error) = account.dispute(amount_disputed) {
            eprintln!(
                "An error occurred: {:?}",
                error,
            )
        };
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

        if let Err(error) = account.resolve(amount_resolved) {
            eprintln!(
                "An error occurred: {:?}",
                error,
            )
        };


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

        if let Err(error) = account.chargeback(chargedback_amount) {
            eprintln!(
                "An error occurred: {:?}",
                error,
            )
        };


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