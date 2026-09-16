//! Error types for statement ingestion and parsing.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_normalize::NormalizationError;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum IngestError {
    #[error("Unsupported container format or unrecognized bank for file: {filename}")]
    UnsupportedFormat { filename: String },

    #[error("Excel parsing error: {0}")]
    Excel(String),

    #[error("CSV parsing error: {0}")]
    Csv(String),

    #[error("PDF parsing error: {0}")]
    Pdf(String),

    #[error("XML parsing error: {0}")]
    Xml(String),

    #[error("SWIFT MT940 parsing error: {0}")]
    Swift(String),

    #[error("HTML parsing error: {0}")]
    Html(String),

    #[error("Invalid statement structure: {0}")]
    InvalidStructure(String),

    #[error("Normalization error: {0}")]
    Normalization(#[from] NormalizationError),

    #[error("I/O error: {0}")]
    Io(String),
}
