use crate::transactions;

use std::env;
use std::io;
use transactions::TransactionAttempt;


pub fn get_transactions_from_csv() -> Result<Vec<TransactionAttempt>, Box<dyn std::error::Error>> {

    let file_path = env::args()
    .nth(1)
    .ok_or_else( || { 
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not load csv file."
        )
        })?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&file_path)?;

    let mut transactions: Vec<TransactionAttempt> = Vec::new();

    for transaction_result in csv_reader.deserialize() {
        let transaction: TransactionAttempt = transaction_result?;
        transactions.push(transaction);
    }

    Ok(transactions)
}