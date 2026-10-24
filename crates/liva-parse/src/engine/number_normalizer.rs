//! Zero-float monetary amount normalizer.
//! Parses string or tabular cell values into atomic minor units (`Money`)
//! using integer checked arithmetic only.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::ParseError;
use calamine::Data;
use liva_money::{Currency, Money};

/// Parses a string representation of a monetary value into atomic minor units (`Money`).
///
/// Ensures 100% integer arithmetic without using floating-point types (`f32`/`f64`).
pub fn parse_dsl_amount(
    raw: &str,
    thousand_sep: &str,
    decimal_sep: &str,
    decimal_places: u32,
    strip_tokens: &[String],
    currency: Currency,
) -> Result<Money, ParseError> {
    let mut clean = raw.trim().to_string();
    if clean.is_empty() || clean == "-" || clean == "—" {
        return Ok(Money::zero(currency));
    }

    // Strip custom tokens (e.g. "VND", "VNĐ", "đ", "$")
    for token in strip_tokens {
        clean = clean.replace(token, "");
    }
    let clean = clean.trim();
    if clean.is_empty() {
        return Ok(Money::zero(currency));
    }

    // Detect negative indicators
    let is_negative = clean.starts_with('-')
        || (clean.starts_with('(') && clean.ends_with(')'))
        || clean.ends_with('-')
        || clean.starts_with("CR")
        || clean.ends_with("CR")
        || clean.starts_with("DR")
        || clean.ends_with("DR");

    // Remove negative markers and parentheses
    let stripped: String = clean
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == ' ')
        .collect();
    let stripped = stripped.trim();

    if stripped.is_empty() {
        return Ok(Money::zero(currency));
    }

    // Determine integer and fractional parts
    let (int_part_str, frac_part_str) = resolve_int_frac(stripped, thousand_sep, decimal_sep);

    // Parse integer part
    let int_val: i64 = if int_part_str.is_empty() {
        0
    } else {
        int_part_str.parse::<i64>().map_err(|e| {
            ParseError::AmountParseError(
                raw.to_string(),
                format!("Failed to parse integer component '{int_part_str}': {e}"),
            )
        })?
    };

    // Calculate scale factor: 10^decimal_places via checked_mul
    let mut scale_factor: i64 = 1;
    for _ in 0..decimal_places {
        scale_factor = scale_factor
            .checked_mul(10)
            .ok_or(ParseError::Overflow)?;
    }

    // Base minor units from integer part
    let mut minor_units = int_val
        .checked_mul(scale_factor)
        .ok_or(ParseError::Overflow)?;

    // Add fractional part if decimal_places > 0
    if decimal_places > 0 && !frac_part_str.is_empty() {
        let frac_padded = if frac_part_str.len() >= decimal_places as usize {
            &frac_part_str[..decimal_places as usize]
        } else {
            // Right-pad with zeros
            &format!(
                "{:0<width$}",
                frac_part_str,
                width = decimal_places as usize
            )
        };

        let frac_val: i64 = frac_padded.parse().unwrap_or(0);
        minor_units = minor_units
            .checked_add(frac_val)
            .ok_or(ParseError::Overflow)?;
    }

    if is_negative {
        minor_units = -minor_units;
    }

    Ok(Money::from_minor(minor_units, currency))
}

/// Helper to split digits into integer and fractional components based on separators.
fn resolve_int_frac<'a>(
    stripped: &'a str,
    thousand_sep: &str,
    decimal_sep: &str,
) -> (String, String) {
    if decimal_sep == "none" || decimal_sep.is_empty() {
        // No decimal separator: entire string is integer minus thousand separator
        let cleaned = if thousand_sep == "none" || thousand_sep.is_empty() {
            stripped.replace(' ', "")
        } else {
            stripped.replace(thousand_sep, "").replace(' ', "")
        };
        (cleaned, String::new())
    } else if thousand_sep == "auto" || decimal_sep == "auto" {
        // Auto-detection heuristics
        auto_detect_int_frac(stripped)
    } else if stripped.contains(decimal_sep) {
        let mut parts = stripped.split(decimal_sep);
        let int_raw = parts.next().unwrap_or("");
        let frac_raw = parts.next().unwrap_or("");

        let int_clean = if thousand_sep == "none" || thousand_sep.is_empty() {
            int_raw.replace(' ', "")
        } else {
            int_raw.replace(thousand_sep, "").replace(' ', "")
        };
        (int_clean, frac_raw.to_string())
    } else {
        // Decimal sep configured but not present in string
        let cleaned = if thousand_sep == "none" || thousand_sep.is_empty() {
            stripped.replace(' ', "")
        } else {
            stripped.replace(thousand_sep, "").replace(' ', "")
        };
        (cleaned, String::new())
    }
}

