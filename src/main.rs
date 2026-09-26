// SIMPLE TOY PAYMENTS ENGINE
mod reader;
mod accounts;
mod transactions;
mod engine;
mod state;

fn main() {

    // Initiate state
    let mut state = state::State::new();

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
        engine::process(&mut state, &transaction);
    }

    // Output the list of accounts
    state.get_all_accounts_in_csv();

}
