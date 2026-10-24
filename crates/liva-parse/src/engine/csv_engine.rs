//! Generic CSV statement parser driven by StatementTemplate DSL.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::engine::date_normalizer::parse_banking_date;
use crate::engine::invariant::verify_statement_invariants;
use crate::engine::number_normalizer::parse_dsl_amount;
use crate::error::ParseError;
use crate::template::schema::{AmountMode, FieldExtractor, StatementTemplate};
use crate::{CanonicalStatement, CanonicalTx};
use liva_ledger::PostingType;
use liva_money::{Currency, Money};
use regex::Regex;

/// Parses raw CSV bytes into a CanonicalStatement using the provided DSL template.
pub fn parse_csv_statement(
    raw: &[u8],
    _filename: &str,
    template: &StatementTemplate,
) -> Result<CanonicalStatement, ParseError> {
    // 1. Decode text
    let mut text = String::from_utf8_lossy(raw).to_string();
    if template.container_options.csv.strip_bom {
        if text.starts_with('\u{feff}') {
            text.remove(0);
        }
    }

    let currency = template
        .currency
        .parse::<Currency>()
        .unwrap_or(Currency::VND);

    // 2. Resolve delimiter
    let lines: Vec<&str> = text.lines().collect();
    let delimiter = resolve_delimiter(&lines, &template.container_options.csv.delimiter);

    // 3. Extract metadata
    let account_number = extract_meta_field(&lines, template.metadata.fields.account_number.as_ref(), template.metadata.scan_range.header_rows);
    let _account_name = extract_meta_field(&lines, template.metadata.fields.account_name.as_ref(), template.metadata.scan_range.header_rows);
    let opening_balance = extract_balance_field(&lines, template.metadata.fields.opening_balance.as_ref(), template.metadata.scan_range.header_rows, template, currency);
    let closing_balance = extract_balance_field(&lines, template.metadata.fields.closing_balance.as_ref(), template.metadata.scan_range.header_rows, template, currency);

    // 4. Detect table header row
    let header_idx = detect_header_row(&lines, delimiter, template)?;
    let col_map = map_columns(&lines[header_idx], delimiter, template);

    // Stop conditions regexes
    let stop_regexes: Vec<Regex> = template
        .table
        .row_filters
        .stop_conditions
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();

    // 5. Parse data rows
    let mut transactions: Vec<CanonicalTx> = Vec::new();
    let mut row_counter = 0;
    let mut last_tx_date: Option<i64> = None;

    for &line in lines.iter().skip(header_idx + 1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check stop conditions
        let is_stop = stop_regexes.iter().any(|re| re.is_match(trimmed));
        if is_stop {
            break;
        }

        let cells: Vec<&str> = trimmed.split(delimiter).map(|c| c.trim()).collect();
        if cells.is_empty() {
            continue;
        }

        // Check stop condition on first cell
        if let Some(first) = cells.first() {
            if stop_regexes.iter().any(|re| re.is_match(first)) {
                break;
            }
        }

        // Extract date
        let date_str = col_map.date_idx.and_then(|idx| cells.get(idx)).copied().unwrap_or("");
        let tx_date = match parse_banking_date(date_str, &template.formatting.date.formats) {
            Ok(ts) => {
                last_tx_date = Some(ts);
                ts
            }
            Err(_) => {
                if template.table.continuation_lines.enabled && !transactions.is_empty() {
                    // Possible continuation line: append narration
                    let narr = col_map.narration_idx.and_then(|idx| cells.get(idx)).copied().unwrap_or("");
                    if !narr.is_empty() {
                        if let Some(last_tx) = transactions.last_mut() {
                            last_tx.narration.push_str(&template.table.continuation_lines.delimiter);
                            last_tx.narration.push_str(narr);
                        }
                    }
                    continue;
                }
                if template.table.row_filters.skip_if_date_missing {
                    continue;
                } else if let Some(last) = last_tx_date {
                    last
                } else {
                    continue;
                }
            }
        };

        // Extract amount & posting type
        let (posting_type, amount) = match template.table.columns.amount_mode {
            AmountMode::SeparateDebitCredit => {
                let debit_str = col_map.debit_idx.and_then(|i| cells.get(i)).copied().unwrap_or("");
                let credit_str = col_map.credit_idx.and_then(|i| cells.get(i)).copied().unwrap_or("");

                let debit_amt = parse_dsl_amount(
                    debit_str,
                    &template.formatting.number.thousand_separator,
                    &template.formatting.number.decimal_separator,
                    template.formatting.number.decimal_places,
                    &template.formatting.number.strip_tokens,
                    currency,
                ).unwrap_or(Money::zero(currency));

                let credit_amt = parse_dsl_amount(
                    credit_str,
                    &template.formatting.number.thousand_separator,
                    &template.formatting.number.decimal_separator,
                    template.formatting.number.decimal_places,
                    &template.formatting.number.strip_tokens,
                    currency,
                ).unwrap_or(Money::zero(currency));

                if credit_amt.amount() > 0 {
                    (PostingType::Credit, credit_amt)
                } else if debit_amt.amount() > 0 {
                    (PostingType::Debit, debit_amt)
                } else {
                    continue; // 0 or empty amount row
                }
            }
            AmountMode::SingleWithIndicator => {
                let amt_str = col_map.amount_idx.and_then(|i| cells.get(i)).copied().unwrap_or("");
                let ind_str = col_map.indicator_idx.and_then(|i| cells.get(i)).copied().unwrap_or("").to_uppercase();

                let amt = parse_dsl_amount(
                    amt_str,
                    &template.formatting.number.thousand_separator,
                    &template.formatting.number.decimal_separator,
                    template.formatting.number.decimal_places,
                    &template.formatting.number.strip_tokens,
                    currency,
                ).unwrap_or(Money::zero(currency));

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
                let amt_str = col_map.amount_idx.and_then(|i| cells.get(i)).copied().unwrap_or("");
                let amt = parse_dsl_amount(
                    amt_str,
                    &template.formatting.number.thousand_separator,
                    &template.formatting.number.decimal_separator,
                    template.formatting.number.decimal_places,
                    &template.formatting.number.strip_tokens,
                    currency,
                ).unwrap_or(Money::zero(currency));

                if amt.amount() < 0 {
                    (PostingType::Debit, Money::from_minor(-amt.amount(), currency))
                } else {
                    (PostingType::Credit, amt)
                }
            }
        };

        // Extract balance after
        let balance_after = col_map.balance_idx.and_then(|i| cells.get(i)).and_then(|s| {
            parse_dsl_amount(
                s,
                &template.formatting.number.thousand_separator,
                &template.formatting.number.decimal_separator,
                template.formatting.number.decimal_places,
                &template.formatting.number.strip_tokens,
                currency,
            ).ok()
        });

        // Narration
        let narration = col_map
            .narration_idx
            .and_then(|i| cells.get(i))
            .copied()
            .unwrap_or("")
            .trim()
            .to_string();

        // Ref code
        let mut ref_code = col_map
            .ref_idx
            .and_then(|i| cells.get(i))
            .copied()
            .unwrap_or("")
            .trim()
            .to_string();

        // Fallback ref from narration if empty
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
            .and_then(|i| cells.get(i))
            .copied()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let counterparty_account = col_map
            .counterparty_acc_idx
            .and_then(|i| cells.get(i))
            .copied()
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
            tx_timestamp: tx_date,
        });
    }

    // Infer closing balance if missing
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

