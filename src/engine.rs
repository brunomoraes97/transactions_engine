use crate::{
    output,
    state::State,
    transactions::{TransactionAttempt, TransactionType},
};
use std::io::Write;

pub struct Engine {
    state: State,
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
            TransactionType::Resolve => self.resolve(transaction_attempt),
            TransactionType::Chargeback => self.chargeback(transaction_attempt),
        }
    }

    pub fn write_accounts_to_csv<W: Write>(&self, writer: W) -> Result<(), csv::Error> {
        output::write_accounts_to_csv(&self.state, writer)
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
                self.state.save_successful_transaction(transaction_attempt);
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
                self.state.save_successful_transaction(transaction_attempt);
            }
            Err(error) => {
                eprintln!("An error occurred: {:?}", error);
            }
        }
    }

    pub fn dispute(&mut self, transaction_attempt: TransactionAttempt) {
        let tx = *transaction_attempt.tx();

        let (client_id, amount_disputed) = match self.state.get_successful_transaction(&tx) {
            Some(transaction) if !*transaction.in_dispute() => {
                (*transaction.client_id(), *transaction.amount())
            }
            Some(_) | None => return,
        };

        let result = {
            let account = self.state.get_or_create_account(&client_id);
            account.dispute(&amount_disputed)
        };

        if let Err(error) = result {
            eprintln!("An error occurred: {:?}", error);
        } else {
            self.state.mark_transaction_as_disputed(&tx);
        }
    }

    pub fn resolve(&mut self, transaction_attempt: TransactionAttempt) {
        let tx = *transaction_attempt.tx();

        let (client_id, amount_disputed) = match self.state.get_successful_transaction(&tx) {
            Some(transaction) if *transaction.in_dispute() => {
                (*transaction.client_id(), *transaction.amount())
            }
            Some(_) | None => return,
        };

        let result = {
            let account = self.state.get_or_create_account(&client_id);
            account.resolve(&amount_disputed)
        };

        if let Err(error) = result {
            eprintln!("An error occurred: {:?}", error);
        } else {
            self.state.clear_transaction_dispute(&tx);
        }
    }

    pub fn chargeback(&mut self, transaction_attempt: TransactionAttempt) {
        let tx = *transaction_attempt.tx();

        let (client_id, amount_disputed) = match self.state.get_successful_transaction(&tx) {
            Some(transaction) if *transaction.in_dispute() => {
                (*transaction.client_id(), *transaction.amount())
            }
            Some(_) | None => return,
        };

        let result = {
            let account = self.state.get_or_create_account(&client_id);
            account.chargeback(&amount_disputed)
        };

        if let Err(error) = result {
            eprintln!("An error occurred: {:?}", error);
        } else {
            self.state.clear_transaction_dispute(&tx);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::accounts::Account;
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn state_is_correctly_changed_after_successful_deposit() {
        let mut engine = Engine::new();
        let client_id = 1;
        let attempt = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            1,
            Decimal::new(5, 0),
        );

        engine.deposit(attempt);

        let account_deposited = engine
            .state
            .get_account(&client_id)
            .expect("account should be created");

        assert_eq!(account_deposited.available(), &Decimal::new(5, 0));
    }

    #[test]
    fn state_is_not_changed_after_unsuccessful_deposit() {
        let mut engine = Engine::new();
        let client_id = 1;
        let attempt = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            1,
            Decimal::new(-1, 2),
        );

        engine.deposit(attempt);
        let account_deposited = engine
            .state
            .get_account(&client_id)
            .expect("account should be created with default values");

        assert_eq!(account_deposited.available(), &Decimal::new(0, 0));
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

        let account = engine.state.get_or_create_account(&client_id);

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

        *engine.state.get_or_create_account(&client_id) = mock_account;

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
        *engine.state.get_or_create_account(&client_id) = account;

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

        let transaction = engine
            .state
            .get_successful_transaction(&transaction_id)
            .expect("transaction should exist after a successful dispute");

        assert_eq!(transaction.in_dispute(), &true);
    }

    #[test]
    fn second_dispute_of_the_same_transaction_is_ignored() {
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
        *engine.state.get_or_create_account(&client_id) = account;

        let successful_transaction = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            transaction_id,
            Decimal::new(5, 0),
        );
        engine
            .state
            .save_successful_transaction(successful_transaction);

        let first_dispute = TransactionAttempt::new_with_fields(
            TransactionType::Dispute,
            client_id,
            transaction_id,
            Decimal::ZERO,
        );
        engine.process(first_dispute);

        let second_dispute = TransactionAttempt::new_with_fields(
            TransactionType::Dispute,
            client_id,
            transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(second_dispute);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should exist after a repeated dispute");

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
        *engine.state.get_or_create_account(&client_id) = account;

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

    #[test]
    fn dispute_is_ignored_when_transaction_belongs_to_another_client() {
        // Arrange
        let mut engine = Engine::new();
        let transaction_owner_id = 7;
        let requesting_client_id = 8;
        let transaction_id = 20;

        let owner_account = Account::new_with_fields(
            transaction_owner_id,
            Decimal::new(4, 0),
            Decimal::ZERO,
            false,
            Decimal::new(4, 0),
        );
        *engine
            .state
            .get_or_create_account(&transaction_owner_id) = owner_account;

        let successful_transaction = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            transaction_owner_id,
            transaction_id,
            Decimal::new(4, 0),
        );
        engine
            .state
            .save_successful_transaction(successful_transaction);

        let dispute_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Dispute,
            requesting_client_id,
            transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(dispute_attempt);

        // Assert
        let owner_account = engine
            .state
            .get_account(&transaction_owner_id)
            .expect("transaction owner's account should still exist");

        assert_eq!(owner_account.available(), &Decimal::new(4, 0));
        assert_eq!(owner_account.held(), &Decimal::ZERO);
        assert_eq!(owner_account.total(), &Decimal::new(4, 0));
        assert_eq!(owner_account.locked(), &false);
        assert!(engine.state.get_account(&requesting_client_id).is_none());

        let transaction = engine
            .state
            .get_successful_transaction(&transaction_id)
            .expect("successful transaction should still exist");

        assert_eq!(transaction.in_dispute(), &false);
    }

    #[test]
    fn state_is_correctly_changed_after_successful_resolve() {
        // Arrange
        let mut engine = Engine::new();
        let client_id = 1;
        let transaction_id = 1;

        let account = Account::new_with_fields(
            client_id,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        *engine.state.get_or_create_account(&client_id) = account;

        let successful_transaction = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            transaction_id,
            Decimal::new(5, 0),
        );
        engine
            .state
            .save_successful_transaction(successful_transaction);
        engine.state.mark_transaction_as_disputed(&transaction_id);

        let resolve_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Resolve,
            client_id,
            transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(resolve_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should exist after a successful resolve");

        assert_eq!(account.available(), &Decimal::new(10, 0));
        assert_eq!(account.held(), &Decimal::ZERO);
        assert_eq!(account.total(), &Decimal::new(10, 0));
        assert_eq!(account.locked(), &false);

        let transaction = engine
            .state
            .get_successful_transaction(&transaction_id)
            .expect("transaction should exist after a successful resolve");

        assert_eq!(transaction.in_dispute(), &false);
    }

    #[test]
    fn state_is_not_changed_after_unsuccessful_resolve() {
        // Arrange
        let mut engine = Engine::new();
        let client_id = 1;
        let missing_transaction_id = 999;

        let account = Account::new_with_fields(
            client_id,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        *engine.state.get_or_create_account(&client_id) = account;

        let resolve_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Resolve,
            client_id,
            missing_transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(resolve_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should still exist after a rejected resolve");

        assert_eq!(account.available(), &Decimal::new(5, 0));
        assert_eq!(account.held(), &Decimal::new(5, 0));
        assert_eq!(account.total(), &Decimal::new(10, 0));
        assert_eq!(account.locked(), &false);
    }

    #[test]
    fn state_is_not_changed_when_resolve_transaction_is_not_in_dispute() {
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
        *engine.state.get_or_create_account(&client_id) = account;

        let successful_transaction = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            transaction_id,
            Decimal::new(5, 0),
        );
        engine
            .state
            .save_successful_transaction(successful_transaction);

        let resolve_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Resolve,
            client_id,
            transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(resolve_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should still exist after a rejected resolve");

        assert_eq!(account.available(), &Decimal::new(10, 0));
        assert_eq!(account.held(), &Decimal::ZERO);
        assert_eq!(account.total(), &Decimal::new(10, 0));
        assert_eq!(account.locked(), &false);
    }

    #[test]
    fn state_is_correctly_changed_after_successful_chargeback() {
        // Arrange
        let mut engine = Engine::new();
        let client_id = 1;
        let transaction_id = 1;

        let account = Account::new_with_fields(
            client_id,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        *engine.state.get_or_create_account(&client_id) = account;

        let successful_transaction = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            transaction_id,
            Decimal::new(5, 0),
        );
        engine
            .state
            .save_successful_transaction(successful_transaction);
        engine.state.mark_transaction_as_disputed(&transaction_id);

        let chargeback_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Chargeback,
            client_id,
            transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(chargeback_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should exist after a successful chargeback");

        assert_eq!(account.available(), &Decimal::new(5, 0));
        assert_eq!(account.held(), &Decimal::ZERO);
        assert_eq!(account.total(), &Decimal::new(5, 0));
        assert_eq!(account.locked(), &true);
    }

    #[test]
    fn state_is_not_changed_after_unsuccessful_chargeback() {
        // Arrange
        let mut engine = Engine::new();
        let client_id = 1;
        let missing_transaction_id = 999;

        let account = Account::new_with_fields(
            client_id,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        *engine.state.get_or_create_account(&client_id) = account;

        let chargeback_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Chargeback,
            client_id,
            missing_transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(chargeback_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should still exist after a rejected chargeback");

        assert_eq!(account.available(), &Decimal::new(5, 0));
        assert_eq!(account.held(), &Decimal::new(5, 0));
        assert_eq!(account.total(), &Decimal::new(10, 0));
        assert_eq!(account.locked(), &false);
    }

    #[test]
    fn state_is_not_changed_when_chargeback_transaction_is_not_in_dispute() {
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
        *engine.state.get_or_create_account(&client_id) = account;

        let successful_transaction = TransactionAttempt::new_with_fields(
            TransactionType::Deposit,
            client_id,
            transaction_id,
            Decimal::new(5, 0),
        );
        engine
            .state
            .save_successful_transaction(successful_transaction);

        let chargeback_attempt = TransactionAttempt::new_with_fields(
            TransactionType::Chargeback,
            client_id,
            transaction_id,
            Decimal::ZERO,
        );

        // Act
        engine.process(chargeback_attempt);

        // Assert
        let account = engine
            .state
            .get_account(&client_id)
            .expect("account should still exist after a rejected chargeback");

        assert_eq!(account.available(), &Decimal::new(10, 0));
        assert_eq!(account.held(), &Decimal::ZERO);
        assert_eq!(account.total(), &Decimal::new(10, 0));
        assert_eq!(account.locked(), &false);
    }
}
