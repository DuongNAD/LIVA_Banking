//! Error types for bank statement parsing and template evaluation.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("Unsupported bank statement format for file: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to parse row {row}: {reason}")]
    RowParseError { row: usize, reason: String },

    #[error("XML parsing error: {0}")]
    XmlError(String),

    #[error("UTF-8 decoding error: {0}")]
    EncodingError(String),

    #[error("Mathematical invariant failure in statement balance: {0}")]
    InvariantError(String),

    #[error("Template syntax or configuration error: {0}")]
    TemplateError(String),

    #[error("Template schema validation error: {0}")]
    ValidationError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Excel workbook parsing error: {0}")]
    ExcelError(String),

    #[error("CSV parsing error: {0}")]
    CsvError(String),

    #[error("Failed to parse banking date '{0}'")]
    DateParseError(String),

    #[error("Failed to parse monetary amount '{0}': {1}")]
    AmountParseError(String, String),

    #[error("Integer arithmetic overflow during monetary scaling")]
    Overflow,

    #[error("SWIFT MT940 parsing error: {0}")]
    SwiftError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}
