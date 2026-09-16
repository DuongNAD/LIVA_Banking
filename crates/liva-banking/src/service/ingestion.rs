//! Ingestion Pipeline Service with SHA-256 Deduplication and Balance Quarantine.

use crate::models::{BankStatement, StatementStatus};
use crate::parser::{ParserError, compute_sha256, sniff_and_parse};
use std::collections::HashSet;

pub struct IngestionService {
    seen_file_hashes: HashSet<String>,
}

impl IngestionService {
    pub fn new() -> Self {
        Self {
            seen_file_hashes: HashSet::new(),
        }
    }

    /// Ingests a raw statement file, enforces SHA-256 deduplication, parses content,
    /// and checks balance invariants before persisting.
    pub fn ingest_statement(
        &mut self,
        bytes: &[u8],
        filename: &str,
    ) -> Result<BankStatement, ParserError> {
        let hash = compute_sha256(bytes);

        // Deduplication Check (Feature F05: Returns 409 Conflict on replay)
        if self.seen_file_hashes.contains(&hash) {
            return Err(ParserError::DuplicateStatementFile { hash });
        }

        // Parse statement content
        let mut statement = sniff_and_parse(bytes, filename)?;

        // Balance Invariant Check (Feature F06)
        let is_balanced = statement.verify_balance_invariant();
        if !is_balanced {
            statement.status = StatementStatus::QuarantinedUnbalanced;
        } else {
            statement.status = StatementStatus::VerifiedBalanced;
        }

        self.seen_file_hashes.insert(hash);
        Ok(statement)
    }
}
