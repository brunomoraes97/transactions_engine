# Final Review Against `notes.md`

This document describes the five points identified during the final review, the current state of each one, and a proposed solution.

## 1. Client Validation in `resolve` and `chargeback`

### Current state

Resolved in `dispute`, `resolve`, and `chargeback`.

In `Engine::dispute`, the stored transaction is used only when:

```rust
transaction.client_id() == transaction_attempt.client_id()
    && !*transaction.in_dispute()
```

There is also a unit test named `dispute_is_ignored_when_transaction_belongs_to_another_client`.

`Engine::resolve` and `Engine::chargeback` now perform the same ownership check, in addition to verifying that the transaction is in dispute. An attempt containing the correct `tx` but a different `client` is ignored without changing the account or the dispute state.

### Implemented solution

The same ownership check already used in `dispute` was applied to both methods:

```rust
Some(transaction)
    if transaction.client_id() == transaction_attempt.client_id()
        && *transaction.in_dispute() =>
{
    (*transaction.client_id(), *transaction.amount())
}
```

Two unit tests were also added:

- `resolve_is_ignored_when_transaction_belongs_to_another_client`;
- `chargeback_is_ignored_when_transaction_belongs_to_another_client`.

Each test confirms that the owner's balances and the dispute state remain unchanged and that no account is created for the incorrect client.

## 2. Pending Operations After an Account Is Locked

### Current state

`Account::deposit`, `Account::withdraw`, and `Account::dispute` return `AccountError::AccountLocked` when the account is locked. Tests exist for these behaviors.

However, `Account::resolve` and `Account::chargeback` do not check `locked`. Therefore, if two transactions are simultaneously in dispute, a chargeback on one of them locks the account, but the other can still be resolved or charged back afterward.

There is currently no unit test for this sequence.

### Adopted business decision

The specification says that an account must be frozen immediately after a chargeback, but it does not explain what should happen to disputes that were already open. There are two possible interpretations:

1. no operation may change a locked account, including pending `resolve` and `chargeback` operations;
2. new account activity is forbidden, but disputes that were already open may still be completed.

The second interpretation was adopted: the lock prevents new account activity, while `resolve` and `chargeback` may complete disputes that were already open. This decision should later be documented in `README.md`.

### Implementation consequence

A `locked` check should not be added to `Account::resolve` or `Account::chargeback`. Future tests for this rule should demonstrate that deposits, withdrawals, and new disputes are rejected, while disputes opened before the lock can still be completed.

## 3. Streaming CSV Processing

### Current state

Resolved. `reader::process_transactions_from_csv` iterates over `csv_reader.deserialize()` and sends each `TransactionAttempt` directly to the engine.

Input memory usage no longer grows linearly with the number of transactions. Memory still grows according to the required account and stored-transaction state, but the complete file contents are no longer retained in a `Vec`.

### How streaming works

`csv::Reader` already produces an iterator through `deserialize()`. There is no need to create a thread, use asynchronous code, or implement an iterator manually. The reader only needs to remain open while each record is passed to the engine as soon as it is deserialized:

```text
file -> csv::Reader -> one TransactionAttempt -> Engine::process
                      one TransactionAttempt -> Engine::process
                      ...
```

At any point, memory contains only the reader, the accumulated account and transaction state, and approximately one CSV row.

### Implemented solution

The loop was moved into a function that receives the engine:

```rust
pub fn process_transactions_from_csv(
    engine: &mut Engine,
) -> Result<(), Box<dyn std::error::Error>> {
    // opens the file and creates csv_reader

    for transaction_result in csv_reader.deserialize() {
        let transaction: TransactionAttempt = transaction_result?;
        engine.process(transaction);
    }

    Ok(())
}
```

`main` now calls this function and does not retain a `Vec`. A more decoupled architectural alternative would be for `reader` to return an iterator, but that would introduce more complex types and lifetimes without providing a relevant benefit for this small project.

To preserve testability, an even better version could accept any type that implements `std::io::Read`; opening the path would remain in a small function, while CSV reading could be tested with in-memory bytes.

## 4. Exit Status on Error

### Previous problem

The program printed read and write errors but executed `return` from a `main` function that returned `()`. To the operating system, this represented successful termination with exit code `0`.

This matters in scripts and pipelines: the consuming process must be able to distinguish a valid execution from a failure without parsing the text written to `stderr`.

### Implemented solution

`main` now returns `std::process::ExitCode`:

- `ExitCode::FAILURE` after a read or write error;
- `ExitCode::SUCCESS` when processing and writing complete successfully.

Messages continue to be written to `stderr`, so they do not contaminate the CSV written to `stdout`.

## 5. `cargo clippy` and `cargo fmt` Failures

### Clippy

The current warning occurs because this method:

```rust
pub fn process(self: &mut Self, transaction_attempt: TransactionAttempt)
```

uses a valid but unnecessarily explicit receiver type. For ordinary methods, the idiomatic form is:

```rust
pub fn process(&mut self, transaction_attempt: TransactionAttempt)
```

Both signatures have the same behavior and ownership semantics. The second simply uses the conventional syntax recognized by Clippy.

After making the change, run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

`-D warnings` turns every warning into an error, which is useful as a final verification step or in CI.

### Formatting

`cargo fmt --check` only verifies formatting and does not modify files. To apply formatting automatically:

```bash
cargo fmt
```

Then confirm it with:

```bash
cargo fmt --check
```

## Recommended Order

1. fix client ownership validation in `resolve` and `chargeback` and add the tests;
2. decide and document the policy for open disputes on locked accounts;
3. implement and test that policy;
4. replace `Vec`-based input reading with streaming processing;
5. apply the simple Clippy fix and run `cargo fmt`;
6. run all tests, Clippy, and the formatting check.
