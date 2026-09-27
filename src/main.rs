mod accounts;
mod engine;
mod output;
mod reader;
mod state;
mod transactions;

use engine::Engine;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Initiate engine
    let mut engine = Engine::new();

    // Read and process transactions from the csv file in arguments
    if let Err(error) = reader::process_transactions_from_csv(&mut engine) {
        eprintln!("Could not process transactions: {}", error);
        return ExitCode::FAILURE;
    }

    // Output the list of accounts
    if let Err(error) = engine.write_accounts_to_csv(std::io::stdout()) {
        eprintln!("Could not write accounts: {}", error);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
