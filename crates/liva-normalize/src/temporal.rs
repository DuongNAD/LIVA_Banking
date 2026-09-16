//! Temporal normalization for banking statements.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::models::{NormalizationError, NormalizedDate};
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, TimeZone};

/// Indochina Time offset (UTC+7, Vietnam Standard Time).
pub const ICT_OFFSET_SECONDS: i32 = 7 * 3600;

/// Parses a raw date or datetime string into a canonical `NormalizedDate`.
///
/// Supported input formats:
/// 1. `DD/MM/YYYY HH:mm:ss` or `DD-MM-YYYY HH:mm:ss` (e.g. "01/08/2026 08:00:00")
/// 2. `DD/MM/YYYY` or `DD-MM-YYYY` (e.g. "01/08/2026")
/// 3. `YYYY-MM-DDTHH:mm:ssZ` or `YYYY-MM-DDTHH:mm:ss+07:00`
/// 4. `YYYY-MM-DD` (e.g. "2026-08-01")
/// 5. `YYMMDD` (SWIFT MT940 date format, e.g. "260801")
pub fn normalize_datetime(input: &str) -> Result<NormalizedDate, NormalizationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(NormalizationError::InvalidDateFormat(
            "Empty date string".to_string(),
        ));
    }

    let ict = FixedOffset::east_opt(ICT_OFFSET_SECONDS).ok_or_else(|| {
        NormalizationError::InvalidDateFormat("Failed to create ICT offset".to_string())
    })?;

    // 1. Check SWIFT MT940 format: 6 digits (YYMMDD)
    if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let yy: i32 = trimmed[0..2]
            .parse()
            .map_err(|_| NormalizationError::InvalidDateFormat(trimmed.to_string()))?;
        let mm: u32 = trimmed[2..4]
            .parse()
            .map_err(|_| NormalizationError::InvalidDateFormat(trimmed.to_string()))?;
        let dd: u32 = trimmed[4..6]
            .parse()
            .map_err(|_| NormalizationError::InvalidDateFormat(trimmed.to_string()))?;

        // Determine 4-digit century: if yy >= 70 assume 1900s, else 2000s
        let yyyy = if yy >= 70 { 1900 + yy } else { 2000 + yy };
        let naive_date = NaiveDate::from_ymd_opt(yyyy, mm, dd).ok_or_else(|| {
            NormalizationError::InvalidDateFormat(format!("Invalid YYMMDD date: {trimmed}"))
        })?;
        let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
        let dt = ict.from_local_datetime(&naive_dt).single().ok_or_else(|| {
            NormalizationError::InvalidDateFormat(format!("Ambiguous local date: {trimmed}"))
        })?;

        return Ok(NormalizedDate {
            iso_date: format!("{yyyy:04}-{mm:02}-{dd:02}"),
            epoch_seconds: dt.timestamp(),
        });
    }

    // 2. ISO 8601 full datetime with timezone: YYYY-MM-DDTHH:mm:ssZ or +07:00
    if trimmed.contains('T') {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
            return Ok(NormalizedDate {
                iso_date: dt.to_rfc3339(),
                epoch_seconds: dt.timestamp(),
            });
        }
        // Fallback for YYYY-MM-DDTHH:mm:ss without timezone
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
            let dt = ict.from_local_datetime(&naive_dt).single().ok_or_else(|| {
                NormalizationError::InvalidDateFormat(format!("Ambiguous local date: {trimmed}"))
            })?;
            return Ok(NormalizedDate {
                iso_date: format!("{}+07:00", naive_dt.format("%Y-%m-%dT%H:%M:%S")),
                epoch_seconds: dt.timestamp(),
            });
        }
    }

    // 3. DD/MM/YYYY HH:mm:ss or DD-MM-YYYY HH:mm:ss
    if trimmed.contains(':') {
        let fmt = if trimmed.contains('/') {
            "%d/%m/%Y %H:%M:%S"
        } else if trimmed.contains('-') {
            "%d-%m-%Y %H:%M:%S"
        } else {
            "%d/%m/%Y %H:%M:%S"
        };
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            let dt = ict.from_local_datetime(&naive_dt).single().ok_or_else(|| {
                NormalizationError::InvalidDateFormat(format!("Ambiguous local date: {trimmed}"))
            })?;
            return Ok(NormalizedDate {
                iso_date: format!("{}+07:00", naive_dt.format("%Y-%m-%dT%H:%M:%S")),
                epoch_seconds: dt.timestamp(),
            });
        }
    }

    // 4. ISO Date: YYYY-MM-DD
    if trimmed.len() == 10 && trimmed.chars().nth(4) == Some('-') {
        if let Ok(naive_date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
            let dt = ict.from_local_datetime(&naive_dt).single().ok_or_else(|| {
                NormalizationError::InvalidDateFormat(format!("Ambiguous local date: {trimmed}"))
            })?;
            return Ok(NormalizedDate {
                iso_date: trimmed.to_string(),
                epoch_seconds: dt.timestamp(),
            });
        }
    }

    // 5. Vietnamese Date: DD/MM/YYYY or DD-MM-YYYY
    let cleaned = trimmed.replace('-', "/");
    let parts: Vec<&str> = cleaned.split('/').collect();
    if parts.len() == 3 {
        let d_res = parts[0].trim().parse::<u32>();
        let m_res = parts[1].trim().parse::<u32>();
        let y_res = parts[2].trim().parse::<i32>();

        if let (Ok(d), Ok(m), Ok(y)) = (d_res, m_res, y_res) {
            let full_year = if y < 100 { 2000 + y } else { y };
            if let Some(naive_date) = NaiveDate::from_ymd_opt(full_year, m, d) {
                let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
                let dt = ict.from_local_datetime(&naive_dt).single().ok_or_else(|| {
                    NormalizationError::InvalidDateFormat(format!(
                        "Ambiguous local date: {trimmed}"
                    ))
                })?;
                return Ok(NormalizedDate {
                    iso_date: format!("{full_year:04}-{m:02}-{d:02}"),
                    epoch_seconds: dt.timestamp(),
                });
            }
        }
    }

    Err(NormalizationError::InvalidDateFormat(trimmed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_dd_mm_yyyy() {
        let res = normalize_datetime("01/08/2026").unwrap();
        assert_eq!(res.iso_date, "2026-08-01");
        // 2026-08-01 00:00:00 ICT = 2026-07-31 17:00:00 UTC
        assert_eq!(res.epoch_seconds, 1785517200);
    }

    #[test]
    fn test_normalize_dd_mm_yyyy_hh_mm_ss() {
        let res = normalize_datetime("01/08/2026 08:00:00").unwrap();
        assert_eq!(res.iso_date, "2026-08-01T08:00:00+07:00");
        assert_eq!(res.epoch_seconds, 1785517200 + 8 * 3600);
    }

    #[test]
    fn test_normalize_yyyy_mm_dd() {
        let res = normalize_datetime("2026-08-31").unwrap();
        assert_eq!(res.iso_date, "2026-08-31");
    }

    #[test]
    fn test_normalize_swift_yymmdd() {
        let res = normalize_datetime("260801").unwrap();
        assert_eq!(res.iso_date, "2026-08-01");
        assert_eq!(res.epoch_seconds, 1785517200);
    }
}
