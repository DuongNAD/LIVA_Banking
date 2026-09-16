//! Bank and ERP Statement Parsers.
//!
//! Provides deterministic streaming parsers for:
//! - Vietcombank (VCB) Excel (.xlsx)
//! - Techcombank (TCB) CSV
//! - ERP General Ledger open vouchers / invoices (CSV / XLSX)
//! - ISO 20022 CAMT.053 XML

pub mod erp;
pub mod iso20022_xml;
pub mod tcb_csv;
pub mod vcb_excel;

pub use erp::ErpInvoiceParser;
pub use iso20022_xml::Iso20022XmlParser;
pub use tcb_csv::TcbCsvParser;
pub use vcb_excel::VcbExcelParser;

use crate::models::BankStatement;
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParserError {
    UnsupportedFormat { filename: String },
    ExcelError(String),
    CsvError(String),
    IoError(String),
    InvalidStructure(String),
    DuplicateStatementFile { hash: String },
    BalanceInvariantFailed { opening: u64, closing: u64, calculated: u64 },
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::UnsupportedFormat { filename } => {
                write!(f, "Unsupported statement format for: {filename}")
            }
            ParserError::ExcelError(msg) => write!(f, "Excel parser error: {msg}"),
            ParserError::CsvError(msg) => write!(f, "CSV parser error: {msg}"),
            ParserError::IoError(msg) => write!(f, "IO error: {msg}"),
            ParserError::InvalidStructure(msg) => write!(f, "Invalid structure: {msg}"),
            ParserError::DuplicateStatementFile { hash } => {
                write!(f, "Duplicate statement file detected (SHA-256: {hash})")
            }
            ParserError::BalanceInvariantFailed { opening, closing, calculated } => {
                write!(f, "Balance invariant failed: opening={opening}, closing={closing}, expected={calculated}")
            }
        }
    }
}

impl std::error::Error for ParserError {}

/// Core trait for Bank Statement Parsers.
pub trait BankStatementParser: Send + Sync {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool;
    fn parse(&self, bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError>;
}

/// Computes SHA-256 hash of a file for deduplication.
pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Intelligent dispatcher that sniffs and parses a statement file,
/// recording the SHA-256 hash and checking balance invariants.
pub fn sniff_and_parse(bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
    let parsers: Vec<Box<dyn BankStatementParser>> = vec![
        Box::new(VcbExcelParser),
        Box::new(TcbCsvParser),
        Box::new(Iso20022XmlParser),
    ];

    let mut parsed: Option<BankStatement> = None;
    for p in &parsers {
        if p.sniff(bytes, filename) {
            match p.parse(bytes, filename) {
                Ok(stmt) => {
                    parsed = Some(stmt);
                    break;
                }
                Err(_) => continue,
            }
        }
    }

    let mut statement = match parsed {
        Some(s) => s,
        None => {
            // Fallback trial
            for p in &parsers {
                if let Ok(s) = p.parse(bytes, filename) {
                    parsed = Some(s);
                    break;
                }
            }
            parsed.ok_or_else(|| ParserError::UnsupportedFormat {
                filename: filename.to_string(),
            })?
        }
    };

    statement.file_hash_sha256 = Some(compute_sha256(bytes));
    statement.verify_balance_invariant();
    Ok(statement)
}
