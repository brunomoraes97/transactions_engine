# Transactions Engine

A small payments engine written in Rust. It reads a chronological stream of transactions from a CSV file, applies them to client accounts, and writes the final account state as CSV to standard output.

## Running the application

Requirements:

- a stable Rust toolchain;
- Cargo.

Run the engine with the input file as the first argument:

```bash
cargo run -- transactions.csv > accounts.csv
```

The account CSV is written to `stdout`. Technical errors are written to `stderr` and cause the process to exit with a non-zero status, so diagnostic messages do not contaminate the output file.

## Input

The input must be a CSV file with these columns:

| Column | Type | Description |
|---|---|---|
| `type` | string | `deposit`, `withdrawal`, `dispute`, `resolve`, or `chargeback` |
| `client` | `u16` | Client identifier |
| `tx` | `u32` | Globally unique transaction identifier |
| `amount` | decimal | Required for deposits and withdrawals; empty for dispute operations |

Example:

```csv
type,client,tx,amount
deposit,1,1,10.0
withdrawal,1,2,2.5
deposit,2,3,5.0
dispute,1,1,
resolve,1,1,
```

Whitespace around fields is accepted. Monetary values are represented with `rust_decimal::Decimal` rather than floating-point numbers.

## Output

The output contains one row for every client account created during processing:

| Column | Description |
|---|---|
| `client` | Client identifier |
| `available` | Funds available for use |
| `held` | Funds held by active disputes |
| `total` | Sum of available and held funds |
| `locked` | Whether the account has been frozen by a chargeback |

Example:

```csv
client,available,held,total,locked
1,7.5,0,7.5,false
2,5.0,0,5.0,false
```

Account row ordering is not guaranteed. The following invariant is maintained:

```text
total = available + held
```

## Transaction behavior

### Deposit

Adds the transaction amount to the client's available and total funds.

### Withdrawal

Subtracts the transaction amount from the client's available and total funds. A withdrawal is ignored when the client has insufficient available funds.

### Dispute

References a previously successful transaction by `tx`. The transaction amount moves from available to held funds while total funds remain unchanged.

A dispute is ignored when:

- the referenced transaction does not exist;
- the transaction belongs to another client;
- the transaction is already in dispute;
- the account is locked.

### Resolve

Releases the held amount back to available funds. A resolve is ignored when the referenced transaction does not exist, belongs to another client, or is not currently in dispute.

### Chargeback

Removes the held amount from the client's total funds and immediately locks the account. A chargeback is ignored when the referenced transaction does not exist, belongs to another client, or is not currently in dispute.

## Design decisions and assumptions

- Input records are processed in file order.
- The input is streamed: transactions are deserialized and processed one at a time instead of being collected into a `Vec`.
- Successful transactions remain indexed by `tx` because a later dispute may reference any previously processed transaction.
- Transaction IDs are assumed to be globally unique, as guaranteed by the assessment specification.
- Invalid business events are ignored without writing one error per event. Structural CSV and I/O errors fail the complete process.
- A locked account rejects new deposits, withdrawals, and disputes.
- Disputes opened before an account was locked may still be completed with a resolve or chargeback. This is an explicit interpretation of the specification's account-freezing rule.
- Successful deposits and withdrawals are stored and may be referenced by a dispute. The general dispute balance formula from the specification is applied to the stored amount.

More detail is available in the [thought process and design evolution](docs/thought-process.md).

## Architecture

```text
CSV file
   |
   v
reader -> TransactionAttempt -> Engine -> State -> Account
                                      |
                                      v
                              output -> stdout
```

| Module | Responsibility |
|---|---|
| `main` | Application composition, error reporting, and process exit status |
| `reader` | Streaming CSV parsing and transaction delivery |
| `transactions` | Input events and stored successful transactions |
| `engine` | Transaction coordination and business validation |
| `state` | Account and successful transaction registries |
| `accounts` | Balance changes and account-level invariants |
| `output` | Final account CSV serialization |

See the [thought process and design evolution](docs/thought-process.md) for the detailed design.

## Tests and quality checks

Run the complete test suite:

```bash
cargo test
```

The suite contains unit tests for account operations and engine state transitions, plus integration tests that execute the compiled binary with real CSV input. Failure cases include a missing argument, a nonexistent input file, and malformed CSV data.

Check formatting and linting:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

See the [thought process and design evolution](docs/thought-process.md) for the complete testing strategy.

## AI assistance disclosure

AI-assisted tools were used transparently during this assessment under the following boundaries:

- Neither vibe coding nor spec-driven development was used. The author independently designed the architecture, researched questions through Google, read the Rust documentation, and wrote approximately 80% of the code manually.
- The remaining approximately 20% was written with OpenAI Codex after discussion and explicit decisions by the author, primarily to reduce repetitive writing. For example, Codex wrote unit tests from instructions placed by the author in `TODO` comments describing the behavior and structure expected from those tests.
- Several Google searches returned Google AI-generated overviews. These helped answer brief questions encountered during development, including topics related to borrowing and lifetimes. Some searches also surfaced useful insights from solutions to similar problems shared by users on Stack Overflow and Reddit.
- Codex was used to generate extensive CSV files for exercising the program with larger and more varied inputs.
- The author copied the most relevant information from the original assessment into a Git-excluded file named `personal/notes.md`. Codex was then asked to compare the implementation against those functional and non-functional requirements and to simulate an assessment score for the project.
- At the end of development, Codex was used to prepare a plan for resolving remaining issues, including uncovered edge cases, formatting problems, and compiler or linter warnings. Two review rounds were performed and will be included with the other [AI-generated material](docs/ai-generated-material/).
- This `README.md` was drafted by Codex from the author's instructions and subsequently reviewed by the author.
- Every line of code and documentation contained in this repository was reviewed by the author.

The author selected the business interpretations, accepted or rejected proposed changes, requested revisions where necessary, and validated the resulting implementation using unit tests, integration tests, Clippy, rustfmt, and an extended local CSV data set.

The development narrative and selected AI-generated supporting material are available in:

- [Thought process and design evolution](docs/thought-process.md);
- [Selected AI transcripts](docs/ai-transcripts.md);
- [AI-generated material](docs/ai-generated-material/).

The transcript documentation is a curated selection of interactions that materially contributed to the project's architecture, implementation, tests, reviews, and documentation. It is not a complete interaction history.
