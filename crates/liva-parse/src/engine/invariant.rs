//! Mathematical balance invariant verification.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::ParseError;
use crate::template::schema::StatementTemplate;
use crate::CanonicalStatement;
use liva_ledger::{Ledger, PostingType};
use liva_money::Money;

/// Enforces the mathematical balance invariant on a CanonicalStatement according to the template rules.
pub fn verify_statement_invariants(
    stmt: &mut CanonicalStatement,
    template: &StatementTemplate,
) -> Result<bool, ParseError> {
    let policy = template.invariants.policy.as_str();
    if policy == "relaxed" {
        return Ok(true);
    }

    let tx_pairs: Vec<(PostingType, Money)> = stmt
        .transactions
        .iter()
        .map(|t| (t.posting_type, t.amount))
        .collect();

    // Auto-compute missing balances if allowed
    match (stmt.opening_balance, stmt.closing_balance) {
        (Some(opening), None) => {
            if template.invariants.allow_missing_balances {
                let mut running = opening;
                for (pt, amt) in &tx_pairs {
                    match pt {
                        PostingType::Credit => {
                            running = running.checked_add(*amt).map_err(|e| {
                                ParseError::InvariantError(format!("Checked add overflow: {e}"))
                            })?;
                        }
                        PostingType::Debit => {
                            running = running.checked_sub(*amt).map_err(|e| {
                                ParseError::InvariantError(format!("Checked sub overflow: {e}"))
                            })?;
                        }
                    }
                }
                stmt.closing_balance = Some(running);
                return Ok(true);
            } else if policy == "strict" {
                return Err(ParseError::InvariantError(
                    "Missing closing balance in strict invariant policy".to_string(),
                ));
            }
        }
        (None, Some(closing)) => {
            if template.invariants.allow_missing_balances {
                let mut running = closing;
                for (pt, amt) in &tx_pairs {
                    match pt {
                        PostingType::Credit => {
                            running = running.checked_sub(*amt).map_err(|e| {
                                ParseError::InvariantError(format!("Checked sub overflow: {e}"))
                            })?;
                        }
                        PostingType::Debit => {
                            running = running.checked_add(*amt).map_err(|e| {
                                ParseError::InvariantError(format!("Checked add overflow: {e}"))
                            })?;
                        }
                    }
                }
                stmt.opening_balance = Some(running);
                return Ok(true);
            } else if policy == "strict" {
                return Err(ParseError::InvariantError(
                    "Missing opening balance in strict invariant policy".to_string(),
                ));
            }
        }
        (None, None) => {
            if stmt.transactions.is_empty() {
                return Ok(true);
            }
            if template.invariants.allow_missing_balances {
                // If transactions exist and have balance_after
                if let Some(first_bal) = stmt.transactions.first().and_then(|t| t.balance_after) {
                    let first_pt = stmt.transactions[0].posting_type;
                    let first_amt = stmt.transactions[0].amount;
                    let inferred_opening = match first_pt {
                        PostingType::Credit => first_bal.checked_sub(first_amt).ok(),
                        PostingType::Debit => first_bal.checked_add(first_amt).ok(),
                    };
                    stmt.opening_balance = inferred_opening;
                }
                if let Some(last_bal) = stmt.transactions.last().and_then(|t| t.balance_after) {
                    stmt.closing_balance = Some(last_bal);
                }
            } else if policy == "strict" {
                return Err(ParseError::InvariantError(
                    "Both opening and closing balances are missing".to_string(),
                ));
            }
        }
        (Some(opening), Some(closing)) => {
            // Both present: evaluate verification formula
            let is_valid = Ledger::verify_statement_balance(opening, closing, &tx_pairs)
                .map_err(|e| ParseError::InvariantError(e.to_string()))?;

            if !is_valid {
                let tol = template.invariants.tolerance_minor_units;
                if tol > 0 {
                    // Check within tolerance
                    let mut running = opening;
                    for (pt, amt) in &tx_pairs {
                        if *pt == PostingType::Credit {
                            running = running.checked_add(*amt).map_err(|e| {
                                ParseError::InvariantError(format!("Checked add overflow: {e}"))
                            })?;
                        } else {
                            running = running.checked_sub(*amt).map_err(|e| {
                                ParseError::InvariantError(format!("Checked sub overflow: {e}"))
                            })?;
                        }
                    }
                    let diff = (running.amount() - closing.amount()).abs();
                    if diff <= tol {
                        return Ok(true);
                    }
                }

                if policy == "strict" {
                    return Err(ParseError::InvariantError(format!(
                        "Statement balance mismatch: Opening ({}) + Credits - Debits != Closing ({})",
                        opening.amount(),
                        closing.amount()
                    )));
                }
            }
        }
    }

    Ok(true)
}
