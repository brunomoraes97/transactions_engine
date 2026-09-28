# Second Final Review Against `notes.md`

## Conclusion

The current code is functional according to the rules described in `notes.md` and the adopted business interpretation for locked accounts. No new confirmed functional defect was found in this review.

The project would not yet receive the maximum score because there is no version-controlled `README.md` explaining the decisions and assumptions.

Current estimate: **9.8/10**. Once this remaining issue is resolved, the project should be close to the maximum score.

## Evidence Collected

### Build and tests

The following commands were run:

```bash
cargo build
cargo test
```

Results:

- build completed successfully;
- 32 unit tests passed;
- 6 integration tests passed;
- no tests failed.

### Formatting

The following command was run:

```bash
cargo fmt --check
```

Result: success. The code conforms to `rustfmt`.

### Clippy

The following command was run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

The previous result was a failure caused by a single warning in `src/engine.rs`:

```rust
pub fn process(self: &mut Self, transaction_attempt: TransactionAttempt)
```

The equivalent idiomatic form is:

```rust
pub fn process(&mut self, transaction_attempt: TransactionAttempt)
```

This change was applied. It does not change ownership, borrowing, or behavior; it only uses the conventional method-receiver syntax.

Current result: Clippy passes without warnings, including with `-D warnings`.

### Extensive execution

The program processed `personal/test-extensive.csv`:

- 500 input transactions;
- exit code `0`;
- no content in `stderr`;
- 110 accounts in the result;
- no violations of `total == available + held`.

The extensive file is stored under `personal/`, which is not version-controlled. It therefore supports local validation but will not be available to an evaluator who clones the repository.

### Input failure

The binary was also executed without the CSV argument:

- exit code `1`;
- no content in `stdout`;
- error message only in `stderr`.

This confirms that a failure does not produce a partial CSV in `stdout` and is correctly communicated to the operating system.

## Review by Criterion

### 1. Basics

**Status: satisfied.**

- the project builds with `cargo build`;
- it can be executed with `cargo run -- file.csv`;
- it reads the CSV specified by the first argument;
- it writes only the resulting CSV to `stdout` during a successful execution;
- it writes diagnostics to `stderr`;
- it returns a non-zero exit code on read or write failures;
- `cargo fmt --check` passes.

Small improvement: the program uses the first argument but does not reject additional arguments. Because the specification says that the input file is the first and only argument, this validation could be added, although it does not affect the expected normal flow.

### 2. Completeness

**Status: satisfied.**

The following are implemented:

- deposit;
- withdrawal;
- dispute;
- resolve;
- chargeback;
- account locking after a chargeback;
- rejection of new account activity on a locked account;
- lookup of previous transactions by `tx`;
- validation that dispute, resolve, and chargeback belong to the specified client;
- rejection of resolve and chargeback when the transaction is not in dispute.

Events without an amount use `amount: Option<Decimal>`. For a deposit or withdrawal without an amount, `TransactionAttempt::amount()` supplies zero and the operation is rejected as an invalid amount.

### 3. Correctness

**Status: satisfied.**

Operations preserve the expected relationships:

- deposit increases `available` and `total`;
- withdrawal reduces `available` and `total` only when sufficient funds are available;
- dispute reduces `available`, increases `held`, and preserves `total`;
- resolve reduces `held`, increases `available`, and preserves `total`;
- chargeback reduces `held` and `total` and locks the account.

Ownership checks now exist in `dispute`, `resolve`, and `chargeback`. Unit tests confirm that another client cannot act on the owner's transaction.

#### Decision on locked accounts

The following interpretation was adopted:

> The lock rejects new account activity but allows disputes that were already open before the chargeback to be completed.

For this reason, `Account::resolve` and `Account::chargeback` intentionally do not check `locked`. This should not be treated as a defect, because it is a deliberate decision addressing an ambiguity in the specification.

The `pending_disputes_can_be_completed_after_account_is_locked` test makes this decision executable and prevents future maintenance from mistakenly adding a lock check to these methods. The tested scenario:

1. performs three deposits for the same client;
2. places all three in dispute;
3. charges back the first, locking the account;
4. resolves the second and charges back the third;
5. confirms that both pending disputes were completed even though the account was locked.

#### Assumption about disputed withdrawals

The code stores successful deposits and withdrawals and allows both to be referenced by a dispute. It applies the general formula from the specification: reduce `available` and increase `held` by the transaction amount.

