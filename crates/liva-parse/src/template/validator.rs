//! Template validation logic for StatementTemplate.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use super::schema::{AmountMode, StatementTemplate};
use crate::error::ParseError;
use regex::Regex;

/// Validates a StatementTemplate for structural correctness and syntax validity.
pub fn validate_template(template: &StatementTemplate) -> Result<(), ParseError> {
    // 1. Validate Bank Code and Name
    if template.bank_code.trim().is_empty() {
        return Err(ParseError::ValidationError(
            "bank_code cannot be empty".to_string(),
        ));
    }
    if template.name.trim().is_empty() {
        return Err(ParseError::ValidationError(
            "template name cannot be empty".to_string(),
        ));
    }

    // 2. Validate Currency
    if template.currency.trim().len() != 3 {
        return Err(ParseError::ValidationError(format!(
            "Invalid ISO-4217 currency code: '{}'",
            template.currency
        )));
    }

    // 3. Validate Sniffing Regexes
    for rule in &template.sniffing.content_rules.rules {
        Regex::new(&rule.pattern).map_err(|e| {
            ParseError::ValidationError(format!(
                "Invalid sniffing regex '{}': {e}",
                rule.pattern
            ))
        })?;
    }

    // 4. Validate Metadata Patterns
    if let Some(ref field) = template.metadata.fields.account_number {
        for pat in &field.patterns {
            Regex::new(pat).map_err(|e| {
                ParseError::ValidationError(format!("Invalid account_number regex '{pat}': {e}"))
            })?;
        }
    }
    if let Some(ref field) = template.metadata.fields.account_name {
        for pat in &field.patterns {
            Regex::new(pat).map_err(|e| {
                ParseError::ValidationError(format!("Invalid account_name regex '{pat}': {e}"))
            })?;
        }
    }
    if let Some(ref field) = template.metadata.fields.opening_balance {
        for pat in &field.patterns {
            Regex::new(pat).map_err(|e| {
                ParseError::ValidationError(format!("Invalid opening_balance regex '{pat}': {e}"))
            })?;
        }
    }
    if let Some(ref field) = template.metadata.fields.closing_balance {
        for pat in &field.patterns {
            Regex::new(pat).map_err(|e| {
                ParseError::ValidationError(format!("Invalid closing_balance regex '{pat}': {e}"))
            })?;
        }
    }

    // 5. Validate Table Column Config & Amount Mode
    match template.table.columns.amount_mode {
        AmountMode::SeparateDebitCredit => {
            let has_debit = template.table.columns.debit.as_ref().is_some_and(|d| {
                !d.aliases.is_empty() || d.fixed_index.is_some() || d.xml_tag.is_some()
            });
            let has_credit = template.table.columns.credit.as_ref().is_some_and(|c| {
                !c.aliases.is_empty() || c.fixed_index.is_some() || c.xml_tag.is_some()
            });
            let has_swift = template.table.columns.swift_statement_line.is_some();
            if !has_swift && (!has_debit && !has_credit) {
                return Err(ParseError::ValidationError(
                    "AmountMode 'separate_debit_credit' requires debit or credit column definition"
                        .to_string(),
                ));
            }
        }
        AmountMode::SingleWithIndicator => {
            let has_amt = template.table.columns.amount.as_ref().is_some_and(|a| {
                !a.aliases.is_empty() || a.fixed_index.is_some() || a.xml_tag.is_some()
            });
            let has_ind = template.table.columns.indicator.as_ref().is_some_and(|i| {
                !i.aliases.is_empty() || i.xml_tag.is_some()
            });
            let has_swift = template.table.columns.swift_statement_line.is_some();
            if !has_swift && (!has_amt || !has_ind) {
                return Err(ParseError::ValidationError(
                    "AmountMode 'single_with_indicator' requires both amount and indicator column definitions"
                        .to_string(),
                ));
            }
        }
        AmountMode::SignedAmount => {
            let has_amt = template.table.columns.amount.as_ref().is_some_and(|a| {
                !a.aliases.is_empty() || a.fixed_index.is_some() || a.xml_tag.is_some()
            });
            let has_swift = template.table.columns.swift_statement_line.is_some();
            if !has_swift && !has_amt {
                return Err(ParseError::ValidationError(
                    "AmountMode 'signed_amount' requires amount column definition".to_string(),
                ));
            }
        }
    }

    // 6. Validate Stop Conditions Regexes
    for stop_pat in &template.table.row_filters.stop_conditions {
        Regex::new(stop_pat).map_err(|e| {
            ParseError::ValidationError(format!("Invalid stop_condition regex '{stop_pat}': {e}"))
        })?;
    }

    // 7. Validate Fallback Regexes
    if let Some(ref ref_col) = template.table.columns.ref_code {
        for pat in &ref_col.fallback_from_narration_regex {
            Regex::new(pat).map_err(|e| {
                ParseError::ValidationError(format!("Invalid ref_code fallback regex '{pat}': {e}"))
            })?;
        }
    }

    // 8. Validate Separators
    let t_sep = &template.formatting.number.thousand_separator;
    let d_sep = &template.formatting.number.decimal_separator;
    if t_sep == d_sep && t_sep != "none" && t_sep != "auto" {
        return Err(ParseError::ValidationError(format!(
            "Thousand separator and decimal separator cannot be the same ('{t_sep}')"
        )));
    }

    Ok(())
}
