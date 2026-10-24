//! Date parsing and timestamp normalization using chrono.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::ParseError;
use chrono::{NaiveDate, NaiveDateTime};

/// Parses a date string into an epoch timestamp (seconds since 1970-01-01 00:00:00 UTC)
/// trying the candidate formats in sequence.
pub fn parse_banking_date(raw: &str, formats: &[String]) -> Result<i64, ParseError> {
    let clean = raw.trim();
    if clean.is_empty() {
        return Err(ParseError::DateParseError("Empty date string".to_string()));
    }

    // Try user-configured formats
    for fmt in formats {
        // Try as DateTime first
        if let Ok(dt) = NaiveDateTime::parse_from_str(clean, fmt) {
            return Ok(dt.and_utc().timestamp());
        }
        // Try as Date only
        if let Ok(d) = NaiveDate::parse_from_str(clean, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return Ok(dt.and_utc().timestamp());
            }
        }
    }

    // Fallback standard formats
    let fallbacks = [
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
        "%d/%m/%Y",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
        "%d-%m-%Y %H:%M:%S",
        "%d-%m-%Y",
        "%y%m%d",
        "%Y%m%d",
    ];

    for fmt in fallbacks {
        if let Ok(dt) = NaiveDateTime::parse_from_str(clean, fmt) {
            return Ok(dt.and_utc().timestamp());
        }
        if let Ok(d) = NaiveDate::parse_from_str(clean, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return Ok(dt.and_utc().timestamp());
            }
        }
    }

    Err(ParseError::DateParseError(raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dates() {
        let formats = vec![
            "%d/%m/%Y %H:%M:%S".to_string(),
            "%d/%m/%Y".to_string(),
            "%Y-%m-%d".to_string(),
        ];

        let ts1 = parse_banking_date("01/08/2026 08:30:00", &formats).unwrap();
        assert!(ts1 > 1750000000);

        let ts2 = parse_banking_date("01/08/2026", &formats).unwrap();
        assert!(ts2 > 1750000000);

        let ts3 = parse_banking_date("2026-08-01", &formats).unwrap();
        assert_eq!(ts2, ts3);
    }
}
