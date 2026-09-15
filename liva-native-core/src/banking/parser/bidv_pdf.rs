//! BIDV PDF Tabular Statement Parser.
//!
//! Features:
//! - 2D text coordinate sorting (grouping text by Y coordinate tolerance and sorting by X).
//! - Multi-line narration wrapping and concatenation.
//! - Page balance checksums: `Balance_end == Balance_start + Sum(Credit) - Sum(Debit)`.

use lopdf::content::Content;
use lopdf::{Document, Object};

use super::{BankStatementParser, ParserError};
use crate::banking::models::{
    BankStatement, BankType, StatementFormat, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};

pub struct BidvPdfParser;

#[derive(Debug, Clone)]
struct TextElement {
    x: f32,
    y: f32,
    text: String,
}

impl BankStatementParser for BidvPdfParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("bidv") {
            return true;
        }

        if bytes.starts_with(b"%PDF-") {
            if let Ok(doc) = Document::load_mem(bytes) {
                let pages = doc.get_pages();
                for (&page_num, _) in pages.iter().take(2) {
                    if let Ok(text) = doc.extract_text(&[page_num]) {
                        let t_lower = text.to_lowercase();
                        if t_lower.contains("bidv")
                            || t_lower.contains("đầu tư và phát triển")
                            || t_lower.contains("sao kê tài khoản")
                        {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn parse(&self, bytes: &[u8], _filename: &str) -> Result<BankStatement, ParserError> {
        let start_time = std::time::Instant::now();

        let doc = Document::load_mem(bytes)
            .map_err(|e| ParserError::PdfError(format!("Failed to load PDF bytes: {e}")))?;

        let pages = doc.get_pages();
        if pages.is_empty() {
            return Err(ParserError::InvalidStructure(
                "PDF has no pages".to_string(),
            ));
        }

        let mut account_number: Option<String> = None;
        let mut account_name: Option<String> = None;
        let mut global_opening_balance: Option<u64> = None;
        let mut global_closing_balance: Option<u64> = None;

        let mut all_transactions: Vec<TransactionRecord> = Vec::new();
        let mut min_tx_date: Option<i64> = None;
        let mut max_tx_date: Option<i64> = None;

        let mut x_debit: Option<f32> = None;
        let mut x_credit: Option<f32> = None;
        let mut x_balance: Option<f32> = None;
        let mut _x_narration: Option<f32> = None;

        for (&page_num, &page_id) in &pages {
            // Extract coordinate text elements from page
            let mut elements = extract_page_elements(&doc, page_id);
            if elements.is_empty() {
                // Fallback to text extraction string
                if let Ok(text) = doc.extract_text(&[page_num]) {
                    elements = text_to_synthetic_elements(&text);
                }
            }

            // Group into 2D lines with spatial clustering delta_y <= 2.5 pt
            let lines = group_elements_into_lines(elements);

            let mut page_opening_balance: Option<u64> = None;
            let mut page_closing_balance: Option<u64> = None;
            let mut page_sum_credit: u64 = 0;
            let mut page_sum_debit: u64 = 0;

            let mut in_table = false;

            for line in &lines {
                let line_text: String = line
                    .iter()
                    .map(|e| e.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let line_lower = line_text.to_lowercase();

                // Check metadata
                if account_number.is_none()
                    && (line_lower.contains("số tài khoản") || line_lower.contains("account no"))
                {
                    account_number = extract_account_number(&line_text);
                }
                if account_name.is_none()
                    && (line_lower.contains("tên khách hàng")
                        || line_lower.contains("customer name"))
                {
                    account_name = extract_account_name(&line_text);
                }
                if line_lower.contains("số dư đầu kỳ") || line_lower.contains("opening balance")
                {
                    if let Some(amt) = extract_amount_from_line(line) {
                        page_opening_balance = Some(amt);
                        if global_opening_balance.is_none() {
                            global_opening_balance = Some(amt);
                        }
                    }
                }
                if line_lower.contains("số dư cuối kỳ") || line_lower.contains("closing balance")
                {
                    if let Some(amt) = extract_amount_from_line(line) {
                        page_closing_balance = Some(amt);
                        global_closing_balance = Some(amt);
                    }
                }

                // Detect table header row and calibrate column X-coordinates
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
                            x_debit = Some(elem.x);
                        } else if h.contains("có") || h.contains("credit") {
                            x_credit = Some(elem.x);
                        } else if h.contains("số dư") || h.contains("balance") {
                            x_balance = Some(elem.x);
                        } else if h.contains("nội dung")
                            || h.contains("diễn giải")
                            || h.contains("description")
                        {
                            _x_narration = Some(elem.x);
                        }
                    }
                    continue;
                }

                if !in_table {
                    continue;
                }

                // Stop if summary or footer
                if line_lower.contains("tổng phát sinh")
                    || line_lower.contains("trang ")
                    || line_lower.contains("page ")
                {
                    in_table = false;
                    continue;
                }

                let last_bal = all_transactions
                    .last()
                    .and_then(|t| t.balance_after)
                    .or(page_opening_balance)
                    .or(global_opening_balance);

                // Try parsing line as a new transaction
                if let Some(mut tx) =
                    try_parse_bidv_row(line, x_debit, x_credit, x_balance, last_bal)
                {
                    tx.row_id = all_transactions.len() + 1;
                    if tx.tx_type == TransactionType::Credit {
                        page_sum_credit = page_sum_credit.saturating_add(tx.amount);
                    } else {
                        page_sum_debit = page_sum_debit.saturating_add(tx.amount);
                    }

                    min_tx_date = Some(min_tx_date.map_or(tx.tx_date, |m| m.min(tx.tx_date)));
                    max_tx_date = Some(max_tx_date.map_or(tx.tx_date, |m| m.max(tx.tx_date)));

                    all_transactions.push(tx);
                } else if let Some(last_tx) = all_transactions.last_mut() {
                    // Multi-line narration wrapping: belongs to narration of previous transaction
                    let addition = line_text.trim();
                    let starts_with_date = line
                        .first()
                        .and_then(|e| parse_banking_date(&e.text))
                        .is_some();
                    let has_calibrated_amount = line.iter().any(|elem| {
                        if parse_vietnamese_amount(&elem.text).is_none() {
                            return false;
                        }
                        let in_debit = x_debit.map_or(false, |xd| (elem.x - xd).abs() <= 35.0);
                        let in_credit = x_credit.map_or(false, |xc| (elem.x - xc).abs() <= 35.0);
                        let in_balance = x_balance.map_or(false, |xb| (elem.x - xb).abs() <= 35.0);
                        in_debit || in_credit || in_balance
                    });
                    if !addition.is_empty()
                        && !is_noise_footer(addition)
                        && !starts_with_date
                        && !has_calibrated_amount
                    {
                        last_tx.narration.push(' ');
                        last_tx.narration.push_str(addition);
                        if last_tx.doc_ref.is_none() {
                            if let Some(ft) = extract_ft_reference(addition) {
                                last_tx.doc_ref = Some(ft);
                            }
                        }
                    }
                }
            }

            // Page balance checksum calculation:
            // Opening + Credits - Debits == Closing
            if let (Some(open), Some(close)) = (page_opening_balance, page_closing_balance) {
                let calculated_close =
                    (open as i128) + (page_sum_credit as i128) - (page_sum_debit as i128);
                if calculated_close != (close as i128) {
                    tracing::warn!(
                        "Page {page_num} balance checksum discrepancy: Open={open}, +Credit={page_sum_credit}, -Debit={page_sum_debit}, Expected={close}, Calc={calculated_close}"
                    );
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(BankStatement::new(
            "BIDV".to_string(),
            BankType::Bidv,
            StatementFormat::Pdf,
            account_number,
            account_name,
            global_opening_balance,
            global_closing_balance,
            min_tx_date,
            max_tx_date,
            all_transactions,
            duration_ms,
        ))
    }
}

/// Extracts text elements along with their (x, y) coordinates from a PDF page.
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

    let mut cur_x: f32 = 0.0;
    let mut cur_y: f32 = 0.0;

    for op in &content.operations {
        match op.operator.as_str() {
            "Tm" => {
                // Text Matrix: [a, b, c, d, e, f] where e=x, f=y
                if op.operands.len() >= 6 {
                    if let (Ok(x), Ok(y)) = (op.operands[4].as_float(), op.operands[5].as_float()) {
                        cur_x = x;
                        cur_y = y;
                    }
                }
            }
            "Td" | "TD" => {
                // Relative text position: [dx, dy]
                if op.operands.len() >= 2 {
                    if let (Ok(dx), Ok(dy)) = (op.operands[0].as_float(), op.operands[1].as_float())
                    {
                        cur_x += dx;
                        cur_y += dy;
                    }
                }
            }
            "T*" => {
                // Move to next line (approx 12pt down)
                cur_y -= 12.0;
            }
            "Tj" => {
                if let Some(Object::String(bytes, _)) = op.operands.first() {
                    let text = String::from_utf8_lossy(bytes).trim().to_string();
                    if !text.is_empty() {
                        elements.push(TextElement {
                            x: cur_x,
                            y: cur_y,
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
                            x: cur_x,
                            y: cur_y,
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

/// Fallback for text streams: synthetic lines with estimated coordinates
fn text_to_synthetic_elements(text: &str) -> Vec<TextElement> {
    let mut elements = Vec::new();
    let mut y = 1000.0;
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            let mut x = 50.0;
            for token in tokens {
                elements.push(TextElement {
                    x,
                    y,
                    text: token.to_string(),
                });
                x += (token.len() as f32) * 6.0 + 8.0;
            }
        }
        y -= 14.0;
    }
    elements
}

/// Groups 2D elements into table lines based on Y coordinate tolerance (|y1 - y2| <= 2.5pt),
/// and sorts tokens in each line by X coordinate ascending.
fn group_elements_into_lines(mut elements: Vec<TextElement>) -> Vec<Vec<TextElement>> {
    // Sort primarily by Y descending (top to bottom), secondarily by X ascending (left to right)
    elements.sort_by(|a, b| {
        b.y.partial_cmp(&a.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut lines: Vec<Vec<TextElement>> = Vec::new();

    for elem in elements {
        let mut matched_line = false;
        for line in lines.iter_mut().rev() {
            if let Some(first) = line.first() {
                if (first.y - elem.y).abs() <= 2.5 {
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

    for line in &mut lines {
        line.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    }

    lines
}

fn try_parse_bidv_row(
    line: &[TextElement],
    x_debit: Option<f32>,
    x_credit: Option<f32>,
    x_balance: Option<f32>,
    last_balance: Option<u64>,
) -> Option<TransactionRecord> {
    if line.is_empty() {
        return None;
    }

    // A BIDV transaction line typically starts with a date (e.g. 15/08/2026)
    let first_token = &line.first()?.text;
    let tx_date = parse_banking_date(first_token)?;

    let mut doc_ref: Option<String> = None;
    let mut amount: u64 = 0;
    let mut amount_x: f32 = 0.0;
    let mut is_explicit_debit = false;
    let mut balance_after: Option<u64> = None;
    let mut narration_tokens = Vec::new();

    let mut found_amount = false;
    let bal_thresh_x = x_balance.unwrap_or(360.0);

    for (idx, elem) in line.iter().enumerate().skip(1) {
        let token = &elem.text;
        if idx == 1
            && (token.starts_with("FT")
                || (token.len() >= 6 && token.chars().all(|c| c.is_alphanumeric())))
        {
            doc_ref = Some(token.clone());
            continue;
        }

        if let Some(amt) = parse_vietnamese_amount(token) {
            if amt > 0 && !found_amount {
                if elem.x < bal_thresh_x || line.len() <= 3 {
                    amount = amt;
                    amount_x = elem.x;
                    found_amount = true;
                    if token.starts_with('-')
                        || token.to_uppercase() == "NO"
                        || token.to_uppercase() == "DEBIT"
                    {
                        is_explicit_debit = true;
                    }
                    continue;
                }
            } else if found_amount && balance_after.is_none() && amt > 0 {
                balance_after = Some(amt);
                continue;
            }
        }

        narration_tokens.push(token.as_str());
    }

    if !found_amount || amount == 0 {
        return None;
    }

    // Determine Debit vs Credit using:
    // 1. Explicit minus/debit marker
    // 2. Mathematical balance continuity (Delta Balance > 0 -> Credit, < 0 -> Debit)
    // 3. X-coordinate column interval proximity
    let tx_type = if is_explicit_debit {
        TransactionType::Debit
    } else if let (Some(bal), Some(last_bal)) = (balance_after, last_balance) {
        if bal > last_bal {
            TransactionType::Credit
        } else if bal < last_bal {
            TransactionType::Debit
        } else if let (Some(xd), Some(xc)) = (x_debit, x_credit) {
            if (amount_x - xc).abs() < (amount_x - xd).abs() {
                TransactionType::Credit
            } else {
                TransactionType::Debit
            }
        } else {
            TransactionType::Credit
        }
    } else if let (Some(xd), Some(xc)) = (x_debit, x_credit) {
        if (amount_x - xc).abs() < (amount_x - xd).abs() {
            TransactionType::Credit
        } else {
            TransactionType::Debit
        }
    } else {
        TransactionType::Credit
    };

    let narration = narration_tokens.join(" ");

    let ft_number = doc_ref
        .as_ref()
        .filter(|r| r.starts_with("FT"))
        .cloned()
        .or_else(|| extract_ft_reference(&narration));

    let trace_id = extract_trace_reference(&narration);

    Some(TransactionRecord {
        row_id: 0, // Assigned by caller
        tx_date,
        value_date: tx_date,
        doc_ref,
        tx_type,
        amount,
        balance_after,
        counterparty_account: None,
        counterparty_name: None,
        counterparty_bank: None,
        narration,
        ft_number,
        trace_id,
        raw_ref: None,
    })
}

fn extract_amount_from_line(line: &[TextElement]) -> Option<u64> {
    for elem in line.iter().rev() {
        if let Some(amt) = parse_vietnamese_amount(&elem.text) {
            if amt > 0 {
                return Some(amt);
            }
        }
        if let Some(pos) = elem.text.rfind(':') {
            let after = elem.text[pos + 1..].trim();
            if let Some(amt) = parse_vietnamese_amount(after) {
                if amt > 0 {
                    return Some(amt);
                }
            }
        }
    }
    None
}

fn extract_account_number(line: &str) -> Option<String> {
    for part in line.split(|c: char| c == ':' || c == '-' || c == ' ') {
        let clean: String = part.chars().filter(|c| c.is_ascii_digit()).collect();
        if clean.len() >= 9 && clean.len() <= 16 {
            return Some(clean);
        }
    }
    None
}

fn extract_account_name(line: &str) -> Option<String> {
    if let Some(pos) = line.find(':') {
        let sub = line[pos + 1..].trim();
        if !sub.is_empty() {
            return Some(sub.to_string());
        }
    }
    None
}

fn extract_ft_reference(narration: &str) -> Option<String> {
    for token in narration.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.starts_with("FT") && clean.len() >= 10 {
            return Some(clean.to_string());
        }
    }
    None
}

fn extract_trace_reference(narration: &str) -> Option<String> {
    for token in narration.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if (clean.starts_with("NPS") || clean.starts_with("VN")) && clean.len() >= 8 {
            return Some(clean.to_string());
        }
    }
    None
}

fn is_noise_footer(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("ngân hàng tmcp đầu tư")
        || lower.contains("trang ")
        || lower.contains("page ")
        || lower.contains("hotline")
        || lower.contains("bidv.com.vn")
        || lower.contains("tổng phát sinh")
}
