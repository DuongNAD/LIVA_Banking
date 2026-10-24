//! Generic SWIFT MT940 / MT942 statement parser driven by StatementTemplate DSL.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::engine::date_normalizer::parse_banking_date;
use crate::engine::invariant::verify_statement_invariants;
use crate::engine::number_normalizer::parse_dsl_amount;
use crate::error::ParseError;
use crate::template::schema::StatementTemplate;
use crate::{CanonicalStatement, CanonicalTx};
use liva_ledger::PostingType;
use liva_money::{Currency, Money};

/// Parses raw SWIFT MT940 text into a CanonicalStatement using the provided DSL template.
pub fn parse_swift_statement(
    raw: &[u8],
    _filename: &str,
    template: &StatementTemplate,
) -> Result<CanonicalStatement, ParseError> {
    let text = String::from_utf8_lossy(raw);
    let currency = template
        .currency
        .parse::<Currency>()
        .unwrap_or(Currency::VND);

    let mut account_number = None;
    let mut opening_balance = None;
    let mut closing_balance = None;

    let mut transactions = Vec::new();
    let mut row_counter = 0;

    // Temporary storage for current transaction being parsed
    let mut current_tx_line: Option<ParsedSwiftLine> = None;
    let mut current_narration = String::new();

    let lines: Vec<&str> = text.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "-" || trimmed == "{5:}" {
            continue;
        }

        if let Some(val) = trimmed.strip_prefix(":25:") {
            account_number = Some(val.trim().to_string());
        } else if let Some(val) = trimmed.strip_prefix(":60F:").or_else(|| trimmed.strip_prefix(":60M:")) {
            opening_balance = parse_swift_balance(val, template, currency).ok();
        } else if let Some(val) = trimmed.strip_prefix(":62F:").or_else(|| trimmed.strip_prefix(":62M:")) {
            closing_balance = parse_swift_balance(val, template, currency).ok();
        } else if let Some(val) = trimmed.strip_prefix(":61:") {
            // Finalize previous tx if any
            if let Some(parsed_line) = current_tx_line.take() {
                row_counter += 1;
                transactions.push(build_swift_tx(row_counter, parsed_line, current_narration.clone(), template));
                current_narration.clear();
            }

            if let Ok(pl) = parse_line_61(val, template, currency) {
                current_tx_line = Some(pl);
            }
        } else if let Some(val) = trimmed.strip_prefix(":86:") {
            let val = val.trim();
            if !current_narration.is_empty() {
                current_narration.push(' ');
            }
            current_narration.push_str(val);
        } else if !trimmed.starts_with(':') && current_tx_line.is_some() {
            // Continuation line for narration (:86:)
            if !current_narration.is_empty() {
                current_narration.push(' ');
            }
            current_narration.push_str(trimmed);
        }
    }

    // Flush last tx
    if let Some(parsed_line) = current_tx_line.take() {
        row_counter += 1;
        transactions.push(build_swift_tx(row_counter, parsed_line, current_narration, template));
    }

    let mut stmt = CanonicalStatement {
        bank_code: template.bank_code.clone(),
        account_number,
        opening_balance,
        closing_balance,
        transactions,
    };

    // Verify invariants
    verify_statement_invariants(&mut stmt, template)?;

    Ok(stmt)
}

struct ParsedSwiftLine {
    date: i64,
    posting_type: PostingType,
    amount: Money,
    ref_code: String,
}

fn build_swift_tx(
    row_id: usize,
    line: ParsedSwiftLine,
    narration: String,
    template: &StatementTemplate,
) -> CanonicalTx {
    let final_ref = if line.ref_code.is_empty() || line.ref_code == "NONREF" {
        format!("{}-{:04}", template.bank_code, row_id)
    } else {
        line.ref_code
    };

    CanonicalTx {
        row_id,
        bank_code: template.bank_code.clone(),
        ref_code: final_ref,
        posting_type: line.posting_type,
        amount: line.amount,
        balance_after: None,
        counterparty_name: None,
        counterparty_account: None,
        narration,
        tx_timestamp: line.date,
    }
}

