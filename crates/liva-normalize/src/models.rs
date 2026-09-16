//! Core data models and types for LIVA Banking Normalization Engine.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_money::{Currency, Money, MoneyError};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Normalization error types.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NormalizationError {
    #[error("Failed to parse date '{0}': unsupported or invalid datetime format")]
    InvalidDateFormat(String),

    #[error("Failed to parse monetary amount '{0}': invalid numeric format")]
    InvalidMonetaryAmount(String),

    #[error("Arithmetic overflow in monetary calculation")]
    Overflow,

    #[error("Money conversion error: {0}")]
    Money(#[from] MoneyError),

    #[error("Statement invariant check violation: {0}")]
    InvariantViolation(String),
}

/// Official bank identifiers supported across Vietnamese banking and international standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BankIdentifier {
    Vietcombank,
    Techcombank,
    Bidv,
    VietinBank,
    MbBank,
    Agribank,
    Iso20022,
    Swift,
    Unknown,
}

impl BankIdentifier {
    pub const fn as_code(&self) -> &'static str {
        match self {
            BankIdentifier::Vietcombank => "VCB",
            BankIdentifier::Techcombank => "TCB",
            BankIdentifier::Bidv => "BIDV",
            BankIdentifier::VietinBank => "CTG",
            BankIdentifier::MbBank => "MB",
            BankIdentifier::Agribank => "VBA",
            BankIdentifier::Iso20022 => "ISO20022",
            BankIdentifier::Swift => "SWIFT",
            BankIdentifier::Unknown => "UNKNOWN",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.trim().to_uppercase().as_str() {
            "VCB" | "VIETCOMBANK" => BankIdentifier::Vietcombank,
            "TCB" | "TECHCOMBANK" => BankIdentifier::Techcombank,
            "BIDV" => BankIdentifier::Bidv,
            "CTG" | "VIETINBANK" | "ICB" => BankIdentifier::VietinBank,
            "MB" | "MBB" | "MBBANK" => BankIdentifier::MbBank,
            "VBA" | "AGRI" | "AGRIBANK" => BankIdentifier::Agribank,
            "ISO20022" | "CAMT" | "CAMT053" => BankIdentifier::Iso20022,
            "SWIFT" | "MT940" => BankIdentifier::Swift,
            _ => BankIdentifier::Unknown,
        }
    }
}

impl fmt::Display for BankIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_code())
    }
}

impl FromStr for BankIdentifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_code(s))
    }
}

/// Normalized canonical date representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NormalizedDate {
    /// Canonical ISO 8601 representation (e.g. "2026-08-01" or "2026-08-01T08:00:00+07:00").
    pub iso_date: String,
    /// Unix epoch seconds (ICT UTC+7 or UTC).
    pub epoch_seconds: i64,
}

/// Normalized monetary amount in minor units (cents / VND dong).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NormalizedAmount {
    /// Minor currency units (1 VND = 1 minor unit; 1 USD = 100 minor units).
    pub minor_units: u64,
    /// Associated currency.
    pub currency: Currency,
}

impl NormalizedAmount {
    pub const fn vnd(amount: u64) -> Self {
        Self {
            minor_units: amount,
            currency: Currency::VND,
        }
    }

    pub const fn new(minor_units: u64, currency: Currency) -> Self {
        Self {
            minor_units,
            currency,
        }
    }

    pub fn to_money(&self) -> Result<Money, MoneyError> {
        let signed_amt = i64::try_from(self.minor_units).map_err(|_| MoneyError::Overflow)?;
        Ok(Money::from_minor(signed_amt, self.currency))
    }
}

impl From<Money> for NormalizedAmount {
    fn from(m: Money) -> Self {
        let unsigned = if m.amount() < 0 {
            0u64
        } else {
            m.amount() as u64
        };
        Self {
            minor_units: unsigned,
            currency: m.currency(),
        }
    }
}

impl TryFrom<NormalizedAmount> for Money {
    type Error = MoneyError;

    fn try_from(na: NormalizedAmount) -> Result<Self, Self::Error> {
        na.to_money()
    }
}

/// Normalized individual transaction record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedTransaction {
    /// Unique identifier or generated transaction ID.
    pub id: String,
    /// Canonical transaction date in ISO 8601.
    pub date: String,
    /// Optional booking/value date in ISO 8601.
    pub booking_date: Option<String>,
    /// Optional voucher or receipt document number.
    pub voucher_no: Option<String>,
    /// Transaction amount in atomic minor units (VND dong / cents).
    pub amount_cents: u64,
    /// True if credit (inflow), false if debit (outflow).
    pub is_credit: bool,
    /// Running balance in minor units after transaction.
    pub balance_cents: u64,
    /// Transaction description / memo.
    pub narration: String,
    /// Folded and cleaned counterparty name (uppercase unaccented).
    pub counterparty_name: Option<String>,
    /// Tokenized references (bank transaction IDs, invoice numbers).
    pub reference_codes: Vec<String>,
    /// Normalized transaction timestamp in Unix epoch seconds.
    pub tx_timestamp: i64,
    /// Optional booking/value timestamp in Unix epoch seconds.
    pub value_timestamp: Option<i64>,
}

/// Fully normalized bank statement document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedStatement {
    /// Supported bank identifier.
    pub bank: BankIdentifier,
    /// Primary account number.
    pub account_no: String,
    /// Statement opening balance in minor units.
    pub opening_cents: u64,
    /// Statement closing balance in minor units.
    pub closing_cents: u64,
    /// Normalized chronological transactions.
    pub transactions: Vec<NormalizedTransaction>,
}
