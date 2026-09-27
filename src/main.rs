use crate::{engine::Engine, transactions::{Transaction, TransactionType}};

mod reader;
mod accounts;
mod transactions;
mod engine;

fn process(engine: &mut Engine, transaction: Transaction) {

    match transaction.transaction_type() {
        TransactionType::Deposit => engine.deposit(&transaction),
        TransactionType::Withdrawal => engine.withdraw(&transaction),
        TransactionType::Dispute => engine.dispute(&transaction),
        TransactionType::Resolve => engine.resolve(&transaction),
        TransactionType::Chargeback => engine.chargeback(&transaction),
    }

    // Save transaction state
    // TODO: do a Result vs. Error here
    engine.save_transaction(transaction);

}

fn main() {

    // Initiate engine
    let mut engine = engine::Engine::new();

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
        process(&mut engine, transaction);
    }

    // Output the list of accounts
    engine.get_all_accounts_in_csv();

}