The specification does not define different behavior for disputing a withdrawal. This is another assumption that should be declared in the README to avoid leaving the interpretation implicit.

### 4. Safety and Robustness

**Status: satisfied.**

- monetary values use `rust_decimal::Decimal`, avoiding common floating-point errors;
- IDs use `u16` and `u32`, as requested;
- insufficient funds do not change the account;
- non-positive values are rejected;
- a second simultaneous dispute of the same transaction is ignored;
- events that reference nonexistent transactions or incompatible states are ignored;
- events from an incorrect client are ignored;
- structural read and write errors terminate the process with a failure;
- errors are not mixed into the output CSV.

Silence for business failures is consistent with the specification, which allows invalid events to be ignored. Avoiding one error message per line also prevents `stdout` corruption and excessive log volume for large files.

#### Robustness tests

Integration tests were added for:

- execution without an argument, expecting a non-zero exit code;
- a nonexistent path, expecting a non-zero exit code;
- structurally invalid CSV, expecting a non-zero exit code;
- confirmation that `stdout` remains empty in these cases.

These tests confirm the failure exit code, the absence of content in `stdout`, and the presence of a diagnostic in `stderr`.

### 5. Efficiency

**Status: satisfied.**

`reader::process_transactions_from_csv` iterates over `csv_reader.deserialize()` and sends each transaction directly to `Engine::process`. The complete file is no longer accumulated in a `Vec<TransactionAttempt>`.

For input processing, memory remains approximately constant: only the current transaction exists in addition to the reader's internal buffers.

`State` still grows with:

- the accounts encountered;
- successful transactions that may later be referenced by a dispute.

This growth is necessary under the current rules because a dispute may point to any previously processed `tx`.

#### Architectural observation

The `reader` module knows `Engine` directly. This keeps the implementation simple but couples parsing and processing. If the project grows, one possible evolution would separate:

- a function that receives any `std::io::Read` and produces events;
- opening the file specified in the arguments;
- processing the events through the engine.

This is not a necessary correction for the current challenge.

### 6. Maintainability

**Status: good, with clear remaining work.**

Positive points:

- the main responsibilities are separated among `Account`, `State`, `Engine`, `reader`, and `output`;
- balance logic is concentrated in `Account`;
- coordination and transaction-validation logic is concentrated in `Engine`;
- the account and transaction registries are encapsulated in `State`;
- tests use Arrange, Act, and Assert;
- happy paths and several failure modes have unit-test coverage;
- integration tests execute the real binary.

Remaining task: create a version-controlled `README.md`.

## Required README

There is currently no version-controlled `README.md`. Because `notes.md` is stored under `personal/` and that directory is not version-controlled, an evaluator will not see decisions recorded only there.

The future README should include, at minimum:

1. project objective;
2. commands to build, run, and test;
3. input and output examples;
4. architecture summary;
5. policy for technical errors and invalid business events;
6. the decision to allow disputes opened before an account lock to be completed afterward;
7. the adopted assumption for disputes that reference withdrawals;
8. an explanation that input is processed as a stream;
9. a description of the tests and important covered scenarios.

## Estimated Score

| Criterion | Estimate | Rationale |
|---|---:|---|
| Basics | 2.0 / 2.0 | Builds, runs, reads, and writes correctly; formatting passes. |
| Completeness | 2.0 / 2.0 | All event types have been implemented. |
| Correctness | 2.0 / 2.0 | Main rules, client ownership, and the post-lock policy are tested. |
| Safety and robustness | 2.0 / 2.0 | Errors, invalid events, and technical process failures are covered. |
| Efficiency | 1.0 / 1.0 | Input is processed as a stream. |
| Maintainability | 0.8 / 1.0 | Good separation, tests, and clean Clippy results; the README is still missing. |
| **Total** | **9.8 / 10.0** | Functional project; the main remaining issue is version-controlled documentation. |

This score is an estimate because the specification does not define official numerical weights for each criterion.

## Recommended Order for Reaching the Final Version

1. ~~replace `self: &mut Self` with `&mut self` in `Engine::process`;~~
2. ~~run Clippy again and confirm that there are no warnings;~~
3. ~~add the test that formalizes the policy for pending disputes after an account is locked;~~
4. ~~add integration tests for failure exit codes;~~
5. create and version the README with the decisions listed above;
6. ~~run `cargo test`, `cargo fmt --check`, and Clippy as the final verification.~~
