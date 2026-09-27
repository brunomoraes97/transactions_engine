use rust_decimal::Decimal;
use serde;
use getset::Getters;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Chargeback,
    Dispute,
    Resolve,
}

#[derive(Debug, serde::Deserialize)]
pub struct TransactionAttempt {
    #[serde(rename="type")]
    transaction_type: TransactionType,
    #[serde(rename="client")]
    client_id: u16,
    tx: u32,
    amount: Decimal,
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct SuccessfulTransaction {
    transaction_type: TransactionType,
    client_id: u16,
    tx: u32,
    amount: Decimal,
}

impl TransactionAttempt {

    #[cfg(test)]
    pub(crate) fn new_with_fields(
        transaction_type: TransactionType,
        client_id: u16,
        tx: u32,
        amount: Decimal,
    ) -> Self {
        Self {
            transaction_type,
            client_id,
            tx,
            amount,
        }
    }

    pub fn transaction_type(&self) -> &TransactionType {
        &self.transaction_type
    }

    pub fn client_id(&self) -> &u16 {
        &self.client_id
    }

    pub fn tx(&self) -> &u32 {
        &self.tx
    }

    pub fn amount(&self) -> &Decimal {
        &self.amount
    }
}

impl From<TransactionAttempt> for SuccessfulTransaction {

    fn from(attempt: TransactionAttempt) -> Self {
        
        let TransactionAttempt {
            transaction_type,
            client_id,
            tx,
            amount,
        } = attempt;

        Self {
            transaction_type,
            client_id,
            tx,
            amount,
        }
    }
}
