use getset::Getters;
use rust_decimal::Decimal;

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
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    #[serde(rename = "client")]
    client_id: u16,
    tx: u32,
    amount: Option<Decimal>,
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct SuccessfulTransaction {
    client_id: u16,
    amount: Decimal,
    in_dispute: bool,
}

impl SuccessfulTransaction {
    pub fn set_in_dispute(&mut self, setting: bool) {
        self.in_dispute = setting;
    }
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
            amount: Some(amount),
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
        self.amount.as_ref().unwrap_or(&Decimal::ZERO)
    }
}

impl From<TransactionAttempt> for SuccessfulTransaction {
    fn from(attempt: TransactionAttempt) -> Self {
        let TransactionAttempt {
            client_id, amount, ..
        } = attempt;

        Self {
            client_id,
            amount: amount.expect("a successful transaction must have an amount"),
            in_dispute: false,
        }
    }
}
