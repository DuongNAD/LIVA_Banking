//! Dynamic Bank Profile definitions and column mapping resolution.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::models::ContainerFormat;
use liva_normalize::BankIdentifier;
use serde::{Deserialize, Serialize};

/// Dynamic bank statement profile specifying table geometry and column mappings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BankProfile {
    pub id: String,
    pub profile_name: String,
    pub bank_code: String,
    pub format: ContainerFormat,
    pub csv_delimiter: Option<char>,
    pub header_row_index: usize,
    pub data_start_row_index: usize,
    pub footer_skip_rows: usize,
    pub columns: ColumnMappingConfig,
    pub date_format: Option<String>,
    pub decimal_separator: Option<char>,
    pub thousands_separator: Option<char>,
}

impl BankProfile {
    /// Maps profile bank code to standardized BankIdentifier.
    pub fn bank_identifier(&self) -> BankIdentifier {
        match self.bank_code.to_uppercase().trim() {
            "VCB" | "VIETCOMBANK" => BankIdentifier::Vietcombank,
            "TCB" | "TECHCOMBANK" => BankIdentifier::Techcombank,
            "BIDV" => BankIdentifier::Bidv,
            "CTG" | "VIETINBANK" => BankIdentifier::VietinBank,
            "MBB" | "MBBANK" => BankIdentifier::MbBank,
            "VBA" | "AGRIBANK" => BankIdentifier::Agribank,
            "ISO20022" | "CAMT053" => BankIdentifier::Iso20022,
            "SWIFT" | "MT940" => BankIdentifier::Swift,
            _ => BankIdentifier::Unknown,
        }
    }

    /// Resolves column selectors against actual header row string values.
    pub fn resolve_columns(&self, headers: &[String]) -> ResolvedColumnIndices {
        self.columns.resolve(headers)
    }
}

/// Dynamic column mapping configuration for bank statements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnMappingConfig {
    pub date: ColumnSelector,
    pub val_date: Option<ColumnSelector>,
    pub doc_ref: Option<ColumnSelector>,
    pub debit: Option<ColumnSelector>,
    pub credit: Option<ColumnSelector>,
    pub amount: Option<ColumnSelector>,
    pub is_credit_flag: Option<ColumnSelector>,
    pub balance: Option<ColumnSelector>,
    pub narration: ColumnSelector,
    pub counterparty: Option<ColumnSelector>,
}

impl ColumnMappingConfig {
    /// Resolves all column selectors into concrete column indices given a header row.
    pub fn resolve(&self, headers: &[String]) -> ResolvedColumnIndices {
        ResolvedColumnIndices {
            date: self.date.resolve(headers),
            val_date: self.val_date.as_ref().and_then(|s| s.resolve(headers)),
            doc_ref: self.doc_ref.as_ref().and_then(|s| s.resolve(headers)),
            debit: self.debit.as_ref().and_then(|s| s.resolve(headers)),
            credit: self.credit.as_ref().and_then(|s| s.resolve(headers)),
            amount: self.amount.as_ref().and_then(|s| s.resolve(headers)),
            is_credit_flag: self.is_credit_flag.as_ref().and_then(|s| s.resolve(headers)),
            balance: self.balance.as_ref().and_then(|s| s.resolve(headers)),
            narration: self.narration.resolve(headers),
            counterparty: self.counterparty.as_ref().and_then(|s| s.resolve(headers)),
        }
    }
}

/// Selector for a column: either a 0-based column index or header column name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColumnSelector {
    Index(usize),
    Name(String),
}

impl ColumnSelector {
    /// Resolves column index from a list of header names.
    /// If self is Index(idx), returns Some(idx).
    /// If self is Name(target), performs case-insensitive trimmed matching:
    /// 1. Exact equality (case-insensitive, trimmed)
    /// 2. Substring match (case-insensitive)
    pub fn resolve(&self, headers: &[String]) -> Option<usize> {
        match self {
            ColumnSelector::Index(idx) => Some(*idx),
            ColumnSelector::Name(target_name) => {
                let target = target_name.trim();
                if target.is_empty() {
                    return None;
                }

                // 1. Exact match (case-insensitive)
                for (idx, header) in headers.iter().enumerate() {
                    if header.trim().eq_ignore_ascii_case(target) {
                        return Some(idx);
                    }
                }

                // 2. Substring match (case-insensitive)
                let target_lower = target.to_lowercase();
                for (idx, header) in headers.iter().enumerate() {
                    let header_lower = header.trim().to_lowercase();
                    if header_lower.contains(&target_lower) {
                        return Some(idx);
                    }
                }

                None
            }
        }
    }
}

/// Concrete resolved 0-based column indices after header inspection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedColumnIndices {
    pub date: Option<usize>,
    pub val_date: Option<usize>,
    pub doc_ref: Option<usize>,
    pub debit: Option<usize>,
    pub credit: Option<usize>,
    pub amount: Option<usize>,
    pub is_credit_flag: Option<usize>,
    pub balance: Option<usize>,
    pub narration: Option<usize>,
    pub counterparty: Option<usize>,
}
