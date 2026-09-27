mod accounts;
mod engine;
mod reader;
mod state;
mod transactions;

use engine::Engine;

fn main() {
    // Initiate engine and pass the reference to the global state
    let mut engine = Engine::new();

    // Read transactions from the csv file in arguments
    let transactions = match reader::get_transactions_from_csv() {
        Ok(transactions) => transactions,
        Err(error) => {
            eprintln!("Could not read transactions: {}", error);
            return;
        }
    };

    // Process transactions
    for transaction in transactions {
        engine.process(transaction);
    }

    // Output the list of accounts
    todo!();
}
