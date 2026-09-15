//! LIVA Banking — `liva-money` Crate
//!
//! Deterministic, zero-float scaled integer money engine.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Errors arising from money calculations and currency conversions.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MoneyError {
    #[error("Currency mismatch: expected {expected:?}, found {found:?}")]
    CurrencyMismatch { expected: Currency, found: Currency },

    #[error("Integer arithmetic overflow occurred")]
    Overflow,

    #[error("Integer arithmetic underflow occurred")]
    Underflow,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Allocation failed: ratios slice cannot be empty or sum to zero")]
    InvalidAllocationRatios,

    #[error("Failed to parse money string: {0}")]
    ParseError(String),
}

/// Supported currency types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Currency {
    /// Vietnamese Dong (minor unit = 1 VND, 0 decimal places)
    VND,
    /// US Dollar (minor unit = 1 Cent = 0.01 USD, 2 decimal places)
    USD,
    /// Euro (minor unit = 1 Cent = 0.01 EUR, 2 decimal places)
    EUR,
}

impl Currency {
    /// Number of decimal places in the minor unit representation.
    pub const fn decimal_places(&self) -> u32 {
        match self {
            Currency::VND => 0,
            Currency::USD => 2,
            Currency::EUR => 2,
        }
    }

    /// Scaling factor from major units to atomic minor units (e.g. 1 USD = 100 cents).
    pub const fn minor_unit_scale(&self) -> i64 {
        match self {
            Currency::VND => 1,
            Currency::USD => 100,
            Currency::EUR => 100,
        }
    }

    /// ISO-4217 currency code as static string.
    pub const fn code(&self) -> &'static str {
        match self {
            Currency::VND => "VND",
            Currency::USD => "USD",
            Currency::EUR => "EUR",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for Currency {
    type Err = MoneyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "VND" | "VNĐ" | "DONG" | "ĐỒNG" => Ok(Currency::VND),
            "USD" | "$" => Ok(Currency::USD),
            "EUR" | "€" => Ok(Currency::EUR),
            other => Err(MoneyError::ParseError(format!("Unsupported currency code '{other}'"))),
        }
    }
}

/// Atomic representation of monetary amounts in minor units (e.g. 1 Dong or 1 Cent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Money {
    amount: i64,
    currency: Currency,
}

