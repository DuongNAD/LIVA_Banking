//! Integration tests for liva-ingest crate on golden fixtures.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_ingest::{parse_statement, ContainerFormat, IngestEngine};
use liva_normalize::{verify_statement_balance, BankIdentifier};
use std::fs;
use std::path::Path;

fn get_fixtures_dir() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir).join("../../fixtures/statements")
}

#[test]
fn test_sniffing_golden_fixtures() {
    let fixtures_dir = get_fixtures_dir();
    let engine = IngestEngine::new();

    // 1. CAMT.053 XML
    let camt_path = fixtures_dir.join("camt053_aug2026.xml");
    let camt_bytes = fs::read(&camt_path).expect("Read camt053 fixture");
    let (fmt, bank) = engine.sniff(&camt_bytes, "camt053_aug2026.xml");
    assert_eq!(fmt, ContainerFormat::Xml);
    assert!(bank == BankIdentifier::Iso20022 || bank == BankIdentifier::Vietcombank);

    // 2. SWIFT MT940
    let mt940_path = fixtures_dir.join("mt940_aug2026.txt");
    let mt940_bytes = fs::read(&mt940_path).expect("Read mt940 fixture");
    let (fmt, bank) = engine.sniff(&mt940_bytes, "mt940_aug2026.txt");
    assert_eq!(fmt, ContainerFormat::SwiftMt);
    assert!(bank == BankIdentifier::Swift || bank == BankIdentifier::Techcombank);

    // 3. TCB CSV
    let tcb_path = fixtures_dir.join("tcb_aug2026.csv");
    let tcb_bytes = fs::read(&tcb_path).expect("Read tcb fixture");
    let (fmt, bank) = engine.sniff(&tcb_bytes, "tcb_aug2026.csv");
    assert_eq!(fmt, ContainerFormat::TextCsv);
    assert_eq!(bank, BankIdentifier::Techcombank);

    // 4. VCB XLSX
    let vcb_path = fixtures_dir.join("vcb_aug2026.xlsx");
    let vcb_bytes = fs::read(&vcb_path).expect("Read vcb fixture");
    let (fmt, bank) = engine.sniff(&vcb_bytes, "vcb_aug2026.xlsx");
    assert_eq!(fmt, ContainerFormat::ExcelZip);
    assert_eq!(bank, BankIdentifier::Vietcombank);

    // 5. BIDV PDF
    let bidv_path = fixtures_dir.join("bidv_aug2026.pdf");
    let bidv_bytes = fs::read(&bidv_path).expect("Read bidv fixture");
    let (fmt, bank) = engine.sniff(&bidv_bytes, "bidv_aug2026.pdf");
    assert_eq!(fmt, ContainerFormat::Pdf);
    assert_eq!(bank, BankIdentifier::Bidv);
}

#[test]
fn test_parse_camt053_xml_full_statement() {
    let path = get_fixtures_dir().join("camt053_aug2026.xml");
    let bytes = fs::read(&path).expect("Read camt053 fixture");

    let stmt = parse_statement(&bytes, "camt053_aug2026.xml").expect("Parse CAMT.053");

    assert_eq!(stmt.bank, BankIdentifier::Vietcombank);
    assert_eq!(stmt.account_no, "0011001234567");
    assert_eq!(stmt.opening_cents, 1_450_230_000);
    assert_eq!(stmt.closing_cents, 1_629_708_000);
    assert_eq!(stmt.transactions.len(), 14);

    // Verify first transaction details
    let tx1 = &stmt.transactions[0];
    assert_eq!(tx1.amount_cents, 12_500_000);
    assert!(tx1.is_credit);
    assert_eq!(tx1.voucher_no.as_deref(), Some("FT2621500001"));
    assert!(tx1.counterparty_name.is_some());

    // Verify balance invariant: Opening + sum(credit) - sum(debit) == Closing
    let inv = verify_statement_balance(&stmt).expect("Balance invariant calculation");
    assert!(inv.is_valid, "CAMT.053 balance invariant failed");
}

#[test]
fn test_parse_mt940_txt_full_statement() {
    let path = get_fixtures_dir().join("mt940_aug2026.txt");
    let bytes = fs::read(&path).expect("Read mt940 fixture");

    let stmt = parse_statement(&bytes, "mt940_aug2026.txt").expect("Parse SWIFT MT940");

    assert_eq!(stmt.bank, BankIdentifier::Techcombank);
    assert_eq!(stmt.account_no, "19034567890123");
    assert_eq!(stmt.opening_cents, 785_600_000);
    assert_eq!(stmt.closing_cents, 1_004_589_000);
    assert_eq!(stmt.transactions.len(), 14);

    let tx1 = &stmt.transactions[0];
    assert_eq!(tx1.amount_cents, 11_500_000);
    assert!(tx1.is_credit);
    assert_eq!(tx1.voucher_no.as_deref(), Some("FT2621400001"));

    let inv = verify_statement_balance(&stmt).expect("Balance invariant calculation");
    assert!(inv.is_valid, "MT940 balance invariant failed");
}

#[test]
fn test_parse_tcb_csv_full_statement() {
    let path = get_fixtures_dir().join("tcb_aug2026.csv");
    let bytes = fs::read(&path).expect("Read TCB CSV fixture");

    let stmt = parse_statement(&bytes, "tcb_aug2026.csv").expect("Parse TCB CSV");

    assert_eq!(stmt.bank, BankIdentifier::Techcombank);
    assert!(stmt.opening_cents > 0);
    assert!(stmt.transactions.len() >= 5);

    let inv = verify_statement_balance(&stmt).expect("Balance invariant calculation");
    assert!(inv.is_valid, "TCB CSV balance invariant failed");
}

#[test]
fn test_parse_vcb_xlsx_full_statement() {
    let path = get_fixtures_dir().join("vcb_aug2026.xlsx");
    let bytes = fs::read(&path).expect("Read VCB XLSX fixture");

    let stmt = parse_statement(&bytes, "vcb_aug2026.xlsx").expect("Parse VCB XLSX");

    assert_eq!(stmt.bank, BankIdentifier::Vietcombank);
    assert!(stmt.transactions.len() >= 5);

    let inv = verify_statement_balance(&stmt).expect("Balance invariant calculation");
    assert!(inv.is_valid, "VCB XLSX balance invariant failed");
}

#[test]
fn test_parse_bidv_pdf_full_statement() {
    let path = get_fixtures_dir().join("bidv_aug2026.pdf");
    let bytes = fs::read(&path).expect("Read BIDV PDF fixture");

    let stmt = parse_statement(&bytes, "bidv_aug2026.pdf").expect("Parse BIDV PDF");

    assert_eq!(stmt.bank, BankIdentifier::Bidv);
    assert!(stmt.transactions.len() >= 5);
}
