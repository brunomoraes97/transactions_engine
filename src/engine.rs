use crate::{accounts::Account, state::State, transactions::{Transaction, TransactionType}};

pub fn process(state: &mut State, transaction: &Transaction) {

    // Get transaction information
    let transaction_type = *transaction.transaction_type();
    let transaction_id: u32 = *transaction.tx();

    // Get or create the account
    let mut account: Account = state.get_or_create_account(transaction.client_id());

    // Process transaction in the account
    match transaction_type {
        TransactionType::Deposit => account.deposit(*transaction.amount()),
        TransactionType::Withdrawal => account.withdraw(*transaction.amount()),
        TransactionType::Dispute => {

            // TODO: do a Result vs. Error here
            let transaction_disputed = state.get_transaction(transaction_id);
            // TODO: do a Result vs. Error here
            let amount_disputed = transaction_disputed.amount();

            // TODO: do a Result vs. Error here
            account.dispute(*amount_disputed);

        },
        TransactionType::Resolve => {

            // TODO: do a Result vs. Error here
            let transaction_resolved = state.get_transaction(transaction_id);
            // TODO: do a Result vs. Error here
            let amount_resolved = transaction_resolved.amount();
            // TODO: do a Result vs. Error here
            account.resolve(*amount_resolved);

        },
        TransactionType::Chargeback => {

            // TODO: do a Result vs. Error here
            let transaction_chargedback = state.clone().get_transaction(transaction_id);
            // TODO: do a Result vs. Error here
            let amount_chargedback = transaction_chargedback.amount();
            // TODO: do a Result vs. Error here
            account.chargeback(*amount_chargedback);


        },
    }

    // Save transaction state
    // TODO: do a Result vs. Error here
    state.save_transaction(transaction);

}