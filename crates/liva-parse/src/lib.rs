//! LIVA Banking — `liva-parse` Crate
//!
//! Declarative Statement Template DSL Engine, Canonical Transaction Schema,
//! and multi-bank deterministic parser engine.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod builtin_templates;
pub mod engine;
pub mod error;
pub mod template;

pub use engine::{StatementEngine, TemplateRegistry};
pub use error::ParseError;
pub use template::schema::*;
pub use template::validator::validate_template;

use liva_ledger::{Ledger, PostingType};
use liva_money::Money;
use serde::{Deserialize, Serialize};

/// Standardized canonical transaction record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalTx {
    pub row_id: usize,
    pub bank_code: String,
    pub ref_code: String,
    pub posting_type: PostingType,
    pub amount: Money,
    pub balance_after: Option<Money>,
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
    pub narration: String,
    pub tx_timestamp: i64,
}

/// Standardized bank statement container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalStatement {
    pub bank_code: String,
    pub account_number: Option<String>,
    pub opening_balance: Option<Money>,
    pub closing_balance: Option<Money>,
    pub transactions: Vec<CanonicalTx>,
}

impl CanonicalStatement {
    /// Evaluates whether the statement satisfies Closing = Opening + Credit - Debit.
    pub fn verify_invariants(&self) -> Result<bool, ParseError> {
        let (opening, closing) = match (self.opening_balance, self.closing_balance) {
            (Some(o), Some(c)) => (o, c),
            _ => return Ok(true), // Cannot verify without explicit opening & closing
        };

        let txs: Vec<(PostingType, Money)> = self
            .transactions
            .iter()
            .map(|t| (t.posting_type, t.amount))
            .collect();

        Ledger::verify_statement_balance(opening, closing, &txs)
            .map_err(|e| ParseError::InvariantError(e.to_string()))
    }
}

/// Contract for multi-bank statement parsers.
pub trait BankParser: Send + Sync {
    /// Returns true if this parser can handle the provided raw payload and filename.
    fn sniff(&self, raw: &[u8], filename: &str) -> bool;

    /// Parses the raw bytes into a standardized canonical bank statement.
    fn parse(&self, raw: &[u8], filename: &str) -> Result<CanonicalStatement, ParseError>;
}

/// Backward compatibility parser for Techcombank (TCB) CSV exports.
pub struct TcbCsvParser;

impl BankParser for TcbCsvParser {
    fn sniff(&self, raw: &[u8], filename: &str) -> bool {
        let name_lower = filename.to_lowercase();
        if !name_lower.ends_with(".csv") && !name_lower.ends_with(".txt") {
            return false;
        }
        let head = String::from_utf8_lossy(&raw[..raw.len().min(1024)]).to_lowercase();
        head.contains("techcombank") || head.contains("tcb") || head.contains("số tài khoản:")
    }

    fn parse(&self, raw: &[u8], filename: &str) -> Result<CanonicalStatement, ParseError> {
        let engine = StatementEngine::new()?;
        if let Some(tcb_template) = engine.registry().find_by_bank_code("TCB") {
            engine.parse_with_template(raw, filename, tcb_template)
        } else {
            engine.parse(raw, filename)
        }
    }
}

/// Backward compatibility parser for ISO 20022 CAMT.053 XML Statements.
pub struct Camt053XmlParser;

impl BankParser for Camt053XmlParser {
    fn sniff(&self, raw: &[u8], filename: &str) -> bool {
        let name_lower = filename.to_lowercase();
        if !name_lower.ends_with(".xml") && !name_lower.ends_with(".camt053") {
            return false;
        }
        let head = String::from_utf8_lossy(&raw[..raw.len().min(1024)]).to_lowercase();
        head.contains("camt.053") || head.contains("<bktocstmrstmt>") || head.contains("<stmt>")
    }

    fn parse(&self, raw: &[u8], filename: &str) -> Result<CanonicalStatement, ParseError> {
        let engine = StatementEngine::new()?;
        if let Some(xml_template) = engine.registry().find_by_bank_code("ISO20022") {
            engine.parse_with_template(raw, filename, xml_template)
        } else {
            engine.parse(raw, filename)
        }
    }
}
