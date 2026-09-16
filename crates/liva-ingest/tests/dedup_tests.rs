//! Comprehensive tests for Ingest Deduplication Engine, dynamic BankProfile,
//! ProfileStatementParser, and two-tier cryptographic deduplication.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_ingest::dedup::{
    compute_statement_fingerprint, compute_txn_hash, compute_txn_hash_from_record,
    DeduplicationEngine, DeduplicationError,
};
use liva_ingest::models::{ContainerFormat, RawTransactionRecord};
use liva_ingest::parsers::ProfileStatementParser;
use liva_ingest::profile::{BankProfile, ColumnMappingConfig, ColumnSelector};
use liva_ingest::{parse_statement, IngestEngine};
use liva_normalize::{verify_statement_balance, BankIdentifier};
use std::fs;
use std::path::Path;

fn get_fixtures_dir() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir).join("../../fixtures/statements")
}

/// Test 1: Dynamic BankProfile parsing of CSV statement with semicolon and custom column mappings.
#[test]
fn test_1_dynamic_bank_profile_csv_semicolon_custom_mappings() {
    let csv_content = "\
Số tài khoản: 19034567890123
Tên tài khoản: CTY TNHH KINH DOANH LIVA
Số dư đầu kỳ: 100.000.000
Ngày;Diễn giải;Mã chứng từ;Tiền ghi nợ;Tiền ghi có;Số dư sau GD;Tên đối tác
01/08/2026;Thanh toan tien dien EVN;FT26001;2.500.000;;97.500.000;EVN HANOI
02/08/2026;Khach hang thanh toan tien hang;FT26002;;15.000.000;112.500.000;CONG TY DUC PHAT
03/08/2026;Rut tien mat ATM chi tieu;FT26003;5.000.000;;107.500.000;
Số dư cuối kỳ: 107.500.000
";

    let profile = BankProfile {
        id: "tcb_custom_semi".to_string(),
        profile_name: "Techcombank Custom Semicolon Profile".to_string(),
        bank_code: "TCB".to_string(),
        format: ContainerFormat::TextCsv,
        csv_delimiter: Some(';'),
        header_row_index: 3,
        data_start_row_index: 4,
        footer_skip_rows: 1,
        columns: ColumnMappingConfig {
            date: ColumnSelector::Name("Ngày".to_string()),
            val_date: None,
            doc_ref: Some(ColumnSelector::Name("Mã chứng từ".to_string())),
            debit: Some(ColumnSelector::Name("Tiền ghi nợ".to_string())),
            credit: Some(ColumnSelector::Name("Tiền ghi có".to_string())),
            amount: None,
            is_credit_flag: None,
            balance: Some(ColumnSelector::Name("Số dư sau GD".to_string())),
            narration: ColumnSelector::Name("Diễn giải".to_string()),
            counterparty: Some(ColumnSelector::Name("Tên đối tác".to_string())),
        },
        date_format: Some("DD/MM/YYYY".to_string()),
        decimal_separator: Some(','),
        thousands_separator: Some('.'),
    };

    let parser = ProfileStatementParser::new(profile);
    let raw = parser
        .parse_bytes(csv_content.as_bytes(), "test_tcb.csv")
        .expect("Parse custom CSV with BankProfile");

    assert_eq!(raw.bank, BankIdentifier::Techcombank);
    assert_eq!(raw.format, ContainerFormat::TextCsv);
    assert_eq!(raw.account_no.as_deref(), Some("19034567890123"));
    assert_eq!(raw.opening_balance, Some(100_000_000));
    assert_eq!(raw.closing_balance, Some(107_500_000));
    assert_eq!(raw.transactions.len(), 3);

    // Verify row 1: Debit 2,500,000
    let tx1 = &raw.transactions[0];
    assert_eq!(tx1.date_str, "01/08/2026");
    assert_eq!(tx1.amount_cents, Some(2_500_000));
    assert!(!tx1.is_credit);
    assert_eq!(tx1.doc_ref.as_deref(), Some("FT26001"));
    assert_eq!(tx1.counterparty_name.as_deref(), Some("EVN HANOI"));

    // Verify row 2: Credit 15,000,000
    let tx2 = &raw.transactions[1];
    assert_eq!(tx2.date_str, "02/08/2026");
    assert_eq!(tx2.amount_cents, Some(15_000_000));
    assert!(tx2.is_credit);
    assert_eq!(tx2.doc_ref.as_deref(), Some("FT26002"));

    // Normalize and verify balance continuity
    let norm = raw.normalize().expect("Normalize parsed statement");
    assert_eq!(norm.transactions.len(), 3);
    let inv = verify_statement_balance(&norm).expect("Verify balance invariant");
    assert!(inv.is_valid, "Balance invariant must hold: 100M - 2.5M + 15M - 5M = 107.5M");
}

