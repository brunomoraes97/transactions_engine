mod accounts;
mod engine;
mod output;
mod reader;
mod state;
mod transactions;

use engine::Engine;

fn main() {
    
    // Initiate engine
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
    if let Err(error) = engine.write_accounts_to_csv(std::io::stdout()) {
        eprintln!("Could not write accounts: {}", error);
    }
}
