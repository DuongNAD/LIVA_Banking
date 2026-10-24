//! LIVA Banking — Dynamic Fee Unbundling & Expense Routing Engine
//!
//! Decomposes gross banking service fees into net expense and deductible input VAT
//! using 100% integer arithmetic (basis points).
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.
//! Guarantees zero penny drift: `Net + Vat == Gross` down to 1 VND.
//! Automatically routes expenses according to Vietnamese Accounting Standards:
//! - Circular 200/2014: `6427` (Purchased services/Bank fees) or `6425` (Taxes & fees), `1331` (Input VAT)
//! - Circular 133/2016: `6422` (General & admin expense), `1331` (Input VAT)

use super::coa_schema::AccountStandard;
use crate::LedgerError;
use liva_money::{Money, MoneyError};
use serde::{Deserialize, Serialize};

/// Divisor for basis points (10,000 bps = 100.00%).
pub const BPS_DIVISOR: i64 = 10_000;

/// Standard Vietnamese VAT rate in basis points: 10% = 1,000 bps.
pub const VAT_STANDARD_RATE_BPS: i64 = 1_000;

/// Reduced Vietnamese VAT rate in basis points: 8% = 800 bps.
pub const VAT_REDUCED_RATE_BPS: i64 = 800;

/// Zero VAT rate in basis points (exempt or 0% rate): 0 bps.
pub const VAT_EXEMPT_RATE_BPS: i64 = 0;

/// Unbundles a gross bank fee into `(Net Expense, Input VAT)` using 100% integer arithmetic.
///
/// # Mathematical Formulation:
///
/// $$\text{Net} = \left\lfloor \frac{\text{Gross} \times 10,000}{10,000 + \text{VatBps}} \right\rfloor$$
/// $$\text{Vat} = \text{Gross} - \text{Net}$$
///
/// # Invariant Guarantees:
/// 1. `Net + Vat == Gross` ($\Delta = 0$).
/// 2. Zero floating-point arithmetic.
/// 3. If `VatBps <= 0`, `Net = Gross` and `Vat = 0`.
/// 4. If `Gross < 2 VND` and `Vat` computes to 0, `Net = Gross`.
pub fn unbundle_gross_fee(gross: Money, vat_rate_bps: i64) -> Result<(Money, Money), LedgerError> {
    if !gross.is_positive() {
        return Err(LedgerError::NonPositiveAmount);
    }
    if vat_rate_bps <= 0 {
        return Ok((gross, Money::zero(gross.currency())));
    }

    let g = gross.amount() as i128;
    let rate = vat_rate_bps as i128;
    let den = (BPS_DIVISOR as i128)
        .checked_add(rate)
        .ok_or(MoneyError::Overflow)?;

    // Net = floor(Gross * 10,000 / (10,000 + vat_rate_bps))
    let numerator = g
        .checked_mul(BPS_DIVISOR as i128)
        .ok_or(MoneyError::Overflow)?;
    let raw_net_128 = numerator / den;

    // Boundary guard: If gross < 2 VND or integer net rounds to 0,
    // net absorbs the full gross amount and VAT is 0.
    let net_128 = if gross.amount() < 2 || raw_net_128 == 0 {
        g
    } else {
        raw_net_128
    };

    let net_amt = i64::try_from(net_128).map_err(|_| LedgerError::Money(MoneyError::Overflow))?;
    let vat_amt = gross
        .amount()
        .checked_sub(net_amt)
        .ok_or(MoneyError::Underflow)?;

    let net = Money::from_minor(net_amt, gross.currency());
    let vat = Money::from_minor(vat_amt, gross.currency());

    // Invariant self-check
    debug_assert_eq!(
        net.checked_add(vat).map_err(LedgerError::Money)?,
        gross,
        "Mathematical invariant violation: Net ({}) + Vat ({}) != Gross ({})",
        net,
        vat,
        gross
    );

    Ok((net, vat))
}

/// Result of unbundling a gross fee with statutory expense and VAT account routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnbundledFeeResult {
    /// Original gross bank fee deducted
    pub gross: Money,
    /// Net bank fee expense portion
    pub net: Money,
    /// Deductible input VAT portion
    pub vat: Money,
    /// VAT rate applied in basis points
    pub vat_rate_bps: i64,
    /// Target expense account code (e.g. "6427", "6425", "6422")
    pub expense_account: String,
    /// Target input VAT account code (e.g. "1331")
    pub vat_account: String,
}

impl UnbundledFeeResult {
    /// Returns true if VAT is positive and should be posted to the ledger.
    pub const fn has_vat(&self) -> bool {
        self.vat.is_positive()
    }
}

/// Unbundles a gross bank fee and routes it to standard statutory accounts
/// according to the specified accounting standard.
///
/// # Routing Rules:
/// - Circular 200/2014:
///   - Default expense account: `6427` (Chi phí dịch vụ mua ngoài)
///   - VAT account: `1331` (Thuế GTGT được khấu trừ)
/// - Circular 133/2016:
///   - Default expense account: `6422` (Chi phí quản lý doanh nghiệp)
///   - VAT account: `1331` (Thuế GTGT được khấu trừ)
/// - Custom standard:
///   - Uses `custom_expense_override` if provided, otherwise defaults to `6427`.
pub fn route_and_unbundle_fee(
    gross: Money,
    vat_rate_bps: i64,
    standard: AccountStandard,
    custom_expense_override: Option<&str>,
    custom_vat_override: Option<&str>,
) -> Result<UnbundledFeeResult, LedgerError> {
    let (net, vat) = unbundle_gross_fee(gross, vat_rate_bps)?;

    let expense_account = if let Some(custom_acc) = custom_expense_override {
        custom_acc.to_string()
    } else {
        match standard {
            AccountStandard::Circular200_2014 => "6427".to_string(),
            AccountStandard::Circular133_2016 => "6422".to_string(),
            AccountStandard::Custom => "6427".to_string(),
        }
    };

    let vat_account = custom_vat_override.unwrap_or("1331").to_string();

    Ok(UnbundledFeeResult {
        gross,
        net,
        vat,
        vat_rate_bps,
        expense_account,
        vat_account,
    })
}