/// Test 2: Dynamic BankProfile parsing of Excel statement with column index mappings and merged header skip.
#[test]
fn test_2_dynamic_bank_profile_excel_index_mappings() {
    let path = get_fixtures_dir().join("vcb_aug2026.xlsx");
    let bytes = fs::read(&path).expect("Read VCB XLSX fixture");

    // VCB XLSX header is at row index 9, transactions start at row index 10
    let profile = BankProfile {
        id: "vcb_index_profile".to_string(),
        profile_name: "VCB Direct Index Mapping".to_string(),
        bank_code: "VCB".to_string(),
        format: ContainerFormat::ExcelZip,
        csv_delimiter: None,
        header_row_index: 9,
        data_start_row_index: 10,
        footer_skip_rows: 0,
        columns: ColumnMappingConfig {
            date: ColumnSelector::Index(0),
            val_date: Some(ColumnSelector::Index(1)),
            doc_ref: Some(ColumnSelector::Index(2)),
            debit: Some(ColumnSelector::Index(3)),
            credit: Some(ColumnSelector::Index(4)),
            amount: None,
            is_credit_flag: None,
            balance: Some(ColumnSelector::Index(5)),
            narration: ColumnSelector::Index(6),
            counterparty: Some(ColumnSelector::Index(7)),
        },
        date_format: Some("DD/MM/YYYY".to_string()),
        decimal_separator: Some(','),
        thousands_separator: Some('.'),
    };

    let parser = ProfileStatementParser::new(profile);
    let raw = parser
        .parse_bytes(&bytes, "vcb_aug2026.xlsx")
        .expect("Parse VCB XLSX via ProfileStatementParser");

    assert_eq!(raw.bank, BankIdentifier::Vietcombank);
    assert_eq!(raw.format, ContainerFormat::ExcelZip);
    assert_eq!(raw.transactions.len(), 50);
    assert_eq!(raw.opening_balance, Some(1_450_230_000));
    assert_eq!(raw.closing_balance, Some(2_658_070_400));

    // Verify all transactions have valid dates and amounts
    for tx in &raw.transactions {
        assert!(!tx.date_str.is_empty());
        assert!(tx.amount_cents.unwrap_or(0) > 0);
    }

    let norm = raw.normalize().expect("Normalize VCB statement");
    assert_eq!(norm.transactions.len(), 50);
    let inv = verify_statement_balance(&norm).expect("Balance check");
    assert!(inv.is_valid, "Balance invariant must hold on VCB XLSX");
}

/// Test 3: compute_statement_fingerprint idempotency (same bytes & period produce exact same 64-char hex hash).
#[test]
fn test_3_compute_statement_fingerprint_idempotency() {
    let dummy_bytes = b"LIVA_BANKING_TEST_STATEMENT_CONTENT_2026_AUG";
    let account = "0011001234567";
    let start_date = "2026-08-01";
    let end_date = "2026-08-31";
    let opening = 1_000_000_000;
    let closing = 1_250_000_000;

    let fp1 = compute_statement_fingerprint(dummy_bytes, account, start_date, end_date, opening, closing);
    let fp2 = compute_statement_fingerprint(dummy_bytes, account, start_date, end_date, opening, closing);

    assert_eq!(fp1, fp2, "Fingerprint computation must be deterministic");
    assert_eq!(fp1.len(), 64, "SHA-256 hex digest must be exactly 64 characters");
    assert!(fp1.chars().all(|c| c.is_ascii_hexdigit()), "Fingerprint must be hexadecimal");

    // Mutation in any field changes the fingerprint
    let fp_diff_acc = compute_statement_fingerprint(dummy_bytes, "0011001234568", start_date, end_date, opening, closing);
    assert_ne!(fp1, fp_diff_acc, "Different account must produce different fingerprint");

    let fp_diff_start = compute_statement_fingerprint(dummy_bytes, account, "2026-08-02", end_date, opening, closing);
    assert_ne!(fp1, fp_diff_start, "Different start date must produce different fingerprint");

    let fp_diff_end = compute_statement_fingerprint(dummy_bytes, account, start_date, "2026-08-30", opening, closing);
    assert_ne!(fp1, fp_diff_end, "Different end date must produce different fingerprint");

    let fp_diff_open = compute_statement_fingerprint(dummy_bytes, account, start_date, end_date, opening + 1, closing);
    assert_ne!(fp1, fp_diff_open, "Different opening balance must produce different fingerprint");

    let fp_diff_close = compute_statement_fingerprint(dummy_bytes, account, start_date, end_date, opening, closing - 1);
    assert_ne!(fp1, fp_diff_close, "Different closing balance must produce different fingerprint");

    let fp_diff_bytes = compute_statement_fingerprint(b"DIFFERENT_BYTES", account, start_date, end_date, opening, closing);
    assert_ne!(fp1, fp_diff_bytes, "Different bytes must produce different fingerprint");
}