/// Heuristic auto-detection of thousand and decimal separators.
fn auto_detect_int_frac(stripped: &str) -> (String, String) {
    let has_dot = stripped.contains('.');
    let has_comma = stripped.contains(',');

    match (has_dot, has_comma) {
        (true, true) => {
            // Both present: whichever appears last is the decimal separator
            let last_dot = stripped.rfind('.').unwrap_or(0);
            let last_comma = stripped.rfind(',').unwrap_or(0);
            if last_dot > last_comma {
                // Dot is decimal, comma is thousand (US/UK style: 1,234.56)
                let mut parts = stripped.split('.');
                let int_part = parts.next().unwrap_or("").replace(',', "").replace(' ', "");
                let frac_part = parts.next().unwrap_or("").to_string();
                (int_part, frac_part)
            } else {
                // Comma is decimal, dot is thousand (VN/EU style: 1.234,56)
                let mut parts = stripped.split(',');
                let int_part = parts.next().unwrap_or("").replace('.', "").replace(' ', "");
                let frac_part = parts.next().unwrap_or("").to_string();
                (int_part, frac_part)
            }
        }
        (true, false) => {
            // Only dot present: if there are multiple dots or 3 digits after dot, likely thousands (VN: 45.000.000)
            let dots_count = stripped.matches('.').count();
            if dots_count > 1 {
                (stripped.replace('.', "").replace(' ', ""), String::new())
            } else {
                let parts: Vec<&str> = stripped.split('.').collect();
                if parts.len() == 2 && parts[1].len() == 3 {
                    // e.g. "500.000" -> thousand separator in VN
                    (stripped.replace('.', "").replace(' ', ""), String::new())
                } else if parts.len() == 2 && (parts[1].len() == 1 || parts[1].len() == 2) {
                    // e.g. "500.5" or "500.50" -> decimal
                    (parts[0].replace(' ', ""), parts[1].to_string())
                } else {
                    (stripped.replace('.', "").replace(' ', ""), String::new())
                }
            }
        }
        (false, true) => {
            // Only comma present
            let comma_count = stripped.matches(',').count();
            if comma_count > 1 {
                (stripped.replace(',', "").replace(' ', ""), String::new())
            } else {
                let parts: Vec<&str> = stripped.split(',').collect();
                if parts.len() == 2 && (parts[1].len() == 1 || parts[1].len() == 2) {
                    // e.g. "500,50" -> decimal in VN/EU
                    (parts[0].replace(' ', ""), parts[1].to_string())
                } else {
                    (stripped.replace(',', "").replace(' ', ""), String::new())
                }
            }
        }
        (false, false) => (stripped.replace(' ', ""), String::new()),
    }
}

/// Converts a Calamine Excel Data cell to string representation without float operations.
pub fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            // Calamine float cell: Display formatting only, no float arithmetic
            let s = format!("{f}");
            // If it ends with .0, strip it for clean integer presentation
            if let Some(stripped) = s.strip_suffix(".0") {
                stripped.to_string()
            } else {
                s
            }
        }
        Data::DateTime(d) => format!("{d}"),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("{e:?}"),
    }
}

/// Converts a Calamine Data cell into a parsed Money amount using DSL formatting.
pub fn cell_to_dsl_amount(
    cell: &Data,
    thousand_sep: &str,
    decimal_sep: &str,
    decimal_places: u32,
    strip_tokens: &[String],
    currency: Currency,
) -> Result<Option<Money>, ParseError> {
    match cell {
        Data::Empty => Ok(None),
        Data::Int(i) => {
            let mut minor = *i;
            for _ in 0..decimal_places {
                minor = minor.checked_mul(10).ok_or(ParseError::Overflow)?;
            }
            Ok(Some(Money::from_minor(minor, currency)))
        }
        Data::Float(_) | Data::String(_) => {
            let raw_str = cell_to_string(cell);
            if raw_str.trim().is_empty() {
                Ok(None)
            } else {
                let money = parse_dsl_amount(
                    &raw_str,
                    thousand_sep,
                    decimal_sep,
                    decimal_places,
                    strip_tokens,
                    currency,
                )?;
                Ok(Some(money))
            }
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vietnamese_dong_amounts() {
        let money = parse_dsl_amount("45.000.000", ".", ",", 0, &[], Currency::VND).unwrap();
        assert_eq!(money, Money::vnd(45_000_000));

        let money2 =
            parse_dsl_amount("1.380.600.000", ".", ",", 0, &[], Currency::VND).unwrap();
        assert_eq!(money2, Money::vnd(1_380_600_000));

        let money3 =
            parse_dsl_amount("15.000.000,00", ".", ",", 0, &[], Currency::VND).unwrap();
        assert_eq!(money3, Money::vnd(15_000_000));

        let money_neg =
            parse_dsl_amount("-2.200.000", ".", ",", 0, &[], Currency::VND).unwrap();
        assert_eq!(money_neg, Money::vnd(-2_200_000));

        let money_paren =
            parse_dsl_amount("(500.000)", ".", ",", 0, &[], Currency::VND).unwrap();
        assert_eq!(money_paren, Money::vnd(-500_000));
    }

    #[test]
    fn test_parse_usd_with_cents() {
        let money = parse_dsl_amount("1,234.56", ",", ".", 2, &[], Currency::USD).unwrap();
        assert_eq!(money, Money::usd_cents(123456));

        let money_zero_cents =
            parse_dsl_amount("1,000", ",", ".", 2, &[], Currency::USD).unwrap();
        assert_eq!(money_zero_cents, Money::usd_cents(100000));
    }

    #[test]
    fn test_strip_currency_tokens() {
        let strip = vec!["VND".to_string(), "VNĐ".to_string(), "đ".to_string()];
        let money = parse_dsl_amount("45.000.000 VND", ".", ",", 0, &strip, Currency::VND).unwrap();
        assert_eq!(money, Money::vnd(45_000_000));
    }
}