impl Money {
    /// Creates a new `Money` instance from atomic minor units.
    pub const fn from_minor(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Creates a new `Money` instance from major currency units (e.g. 100 USD = 10,000 cents).
    pub fn from_major(major: i64, currency: Currency) -> Result<Self, MoneyError> {
        let scale = currency.minor_unit_scale();
        let amount = major.checked_mul(scale).ok_or(MoneyError::Overflow)?;
        Ok(Self { amount, currency })
    }

    /// Convenience constructor for Vietnamese Dong (minor unit = 1 VND).
    pub const fn vnd(amount: i64) -> Self {
        Self {
            amount,
            currency: Currency::VND,
        }
    }

    /// Convenience constructor for USD cents (100 cents = 1 USD).
    pub const fn usd_cents(cents: i64) -> Self {
        Self {
            amount: cents,
            currency: Currency::USD,
        }
    }

    /// Returns a zero-value `Money` for the given currency.
    pub const fn zero(currency: Currency) -> Self {
        Self {
            amount: 0,
            currency,
        }
    }

    /// Returns the raw amount in atomic minor units.
    pub const fn amount(&self) -> i64 {
        self.amount
    }

    /// Returns the currency.
    pub const fn currency(&self) -> Currency {
        self.currency
    }

    /// Returns true if the monetary value is zero.
    pub const fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Returns true if the monetary value is strictly positive (> 0).
    pub const fn is_positive(&self) -> bool {
        self.amount > 0
    }

    /// Returns true if the monetary value is strictly negative (< 0).
    pub const fn is_negative(&self) -> bool {
        self.amount < 0
    }

    /// Returns the absolute value of the monetary amount.
    pub fn abs(&self) -> Result<Self, MoneyError> {
        let amount = self.amount.checked_abs().ok_or(MoneyError::Overflow)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Checked addition with currency check.
    pub fn checked_add(&self, rhs: Self) -> Result<Self, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency,
                found: rhs.currency,
            });
        }
        let amount = self.amount.checked_add(rhs.amount).ok_or(MoneyError::Overflow)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Checked subtraction with currency check.
    pub fn checked_sub(&self, rhs: Self) -> Result<Self, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency,
                found: rhs.currency,
            });
        }
        let amount = self.amount.checked_sub(rhs.amount).ok_or(MoneyError::Overflow)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Checked integer multiplication.
    pub fn checked_mul(&self, factor: i64) -> Result<Self, MoneyError> {
        let amount = self.amount.checked_mul(factor).ok_or(MoneyError::Overflow)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Multiplies money by a rational fraction `numerator / denominator` using 128-bit
    /// integer arithmetic with **Banker's Rounding (Round-Half-to-Even)**.
    ///
    /// Eliminates floating-point drift and guarantees exact deterministic rounding.
    pub fn checked_mul_ratio(&self, numerator: i64, denominator: i64) -> Result<Self, MoneyError> {
        if denominator == 0 {
            return Err(MoneyError::DivisionByZero);
        }

        let num_128 = (self.amount as i128)
            .checked_mul(numerator as i128)
            .ok_or(MoneyError::Overflow)?;
        let den_128 = denominator as i128;

        // Standard quotient and remainder
        let quotient = num_128 / den_128;
        let remainder = num_128 % den_128;

        if remainder == 0 {
            let res = i64::try_from(quotient).map_err(|_| MoneyError::Overflow)?;
            return Ok(Self {
                amount: res,
                currency: self.currency,
            });
        }

        // Half-to-even (Banker's rounding) logic in pure integer math
        let abs_rem = remainder.abs();
        let abs_den = den_128.abs();
        let twice_rem = abs_rem.checked_mul(2).ok_or(MoneyError::Overflow)?;

        let rounded_quotient = if twice_rem > abs_den {
            // Strictly greater than half: round away from zero
            if (num_128 > 0) == (den_128 > 0) {
                quotient + 1
            } else {
                quotient - 1
            }
        } else if twice_rem == abs_den {
            // Exactly halfway: round to nearest even quotient
            if quotient % 2 != 0 {
                if (num_128 > 0) == (den_128 > 0) {
                    quotient + 1
                } else {
                    quotient - 1
                }
            } else {
                quotient
            }
        } else {
            // Less than half: round towards zero (truncate)
            quotient
        };

        let res = i64::try_from(rounded_quotient).map_err(|_| MoneyError::Overflow)?;
        Ok(Self {
            amount: res,
            currency: self.currency,
        })
    }

    /// Deterministically allocates this monetary amount across multiple parts according to integer ratios.
    ///
    /// **Zero Penny Drift Guarantee**: `sum(results) == self.amount`.
    /// Residual minor units from division are distributed one unit at a time to the largest remainders.
    pub fn allocate(&self, ratios: &[u32]) -> Result<Vec<Self>, MoneyError> {
        if ratios.is_empty() {
            return Err(MoneyError::InvalidAllocationRatios);
        }

        let total_ratio: u64 = ratios.iter().map(|&r| r as u64).sum();
        if total_ratio == 0 {
            return Err(MoneyError::InvalidAllocationRatios);
        }

        let mut results = Vec::with_capacity(ratios.len());
        let mut allocated_sum: i64 = 0;
        let mut remainders: Vec<(usize, i64)> = Vec::with_capacity(ratios.len());

        let total_ratio_128 = total_ratio as i128;
        let is_neg = self.amount < 0;
        let abs_amount = self.amount.abs();

        for (idx, &ratio) in ratios.iter().enumerate() {
            if ratio == 0 {
                results.push(0i64);
                continue;
            }
            let prod = (abs_amount as i128) * (ratio as i128);
            let share = (prod / total_ratio_128) as i64;
            let rem = (prod % total_ratio_128) as i64;

            results.push(share);
            allocated_sum += share;
            remainders.push((idx, rem));
        }

        // Remainder minor units to distribute
        let mut leftover = abs_amount - allocated_sum;

        // Sort descending by remainder to allocate leftover minor units fairly
        remainders.sort_by_key(|a| std::cmp::Reverse(a.1));

        for (idx, _) in remainders.iter() {
            if leftover == 0 {
                break;
            }
            results[*idx] += 1;
            leftover -= 1;
        }

        let final_money: Vec<Money> = results
            .into_iter()
            .map(|val| {
                let actual = if is_neg { -val } else { val };
                Self {
                    amount: actual,
                    currency: self.currency,
                }
            })
            .collect();

        Ok(final_money)
    }

    /// Formats the monetary amount according to Vietnamese banking convention:
    /// e.g. `750.000.000 VND` or `-15.000.000 VND` with dot `.` thousand separators.
    pub fn to_vietnamese_display(&self) -> String {
        let is_negative = self.amount < 0;
        let abs_val = self.amount.abs();
        let s = abs_val.to_string();

        let mut formatted = String::new();
        let len = s.len();

        for (i, ch) in s.chars().enumerate() {
            if i > 0 && (len - i).is_multiple_of(3) {
                formatted.push('.');
            }
            formatted.push(ch);
        }

        if is_negative {
            format!("-{} {}", formatted, self.currency)
        } else {
            format!("{} {}", formatted, self.currency)
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_vietnamese_display())
    }
}

