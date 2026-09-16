//! Monetary amount normalization and zero-float parsing.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::models::{NormalizationError, NormalizedAmount};
use liva_money::{Currency, Money};

/// Result of parsing an amount string, including magnitude in minor units and negative flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedAmount {
    pub minor_units: u64,
    pub is_negative: bool,
    pub currency: Currency,
}

impl ParsedAmount {
    pub fn to_normalized_amount(&self) -> NormalizedAmount {
        NormalizedAmount::new(self.minor_units, self.currency)
    }

    pub fn to_money(&self) -> Result<Money, NormalizationError> {
        let signed = if self.is_negative {
            -i64::try_from(self.minor_units).map_err(|_| NormalizationError::Overflow)?
        } else {
            i64::try_from(self.minor_units).map_err(|_| NormalizationError::Overflow)?
        };
        Ok(Money::from_minor(signed, self.currency))
    }
}

/// Parses a formatted monetary string into integer minor units (VND cents/dong).
///
/// Handles:
/// - Vietnamese / European format: `12.500.000,00` or `12.500.000` -> `12500000`
/// - US / UK format: `12,500,000.00` or `12,500,000` -> `12500000`
/// - Plain integer: `12500000` -> `12500000`
/// - SWIFT trailing comma: `11500000,` -> `11500000`
/// - Explicit signs: `-25.000.000`, `(25.000.000)`, `+10.000.000`
/// - Currency markers: `VND`, `VNĐ`, `đ`, `USD`, `EUR`
pub fn parse_monetary_amount(input: &str) -> Result<ParsedAmount, NormalizationError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(NormalizationError::InvalidMonetaryAmount(
            "Empty string".to_string(),
        ));
    }

    let mut upper = raw.to_uppercase();
    let mut currency = Currency::VND;
    if upper.contains("USD") || upper.contains('$') {
        currency = Currency::USD;
    } else if upper.contains("EUR") || upper.contains('€') {
        currency = Currency::EUR;
    }

    // Strip currency symbols and letters
    let mut cleaned = String::with_capacity(upper.len());
    let mut is_negative = false;

    // Check accounting parenthesis: (123.456)
    if upper.starts_with('(') && upper.ends_with(')') {
        is_negative = true;
        upper = upper[1..upper.len() - 1].to_string();
    }

    for ch in upper.chars() {
        match ch {
            '-' => is_negative = true,
            '+' => {}
            '0'..='9' | '.' | ',' => cleaned.push(ch),
            _ => {} // Ignore whitespace and letters
        }
    }

    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return Err(NormalizationError::InvalidMonetaryAmount(raw.to_string()));
    }

    // Determine decimal vs thousand separator
    let has_dot = cleaned.contains('.');
    let has_comma = cleaned.contains(',');

    let minor_units: u64 = if has_dot && has_comma {
        let last_dot = cleaned.rfind('.').unwrap();
        let last_comma = cleaned.rfind(',').unwrap();

        if last_comma > last_dot {
            // Dot thousands, Comma decimal: e.g. "12.500.000,00"
            let integer_part = &cleaned[..last_comma].replace('.', "");
            let decimal_part = &cleaned[last_comma + 1..];
            parse_units_with_decimals(integer_part, decimal_part, currency)?
        } else {
            // Comma thousands, Dot decimal: e.g. "12,500,000.00"
            let integer_part = &cleaned[..last_dot].replace(',', "");
            let decimal_part = &cleaned[last_dot + 1..];
            parse_units_with_decimals(integer_part, decimal_part, currency)?
        }
    } else if has_comma {
        // Only commas present: could be thousands ("12,500,000") or decimal ("12500000,00" or SWIFT "11500000,")
        let comma_count = cleaned.chars().filter(|&c| c == ',').count();
        let last_comma = cleaned.rfind(',').unwrap();
        let after_comma = &cleaned[last_comma + 1..];

        if comma_count > 1 {
            // Multiple commas -> thousands separator: "1,000,000"
            let integer_part = cleaned.replace(',', "");
            integer_part
                .parse::<u64>()
                .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
        } else if after_comma.is_empty() {
            // SWIFT trailing comma: "11500000,"
            let integer_part = &cleaned[..last_comma];
            integer_part
                .parse::<u64>()
                .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
        } else if after_comma.len() <= 2 {
            // Likely decimal comma: "12500000,50"
            let integer_part = &cleaned[..last_comma];
            parse_units_with_decimals(integer_part, after_comma, currency)?
        } else if after_comma.len() == 3 {
            // Likely thousands comma: "125,000"
            let integer_part = cleaned.replace(',', "");
            integer_part
                .parse::<u64>()
                .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
        } else {
            let integer_part = cleaned.replace(',', "");
            integer_part
                .parse::<u64>()
                .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
        }
    } else if has_dot {
        // Only dots present: could be thousands ("12.500.000") or decimal ("12500000.00")
        let dot_count = cleaned.chars().filter(|&c| c == '.').count();
        let last_dot = cleaned.rfind('.').unwrap();
        let after_dot = &cleaned[last_dot + 1..];

        if dot_count > 1 {
            // Multiple dots -> thousands separator: "12.500.000"
            let integer_part = cleaned.replace('.', "");
            integer_part
                .parse::<u64>()
                .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
        } else if currency == Currency::VND && after_dot.len() == 3 {
            // Vietnamese convention: single dot followed by 3 digits is thousands (e.g. 500.000)
            let integer_part = cleaned.replace('.', "");
            integer_part
                .parse::<u64>()
                .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
        } else if after_dot.len() <= 2 {
            // Decimal dot: "12500000.50"
            let integer_part = &cleaned[..last_dot];
            parse_units_with_decimals(integer_part, after_dot, currency)?
        } else {
            // Fallback: strip dot
            let integer_part = cleaned.replace('.', "");
            integer_part
                .parse::<u64>()
                .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
        }
    } else {
        // Plain integer
        cleaned
            .parse::<u64>()
            .map_err(|_| NormalizationError::InvalidMonetaryAmount(raw.to_string()))?
    };

    Ok(ParsedAmount {
        minor_units,
        is_negative,
        currency,
    })
}

