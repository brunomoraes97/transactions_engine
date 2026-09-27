use crate::{state::State, transactions::{TransactionAttempt, TransactionType}};

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
            TransactionType::Withdrawal => self.withdraw(transaction_attempt),
            TransactionType::Dispute => self.dispute(transaction_attempt),
            TransactionType::Resolve => todo!(),
            TransactionType::Chargeback => todo!(),
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


    pub fn withdraw(&mut self, transaction_attempt: TransactionAttempt) {

        let result = {

            let account = self
                .state
                .get_or_create_account(transaction_attempt.client_id());

            account.withdraw(transaction_attempt.amount())

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

    pub fn dispute(&mut self, transaction_attempt: TransactionAttempt) {

        let search_transaction = self
            .state
            .get_successful_transaction(transaction_attempt.tx());

        let successful_transaction = match search_transaction {
                Some(transaction) => transaction,
                None => return,
            };

        let client_id = *successful_transaction.client_id();
        let amount_disputed = *successful_transaction.amount();

        let result = {
            let account = self
                .state
                .get_or_create_account(&client_id);
            account.dispute(&amount_disputed)
        };

        if let Err(error) = result {
            eprintln!("An error occurred: {:?}", error);
        }
    }
    /*

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


    */
}

#[cfg(test)]
mod tests {

    use rust_decimal::Decimal;
    use crate::accounts::Account;

use super::*;

    #[test]
    fn state_is_correctly_changed_after_successful_deposit() {
        let mut engine = Engine::new();
        let client_id = 1;
        let attempt = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            1,
            Decimal::new(5,0)
        );

        engine.deposit(attempt);

        let account_deposited = engine
            .state
            .get_account(&client_id)
            .expect("account should be created");

        assert_eq!(
            account_deposited.available(),
            &Decimal::new(5,0)
        );
    }

    #[test]
    fn state_is_not_changed_after_unsuccessful_deposit() {
        
        let mut engine = Engine::new();
        let client_id = 1;
        let attempt = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            1,
            Decimal::new(-1,2)
        );

        engine.deposit(attempt);
        let account_deposited = engine
            .state
            .get_account(&client_id)
            .expect("account should be created with default values");

        assert_eq!(
            account_deposited.available(),
            &Decimal::new(0,0)
        );
    }

    #[test]
    fn state_is_correctly_changed_after_successful_withdrawal() {
        
        let mut engine = Engine::new();
        let client_id = 1;

        let mock_account = Account::new_with_fields(
            client_id,
            Decimal::new(10, 0),
            Decimal::ZERO,
            false,
            Decimal::new(10, 0),
        );

        let account = engine
            .state
            .get_or_create_account(&client_id);

        *account = mock_account;

        let attempt = TransactionAttempt::new_with_fields(
            TransactionType::Withdrawal,
            client_id,
            1,
            Decimal::new(5, 0),
        );

        engine.withdraw(attempt);

        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should exist");

        assert_eq!(account.available(), &Decimal::new(5, 0));
        assert_eq!(account.total(), &Decimal::new(5, 0));

    }

    #[test]
    fn state_is_not_changed_after_unsuccessful_withdrawal() {
        
        let mut engine = Engine::new();
        let client_id = 1;
        let transaction_id = 1;

        let mock_account = Account::new_with_fields(
            client_id,
            Decimal::new(10, 0),
            Decimal::ZERO,
            false,
            Decimal::new(10, 0),
        );

        *engine
            .state
            .get_or_create_account(&client_id) = mock_account;

        let attempt = TransactionAttempt::new_with_fields(
            TransactionType::Withdrawal,
            client_id,
            transaction_id,
            Decimal::new(15, 0),
        );

        engine.withdraw(attempt);

        {
            let account = engine
                .state
                .get_account(&client_id)
                .expect("account should still exist");

            assert_eq!(account.available(), &Decimal::new(10, 0));
            assert_eq!(account.held(), &Decimal::ZERO);
            assert_eq!(account.total(), &Decimal::new(10, 0));
            assert_eq!(account.locked(), &false);
        }

        assert!(
            engine
                .state
                .get_successful_transaction(&transaction_id)
                .is_none()
        );
    }

    #[test]
    fn state_is_correctly_changed_after_successful_dispute() {
        // Arrange
        let mut engine = Engine::new();
        let client_id = 1;
        let transaction_id = 1;

        let account = Account::new_with_fields(
            client_id,
            Decimal::new(10, 0),
            Decimal::ZERO,
            false,
            Decimal::new(10, 0),
        );
        *engine
            .state
            .get_or_create_account(&client_id) = account;

        let successful_transaction = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            transaction_id,
            Decimal::new(5, 0),
        );
        engine
            .state
            .save_successful_transaction(successful_transaction);

        let dispute_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Dispute,
            client_id,
            transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(dispute_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should exist after a successful dispute");

        assert_eq!(account.available(), &Decimal::new(5, 0));
        assert_eq!(account.held(), &Decimal::new(5, 0));
        assert_eq!(account.total(), &Decimal::new(10, 0));
        assert_eq!(account.locked(), &false);
    }

    #[test]
    fn state_is_not_changed_after_unsuccessful_dispute() {
        // Arrange
        let mut engine = Engine::new();
        let client_id = 1;
        let missing_transaction_id = 999;

        let account = Account::new_with_fields(
            client_id,
            Decimal::new(10, 0),
            Decimal::ZERO,
            false,
            Decimal::new(10, 0),
        );
        *engine
            .state
            .get_or_create_account(&client_id) = account;

        let dispute_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Dispute,
            client_id,
            missing_transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(dispute_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should still exist after a rejected dispute");

        assert_eq!(account.available(), &Decimal::new(10, 0));
        assert_eq!(account.held(), &Decimal::ZERO);
        assert_eq!(account.total(), &Decimal::new(10, 0));
        assert_eq!(account.locked(), &false);
    }

}