struct ColumnMapping {
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

fn map_columns(header_line: &str, delimiter: char, template: &StatementTemplate) -> ColumnMapping {
    let cells: Vec<String> = header_line
        .split(delimiter)
        .map(|s| s.trim().to_lowercase())
        .collect();

    let mut map = ColumnMapping {
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
        if matches_aliases(cell, template.table.columns.date.as_ref()) {
            map.date_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.value_date.as_ref()) {
            map.value_date_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.ref_code.as_ref()) {
            map.ref_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.debit.as_ref()) {
            map.debit_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.credit.as_ref()) {
            map.credit_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.amount.as_ref()) {
            map.amount_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.balance_after.as_ref()) {
            map.balance_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.narration.as_ref()) {
            map.narration_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.counterparty_name.as_ref()) {
            map.counterparty_name_idx = Some(idx);
        } else if matches_aliases(cell, template.table.columns.counterparty_account.as_ref()) {
            map.counterparty_acc_idx = Some(idx);
        }
    }

    // Check indicator
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

fn matches_aliases(cell: &str, spec: Option<&crate::template::schema::ColumnSpec>) -> bool {
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

fn detect_header_row(lines: &[&str], delimiter: char, template: &StatementTemplate) -> Result<usize, ParseError> {
    if let Some(fixed_idx) = template.table.header_detection.fixed_row_index {
        if fixed_idx < lines.len() {
            return Ok(fixed_idx);
        }
    }

    let stop_row = template.table.header_detection.stop_at_row.min(lines.len());
    let min_matches = template.table.header_detection.min_matching_columns;

    for (idx, line) in lines.iter().take(stop_row).enumerate() {
        let cells: Vec<String> = line.split(delimiter).map(|s| s.trim().to_lowercase()).collect();
        let mut matches = 0;

        for cell in &cells {
            if matches_aliases(cell, template.table.columns.date.as_ref())
                || matches_aliases(cell, template.table.columns.ref_code.as_ref())
                || matches_aliases(cell, template.table.columns.debit.as_ref())
                || matches_aliases(cell, template.table.columns.credit.as_ref())
                || matches_aliases(cell, template.table.columns.amount.as_ref())
                || matches_aliases(cell, template.table.columns.balance_after.as_ref())
                || matches_aliases(cell, template.table.columns.narration.as_ref())
                || matches_aliases(cell, template.table.columns.counterparty_name.as_ref())
            {
                matches += 1;
            }
        }

        if matches >= min_matches {
            return Ok(idx);
        }
    }

    Err(ParseError::CsvError(format!(
        "Could not detect table header row in CSV statement for bank {}",
        template.bank_code
    )))
}

fn resolve_delimiter(lines: &[&str], configured: &str) -> char {
    match configured {
        ";" => ';',
        "," => ',',
        "\t" => '\t',
        "|" => '|',
        _ => {
            // Auto detect
            let mut comma = 0;
            let mut semi = 0;
            let mut tab = 0;
            let mut pipe = 0;

            for line in lines.iter().take(20) {
                comma += line.matches(',').count();
                semi += line.matches(';').count();
                tab += line.matches('\t').count();
                pipe += line.matches('|').count();
            }

            if semi >= comma && semi >= tab && semi >= pipe && semi > 0 {
                ';'
            } else if tab >= comma && tab >= semi && tab >= pipe && tab > 0 {
                '\t'
            } else if pipe >= comma && pipe >= semi && pipe >= tab && pipe > 0 {
                '|'
            } else {
                ','
            }
        }
    }
}

fn extract_meta_field(lines: &[&str], extractor: Option<&FieldExtractor>, scan_rows: usize) -> Option<String> {
    let ext = extractor?;
    for line in lines.iter().take(scan_rows) {
        for pat in &ext.patterns {
            if let Ok(re) = Regex::new(pat) {
                if let Some(caps) = re.captures(line) {
                    if let Some(m) = caps.get(1) {
                        return Some(m.as_str().trim().to_string());
                    }
                }
            }
        }
        for kw in &ext.keywords {
            let kw_lower = kw.to_lowercase();
            let line_lower = line.to_lowercase();
            if line_lower.contains(&kw_lower) {
                if let Some(idx) = line_lower.find(&kw_lower) {
                    let rest = &line[idx + kw.len()..];
                    let val = rest.trim_start_matches(|c| c == ':' || c == ';' || c == '\t' || c == ' ');
                    let val = val.split(';').next().unwrap_or("").trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_balance_field(
    lines: &[&str],
    extractor: Option<&FieldExtractor>,
    scan_rows: usize,
    template: &StatementTemplate,
    currency: Currency,
) -> Option<Money> {
    let raw_val = extract_meta_field(lines, extractor, scan_rows)?;
    parse_dsl_amount(
        &raw_val,
        &template.formatting.number.thousand_separator,
        &template.formatting.number.decimal_separator,
        template.formatting.number.decimal_places,
        &template.formatting.number.strip_tokens,
        currency,
    ).ok()
}
