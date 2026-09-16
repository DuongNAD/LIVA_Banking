//! LIVA Banking — `liva-normalize` Crate
//!
//! Deterministic statement normalization engine:
//! - Temporal normalization (ISO 8601 strings and Unix epoch seconds in UTC+7 ICT).
//! - Zero-float scaled integer monetary parsing (minor units u64, VND cents/dong).
//! - Deterministic uppercase unaccented Vietnamese partner name folding.
//! - Bank reference codes (FT, NPS, VN) and invoice token (HD, INV) extraction.
//! - Bank statement balance invariant verification.
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod invariant;
pub mod models;
pub mod monetary;
pub mod partner;
pub mod reference;
pub mod temporal;

pub use invariant::{verify_statement_balance, InvariantCheckResult};
pub use models::{
    BankIdentifier, NormalizationError, NormalizedAmount, NormalizedDate, NormalizedStatement,
    NormalizedTransaction,
};
pub use monetary::{parse_monetary_amount, ParsedAmount};
pub use partner::{normalize_partner_name, strip_vietnamese_diacritics};
pub use reference::extract_reference_codes;
pub use temporal::{normalize_datetime, ICT_OFFSET_SECONDS};
