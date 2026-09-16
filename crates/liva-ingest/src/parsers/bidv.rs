//! BIDV PDF statement parser.
//! Uses 2D spatial clustering (|Δy| <= 2.5 pt / 2500 mpt), multi-line narration concatenation,
//! and signed debit handling without floating point arithmetic.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use crate::models::{ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser};
use liva_normalize::{normalize_datetime, parse_monetary_amount, BankIdentifier};
use lopdf::content::Content;
use lopdf::{Document, Object};

pub struct BidvParser;

#[derive(Debug, Clone)]
struct TextElement {
    x_mpt: i64,
    y_mpt: i64,
    text: String,
}

impl StatementParser for BidvParser {
    fn can_parse(&self, container: ContainerFormat, bank: BankIdentifier) -> bool {
        container == ContainerFormat::Pdf
            && (bank == BankIdentifier::Bidv || bank == BankIdentifier::Unknown)
    }

    fn parse(&self, bytes: &[u8], _filename: &str) -> Result<RawStatementRecord, IngestError> {
        let doc = Document::load_mem(bytes)
            .map_err(|e| IngestError::Pdf(format!("Failed to load PDF bytes: {e}")))?;

        let pages = doc.get_pages();
        if pages.is_empty() {
            return Err(IngestError::InvalidStructure("PDF contains no pages".to_string()));
        }

        let mut account_number: Option<String> = None;
        let mut account_name: Option<String> = None;
        let mut global_opening_balance: Option<u64> = None;
        let mut global_closing_balance: Option<u64> = None;

        let mut all_transactions: Vec<RawTransactionRecord> = Vec::new();

        let mut x_debit_mpt: Option<i64> = None;
        let mut x_credit_mpt: Option<i64> = None;
        let mut x_balance_mpt: Option<i64> = None;

        for (&page_num, &page_id) in &pages {
            let mut elements = extract_page_elements(&doc, page_id);
            if elements.is_empty() {
                if let Ok(text) = doc.extract_text(&[page_num]) {
                    elements = text_to_synthetic_elements(&text);
                }
            }

            // Group into 2D lines with spatial clustering delta_y <= 2500 mpt (2.5 pt)
            let lines = group_elements_into_lines(elements);
            let mut in_table = false;
            let mut page_opening_balance: Option<u64> = None;

            for line in &lines {
                let line_text: String = line
                    .iter()
                    .map(|e| e.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let line_lower = line_text.to_lowercase();

                if account_number.is_none() && (line_lower.contains("số tài khoản") || line_lower.contains("account no")) {
                    if let Some(pos) = line_text.find(':') {
                        let digits: String = line_text[pos + 1..].chars().filter(|c| c.is_ascii_digit()).collect();
                        if !digits.is_empty() {
                            account_number = Some(digits);
                        }
                    }
                }

                if account_name.is_none() && (line_lower.contains("tên khách hàng") || line_lower.contains("customer name")) {
                    if let Some(pos) = line_text.find(':') {
                        let name = line_text[pos + 1..].trim().to_string();
                        if !name.is_empty() {
                            account_name = Some(name);
                        }
                    }
                }

                if line_lower.contains("số dư đầu kỳ") || line_lower.contains("opening balance") {
                    if let Some(amt) = extract_amount_from_line(line) {
                        page_opening_balance = Some(amt);
                        if global_opening_balance.is_none() {
                            global_opening_balance = Some(amt);
                        }
                    }
                }

                if line_lower.contains("số dư cuối kỳ") || line_lower.contains("closing balance") {
                    if let Some(amt) = extract_amount_from_line(line) {
                        global_closing_balance = Some(amt);
                    }
                }

                // Table header row calibration
                if (line_lower.contains("ngày") || line_lower.contains("date"))
                    && (line_lower.contains("chứng từ")
                        || line_lower.contains("số tiền")
                        || line_lower.contains("nợ")
                        || line_lower.contains("có"))
                {
                    in_table = true;
                    for elem in line {
                        let h = elem.text.trim().to_lowercase();
                        if h.contains("nợ") || h.contains("debit") {
                            x_debit_mpt = Some(elem.x_mpt);
                        } else if h.contains("có") || h.contains("credit") {
                            x_credit_mpt = Some(elem.x_mpt);
                        } else if h.contains("số dư") || h.contains("balance") {
                            x_balance_mpt = Some(elem.x_mpt);
                        }
                    }
                    continue;
                }

                if !in_table {
                    continue;
                }

                // Stop if footer
                if line_lower.contains("tổng phát sinh")
                    || line_lower.contains("trang ")
                    || line_lower.contains("page ")
                {
                    in_table = false;
                    continue;
                }

                let last_bal = all_transactions
                    .last()
                    .and_then(|t| t.balance_cents)
                    .or(page_opening_balance)
                    .or(global_opening_balance);

                if let Some(tx) = try_parse_bidv_row(line, x_debit_mpt, x_credit_mpt, x_balance_mpt, last_bal) {
                    all_transactions.push(tx);
                } else if let Some(last_tx) = all_transactions.last_mut() {
                    // Multi-line narration concatenation
                    let addition = line_text.trim();
                    let starts_with_date = line
                        .first()
                        .and_then(|e| normalize_datetime(&e.text).ok())
                        .is_some();
                    let has_amount = line.iter().any(|elem| parse_monetary_amount(&elem.text).is_ok());

                    if !addition.is_empty()
                        && !starts_with_date
                        && !has_amount
                        && !addition.to_lowercase().contains("tổng phát sinh")
                    {
                        last_tx.narration.push(' ');
                        last_tx.narration.push_str(addition);
                    }
                }
            }
        }

        if all_transactions.is_empty() {
            return Err(IngestError::InvalidStructure(
                "No transactions extracted from BIDV PDF".to_string(),
            ));
        }

        // Set consecutive row IDs
        for (i, tx) in all_transactions.iter_mut().enumerate() {
            tx.row_id = i + 1;
        }

        Ok(RawStatementRecord {
            bank: BankIdentifier::Bidv,
            format: ContainerFormat::Pdf,
            account_no: account_number,
            account_name,
            opening_balance: global_opening_balance,
            closing_balance: global_closing_balance,
            transactions: all_transactions,
        })
    }
}

/// Converts an Object coordinate to integer millipoints (1 pt = 1000 mpt) without floating point arithmetic.
fn object_to_millipoints(obj: &Object) -> Option<i64> {
    match obj {
        Object::Integer(i) => Some((*i).saturating_mul(1000)),
        Object::Real(f) => {
            let s = format!("{f:.3}");
            let (int_part, dec_part) = if let Some((i, d)) = s.split_once('.') {
                (i, d)
            } else {
                (s.as_str(), "000")
            };
            let sign = if int_part.starts_with('-') { -1i64 } else { 1i64 };
            let abs_int: i64 = int_part.trim_start_matches('-').parse().unwrap_or(0);
            let abs_dec: i64 = dec_part.parse().unwrap_or(0);
            let total = abs_int.saturating_mul(1000).saturating_add(abs_dec);
            Some(sign.saturating_mul(total))
        }
        _ => None,
    }
}

fn extract_page_elements(doc: &Document, page_id: (u32, u16)) -> Vec<TextElement> {
    let mut elements = Vec::new();
    let content_bytes = match doc.get_page_content(page_id) {
        Ok(b) => b,
        Err(_) => return elements,
    };

    let content = match Content::decode(&content_bytes) {
        Ok(c) => c,
        Err(_) => return elements,
    };

    let mut cur_x_mpt: i64 = 0;
    let mut cur_y_mpt: i64 = 0;

    for op in &content.operations {
        match op.operator.as_str() {
            "Tm" => {
                if op.operands.len() >= 6 {
                    if let (Some(x), Some(y)) = (
                        object_to_millipoints(&op.operands[4]),
                        object_to_millipoints(&op.operands[5]),
                    ) {
                        cur_x_mpt = x;
                        cur_y_mpt = y;
                    }
                }
            }
            "Td" | "TD" => {
                if op.operands.len() >= 2 {
                    if let (Some(dx), Some(dy)) = (
                        object_to_millipoints(&op.operands[0]),
                        object_to_millipoints(&op.operands[1]),
                    ) {
                        cur_x_mpt = cur_x_mpt.saturating_add(dx);
                        cur_y_mpt = cur_y_mpt.saturating_add(dy);
                    }
                }
            }
            "T*" => {
                // Move down approx 12 pt (12,000 mpt)
                cur_y_mpt = cur_y_mpt.saturating_sub(12_000);
            }
            "Tj" => {
                if let Some(Object::String(bytes, _)) = op.operands.first() {
                    let text = String::from_utf8_lossy(bytes).trim().to_string();
                    if !text.is_empty() {
                        elements.push(TextElement {
                            x_mpt: cur_x_mpt,
                            y_mpt: cur_y_mpt,
                            text,
                        });
                    }
                }
            }
            "TJ" => {
                if let Some(Object::Array(arr)) = op.operands.first() {
                    let mut combined = String::new();
                    for item in arr {
                        if let Object::String(bytes, _) = item {
                            combined.push_str(&String::from_utf8_lossy(bytes));
                        }
                    }
                    let trimmed = combined.trim().to_string();
                    if !trimmed.is_empty() {
                        elements.push(TextElement {
                            x_mpt: cur_x_mpt,
                            y_mpt: cur_y_mpt,
                            text: trimmed,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    elements
}

fn text_to_synthetic_elements(text: &str) -> Vec<TextElement> {
    let mut elements = Vec::new();
    let mut y_mpt: i64 = 1_000_000;
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            let mut x_mpt: i64 = 50_000;
            for token in tokens {
                elements.push(TextElement {
                    x_mpt,
                    y_mpt,
                    text: token.to_string(),
                });
                let tok_len = token.len() as i64;
                x_mpt = x_mpt.saturating_add(tok_len.saturating_mul(6000).saturating_add(8000));
            }
        }
        y_mpt = y_mpt.saturating_sub(14_000);
    }
    elements
}

/// Groups 2D elements into table lines based on vertical tolerance |y1 - y2| <= 2500 mpt (2.5 pt)
/// and sorts tokens in each line by X coordinate ascending.
fn group_elements_into_lines(mut elements: Vec<TextElement>) -> Vec<Vec<TextElement>> {
    // Sort primarily by Y descending (top to bottom), secondarily by X ascending
    elements.sort_by(|a, b| {
        b.y_mpt
            .cmp(&a.y_mpt)
            .then_with(|| a.x_mpt.cmp(&b.x_mpt))
    });

    let mut lines: Vec<Vec<TextElement>> = Vec::new();

    for elem in elements {
        let mut matched_line = false;
        for line in lines.iter_mut().rev() {
            if let Some(first) = line.first() {
                if (first.y_mpt.saturating_sub(elem.y_mpt)).abs() <= 2500 {
                    line.push(elem.clone());
                    matched_line = true;
                    break;
                }
            }
        }
        if !matched_line {
            lines.push(vec![elem]);
        }
    }

    // Sort each line horizontally by X ascending
    for line in &mut lines {
        line.sort_by_key(|e| e.x_mpt);
    }

    lines
}

fn try_parse_bidv_row(
    line: &[TextElement],
    x_debit: Option<i64>,
    x_credit: Option<i64>,
    x_balance: Option<i64>,
    last_balance: Option<u64>,
) -> Option<RawTransactionRecord> {
    if line.is_empty() {
        return None;
    }

    let first_token = &line.first()?.text;
    if normalize_datetime(first_token).is_err() {
        return None;
    }

    let mut doc_ref: Option<String> = None;
    let mut amount_cents: u64 = 0;
    let mut amount_x_mpt: i64 = 0;
    let mut is_explicit_debit = false;
    let mut balance_cents: Option<u64> = None;
    let mut narration_tokens = Vec::new();

    let mut found_amount = false;
    let bal_thresh_x = x_balance.unwrap_or(360_000);

    for (idx, elem) in line.iter().enumerate().skip(1) {
        let token = &elem.text;
        if idx == 1
            && (token.starts_with("FT") || (token.len() >= 6 && token.chars().all(|c| c.is_alphanumeric())))
        {
            doc_ref = Some(token.clone());
            continue;
        }

        if let Ok(parsed_amt) = parse_monetary_amount(token) {
            let amt = parsed_amt.minor_units;
            if amt > 0 && !found_amount {
                if elem.x_mpt < bal_thresh_x || line.len() <= 3 {
                    amount_cents = amt;
                    amount_x_mpt = elem.x_mpt;
                    found_amount = true;
                    if parsed_amt.is_negative
                        || token.starts_with('-')
                        || token.eq_ignore_ascii_case("NO")
                        || token.eq_ignore_ascii_case("DEBIT")
                    {
                        is_explicit_debit = true;
                    }
                    continue;
                }
            } else if found_amount && balance_cents.is_none() && amt > 0 {
                balance_cents = Some(amt);
                continue;
            }
        }

        narration_tokens.push(token.as_str());
    }

    if !found_amount || amount_cents == 0 {
        return None;
    }

    // Determine Debit vs Credit
    let is_credit = if is_explicit_debit {
        false
    } else if let (Some(bal), Some(last_bal)) = (balance_cents, last_balance) {
        if bal > last_bal {
            true
        } else if bal < last_bal {
            false
        } else if let (Some(xd), Some(xc)) = (x_debit, x_credit) {
            (amount_x_mpt.saturating_sub(xc)).abs() < (amount_x_mpt.saturating_sub(xd)).abs()
        } else {
            true
        }
    } else if let (Some(xd), Some(xc)) = (x_debit, x_credit) {
        (amount_x_mpt.saturating_sub(xc)).abs() < (amount_x_mpt.saturating_sub(xd)).abs()
    } else {
        true
    };

    Some(RawTransactionRecord {
        row_id: 0,
        date_str: first_token.clone(),
        val_date_str: None,
        doc_ref,
        debit_amt_str: if !is_credit { Some(amount_cents.to_string()) } else { None },
        credit_amt_str: if is_credit { Some(amount_cents.to_string()) } else { None },
        amount_cents: Some(amount_cents),
        is_credit,
        balance_str: balance_cents.map(|b| b.to_string()),
        balance_cents,
        narration: narration_tokens.join(" "),
        counterparty_name: None,
    })
}

fn extract_amount_from_line(line: &[TextElement]) -> Option<u64> {
    for elem in line.iter().rev() {
        if let Ok(parsed) = parse_monetary_amount(&elem.text) {
            if parsed.minor_units > 0 {
                return Some(parsed.minor_units);
            }
        }
    }
    None
}