/// Test 4: Re-importing exact same statement file triggers DeduplicationError::DuplicateStatement.
#[test]
fn test_4_reimporting_same_statement_triggers_duplicate_statement() {
    let mut engine = DeduplicationEngine::new();
    let fp = "a3f8c9b102938475610293847561029384756102938475610293847561029384";
    let account = "0011001234567";
    let start_date = "2026-08-01";
    let end_date = "2026-08-31";

    // First registration succeeds
    let res1 = engine.check_and_register_statement(fp, account, start_date, end_date);
    assert!(res1.is_ok(), "First statement registration must succeed");
    assert!(engine.is_fingerprint_registered(fp));

    // Second registration with exact same fingerprint fails with DuplicateStatement
    let res2 = engine.check_and_register_statement(fp, account, start_date, end_date);
    assert!(res2.is_err(), "Duplicate statement must be rejected");

    match res2 {
        Err(DeduplicationError::DuplicateStatement { fingerprint }) => {
            assert_eq!(fingerprint, fp);
        }
        other => panic!("Expected DuplicateStatement error, got {other:?}"),
    }
}

/// Test 5: Overlapping statement period triggers DeduplicationError::OverlappingPeriod.
#[test]
fn test_5_overlapping_period_triggers_overlapping_period_error() {
    let mut engine = DeduplicationEngine::new();
    let account = "0011001234567";

    // Finalize period 2026-08-01 to 2026-08-31
    let fp1 = "1111111111111111111111111111111111111111111111111111111111111111";
    engine
        .check_and_register_statement(fp1, account, "2026-08-01", "2026-08-31")
        .expect("Register initial period");

    // Attempt to register overlapping period: 2026-08-15 to 2026-09-15
    let fp2 = "2222222222222222222222222222222222222222222222222222222222222222";
    let res = engine.check_and_register_statement(fp2, account, "2026-08-15", "2026-09-15");

    assert!(res.is_err(), "Overlapping period must be rejected");
    match res {
        Err(DeduplicationError::OverlappingPeriod {
            account_no,
            requested_period,
            conflicting_period,
        }) => {
            assert_eq!(account_no, account);
            assert_eq!(requested_period, ("2026-08-15".to_string(), "2026-09-15".to_string()));
            assert_eq!(conflicting_period, ("2026-08-01".to_string(), "2026-08-31".to_string()));
        }
        other => panic!("Expected OverlappingPeriod error, got {other:?}"),
    }
}

/// Test 6: Non-overlapping periods succeed without conflict.
#[test]
fn test_6_non_overlapping_periods_succeed() {
    let mut engine = DeduplicationEngine::new();
    let account = "0011001234567";

    // August statement: 2026-08-01 to 2026-08-31
    let fp_aug = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    engine
        .check_and_register_statement(fp_aug, account, "2026-08-01", "2026-08-31")
        .expect("Register August statement");

    // September statement: 2026-09-01 to 2026-09-30 (adjacent, non-overlapping)
    let fp_sep = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let res_sep = engine.check_and_register_statement(fp_sep, account, "2026-09-01", "2026-09-30");
    assert!(res_sep.is_ok(), "September statement must register without conflict");

    // October statement: 2026-10-01 to 2026-10-31
    let fp_oct = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    let res_oct = engine.check_and_register_statement(fp_oct, account, "2026-10-01", "2026-10-31");
    assert!(res_oct.is_ok(), "October statement must register without conflict");

    // Different account with overlapping dates to August succeeds
    let fp_diff_acc = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    let res_diff_acc = engine.check_and_register_statement(fp_diff_acc, "9999999999", "2026-08-01", "2026-08-31");
    assert!(res_diff_acc.is_ok(), "Different account with same date range must succeed");

    // Verify account periods count
    let periods = engine.get_account_periods(account).expect("Get account periods");
    assert_eq!(periods.len(), 3);
}

