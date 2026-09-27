use getset::Getters;
use rust_decimal::Decimal;

#[derive(Debug, serde::Serialize, Getters)]
#[getset(get = "pub")]
pub struct Account {
    client_id: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
    total: Decimal,
}
#[derive(Debug)]
pub enum AccountError {
    InsufficientFunds,
    InvalidAmount,
    AccountLocked,
}

impl Account {
    // constructor
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
            total: Decimal::ZERO,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_fields(
        client_id: u16,
        available: Decimal,
        held: Decimal,
        locked: bool,
        total: Decimal,
    ) -> Self {
        Self {
            client_id,
            available,
            held,
            locked,
            total,
        }
    }

    pub fn deposit(&mut self, amount: &Decimal) -> Result<(), AccountError> {
        if *amount <= Decimal::ZERO {
            return Err(AccountError::InvalidAmount);
        }

        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        self.available += amount;
        self.total += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: &Decimal) -> Result<(), AccountError> {
        if *amount <= Decimal::ZERO {
            return Err(AccountError::InvalidAmount);
        }

        if *amount > self.available {
            return Err(AccountError::InsufficientFunds);
        }

        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        self.available -= amount;
        self.total -= amount;

        Ok(())
    }

    pub fn dispute(&mut self, amount_disputed: &Decimal) -> Result<(), AccountError> {
        if *amount_disputed <= Decimal::ZERO {
            return Err(AccountError::InvalidAmount);
        }

        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        self.available -= amount_disputed;
        self.held += amount_disputed;

        Ok(())
    }

    pub fn resolve(&mut self, amount_resolved: &Decimal) -> Result<(), AccountError> {
        if *amount_resolved <= Decimal::ZERO {
            return Err(AccountError::InvalidAmount);
        }

        self.held -= amount_resolved;
        self.available += amount_resolved;

        Ok(())
    }

    pub fn chargeback(&mut self, amount_chargedback: &Decimal) -> Result<(), AccountError> {
        if *amount_chargedback <= Decimal::ZERO {
            return Err(AccountError::InvalidAmount);
        }

        self.held -= amount_chargedback;
        self.total -= amount_chargedback;
        self.locked = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_be_able_to_deposit_normally() {
        let mut account = Account::new(1);
        let amount = Decimal::new(5, 0);

        assert!(account.deposit(&amount).is_ok());
        assert_eq!(account.available, amount);
        assert_eq!(account.total, (account.available + account.held));
    }

    #[test]
    fn depositing_invalid_amount_fails() {
        let mut account = Account::new(1);
        let invalid_amount = Decimal::new(0, 0);

        assert!(matches!(
            account.deposit(&invalid_amount),
            Err(AccountError::InvalidAmount),
        ));
    }

    #[test]
    fn deposit_with_locked_account_fails() {
        let mut account = Account::new(1);
        let amount = Decimal::new(10, 0);
        account.locked = true;

        assert!(matches!(
            account.deposit(&amount),
            Err(AccountError::AccountLocked),
        ));
    }

    #[test]
    fn withdrawal_happens_normally_happy_path() {
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(10, 0),
            Decimal::ZERO,
            false,
            Decimal::new(10, 0),
        );

        let amount = Decimal::new(5, 0);

        assert!(account.withdraw(&amount).is_ok());
        assert_eq!(account.available, Decimal::new(5, 0));
        assert_eq!(account.total, (account.available + account.held));
    }

    #[test]
    fn withdrawal_does_not_happen_with_invalid_amount() {
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(0, 0),
            Decimal::ZERO,
            false,
            Decimal::new(0, 0),
        );

        let amount = Decimal::new(-5, 0);

