use rust_decimal::Decimal;
use serde;

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
pub struct Transaction {
    #[serde(rename="type")]
    transaction_type: TransactionType,
    #[serde(rename="client")]
    client_id: u16,
    tx: u32,
    amount: Decimal,
}

impl Transaction {

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