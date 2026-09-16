//! - Tier 1: `compute_statement_fingerprint` for statement file/period idempotency.
//! - Tier 2: `compute_txn_hash` for row-level idempotency and collision resistance.
//! - `DeduplicationEngine`: Stateful tracker preventing duplicate imports and overlapping periods.
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::models::RawTransactionRecord;
use chrono::NaiveDate;
use hex::encode as hex_encode;
use liva_normalize::{normalize_datetime, NormalizedTransaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Computes Tier 1 statement-level cryptographic fingerprint.
///
/// Combines the SHA-256 digest of raw file bytes with canonical account number,
/// ISO date period, and opening/closing balances:
/// ```text
/// raw_file_hash = SHA-256(raw_bytes)
/// canonical_string = account_no.trim() | start_date_iso | end_date_iso | opening_cents | closing_cents | raw_file_hash
/// statement_fingerprint = hex(SHA-256(canonical_string))
/// ```
pub fn compute_statement_fingerprint(
    raw_bytes: &[u8],
    account_no: &str,
    start_date_iso: &str,
    end_date_iso: &str,
    opening_cents: i64,
    closing_cents: i64,
) -> String {
    let raw_file_hash = hex_encode(Sha256::digest(raw_bytes));
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}",
        account_no.trim(),
        start_date_iso.trim(),
        end_date_iso.trim(),
        opening_cents,
        closing_cents,
        raw_file_hash
    );
    hex_encode(Sha256::digest(canonical.as_bytes()))
}

/// Computes Tier 2 canonical transaction-level hash.
///
/// Canonical format:
/// `account_no|date_iso|amount_cents|CR/DR|voucher_no|balance_cents|narration`
/// Returns a 64-character lowercase hex SHA-256 digest.
pub fn compute_txn_hash(
    account_no: &str,
    date_iso: &str,
    amount_cents: i64,
    is_credit: bool,
    voucher_no: Option<&str>,
    balance_cents: i64,
    narration: &str,
) -> String {
    let cr_dr = if is_credit { "CR" } else { "DR" };
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        account_no.trim(),
        date_iso.trim(),
        amount_cents,
        cr_dr,
        voucher_no.unwrap_or("").trim(),
        balance_cents,
        narration.trim()
    );
    hex_encode(Sha256::digest(canonical.as_bytes()))
}

/// Computes transaction hash directly from a RawTransactionRecord.
pub fn compute_txn_hash_from_record(account_no: &str, record: &RawTransactionRecord) -> String {
    let date_iso = normalize_datetime(&record.date_str)
        .map(|d| d.iso_date)
        .unwrap_or_else(|_| record.date_str.trim().to_string());

    let amount_cents = record.amount_cents.unwrap_or(0) as i64;
    let balance_cents = record.balance_cents.unwrap_or(0) as i64;

    compute_txn_hash(
        account_no,
        &date_iso,
        amount_cents,
        record.is_credit,
        record.doc_ref.as_deref(),
        balance_cents,
        &record.narration,
    )
}

/// Computes transaction hash from a NormalizedTransaction.
pub fn compute_txn_hash_from_normalized(account_no: &str, tx: &NormalizedTransaction) -> String {
    compute_txn_hash(
        account_no,
        &tx.date,
        tx.amount_cents as i64,
        tx.is_credit,
        tx.voucher_no.as_deref(),
        tx.balance_cents as i64,
        &tx.narration,
    )
}

/// Finalized statement period descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementPeriod {
    pub start_date: String,
    pub end_date: String,
    pub fingerprint: String,
}

/// Deduplication error variants for statements and periods.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeduplicationError {
    #[error("Duplicate statement detected: fingerprint {fingerprint}")]
    DuplicateStatement { fingerprint: String },

    #[error("Overlapping statement period for account {account_no}: requested {requested_period:?} conflicts with existing {conflicting_period:?}")]
    OverlappingPeriod {
        account_no: String,
        requested_period: (String, String),
        conflicting_period: (String, String),
    },

    #[error("Invalid period: start date {start_date} is after end date {end_date}")]
    InvalidPeriod {
        start_date: String,
        end_date: String,
    },
}

/// Report produced during transaction deduplication.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeduplicationReport {
    pub total_input_records: usize,
    pub unique_records_count: usize,
    pub duplicate_records_count: usize,
    pub unique_txn_hashes: Vec<String>,
    pub duplicate_txn_hashes: Vec<String>,
}

impl DeduplicationReport {
    pub fn has_duplicates(&self) -> bool {
        self.duplicate_records_count > 0
    }
}

/// Stateful deduplication engine tracking statement fingerprints, finalized periods,
/// and row-level transaction hashes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeduplicationEngine {
    pub registered_fingerprints: HashSet<String>,
    pub finalized_periods: HashMap<String, Vec<StatementPeriod>>,
    pub seen_txn_hashes: HashSet<String>,
}

