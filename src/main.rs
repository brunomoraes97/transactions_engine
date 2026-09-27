mod reader;
mod accounts;
mod transactions;
mod engine;
mod state;

use engine::Engine;

fn main() {

    // Initiate engine and pass the reference to the global state
    let mut engine= Engine::new();

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
    engine.get_all_accounts_in_csv();

}
