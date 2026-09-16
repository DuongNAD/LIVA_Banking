//! LIVA Banking — `liva-ingest` Crate
//!
//! Autonomous format sniffing and multi-bank statement ingestion:
//! - Container detection (PDF, XLSX, CSV, CAMT.053 XML, SWIFT MT940).
//! - 6 Vietnamese banks: Vietcombank, Techcombank, BIDV, VietinBank, MBBank, Agribank.
//! - Direct normalization into `NormalizedStatement` without floating-point arithmetic.
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod dedup;
pub mod error;
pub mod models;
pub mod parsers;
pub mod profile;
pub mod sniffer;

pub use dedup::{
    compute_statement_fingerprint, compute_txn_hash, compute_txn_hash_from_normalized,
    compute_txn_hash_from_record, DeduplicationEngine, DeduplicationError, DeduplicationReport,
    StatementPeriod,
};
pub use error::IngestError;
pub use models::{
    ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser, StatementSniffer,
};
pub use parsers::{
    default_parsers, BidvParser, Camt053Parser, Mt940Parser, MultiBankTableParser,
    ProfileStatementParser, TcbParser, VcbParser,
};
pub use profile::{
    BankProfile, ColumnMappingConfig, ColumnSelector, ResolvedColumnIndices,
};
pub use sniffer::DefaultSniffer;

use liva_normalize::{BankIdentifier, NormalizedStatement};

/// Main statement ingestion engine orchestrating sniffing, parsing, and normalization.
pub struct IngestEngine {
    sniffer: Box<dyn StatementSniffer + Send + Sync>,
    parsers: Vec<Box<dyn StatementParser>>,
}

impl Default for IngestEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl IngestEngine {
    /// Creates a new ingestion engine with default sniffing and standard parsers.
    pub fn new() -> Self {
        Self {
            sniffer: Box::new(DefaultSniffer),
            parsers: default_parsers(),
        }
    }

    /// Registers a custom parser at the beginning of the parser list.
    pub fn register_parser(&mut self, parser: Box<dyn StatementParser>) {
        self.parsers.insert(0, parser);
    }

    /// Sniffs the physical container format and bank identity from file bytes and filename.
    pub fn sniff(&self, bytes: &[u8], filename: &str) -> (ContainerFormat, BankIdentifier) {
        let container = self.sniffer.sniff_container(bytes, filename);
        let bank = self.sniffer.sniff_bank(bytes, filename);
        (container, bank)
    }

    /// Parses raw statement record from bytes without full normalization.
    pub fn ingest_raw(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        let (container, bank) = self.sniff(bytes, filename);

        for parser in &self.parsers {
            if parser.can_parse(container, bank) {
                match parser.parse(bytes, filename) {
                    Ok(mut record) => {
                        // Inherit sniffed metadata if parser didn't resolve it
                        if record.bank == BankIdentifier::Unknown && bank != BankIdentifier::Unknown {
                            record.bank = bank;
                        }
                        if record.format == ContainerFormat::Unknown && container != ContainerFormat::Unknown {
                            record.format = container;
                        }
                        return Ok(record);
                    }
                    Err(_) => {
                        // If specific parser fails, attempt fallback parsers
                        continue;
                    }
                }
            }
        }

        // Fallback: try all parsers regardless of sniffed format
        for parser in &self.parsers {
            if let Ok(mut record) = parser.parse(bytes, filename) {
                if record.bank == BankIdentifier::Unknown && bank != BankIdentifier::Unknown {
                    record.bank = bank;
                }
                return Ok(record);
            }
        }

        Err(IngestError::UnsupportedFormat {
            filename: filename.to_string(),
        })
    }

    /// Ingests statement file and produces a verified `NormalizedStatement`.
    pub fn ingest_normalized(
        &self,
        bytes: &[u8],
        filename: &str,
    ) -> Result<NormalizedStatement, IngestError> {
        let raw = self.ingest_raw(bytes, filename)?;
        raw.normalize()
    }
}

/// Convenience function to ingest and normalize a statement file using standard configuration.
pub fn parse_statement(bytes: &[u8], filename: &str) -> Result<NormalizedStatement, IngestError> {
    let engine = IngestEngine::new();
    engine.ingest_normalized(bytes, filename)
}