impl std::ops::Add for Money {
    type Output = Result<Money, MoneyError>;
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl std::ops::Sub for Money {
    type Output = Result<Money, MoneyError>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_float_vietnamese_display() {
        let m1 = Money::vnd(750_000_000);
        assert_eq!(m1.to_vietnamese_display(), "750.000.000 VND");

        let m2 = Money::vnd(-15_000_000);
        assert_eq!(m2.to_vietnamese_display(), "-15.000.000 VND");

        let m3 = Money::vnd(0);
        assert_eq!(m3.to_vietnamese_display(), "0 VND");
    }

    #[test]
    fn test_checked_add_sub() {
        let a = Money::vnd(1_000_000);
        let b = Money::vnd(250_000);

        let sum = a.checked_add(b).unwrap();
        assert_eq!(sum.amount(), 1_250_000);

        let diff = a.checked_sub(b).unwrap();
        assert_eq!(diff.amount(), 750_000);
    }

    #[test]
    fn test_currency_mismatch() {
        let a = Money::vnd(1_000_000);
        let b = Money::usd_cents(100);

        assert!(matches!(
            a.checked_add(b),
            Err(MoneyError::CurrencyMismatch { .. })
        ));
    }

    #[test]
    fn test_bankers_rounding_half_to_even() {
        // 25.0 * 1/2 = 12.5 -> rounds to 12 (even)
        let m = Money::vnd(25);
        let half = m.checked_mul_ratio(1, 2).unwrap();
        assert_eq!(half.amount(), 12);

        // 35.0 * 1/2 = 17.5 -> rounds to 18 (even)
        let m2 = Money::vnd(35);
        let half2 = m2.checked_mul_ratio(1, 2).unwrap();
        assert_eq!(half2.amount(), 18);
    }

    #[test]
    fn test_split_allocation_zero_penny_drift() {
        let total = Money::vnd(100);
        // Split into 3 parts (1:1:1): 33 + 33 + 34 = 100
        let parts = total.allocate(&[1, 1, 1]).unwrap();
        assert_eq!(parts.len(), 3);
        let sum: i64 = parts.iter().map(|p| p.amount()).sum();
        assert_eq!(sum, 100);

        // Split into 7 parts
        let total_big = Money::vnd(1_000_000_007);
        let ratios = vec![3, 5, 2, 7, 11, 13, 17];
        let parts_big = total_big.allocate(&ratios).unwrap();
        let sum_big: i64 = parts_big.iter().map(|p| p.amount()).sum();
        assert_eq!(sum_big, 1_000_000_007);
    }
}
