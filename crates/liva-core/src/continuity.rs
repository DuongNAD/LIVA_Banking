//! Inter-period balance continuity verification.
//!
//! Enforces the fundamental accounting invariant:
//! `Opening(Period N) == Closing(Period N-1)`.

#![deny(clippy::float_arithmetic)]

use liva_money::Money;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents summary balances of a fiscal or reconciliation period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeriodBalance {
    pub period_id: String,
    pub account_id: String,
    pub opening_balance: Money,
    pub closing_balance: Money,
}

impl PeriodBalance {
    /// Creates a new `PeriodBalance`.
    pub fn new(
        period_id: impl Into<String>,
        account_id: impl Into<String>,
        opening_balance: Money,
        closing_balance: Money,
    ) -> Self {
        Self {
            period_id: period_id.into(),
            account_id: account_id.into(),
            opening_balance,
            closing_balance,
        }
    }
}

/// Errors occurring during period balance continuity validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContinuityError {
    #[error("Period balance continuity discrepancy for account {account_id}, period {period_id}: expected opening {expected_opening:?}, actual opening {actual_opening:?}, delta {delta}")]
    Discrepancy {
        account_id: String,
        period_id: String,
        expected_opening: Money,
        actual_opening: Money,
        delta: i64,
    },

    #[error("Account mismatch between consecutive periods: expected {expected}, actual {actual}")]
    AccountMismatch {
        expected: String,
        actual: String,
    },

    #[error("Currency mismatch between consecutive periods")]
    CurrencyMismatch,

    #[error("Empty period sequence")]
    EmptySequence,
}

/// Verifies that `Opening(Period N) == Closing(Period N-1)`.
///
/// Ensures both periods belong to the same account and share the same currency.
/// If `curr.opening_balance != prev.closing_balance`, returns a `ContinuityError::Discrepancy`
/// containing the exact difference $\Delta = \text{actual} - \text{expected}$.
pub fn verify_period_transition(
    prev: &PeriodBalance,
    curr: &PeriodBalance,
) -> Result<(), ContinuityError> {
    if prev.account_id != curr.account_id {
        return Err(ContinuityError::AccountMismatch {
            expected: prev.account_id.clone(),
            actual: curr.account_id.clone(),
        });
    }

    if prev.closing_balance.currency() != curr.opening_balance.currency() {
        return Err(ContinuityError::CurrencyMismatch);
    }

    if curr.opening_balance != prev.closing_balance {
        let delta = curr
            .opening_balance
            .amount()
            .saturating_sub(prev.closing_balance.amount());
        return Err(ContinuityError::Discrepancy {
            account_id: curr.account_id.clone(),
            period_id: curr.period_id.clone(),
            expected_opening: prev.closing_balance,
            actual_opening: curr.opening_balance,
            delta,
        });
    }

    Ok(())
}

/// Validates continuity across a chronological sequence of periods for an account.
///
/// Verifies `Opening(Period i) == Closing(Period i-1)` for each consecutive pair in the slice.
/// Returns `Err(ContinuityError::EmptySequence)` if `periods` is empty.
pub fn verify_period_sequence(periods: &[PeriodBalance]) -> Result<(), ContinuityError> {
    if periods.is_empty() {
        return Err(ContinuityError::EmptySequence);
    }

    for window in periods.windows(2) {
        verify_period_transition(&window[0], &window[1])?;
    }

    Ok(())
}
