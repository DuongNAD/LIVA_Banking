//! Statement balance invariant verification.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::models::{NormalizationError, NormalizedStatement};

/// Invariant summary containing calculated totals and check status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvariantCheckResult {
    pub opening_cents: u64,
    pub closing_cents: u64,
    pub total_credit_cents: u64,
    pub total_debit_cents: u64,
    pub calculated_closing_cents: u64,
    pub is_valid: bool,
}

/// Verifies the core banking statement invariant:
/// `closing_cents == opening_cents + SUM(credit) - SUM(debit)`.
pub fn verify_statement_balance(
    stmt: &NormalizedStatement,
) -> Result<InvariantCheckResult, NormalizationError> {
    let mut total_credit: u64 = 0;
    let mut total_debit: u64 = 0;

    for tx in &stmt.transactions {
        if tx.is_credit {
            total_credit = total_credit
                .checked_add(tx.amount_cents)
                .ok_or(NormalizationError::Overflow)?;
        } else {
            total_debit = total_debit
                .checked_add(tx.amount_cents)
                .ok_or(NormalizationError::Overflow)?;
        }
    }

    // Use i128 to prevent overflow in intermediate calculations
    let open_i = stmt.opening_cents as i128;
    let cred_i = total_credit as i128;
    let deb_i = total_debit as i128;
    let calc_closing_i = open_i + cred_i - deb_i;

    let is_valid = if calc_closing_i < 0 {
        false
    } else {
        (calc_closing_i as u64) == stmt.closing_cents
    };

    let calculated_closing_cents = if calc_closing_i >= 0 {
        calc_closing_i as u64
    } else {
        0
    };

    let result = InvariantCheckResult {
        opening_cents: stmt.opening_cents,
        closing_cents: stmt.closing_cents,
        total_credit_cents: total_credit,
        total_debit_cents: total_debit,
        calculated_closing_cents,
        is_valid,
    };

    if !is_valid {
        return Err(NormalizationError::InvariantViolation(format!(
            "Statement balance invariant violated for account '{}': opening {} + credit {} - debit {} = calculated {}, but closing is {}",
            stmt.account_no, stmt.opening_cents, total_credit, total_debit, calculated_closing_cents, stmt.closing_cents
        )));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BankIdentifier, NormalizedTransaction};

    #[test]
    fn test_valid_statement_invariant() {
        let stmt = NormalizedStatement {
            bank: BankIdentifier::Vietcombank,
            account_no: "0011001234567".to_string(),
            opening_cents: 1_000_000,
            closing_cents: 1_500_000,
            transactions: vec![
                NormalizedTransaction {
                    id: "tx-1".to_string(),
                    date: "2026-08-01".to_string(),
                    booking_date: None,
                    voucher_no: None,
                    amount_cents: 600_000,
                    is_credit: true,
                    balance_cents: 1_600_000,
                    narration: "Credit".to_string(),
                    counterparty_name: None,
                    reference_codes: vec![],
                    tx_timestamp: 1785517200,
                    value_timestamp: None,
                },
                NormalizedTransaction {
                    id: "tx-2".to_string(),
                    date: "2026-08-02".to_string(),
                    booking_date: None,
                    voucher_no: None,
                    amount_cents: 100_000,
                    is_credit: false,
                    balance_cents: 1_500_000,
                    narration: "Debit".to_string(),
                    counterparty_name: None,
                    reference_codes: vec![],
                    tx_timestamp: 1785603600,
                    value_timestamp: None,
                },
            ],
        };

        let res = verify_statement_balance(&stmt).unwrap();
        assert!(res.is_valid);
        assert_eq!(res.calculated_closing_cents, 1_500_000);
    }

    #[test]
    fn test_invalid_statement_invariant() {
        let stmt = NormalizedStatement {
            bank: BankIdentifier::Vietcombank,
            account_no: "0011001234567".to_string(),
            opening_cents: 1_000_000,
            closing_cents: 1_499_999, // 1 VND discrepancy
            transactions: vec![NormalizedTransaction {
                id: "tx-1".to_string(),
                date: "2026-08-01".to_string(),
                booking_date: None,
                voucher_no: None,
                amount_cents: 500_000,
                is_credit: true,
                balance_cents: 1_500_000,
                narration: "Credit".to_string(),
                counterparty_name: None,
                reference_codes: vec![],
                tx_timestamp: 1785517200,
                value_timestamp: None,
            }],
        };

        assert!(verify_statement_balance(&stmt).is_err());
    }
}
