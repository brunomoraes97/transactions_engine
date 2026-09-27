use crate::{engine::Engine, transactions::TransactionAttempt};

use std::env;
use std::io;

pub fn process_transactions_from_csv(
    engine: &mut Engine,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = env::args()
        .nth(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Could not load csv file."))?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&file_path)?;

    for transaction_result in csv_reader.deserialize() {
        let transaction: TransactionAttempt = transaction_result?;
        engine.process(transaction);
    }

    Ok(())
}
