//! Generic Excel (XLSX / XLS) statement parser driven by StatementTemplate DSL.
//! Uses `calamine` for zero-allocation tabular reading and strictly enforces
//! `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::engine::date_normalizer::parse_banking_date;
use crate::engine::invariant::verify_statement_invariants;
use crate::engine::number_normalizer::{cell_to_dsl_amount, cell_to_string, parse_dsl_amount};
use crate::error::ParseError;
use crate::template::schema::{AmountMode, ColumnSpec, FieldExtractor, StatementTemplate};
use crate::{CanonicalStatement, CanonicalTx};
use calamine::{Data, Reader, open_workbook_auto_from_rs};
use liva_ledger::PostingType;
use liva_money::{Currency, Money};
use regex::Regex;
use std::io::Cursor;

/// Parses raw Excel (.xlsx / .xls) bytes into a CanonicalStatement using the provided DSL template.
pub fn parse_excel_statement(
    raw: &[u8],
    filename: &str,
    template: &StatementTemplate,
) -> Result<CanonicalStatement, ParseError> {
    let mut workbook = open_workbook_auto_from_rs(Cursor::new(raw))
        .map_err(|e| ParseError::ExcelError(format!("Failed to open Excel workbook '{filename}': {e}")))?;

    // 1. Select sheet
    let sheet_range = match template.container_options.excel.sheet_selection.as_str() {
        "by_name" => {
            let pattern = template
                .container_options
                .excel
                .sheet_name_pattern
                .as_deref()
                .unwrap_or("");
            let re = Regex::new(pattern).map_err(|e| ParseError::ValidationError(e.to_string()))?;
            let sheet_names = workbook.sheet_names();
            let matched_name = sheet_names.iter().find(|name| re.is_match(name)).cloned();
            let target_name = matched_name.ok_or_else(|| {
                ParseError::ExcelError(format!("No sheet found matching pattern '{pattern}' in {filename}"))
            })?;
            workbook
                .worksheet_range(&target_name)
                .map_err(|e| ParseError::ExcelError(format!("Failed to read sheet '{target_name}': {e}")))?
        }
        "by_index" => {
            let idx = template.container_options.excel.sheet_index;
            workbook
                .worksheet_range_at(idx)
                .ok_or_else(|| ParseError::ExcelError(format!("Sheet index {idx} not found in {filename}")))?
                .map_err(|e| ParseError::ExcelError(format!("Failed to read sheet index {idx}: {e}")))?
        }
        _ => {
            // First sheet
            workbook
                .worksheet_range_at(0)
                .ok_or_else(|| ParseError::ExcelError(format!("Workbook {filename} has no sheets")))?
                .map_err(|e| ParseError::ExcelError(format!("Failed to read first sheet: {e}")))?
        }
    };

    let currency = template
        .currency
        .parse::<Currency>()
        .unwrap_or(Currency::VND);

    let rows: Vec<&[Data]> = sheet_range.rows().collect();
    if rows.is_empty() {
        return Err(ParseError::ExcelError("Excel worksheet is empty".to_string()));
    }

    // 2. Extract metadata
    let account_number = extract_excel_meta(&rows, template.metadata.fields.account_number.as_ref(), template.metadata.scan_range.header_rows);
    let _account_name = extract_excel_meta(&rows, template.metadata.fields.account_name.as_ref(), template.metadata.scan_range.header_rows);
    let opening_balance = extract_excel_balance(&rows, template.metadata.fields.opening_balance.as_ref(), template.metadata.scan_range.header_rows, template, currency);
    let mut closing_balance = extract_excel_balance(&rows, template.metadata.fields.closing_balance.as_ref(), template.metadata.scan_range.header_rows, template, currency);

    // Scan bottom footer rows for closing balance if not found in header
    if closing_balance.is_none() {
        let footer_take = template.metadata.scan_range.footer_rows.min(rows.len());
        let footer_rows: Vec<&[Data]> = rows.iter().rev().take(footer_take).cloned().collect();
        closing_balance = extract_excel_balance(&footer_rows, template.metadata.fields.closing_balance.as_ref(), footer_take, template, currency);
    }

    // 3. Detect table header row
    let header_idx = detect_excel_header_row(&rows, template)?;
    let col_map = map_excel_columns(rows[header_idx], template);

    // Stop conditions regexes
    let stop_regexes: Vec<Regex> = template
        .table
        .row_filters
        .stop_conditions
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();

    // 4. Parse transaction rows
    let mut transactions = Vec::new();
    let mut row_counter = 0;
    let mut last_valid_date: Option<i64> = None;

    let thousand_sep = &template.formatting.number.thousand_separator;
    let decimal_sep = &template.formatting.number.decimal_separator;
    let decimal_places = template.formatting.number.decimal_places;
    let strip_tokens = &template.formatting.number.strip_tokens;

    for row in rows.iter().skip(header_idx + 1) {
        if row.is_empty() {
            continue;
        }

        // Check stop conditions on first column
        let first_cell_str = row.first().map(cell_to_string).unwrap_or_default();
        if stop_regexes.iter().any(|re| re.is_match(&first_cell_str)) {
            break;
        }
        let full_row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
        if stop_regexes.iter().any(|re| re.is_match(&full_row_str)) {
            break;
        }

        // Resolve transaction date
        let date_cell_str = col_map.date_idx.and_then(|idx| row.get(idx)).map(cell_to_string).unwrap_or_default();
        let tx_date = match parse_banking_date(&date_cell_str, &template.formatting.date.formats) {
            Ok(d) => {
                last_valid_date = Some(d);
                d
            }
            Err(_) => {
                if template.container_options.excel.merged_cells == "forward_fill" {
                    match last_valid_date {
                        Some(d) => d,
                        None => continue, // Cannot resolve date
                    }
                } else if template.table.row_filters.skip_if_date_missing {
                    continue;
                } else {
                    continue;
                }
            }
        };

        let val_date = col_map
            .value_date_idx
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .as_deref()
            .and_then(|s| parse_banking_date(s, &template.formatting.date.formats).ok())
            .unwrap_or(tx_date);

        // Resolve amount and posting type
        let (posting_type, amount) = match template.table.columns.amount_mode {
            AmountMode::SeparateDebitCredit => {
                let debit_amt = col_map
                    .debit_idx
                    .and_then(|idx| row.get(idx))
                    .and_then(|c| cell_to_dsl_amount(c, thousand_sep, decimal_sep, decimal_places, strip_tokens, currency).ok().flatten())
                    .unwrap_or(Money::zero(currency));

                let credit_amt = col_map
                    .credit_idx
                    .and_then(|idx| row.get(idx))
                    .and_then(|c| cell_to_dsl_amount(c, thousand_sep, decimal_sep, decimal_places, strip_tokens, currency).ok().flatten())
                    .unwrap_or(Money::zero(currency));

                if credit_amt.amount() > 0 {
                    (PostingType::Credit, credit_amt)
                } else if debit_amt.amount() > 0 {
                    (PostingType::Debit, debit_amt)
                } else {
                    continue; // 0 or empty amount
                }
            }
            AmountMode::SingleWithIndicator => {
                let amt = col_map
                    .amount_idx
                    .and_then(|idx| row.get(idx))
                    .and_then(|c| cell_to_dsl_amount(c, thousand_sep, decimal_sep, decimal_places, strip_tokens, currency).ok().flatten())
                    .unwrap_or(Money::zero(currency));

                let ind_str = col_map
                    .indicator_idx
                    .and_then(|idx| row.get(idx))
                    .map(cell_to_string)
                    .unwrap_or_default()
                    .to_uppercase();

                let is_credit = if let Some(ref ind_spec) = template.table.columns.indicator {
                    ind_spec.credit_tokens.iter().any(|t| ind_str.contains(&t.to_uppercase()))
                } else {
                    ind_str.contains("CR") || ind_str.contains("CÓ")
                };

                let post_type = if is_credit {
                    PostingType::Credit
                } else {
                    PostingType::Debit
                };
                (post_type, amt)
            }
            AmountMode::SignedAmount => {
                let amt = col_map
                    .amount_idx
                    .and_then(|idx| row.get(idx))
                    .and_then(|c| cell_to_dsl_amount(c, thousand_sep, decimal_sep, decimal_places, strip_tokens, currency).ok().flatten())
                    .unwrap_or(Money::zero(currency));

                if amt.amount() < 0 {
                    (PostingType::Debit, Money::from_minor(-amt.amount(), currency))
                } else {
                    (PostingType::Credit, amt)
                }
            }
        };

        // Balance after
        let balance_after = col_map
            .balance_idx
            .and_then(|idx| row.get(idx))
            .and_then(|c| cell_to_dsl_amount(c, thousand_sep, decimal_sep, decimal_places, strip_tokens, currency).ok().flatten());

        // Narration
        let narration = col_map
            .narration_idx
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .unwrap_or_default()
            .trim()
            .to_string();

        // Ref code
        let mut ref_code = col_map
            .ref_idx
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .unwrap_or_default()
            .trim()
            .to_string();

        if ref_code.is_empty() {
            if let Some(ref ref_spec) = template.table.columns.ref_code {
                for pat in &ref_spec.fallback_from_narration_regex {
                    if let Ok(re) = Regex::new(pat) {
                        if let Some(caps) = re.captures(&narration) {
                            if let Some(m) = caps.get(1) {
                                ref_code = m.as_str().to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }
        if ref_code.is_empty() {
            row_counter += 1;
            ref_code = format!("{}-{:04}", template.bank_code, row_counter);
        } else {
            row_counter += 1;
        }

        // Counterparty
        let counterparty_name = col_map
            .counterparty_name_idx
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let counterparty_account = col_map
            .counterparty_acc_idx
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        transactions.push(CanonicalTx {
            row_id: row_counter,
            bank_code: template.bank_code.clone(),
            ref_code,
            posting_type,
            amount,
            balance_after,
            counterparty_name,
            counterparty_account,
            narration,
            tx_timestamp: val_date,
        });
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

struct ExcelColumnMapping {
    date_idx: Option<usize>,
    value_date_idx: Option<usize>,
    ref_idx: Option<usize>,
    debit_idx: Option<usize>,
    credit_idx: Option<usize>,
    amount_idx: Option<usize>,
    indicator_idx: Option<usize>,
    balance_idx: Option<usize>,
    narration_idx: Option<usize>,
    counterparty_name_idx: Option<usize>,
    counterparty_acc_idx: Option<usize>,
}

fn map_excel_columns(header_row: &[Data], template: &StatementTemplate) -> ExcelColumnMapping {
    let cells: Vec<String> = header_row
        .iter()
        .map(cell_to_string)
        .map(|s| s.trim().to_lowercase())
        .collect();

    let mut map = ExcelColumnMapping {
        date_idx: None,
        value_date_idx: None,
        ref_idx: None,
        debit_idx: None,
        credit_idx: None,
        amount_idx: None,
        indicator_idx: None,
        balance_idx: None,
        narration_idx: None,
        counterparty_name_idx: None,
        counterparty_acc_idx: None,
    };

    for (idx, cell) in cells.iter().enumerate() {
        if matches_excel_alias(cell, template.table.columns.date.as_ref()) {
            map.date_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.value_date.as_ref()) {
            map.value_date_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.ref_code.as_ref()) {
            map.ref_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.debit.as_ref()) {
            map.debit_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.credit.as_ref()) {
            map.credit_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.amount.as_ref()) {
            map.amount_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.balance_after.as_ref()) {
            map.balance_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.narration.as_ref()) {
            map.narration_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.counterparty_name.as_ref()) {
            map.counterparty_name_idx = Some(idx);
        } else if matches_excel_alias(cell, template.table.columns.counterparty_account.as_ref()) {
            map.counterparty_acc_idx = Some(idx);
        }
    }

    if let Some(ref ind_spec) = template.table.columns.indicator {
        for (idx, cell) in cells.iter().enumerate() {
            if ind_spec.aliases.iter().any(|a| cell.contains(&a.to_lowercase())) {
                map.indicator_idx = Some(idx);
                break;
            }
        }
    }

    map
}

fn matches_excel_alias(cell: &str, spec: Option<&ColumnSpec>) -> bool {
    let cell_trimmed = cell.trim();
    if cell_trimmed.is_empty() {
        return false;
    }
    if let Some(s) = spec {
        s.aliases.iter().any(|alias| {
            let a = alias.trim().to_lowercase();
            if a.is_empty() {
                return false;
            }
            cell_trimmed == a || cell_trimmed.contains(&a) || (cell_trimmed.len() >= 3 && a.contains(cell_trimmed))
        })
    } else {
        false
    }
}

fn detect_excel_header_row(rows: &[&[Data]], template: &StatementTemplate) -> Result<usize, ParseError> {
    if let Some(fixed_idx) = template.table.header_detection.fixed_row_index {
        if fixed_idx < rows.len() {
            return Ok(fixed_idx);
        }
    }

    let stop_row = template.table.header_detection.stop_at_row.min(rows.len());
    let min_matches = template.table.header_detection.min_matching_columns;

    for (idx, row) in rows.iter().take(stop_row).enumerate() {
        let cells: Vec<String> = row.iter().map(cell_to_string).map(|s| s.trim().to_lowercase()).collect();
        let mut matches = 0;

        for cell in &cells {
            if matches_excel_alias(cell, template.table.columns.date.as_ref())
                || matches_excel_alias(cell, template.table.columns.ref_code.as_ref())
                || matches_excel_alias(cell, template.table.columns.debit.as_ref())
                || matches_excel_alias(cell, template.table.columns.credit.as_ref())
                || matches_excel_alias(cell, template.table.columns.amount.as_ref())
                || matches_excel_alias(cell, template.table.columns.balance_after.as_ref())
                || matches_excel_alias(cell, template.table.columns.narration.as_ref())
                || matches_excel_alias(cell, template.table.columns.counterparty_name.as_ref())
            {
                matches += 1;
            }
        }

        if matches >= min_matches {
            return Ok(idx);
        }
    }

    Err(ParseError::ExcelError(format!(
        "Could not detect table header row in Excel statement for bank {}",
        template.bank_code
    )))
}

fn extract_excel_meta(rows: &[&[Data]], extractor: Option<&FieldExtractor>, scan_rows: usize) -> Option<String> {
    let ext = extractor?;
    for row in rows.iter().take(scan_rows) {
        let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
        for pat in &ext.patterns {
            if let Ok(re) = Regex::new(pat) {
                if let Some(caps) = re.captures(&row_str) {
                    if let Some(m) = caps.get(1) {
                        return Some(m.as_str().trim().to_string());
                    }
                }
            }
        }
        for kw in &ext.keywords {
            let kw_lower = kw.to_lowercase();
            let row_lower = row_str.to_lowercase();
            if row_lower.contains(&kw_lower) {
                if let Some(idx) = row_lower.find(&kw_lower) {
                    let rest = &row_str[idx + kw.len()..];
                    let val = rest.trim_start_matches(|c| c == ':' || c == '-' || c == ' ');
                    let val = val.split('\n').next().unwrap_or("").trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_excel_balance(
    rows: &[&[Data]],
    extractor: Option<&FieldExtractor>,
    scan_rows: usize,
    template: &StatementTemplate,
    currency: Currency,
) -> Option<Money> {
    let ext = extractor?;
    for row in rows.iter().take(scan_rows) {
        let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
        let row_lower = row_str.to_lowercase();

        let matched = ext.keywords.iter().any(|kw| row_lower.contains(&kw.to_lowercase()))
            || ext.patterns.iter().any(|pat| Regex::new(pat).map_or(false, |re| re.is_match(&row_str)));

        if matched {
            // Find numeric cell in this row or adjacent
            for cell in row.iter().rev() {
                if let Ok(Some(m)) = cell_to_dsl_amount(
                    cell,
                    &template.formatting.number.thousand_separator,
                    &template.formatting.number.decimal_separator,
                    template.formatting.number.decimal_places,
                    &template.formatting.number.strip_tokens,
                    currency,
                ) {
                    if m.amount() > 0 {
                        return Some(m);
                    }
                }
            }
            // Fallback: parse from row_str
            if let Some(raw) = extract_excel_meta(std::slice::from_ref(row), Some(ext), 1) {
                if let Ok(m) = parse_dsl_amount(
                    &raw,
                    &template.formatting.number.thousand_separator,
                    &template.formatting.number.decimal_separator,
                    template.formatting.number.decimal_places,
                    &template.formatting.number.strip_tokens,
                    currency,
                ) {
                    if m.amount() > 0 {
                        return Some(m);
                    }
                }
            }
        }
    }
    None
}
