use rust_decimal::Decimal;
use getset::Getters;

#[derive(Debug, serde::Serialize, Getters)]
#[getset(get="pub")]
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

    
    pub fn new_with_fields(
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

    /*

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
        self.locked = true;

        Ok(())
    }

    */

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_be_able_to_deposit_normally() {
        
        let mut account = Account::new(1);
        let amount = Decimal::new(5,0);
        
        assert!(account.deposit(&amount).is_ok());
        assert_eq!(account.available, amount);
        assert_eq!(account.total, (account.available + account.held));

    }

    #[test]
    fn depositing_invalid_amount_fails() {

        let mut account = Account::new(1);
        let invalid_amount = Decimal::new(0,0);

        assert!(
            matches!(
                account.deposit(&invalid_amount),
                Err(AccountError::InvalidAmount),
            )
        );
    }

    #[test]
    fn deposit_with_locked_account_fails() {
        let mut account = Account::new(1);
        let amount = Decimal::new(10,0);
        account.locked = true;
        
        assert!(
            matches!(
                account.deposit(&amount),
                Err(AccountError::AccountLocked),
            )
        );
    }

    #[test]
    fn withdrawal_happens_normally_happy_path() {
        
        let mut account = Account::new_with_fields(
            1,
            Decimal::new(10,0),
            Decimal::ZERO,
            false,
            Decimal::new(10,0)
        );


        let amount = Decimal::new(5,0);
        
        assert!(account.withdraw(&amount).is_ok());
        assert_eq!(account.available, Decimal::new(5,0));
        assert_eq!(account.total, (account.available + account.held));
    }

    #[test]
    fn withdrawal_does_not_happen_with_invalid_amount() {

        let mut account = Account::new_with_fields(
    1,
    Decimal::new(0,0),
    Decimal::ZERO,
    false,
    Decimal::new(0,0)
        );


        let amount = Decimal::new(-5,0);
        
        assert!(account.withdraw(&amount).is_err());
        assert_eq!(account.available, Decimal::new(0,0));
        assert_eq!(account.total, (account.available + account.held));

    }

    #[test]
    fn withdrawl_does_not_happen_with_insufficient_funds() {

        let mut account = Account::new_with_fields(
    1,
    Decimal::new(0,0),
    Decimal::ZERO,
    false,
    Decimal::new(0,0)
        );


        let amount = Decimal::new(5,0);
        
        assert!(account.withdraw(&amount).is_err());
        assert_eq!(account.available, Decimal::new(0,0));
        assert_eq!(account.total, (account.available + account.held));
    }   
    
    #[test]
    fn withdrawal_does_not_happen_with_account_locked() {
    
        let mut account = Account::new_with_fields(
    1,
    Decimal::new(10,0),
    Decimal::ZERO,
    true,
    Decimal::new(10,0)
        );


        let amount = Decimal::new(5,0);
        
        assert!(account.withdraw(&amount).is_err());
        assert_eq!(account.available, Decimal::new(10,0));
        assert_eq!(account.total, (account.available + account.held));  
    }


}

