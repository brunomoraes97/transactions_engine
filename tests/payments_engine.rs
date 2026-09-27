use std::{
    fs,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct AccountOutput {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

static NEXT_FILE_ID: AtomicU64 = AtomicU64::new(0);

fn temporary_input_path() -> std::path::PathBuf {
    let unique_id = NEXT_FILE_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "transactions_engine_{}_{}.csv",
        std::process::id(),
        unique_id,
    ))
}

fn run_engine(input: &str) -> Vec<AccountOutput> {
    let input_path = temporary_input_path();

    fs::write(&input_path, input).expect("temporary input CSV should be created");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_transactions_engine"))
        .arg(&input_path)
        .output()
        .expect("payments engine should execute");

    fs::remove_file(&input_path).expect("temporary input CSV should be removed");

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "payments engine failed: {}",
        stderr,
    );

    let mut csv_reader = csv::Reader::from_reader(output.stdout.as_slice());
    let headers: Vec<&str> = csv_reader
        .headers()
        .expect("output should contain CSV headers")
        .iter()
        .collect();
    assert_eq!(
        headers,
        vec!["client", "available", "held", "total", "locked"],
        "unexpected CSV headers; stderr: {}",
        stderr,
    );

    let mut accounts: Vec<AccountOutput> = csv_reader
        .deserialize()
        .collect::<Result<_, _>>()
        .expect("output should contain valid account records");
    accounts.sort_by_key(|account| account.client);

    accounts
}

fn assert_engine_failed_without_output(output: Output) {
    assert!(!output.status.success(), "payments engine should fail");
    assert!(output.stdout.is_empty(), "stdout should remain empty");
    assert!(
        !output.stderr.is_empty(),
        "stderr should describe the error"
    );
}

#[test]
fn processes_deposits_and_withdrawals() {
    // Arrange
    let input = "type,client,tx,amount\n\
                 deposit,1,1,1.0\n\
                 deposit,2,2,2.0\n\
                 deposit,1,3,2.0\n\
                 withdrawal,1,4,1.5\n\
                 withdrawal,2,5,3.0\n";

    // Act
    let accounts = run_engine(input);

    // Assert
    assert_eq!(
        accounts,
        vec![
            AccountOutput {
                client: 1,
                available: Decimal::new(15, 1),
                held: Decimal::ZERO,
                total: Decimal::new(15, 1),
                locked: false,
            },
            AccountOutput {
                client: 2,
                available: Decimal::new(2, 0),
                held: Decimal::ZERO,
                total: Decimal::new(2, 0),
                locked: false,
            },
        ],
    );
}

#[test]
fn processes_dispute_and_resolve() {
    // Arrange
    let input = "type,client,tx,amount\n\
                 deposit,1,1,10.0\n\
                 dispute,1,1,\n\
                 resolve,1,1,\n";

    // Act
    let accounts = run_engine(input);

    // Assert
    assert_eq!(
        accounts,
        vec![AccountOutput {
            client: 1,
            available: Decimal::new(10, 0),
            held: Decimal::ZERO,
            total: Decimal::new(10, 0),
            locked: false,
        }],
    );
}

#[test]
fn processes_dispute_and_chargeback() {
    // Arrange
    let input = "type,client,tx,amount\n\
                 deposit,1,1,8.0\n\
                 dispute,1,1,\n\
                 chargeback,1,1,\n\
                 deposit,1,2,2.0\n";

    // Act
    let accounts = run_engine(input);

    // Assert
    assert_eq!(
        accounts,
        vec![AccountOutput {
            client: 1,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: true,
        }],
    );
}

#[test]
fn fails_when_input_argument_is_missing() {
    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_transactions_engine"))
        .output()
        .expect("payments engine should execute");

    // Assert
    assert_engine_failed_without_output(output);
}

#[test]
fn fails_when_input_file_does_not_exist() {
    // Arrange
    let missing_input_path = temporary_input_path();

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_transactions_engine"))
        .arg(missing_input_path)
        .output()
        .expect("payments engine should execute");

    // Assert
    assert_engine_failed_without_output(output);
}

#[test]
fn fails_when_input_csv_is_invalid() {
    // Arrange
    let input_path = temporary_input_path();
    let invalid_input = "type,client,tx,amount\ndeposit,invalid-client,1,5.0\n";
    fs::write(&input_path, invalid_input).expect("temporary input CSV should be created");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_transactions_engine"))
        .arg(&input_path)
        .output()
        .expect("payments engine should execute");

    fs::remove_file(&input_path).expect("temporary input CSV should be removed");

    // Assert
    assert_engine_failed_without_output(output);
}
