//! Data models and core traits for statement ingestion.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use liva_normalize::{
    extract_reference_codes, normalize_datetime, normalize_partner_name, parse_monetary_amount,
    BankIdentifier, NormalizedStatement, NormalizedTransaction,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Physical file container format detected via magic bytes and heuristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContainerFormat {
    /// Adobe Portable Document Format (%PDF-)
    Pdf,
    /// Office Open XML Workbook / ZIP container (.xlsx)
    ExcelZip,
    /// Microsoft Compound Document File / OLE container (.xls)
    ExcelOle,
    /// HTML table disguised as spreadsheet (common in Agribank exports)
    HtmlTable,
    /// Delimited plaintext (CSV, TSV, semicolon-delimited)
    TextCsv,
    /// ISO 20022 XML / Generic Financial XML
    Xml,
    /// SWIFT MT940 statement message
    SwiftMt,
    /// Unrecognized container
    Unknown,
}

impl ContainerFormat {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ContainerFormat::Pdf => "PDF",
            ContainerFormat::ExcelZip => "EXCEL_ZIP",
            ContainerFormat::ExcelOle => "EXCEL_OLE",
            ContainerFormat::HtmlTable => "HTML_TABLE",
            ContainerFormat::TextCsv => "TEXT_CSV",
            ContainerFormat::Xml => "XML",
            ContainerFormat::SwiftMt => "SWIFT_MT",
            ContainerFormat::Unknown => "UNKNOWN",
        }
    }
}

impl fmt::Display for ContainerFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Raw individual transaction extracted directly from container prior to full normalization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawTransactionRecord {
    pub row_id: usize,
    pub date_str: String,
    pub val_date_str: Option<String>,
    pub doc_ref: Option<String>,
    pub debit_amt_str: Option<String>,
    pub credit_amt_str: Option<String>,
    pub amount_cents: Option<u64>,
    pub is_credit: bool,
    pub balance_str: Option<String>,
    pub balance_cents: Option<u64>,
    pub narration: String,
    pub counterparty_name: Option<String>,
}

/// Raw parsed statement containing metadata and extracted raw rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawStatementRecord {
    pub bank: BankIdentifier,
    pub format: ContainerFormat,
    pub account_no: Option<String>,
    pub account_name: Option<String>,
    pub opening_balance: Option<u64>,
    pub closing_balance: Option<u64>,
    pub transactions: Vec<RawTransactionRecord>,
}

impl RawStatementRecord {
    /// Normalizes this raw record into a fully validated `NormalizedStatement`.
    pub fn normalize(&self) -> Result<NormalizedStatement, IngestError> {
        let account_no = self
            .account_no
            .clone()
            .unwrap_or_else(|| "UNKNOWN_ACCOUNT".to_string());

        let opening_cents = self.opening_balance.unwrap_or(0);
        let mut normalized_txs = Vec::with_capacity(self.transactions.len());

        let mut running_balance = opening_cents;

        for (idx, raw) in self.transactions.iter().enumerate() {
            // 1. Normalize transaction date
            let norm_date = normalize_datetime(&raw.date_str)?;

            // 2. Normalize value/booking date if present
            let (booking_date, val_timestamp) = match &raw.val_date_str {
                Some(vd) if !vd.trim().is_empty() => {
                    if let Ok(v_norm) = normalize_datetime(vd) {
                        (Some(v_norm.iso_date), Some(v_norm.epoch_seconds))
                    } else {
                        (None, None)
                    }
                }
                _ => (None, None),
            };

            // 3. Resolve amount
            let amount_cents = if let Some(amt) = raw.amount_cents {
                amt
            } else if raw.is_credit {
                if let Some(ref c_str) = raw.credit_amt_str {
                    parse_monetary_amount(c_str)?.minor_units
                } else {
                    0
                }
            } else if let Some(ref d_str) = raw.debit_amt_str {
                parse_monetary_amount(d_str)?.minor_units
            } else {
                0
            };

            // Update running balance calculation
            if raw.is_credit {
                running_balance = running_balance.saturating_add(amount_cents);
            } else {
                running_balance = running_balance.saturating_sub(amount_cents);
            }

            // If statement provided an explicit running balance, normalize it
            let balance_cents = if let Some(b) = raw.balance_cents {
                b
            } else if let Some(ref b_str) = raw.balance_str {
                parse_monetary_amount(b_str)
                    .map(|p| p.minor_units)
                    .unwrap_or(running_balance)
            } else {
                running_balance
            };

            // 4. Clean counterparty name
            let counterparty_name = raw
                .counterparty_name
                .as_deref()
                .map(normalize_partner_name)
                .filter(|s| !s.is_empty());

            // 5. Extract reference codes
            let reference_codes =
                extract_reference_codes(&raw.narration, raw.doc_ref.as_deref());

            let tx_id = raw
                .doc_ref
                .clone()
                .filter(|r| !r.is_empty() && r != "NONREF")
                .unwrap_or_else(|| format!("tx-{}", idx + 1));

            normalized_txs.push(NormalizedTransaction {
                id: tx_id,
                date: norm_date.iso_date,
                booking_date,
                voucher_no: raw.doc_ref.clone(),
                amount_cents,
                is_credit: raw.is_credit,
                balance_cents,
                narration: raw.narration.clone(),
                counterparty_name,
                reference_codes,
                tx_timestamp: norm_date.epoch_seconds,
                value_timestamp: val_timestamp,
            });
        }

        let closing_cents = self.closing_balance.unwrap_or(running_balance);

        Ok(NormalizedStatement {
            bank: self.bank,
            account_no,
            opening_cents,
            closing_cents,
            transactions: normalized_txs,
        })
    }
}

/// Core trait for autonomous bank statement sniffers.
pub trait StatementSniffer {
    fn sniff_container(&self, bytes: &[u8], filename: &str) -> ContainerFormat;
    fn sniff_bank(&self, bytes: &[u8], filename: &str) -> BankIdentifier;
}

/// Core trait implemented by specialized bank statement parsers.
pub trait StatementParser: Send + Sync {
    fn can_parse(&self, container: ContainerFormat, bank: BankIdentifier) -> bool;
    fn parse(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError>;
}