        assert!(account.withdraw(&amount).is_err());
        assert_eq!(account.available, Decimal::new(0, 0));
        assert_eq!(account.total, (account.available + account.held));
    }

    #[test]
    fn withdrawl_does_not_happen_with_insufficient_funds() {
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(0, 0),
            Decimal::ZERO,
            false,
            Decimal::new(0, 0),
        );

        let amount = Decimal::new(5, 0);

        assert!(account.withdraw(&amount).is_err());
        assert_eq!(account.available, Decimal::new(0, 0));
        assert_eq!(account.total, (account.available + account.held));
    }

    #[test]
    fn withdrawal_does_not_happen_with_account_locked() {
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(10, 0),
            Decimal::ZERO,
            true,
            Decimal::new(10, 0),
        );

        let amount = Decimal::new(5, 0);

        assert!(account.withdraw(&amount).is_err());
        assert_eq!(account.available, Decimal::new(10, 0));
        assert_eq!(account.total, (account.available + account.held));
    }

    #[test]
    fn dispute_happens_normally() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(10, 0),
            Decimal::ZERO,
            false,
            Decimal::new(10, 0),
        );
        let disputed_amount = Decimal::new(5, 0);

        // Act
        let result = account.dispute(&disputed_amount);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::new(5, 0));
        assert_eq!(account.held, Decimal::new(5, 0));
        assert_eq!(account.total, Decimal::new(10, 0));
        assert!(!account.locked);
    }

    #[test]
    fn dispute_does_not_happen_with_invalid_amount() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(10, 0),
            Decimal::ZERO,
            false,
            Decimal::new(10, 0),
        );
        let invalid_amount = Decimal::ZERO;

        // Act
        let result = account.dispute(&invalid_amount);

        // Assert
        assert!(matches!(result, Err(AccountError::InvalidAmount)));
        assert_eq!(account.available, Decimal::new(10, 0));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(10, 0));
        assert!(!account.locked);
    }

    #[test]
    fn dispute_does_not_happen_with_locked_account() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(10, 0),
            Decimal::ZERO,
            true,
            Decimal::new(10, 0),
        );
        let disputed_amount = Decimal::new(5, 0);

        // Act
        let result = account.dispute(&disputed_amount);

        // Assert
        assert!(matches!(result, Err(AccountError::AccountLocked)));
        assert_eq!(account.available, Decimal::new(10, 0));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(10, 0));
        assert!(account.locked);
    }

    #[test]
    fn resolve_happens_normally() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        let resolved_amount = Decimal::new(5, 0);

        // Act
        let result = account.resolve(&resolved_amount);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::new(10, 0));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(10, 0));
        assert!(!account.locked);
    }

    #[test]
    fn resolve_does_not_happen_with_invalid_amount() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        let invalid_amount = Decimal::ZERO;

        // Act
        let result = account.resolve(&invalid_amount);

        // Assert
        assert!(matches!(result, Err(AccountError::InvalidAmount)));
        assert_eq!(account.available, Decimal::new(5, 0));
        assert_eq!(account.held, Decimal::new(5, 0));
        assert_eq!(account.total, Decimal::new(10, 0));
        assert!(!account.locked);
    }

    #[test]
    fn clients_held_and_total_funds_decrease_by_disputed_amount_after_successful_chargeback() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        let chargeback_amount = Decimal::new(5, 0);

        // Act
        let result = account.chargeback(&chargeback_amount);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::new(5, 0));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(5, 0));
    }

    #[test]
    fn clients_account_is_locked_after_chargeback() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        let chargeback_amount = Decimal::new(5, 0);

        // Act
        let result = account.chargeback(&chargeback_amount);

        // Assert
        assert!(result.is_ok());
        assert!(account.locked);
    }

    #[test]
    fn chargeback_does_not_happen_with_invalid_amount() {
        // Arrange
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(5, 0),
            Decimal::new(5, 0),
            false,
            Decimal::new(10, 0),
        );
        let invalid_amount = Decimal::ZERO;

        // Act
        let result = account.chargeback(&invalid_amount);

        // Assert
        assert!(matches!(result, Err(AccountError::InvalidAmount)));
        assert_eq!(account.available, Decimal::new(5, 0));
        assert_eq!(account.held, Decimal::new(5, 0));
        assert_eq!(account.total, Decimal::new(10, 0));
        assert!(!account.locked);
    }
}