/// Parses SWIFT balance line (:60F:, :62F:), e.g. "C260801VND785600000,"
fn parse_swift_balance(
    raw: &str,
    template: &StatementTemplate,
    default_currency: Currency,
) -> Result<Money, ParseError> {
    if raw.len() < 10 {
        return Err(ParseError::SwiftError("Balance string too short".to_string()));
    }

    let ind_char = &raw[0..1];
    let is_debit = ind_char == "D";

    // 6 chars date: raw[1..7]
    // 3 chars currency: raw[7..10]
    let cur_str = &raw[7..10];
    let currency = cur_str.parse::<Currency>().unwrap_or(default_currency);

    let amount_str = &raw[10..];
    let mut money = parse_dsl_amount(
        amount_str,
        &template.formatting.number.thousand_separator,
        &template.formatting.number.decimal_separator,
        template.formatting.number.decimal_places,
        &template.formatting.number.strip_tokens,
        currency,
    )?;

    if is_debit && money.amount() > 0 {
        money = Money::from_minor(-money.amount(), currency);
    }

    Ok(money)
}

/// Parses SWIFT Statement Line (:61:), e.g. "260801CR45000000,NTRFREF12345//FT67890"
fn parse_line_61(
    raw: &str,
    template: &StatementTemplate,
    currency: Currency,
) -> Result<ParsedSwiftLine, ParseError> {
    if raw.len() < 10 {
        return Err(ParseError::SwiftError("Tag 61 line too short".to_string()));
    }

    // 1. Value Date: YYMMDD (pos 0..6)
    let date_str = &raw[0..6];
    let date = parse_banking_date(date_str, &["%y%m%d".to_string()])
        .unwrap_or(1726400000);

    let mut cursor = 6;
    // Optional Entry Date (MMDD) if 4 digits follow
    let remaining = &raw[cursor..];
    if remaining.len() >= 4 && remaining[..4].chars().all(|c| c.is_ascii_digit()) {
        // Skip entry date
        cursor += 4;
    }

    // 2. Credit/Debit Indicator: "CR", "DR", "RC", "RD", "C", "D"
    let ind_str = &raw[cursor..];
    let (posting_type, ind_len) = if ind_str.starts_with("CR") {
        (PostingType::Credit, 2)
    } else if ind_str.starts_with("DR") {
        (PostingType::Debit, 2)
    } else if ind_str.starts_with("RC") {
        (PostingType::Credit, 2)
    } else if ind_str.starts_with("RD") {
        (PostingType::Debit, 2)
    } else if ind_str.starts_with('C') {
        (PostingType::Credit, 1)
    } else if ind_str.starts_with('D') {
        (PostingType::Debit, 1)
    } else {
        (PostingType::Credit, 1)
    };
    cursor += ind_len;

    // Optional Funds Code (1 letter)
    let after_ind = &raw[cursor..];
    if after_ind.starts_with(|c: char| c.is_ascii_alphabetic() && !c.is_ascii_digit()) && !after_ind.starts_with("N") && !after_ind.starts_with("F") {
        cursor += 1;
    }

    // 3. Amount: digits and comma decimal separator until letter
    let amt_start = cursor;
    let mut amt_end = cursor;
    for c in raw[cursor..].chars() {
        if c.is_ascii_digit() || c == ',' || c == '.' {
            amt_end += c.len_utf8();
        } else {
            break;
        }
    }
    let amt_str = &raw[amt_start..amt_end];
    cursor = amt_end;

    let amount = parse_dsl_amount(
        amt_str,
        "none",
        ",",
        template.formatting.number.decimal_places,
        &[],
        currency,
    )?;

    // 4. Transaction Code (4 chars) + Reference (16 chars)
    let rest = &raw[cursor..];
    let ref_code = if rest.len() >= 4 {
        let after_code = &rest[4..];
        if let Some(slash_idx) = after_code.find("//") {
            after_code[..slash_idx].trim().to_string()
        } else {
            after_code.trim().to_string()
        }
    } else {
        String::new()
    };

    Ok(ParsedSwiftLine {
        date,
        posting_type,
        amount,
        ref_code,
    })
}
