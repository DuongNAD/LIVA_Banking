//! Verification tests against golden sample bank statements in `data/demo_ready/`.
//! Verifies 100% extraction accuracy and mathematical balance invariants.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_ledger::PostingType;
use liva_money::Money;
use liva_parse::StatementEngine;
use std::fs;
use std::path::PathBuf;

fn get_golden_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../data/demo_ready")
}

#[test]
fn test_golden_tcb_csv_parsing() {
    let path = get_golden_dir().join("02_TCB_SaoKe_Thang8_Chuan.csv");
    let raw = fs::read(&path).expect("Failed to read TCB golden sample");

    let engine = StatementEngine::new().expect("Failed to initialize StatementEngine");
    let stmt = engine
        .parse(&raw, "02_TCB_SaoKe_Thang8_Chuan.csv")
        .expect("Failed to parse TCB golden CSV");

    assert_eq!(stmt.bank_code, "TCB");
    assert_eq!(stmt.account_number.as_deref(), Some("19034567890123"));
    assert_eq!(stmt.opening_balance, Some(Money::vnd(785_600_000)));
    assert_eq!(stmt.closing_balance, Some(Money::vnd(1_583_400_000)));

    // Exact 5 transactions
    assert_eq!(stmt.transactions.len(), 5);

    // Tx 1: Credit 45.000.000, Balance 830.600.000, Partner An Phat
    assert_eq!(stmt.transactions[0].amount, Money::vnd(45_000_000));
    assert_eq!(stmt.transactions[0].posting_type, PostingType::Credit);
    assert_eq!(stmt.transactions[0].balance_after, Some(Money::vnd(830_600_000)));
    assert_eq!(stmt.transactions[0].ref_code, "TCB26240001");
    assert_eq!(stmt.transactions[0].counterparty_name.as_deref(), Some("Công ty TNHH TM DV An Phát"));

    // Tx 2: Credit 550.000.000, Balance 1.380.600.000, Masan
    assert_eq!(stmt.transactions[1].amount, Money::vnd(550_000_000));
    assert_eq!(stmt.transactions[1].posting_type, PostingType::Credit);
    assert_eq!(stmt.transactions[1].balance_after, Some(Money::vnd(1_380_600_000)));

    // Tx 3: Debit 2.200.000, Balance 1.378.400.000, Techcombank
    assert_eq!(stmt.transactions[2].amount, Money::vnd(2_200_000));
    assert_eq!(stmt.transactions[2].posting_type, PostingType::Debit);
    assert_eq!(stmt.transactions[2].balance_after, Some(Money::vnd(1_378_400_000)));

    // Tx 4: Credit 120.000.000, Balance 1.498.400.000, Dai Viet
    assert_eq!(stmt.transactions[3].amount, Money::vnd(120_000_000));
    assert_eq!(stmt.transactions[3].posting_type, PostingType::Credit);
    assert_eq!(stmt.transactions[3].balance_after, Some(Money::vnd(1_498_400_000)));

    // Tx 5: Credit 85.000.000, Balance 1.583.400.000, FPT Smart Cloud
    assert_eq!(stmt.transactions[4].amount, Money::vnd(85_000_000));
    assert_eq!(stmt.transactions[4].posting_type, PostingType::Credit);
    assert_eq!(stmt.transactions[4].balance_after, Some(Money::vnd(1_583_400_000)));

    // Mathematical Balance Checksum: 785.6M + 800M - 2.2M == 1.583.4M
    assert!(stmt.verify_invariants().expect("Invariant validation failed"));
}

#[test]
fn test_golden_bidv_csv_parsing() {
    let path = get_golden_dir().join("03_BIDV_SaoKe_NgoaiGio_BatThuong.csv");
    let raw = fs::read(&path).expect("Failed to read BIDV golden sample");

    let engine = StatementEngine::new().expect("Failed to initialize StatementEngine");
    let stmt = engine
        .parse(&raw, "03_BIDV_SaoKe_NgoaiGio_BatThuong.csv")
        .expect("Failed to parse BIDV golden CSV");

    assert_eq!(stmt.bank_code, "BIDV");
    assert_eq!(stmt.account_number.as_deref(), Some("12010001234567"));

    // 4 transactions in BIDV statement
    assert_eq!(stmt.transactions.len(), 4);

    // Tx 1: Credit 420.000.000, Balance 932.000.000
    assert_eq!(stmt.transactions[0].amount, Money::vnd(420_000_000));
    assert_eq!(stmt.transactions[0].posting_type, PostingType::Credit);
    assert_eq!(stmt.transactions[0].balance_after, Some(Money::vnd(932_000_000)));
    assert_eq!(stmt.transactions[0].ref_code, "BIDV2621001");
    assert_eq!(stmt.transactions[0].counterparty_name.as_deref(), Some("VU TRONG PHUONG"));

    // Tx 2: Debit 419.500.000, Balance 512.500.000
    assert_eq!(stmt.transactions[1].amount, Money::vnd(419_500_000));
    assert_eq!(stmt.transactions[1].posting_type, PostingType::Debit);
    assert_eq!(stmt.transactions[1].balance_after, Some(Money::vnd(512_500_000)));

    // Tx 3: Credit 60.000.000, Balance 572.500.000
    assert_eq!(stmt.transactions[2].amount, Money::vnd(60_000_000));
    assert_eq!(stmt.transactions[2].posting_type, PostingType::Credit);
    assert_eq!(stmt.transactions[2].balance_after, Some(Money::vnd(572_500_000)));

    // Tx 4: Debit 500.000, Balance 572.000.000
    assert_eq!(stmt.transactions[3].amount, Money::vnd(500_000));
    assert_eq!(stmt.transactions[3].posting_type, PostingType::Debit);
    assert_eq!(stmt.transactions[3].balance_after, Some(Money::vnd(572_000_000)));

    // Mathematical Balance Checksum: Inferred opening: 512M + 420M - 419.5M + 60M - 0.5M == 572M
    assert_eq!(stmt.closing_balance, Some(Money::vnd(572_000_000)));
    assert!(stmt.verify_invariants().expect("Invariant validation failed"));
}

#[test]
fn test_golden_vcb_excel_parsing() {
    let path = get_golden_dir().join("01_VCB_SaoKe_Thang8_Chuan.xlsx");
    let raw = fs::read(&path).expect("Failed to read VCB golden sample");

    let engine = StatementEngine::new().expect("Failed to initialize StatementEngine");
    let stmt = engine
        .parse(&raw, "01_VCB_SaoKe_Thang8_Chuan.xlsx")
        .expect("Failed to parse VCB golden Excel");

    assert_eq!(stmt.bank_code, "VCB");
    assert!(stmt.account_number.is_some(), "VCB account number should be parsed");
    assert!(!stmt.transactions.is_empty(), "VCB statement should contain transactions");

    // All transactions must have non-zero amounts
    for tx in &stmt.transactions {
        assert!(tx.amount.amount() > 0, "Transaction amount must be positive");
    }

    // Mathematical Balance Invariant
    assert!(stmt.verify_invariants().expect("Invariant validation failed"));
}