/// Helper to parse integer and decimal parts into minor units without floating point math.
fn parse_units_with_decimals(
    integer_part: &str,
    decimal_part: &str,
    currency: Currency,
) -> Result<u64, NormalizationError> {
    let int_val: u64 = if integer_part.is_empty() {
        0
    } else {
        integer_part
            .parse()
            .map_err(|_| NormalizationError::InvalidMonetaryAmount(integer_part.to_string()))?
    };

    let scale = currency.decimal_places(); // VND = 0, USD = 2, EUR = 2

    if scale == 0 {
        // VND: decimal places are discarded or rounded without float
        // If decimal is >= 50, round up
        let dec_prefix: u32 = if decimal_part.len() >= 2 {
            decimal_part[..2].parse().unwrap_or(0)
        } else if decimal_part.len() == 1 {
            decimal_part.parse::<u32>().unwrap_or(0) * 10
        } else {
            0
        };
        let rounded = if dec_prefix >= 50 {
            int_val.saturating_add(1)
        } else {
            int_val
        };
        Ok(rounded)
    } else {
        // 2 decimal places (USD / EUR cents)
        let scaled_int = int_val
            .checked_mul(100)
            .ok_or(NormalizationError::Overflow)?;
        let cents: u64 = if decimal_part.len() >= 2 {
            decimal_part[..2].parse().unwrap_or(0)
        } else if decimal_part.len() == 1 {
            decimal_part.parse::<u64>().unwrap_or(0) * 10
        } else {
            0
        };
        Ok(scaled_int.saturating_add(cents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vietnamese_dot_thousands_comma_decimals() {
        let res = parse_monetary_amount("12.500.000,00").unwrap();
        assert_eq!(res.minor_units, 12_500_000);
        assert!(!res.is_negative);

        let res2 = parse_monetary_amount("1.450.230.000,00").unwrap();
        assert_eq!(res2.minor_units, 1_450_230_000);
    }

    #[test]
    fn test_parse_vietnamese_dot_thousands() {
        let res = parse_monetary_amount("520.000.000").unwrap();
        assert_eq!(res.minor_units, 520_000_000);
    }

    #[test]
    fn test_parse_swift_mt940_amount() {
        let res = parse_monetary_amount("785600000,").unwrap();
        assert_eq!(res.minor_units, 785_600_000);
    }

    #[test]
    fn test_parse_negative_amount() {
        let res = parse_monetary_amount("-25.000.000").unwrap();
        assert_eq!(res.minor_units, 25_000_000);
        assert!(res.is_negative);

        let res2 = parse_monetary_amount("(15.000.000)").unwrap();
        assert_eq!(res2.minor_units, 15_000_000);
        assert!(res2.is_negative);
    }
}