/// Test 7: compute_txn_hash produces collision-resistant 64-char hex hashes;
/// 1-byte mutation in date, amount, voucher, balance, or narration produces a different hash.
#[test]
fn test_7_compute_txn_hash_collision_resistance_and_mutation() {
    let account = "0011001234567";
    let date = "2026-08-15";
    let amount = 50_000_000;
    let is_credit = true;
    let voucher = Some("FT2621500001");
    let balance = 150_000_000;
    let narration = "THANH TOAN TIEN HANG HD 12345";

    let base_hash = compute_txn_hash(account, date, amount, is_credit, voucher, balance, narration);
    assert_eq!(base_hash.len(), 64);
    assert!(base_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // 1-byte mutation in date
    let h_date = compute_txn_hash(account, "2026-08-16", amount, is_credit, voucher, balance, narration);
    assert_ne!(base_hash, h_date, "Date mutation must change hash");

    // 1-byte mutation in amount
    let h_amount = compute_txn_hash(account, date, amount + 1, is_credit, voucher, balance, narration);
    assert_ne!(base_hash, h_amount, "Amount mutation must change hash");

    // Flip CR/DR
    let h_cr = compute_txn_hash(account, date, amount, !is_credit, voucher, balance, narration);
    assert_ne!(base_hash, h_cr, "CR/DR toggle must change hash");

    // 1-byte mutation in voucher
    let h_voucher = compute_txn_hash(account, date, amount, is_credit, Some("FT2621500002"), balance, narration);
    assert_ne!(base_hash, h_voucher, "Voucher mutation must change hash");

    // Voucher None vs Some
    let h_no_voucher = compute_txn_hash(account, date, amount, is_credit, None, balance, narration);
    assert_ne!(base_hash, h_no_voucher, "Missing voucher must change hash");

    // 1-byte mutation in balance
    let h_balance = compute_txn_hash(account, date, amount, is_credit, voucher, balance - 1, narration);
    assert_ne!(base_hash, h_balance, "Balance mutation must change hash");

    // 1-byte mutation in narration
    let h_narration = compute_txn_hash(account, date, amount, is_credit, voucher, balance, "THANH TOAN TIEN HANG HD 12346");
    assert_ne!(base_hash, h_narration, "Narration mutation must change hash");

    // Whitespace trimming normalization: " FT2621500001 " should equal "FT2621500001"
    let h_trimmed = compute_txn_hash(
        account,
        &format!("  {date}  "),
        amount,
        is_credit,
        Some("  FT2621500001  "),
        balance,
        &format!("  {narration}  "),
    );
    assert_eq!(base_hash, h_trimmed, "Whitespace normalization must produce identical hash");
}

/// Test 8: Duplicate transaction rows within batch or across batches are flagged and deduplicated.
#[test]
fn test_8_duplicate_transaction_rows_deduplication() {
    let mut engine = DeduplicationEngine::new();
    let account = "0011001234567";

    let tx_a = RawTransactionRecord {
        row_id: 1,
        date_str: "2026-08-01".to_string(),
        val_date_str: None,
        doc_ref: Some("FT001".to_string()),
        debit_amt_str: None,
        credit_amt_str: Some("10000000".to_string()),
        amount_cents: Some(10_000_000),
        is_credit: true,
        balance_str: Some("110000000".to_string()),
        balance_cents: Some(110_000_000),
        narration: "Transfer A".to_string(),
        counterparty_name: None,
    };

    let tx_b = RawTransactionRecord {
        row_id: 2,
        date_str: "2026-08-02".to_string(),
        val_date_str: None,
        doc_ref: Some("FT002".to_string()),
        debit_amt_str: Some("5000000".to_string()),
        credit_amt_str: None,
        amount_cents: Some(5_000_000),
        is_credit: false,
        balance_str: Some("105000000".to_string()),
        balance_cents: Some(105_000_000),
        narration: "Payment B".to_string(),
        counterparty_name: None,
    };

    let tx_c = RawTransactionRecord {
        row_id: 3,
        date_str: "2026-08-03".to_string(),
        val_date_str: None,
        doc_ref: Some("FT003".to_string()),
        debit_amt_str: None,
        credit_amt_str: Some("20000000".to_string()),
        amount_cents: Some(20_000_000),
        is_credit: true,
        balance_str: Some("125000000".to_string()),
        balance_cents: Some(125_000_000),
        narration: "Deposit C".to_string(),
        counterparty_name: None,
    };

    // Batch 1 has TxA, TxB, and duplicate TxA within the same batch
    let batch_1 = vec![tx_a.clone(), tx_b.clone(), tx_a.clone()];
    let (unique_txns_1, report_1) = engine.filter_transactions(account, &batch_1);

    assert_eq!(report_1.total_input_records, 3);
    assert_eq!(report_1.unique_records_count, 2);
    assert_eq!(report_1.duplicate_records_count, 1);
    assert!(report_1.has_duplicates());
    assert_eq!(unique_txns_1.len(), 2);

    let hash_a = compute_txn_hash_from_record(account, &tx_a);
    assert_eq!(report_1.duplicate_txn_hashes, vec![hash_a.clone()]);
    assert!(engine.is_txn_seen(&hash_a));

    // Batch 2 has TxB (already seen in Batch 1) and new TxC
    let batch_2 = vec![tx_b.clone(), tx_c.clone()];
    let (unique_txns_2, report_2) = engine.filter_transactions(account, &batch_2);

    assert_eq!(report_2.total_input_records, 2);
    assert_eq!(report_2.unique_records_count, 1);
    assert_eq!(report_2.duplicate_records_count, 1);
    assert_eq!(unique_txns_2.len(), 1);
    assert_eq!(unique_txns_2[0].doc_ref.as_deref(), Some("FT003"));

    let hash_b = compute_txn_hash_from_record(account, &tx_b);
    assert_eq!(report_2.duplicate_txn_hashes, vec![hash_b]);
}

/// Test 9: Existing statement fixtures in fixtures/statements/ (vcb_aug2026.xlsx, tcb_aug2026.csv) parse without panic.
#[test]
fn test_9_existing_statement_fixtures_parse_without_panic() {
    let fixtures_dir = get_fixtures_dir();
    let engine = IngestEngine::new();

    // 1. VCB XLSX fixture
    let vcb_path = fixtures_dir.join("vcb_aug2026.xlsx");
    let vcb_bytes = fs::read(&vcb_path).expect("Read VCB XLSX fixture");
    let vcb_stmt = engine
        .ingest_normalized(&vcb_bytes, "vcb_aug2026.xlsx")
        .expect("Ingest and normalize VCB XLSX fixture");

    assert_eq!(vcb_stmt.bank, BankIdentifier::Vietcombank);
    assert!(vcb_stmt.opening_cents > 0);
    assert!(vcb_stmt.closing_cents > 0);
    assert!(!vcb_stmt.transactions.is_empty());

    let vcb_inv = verify_statement_balance(&vcb_stmt).expect("Verify VCB balance invariant");
    assert!(vcb_inv.is_valid, "VCB balance invariant must pass on golden fixture");

    // 2. TCB CSV fixture
    let tcb_path = fixtures_dir.join("tcb_aug2026.csv");
    let tcb_bytes = fs::read(&tcb_path).expect("Read TCB CSV fixture");
    let tcb_stmt = engine
        .ingest_normalized(&tcb_bytes, "tcb_aug2026.csv")
        .expect("Ingest and normalize TCB CSV fixture");

    assert_eq!(tcb_stmt.bank, BankIdentifier::Techcombank);
    assert!(tcb_stmt.opening_cents > 0);
    assert!(!tcb_stmt.transactions.is_empty());

    let tcb_inv = verify_statement_balance(&tcb_stmt).expect("Verify TCB balance invariant");
    assert!(tcb_inv.is_valid, "TCB balance invariant must pass on golden fixture");

    // 3. CAMT.053 XML fixture
    let camt_path = fixtures_dir.join("camt053_aug2026.xml");
    let camt_bytes = fs::read(&camt_path).expect("Read CAMT.053 fixture");
    let camt_stmt = parse_statement(&camt_bytes, "camt053_aug2026.xml").expect("Parse CAMT.053 fixture");
    assert_eq!(camt_stmt.transactions.len(), 14);

    // 4. SWIFT MT940 fixture
    let mt940_path = fixtures_dir.join("mt940_aug2026.txt");
    let mt940_bytes = fs::read(&mt940_path).expect("Read MT940 fixture");
    let mt940_stmt = parse_statement(&mt940_bytes, "mt940_aug2026.txt").expect("Parse MT940 fixture");
    assert_eq!(mt940_stmt.transactions.len(), 14);
}