impl DeduplicationEngine {
    /// Creates a new empty deduplication engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if statement fingerprint already exists or if period overlaps with
    /// an existing finalized period for the same account.
    /// If valid, registers the fingerprint and period.
    pub fn check_and_register_statement(
        &mut self,
        fingerprint: &str,
        account_no: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<(), DeduplicationError> {
        let req_start = start_date.trim();
        let req_end = end_date.trim();

        // 1. Check exact duplicate fingerprint
        if self.registered_fingerprints.contains(fingerprint) {
            return Err(DeduplicationError::DuplicateStatement {
                fingerprint: fingerprint.to_string(),
            });
        }

        // 2. Validate period validity (start <= end)
        if !is_period_valid(req_start, req_end) {
            return Err(DeduplicationError::InvalidPeriod {
                start_date: req_start.to_string(),
                end_date: req_end.to_string(),
            });
        }

        // 3. Check overlapping period for same account
        let acc = account_no.trim();
        if let Some(periods) = self.finalized_periods.get(acc) {
            for existing in periods {
                if dates_overlap(req_start, req_end, &existing.start_date, &existing.end_date) {
                    return Err(DeduplicationError::OverlappingPeriod {
                        account_no: acc.to_string(),
                        requested_period: (req_start.to_string(), req_end.to_string()),
                        conflicting_period: (
                            existing.start_date.clone(),
                            existing.end_date.clone(),
                        ),
                    });
                }
            }
        }

        // 4. Register
        self.registered_fingerprints.insert(fingerprint.to_string());
        self.finalized_periods
            .entry(acc.to_string())
            .or_default()
            .push(StatementPeriod {
                start_date: req_start.to_string(),
                end_date: req_end.to_string(),
                fingerprint: fingerprint.to_string(),
            });

        Ok(())
    }

    /// Deduplicates transaction rows, recording unique hashes into state and returning
    /// a detailed DeduplicationReport.
    pub fn filter_or_check_transactions(
        &mut self,
        account_no: &str,
        txns: &[RawTransactionRecord],
    ) -> DeduplicationReport {
        let (_, report) = self.filter_transactions(account_no, txns);
        report
    }

    /// Filters transaction rows returning only unique records and the deduplication report.
    pub fn filter_transactions(
        &mut self,
        account_no: &str,
        txns: &[RawTransactionRecord],
    ) -> (Vec<RawTransactionRecord>, DeduplicationReport) {
        let mut unique_txns = Vec::with_capacity(txns.len());
        let mut unique_hashes = Vec::with_capacity(txns.len());
        let mut duplicate_hashes = Vec::new();

        for tx in txns {
            let hash = compute_txn_hash_from_record(account_no, tx);
            if self.seen_txn_hashes.contains(&hash) {
                duplicate_hashes.push(hash);
            } else {
                self.seen_txn_hashes.insert(hash.clone());
                unique_hashes.push(hash);
                unique_txns.push(tx.clone());
            }
        }

        let report = DeduplicationReport {
            total_input_records: txns.len(),
            unique_records_count: unique_txns.len(),
            duplicate_records_count: duplicate_hashes.len(),
            unique_txn_hashes: unique_hashes,
            duplicate_txn_hashes: duplicate_hashes,
        };

        (unique_txns, report)
    }

    /// Returns true if the statement fingerprint has already been registered.
    pub fn is_fingerprint_registered(&self, fingerprint: &str) -> bool {
        self.registered_fingerprints.contains(fingerprint)
    }

    /// Returns true if a transaction hash has already been seen.
    pub fn is_txn_seen(&self, txn_hash: &str) -> bool {
        self.seen_txn_hashes.contains(txn_hash)
    }

    /// Returns the finalized periods for a given account.
    pub fn get_account_periods(&self, account_no: &str) -> Option<&[StatementPeriod]> {
        self.finalized_periods.get(account_no.trim()).map(|v| v.as_slice())
    }
}

/// Validates that start_date <= end_date.
fn is_period_valid(start: &str, end: &str) -> bool {
    if let (Ok(s), Ok(e)) = (
        NaiveDate::parse_from_str(start, "%Y-%m-%d"),
        NaiveDate::parse_from_str(end, "%Y-%m-%d"),
    ) {
        s <= e
    } else {
        start <= end
    }
}

/// Checks whether two closed date intervals [s1, e1] and [s2, e2] overlap.
/// Overlap condition: max(s1, s2) <= min(e1, e2) <=> s1 <= e2 && s2 <= e1.
fn dates_overlap(s1: &str, e1: &str, s2: &str, e2: &str) -> bool {
    if let (Ok(d_s1), Ok(d_e1), Ok(d_s2), Ok(d_e2)) = (
        NaiveDate::parse_from_str(s1, "%Y-%m-%d"),
        NaiveDate::parse_from_str(e1, "%Y-%m-%d"),
        NaiveDate::parse_from_str(s2, "%Y-%m-%d"),
        NaiveDate::parse_from_str(e2, "%Y-%m-%d"),
    ) {
        d_s1 <= d_e2 && d_s2 <= d_e1
    } else {
        s1 <= e2 && s2 <= e1
    }
}
