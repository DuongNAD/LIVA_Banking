//! Vietcombank (VCB) Excel Statement Parser.
//!
//! Handles merged header metadata cells, Vietnamese numeric formatting
//! (dot thousand separators `15.000.000,00`), debit/credit columns, and multi-line narrations.

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use std::io::Cursor;

use super::{BankStatementParser, ParserError};
use crate::banking::models::{
    BankStatement, BankType, StatementFormat, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};

pub struct VcbExcelParser;

impl BankStatementParser for VcbExcelParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("vcb") || lower.contains("vietcombank") {
            return true;
        }

        // Check magic bytes for ZIP (.xlsx) or OLE (.xls)
        if bytes.len() >= 4 {
            let is_xlsx = bytes[0..4] == [0x50, 0x4B, 0x03, 0x04];
            let is_xls = bytes[0..4] == [0xD0, 0xCF, 0x11, 0xE0];
            if is_xlsx || is_xls {
                if let Ok(mut workbook) = open_workbook_auto_from_rs(Cursor::new(bytes)) {
                    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
                        for row in range.rows().take(15) {
                            for cell in row {
                                let text = cell_to_string(cell).to_lowercase();
                                if text.contains("vietcombank")
                                    || text.contains("ngoại thương")
                                    || text.contains("vcb")
                                    || text.contains("số tài khoản")
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
        let start_time = std::time::Instant::now();
        let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes))
            .map_err(|e| ParserError::ExcelError(format!("Failed to open Excel workbook: {e}")))?;

        let range = workbook
            .worksheet_range_at(0)
            .ok_or_else(|| ParserError::InvalidStructure("Workbook has no sheets".to_string()))?
            .map_err(|e| ParserError::ExcelError(format!("Failed to read sheet 0: {e}")))?;

        let mut account_number: Option<String> = None;
        let mut account_name: Option<String> = None;
        let mut opening_balance: Option<u64> = None;
        let mut closing_balance: Option<u64> = None;

        let mut header_row_idx: Option<usize> = None;
        let mut col_date: Option<usize> = None;
        let mut col_val_date: Option<usize> = None;
        let mut col_ref: Option<usize> = None;
        let mut col_debit: Option<usize> = None;
        let mut col_credit: Option<usize> = None;
        let mut col_balance: Option<usize> = None;
        let mut col_narration: Option<usize> = None;
        let mut col_counterparty: Option<usize> = None;

        let rows: Vec<&[Data]> = range.rows().collect();

        // 1. Scan metadata and locate table header
        for (r_idx, row) in rows.iter().enumerate() {
            let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
            let row_lower = row_str.to_lowercase();

            // Metadata extraction from header block
            if r_idx < 20 {
                if row_lower.contains("số tài khoản") || row_lower.contains("account no") {
                    account_number = extract_account_number(&row_str);
                }
                if row_lower.contains("tên tài khoản") || row_lower.contains("account name") {
                    account_name = extract_account_name(&row_str);
                }
                if row_lower.contains("số dư đầu kỳ") || row_lower.contains("opening balance")
                {
                    opening_balance = extract_balance_from_row(row);
                }
                if row_lower.contains("số dư cuối kỳ") || row_lower.contains("closing balance")
                {
                    closing_balance = extract_balance_from_row(row);
                }
            }

            // Detect table columns
            if (row_lower.contains("ngày giao dịch")
                || row_lower.contains("ngày gd")
                || row_lower.contains("date"))
                && (row_lower.contains("số tiền")
                    || row_lower.contains("ghi nợ")
                    || row_lower.contains("ghi có")
                    || row_lower.contains("debit")
                    || row_lower.contains("credit"))
            {
                header_row_idx = Some(r_idx);
                for (c_idx, cell) in row.iter().enumerate() {
                    let col_name = cell_to_string(cell).trim().to_lowercase();
                    if col_name.contains("ngày giao dịch")
                        || col_name.contains("ngày gd")
                        || col_name == "date"
                    {
                        col_date = Some(c_idx);
                    } else if col_name.contains("ngày giá trị")
                        || col_name.contains("value date")
                        || col_name.contains("ngày hl")
                    {
                        col_val_date = Some(c_idx);
                    } else if col_name.contains("chứng từ")
                        || col_name.contains("tham chiếu")
                        || col_name.contains("ref")
                        || col_name.contains("mã gd")
                        || col_name.contains("số bút toán")
                    {
                        col_ref = Some(c_idx);
                    } else if col_name.contains("ghi nợ")
                        || col_name.contains("nợ")
                        || col_name.contains("debit")
                    {
                        col_debit = Some(c_idx);
                    } else if col_name.contains("ghi có")
                        || col_name.contains("có")
                        || col_name.contains("credit")
                    {
                        col_credit = Some(c_idx);
                    } else if col_name.contains("số dư") || col_name.contains("balance") {
                        col_balance = Some(c_idx);
                    } else if col_name.contains("nội dung")
                        || col_name.contains("diễn giải")
                        || col_name.contains("narration")
                        || col_name.contains("description")
                    {
                        col_narration = Some(c_idx);
                    } else if col_name.contains("đối tác")
                        || col_name.contains("đối ứng")
                        || col_name.contains("counterparty")
                    {
                        col_counterparty = Some(c_idx);
                    }
                }
                break;
            }
        }

        // Bottom-up scan for closing balance if not found in top 20 rows
        if closing_balance.is_none() {
            for row in rows.iter().rev().take(30) {
                let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
                let row_lower = row_str.to_lowercase();
                if row_lower.contains("số dư cuối kỳ") || row_lower.contains("closing balance")
                {
                    closing_balance = extract_balance_from_row(row);
                    if closing_balance.is_some() {
                        break;
                    }
                }
            }
        }

        let header_idx = header_row_idx.ok_or_else(|| {
            ParserError::InvalidStructure(format!(
                "Could not find transaction header row in VCB statement: {filename}"
            ))
        })?;

        let c_date = col_date.unwrap_or(0);
        let c_debit = col_debit.unwrap_or(2);
        let c_credit = col_credit.unwrap_or(3);
        let c_balance = col_balance.unwrap_or(4);
        let c_narration = col_narration.unwrap_or(5);

        let mut transactions = Vec::new();
        let mut min_tx_date: Option<i64> = None;
        let mut max_tx_date: Option<i64> = None;
        let mut last_valid_date: Option<i64> = None;

        // 2. Parse transaction rows
        for row in rows.iter().skip(header_idx + 1) {
            if row.is_empty() {
                continue;
            }

            let first_col_str = row.first().map(cell_to_string).unwrap_or_default();
            let lower_first = first_col_str.to_lowercase();
            if lower_first.contains("tổng")
                || lower_first.contains("số dư cuối kỳ")
                || lower_first.contains("total")
            {
                break;
            }

            let debit_amt = row.get(c_debit).and_then(cell_to_amount).unwrap_or(0);
            let credit_amt = row.get(c_credit).and_then(cell_to_amount).unwrap_or(0);

            let (tx_type, amount) = if debit_amt > 0 {
                (TransactionType::Debit, debit_amt)
            } else if credit_amt > 0 {
                (TransactionType::Credit, credit_amt)
            } else {
                continue; // 0 amount transaction or empty row
            };

            // Merged cells forward-fill: when date cell is empty under a merged cell,
            // carry forward the previous valid transaction date.
            let date_cell = row.get(c_date).map(cell_to_string).unwrap_or_default();
            let tx_date = match parse_banking_date(&date_cell) {
                Some(d) => {
                    last_valid_date = Some(d);
                    d
                }
                None => match last_valid_date {
                    Some(d) => d,
                    None => continue, // Cannot resolve transaction date
                },
            };

            let val_date = col_val_date
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .as_deref()
                .and_then(parse_banking_date)
                .unwrap_or(tx_date);

            let balance_after = row.get(c_balance).and_then(cell_to_amount);

            let narration = row
                .get(c_narration)
                .map(cell_to_string)
                .unwrap_or_default()
                .trim()
                .to_string();

            let mut doc_ref = col_ref
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            if doc_ref.is_none() {
                doc_ref = extract_ft_reference(&narration);
            }

            let ft_number = doc_ref
                .as_ref()
                .filter(|r| r.starts_with("FT"))
                .cloned()
                .or_else(|| extract_ft_reference(&narration));

            let trace_id = extract_trace_reference(&narration);

            let counterparty_name = col_counterparty
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            min_tx_date = Some(min_tx_date.map_or(tx_date, |m| m.min(tx_date)));
            max_tx_date = Some(max_tx_date.map_or(tx_date, |m| m.max(tx_date)));

            transactions.push(TransactionRecord {
                row_id: transactions.len() + 1,
                tx_date,
                value_date: val_date,
                doc_ref,
                tx_type,
                amount,
                balance_after,
                counterparty_account: None,
                counterparty_name,
                counterparty_bank: None,
                narration,
                ft_number,
                trace_id,
                raw_ref: None,
            });
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(BankStatement::new(
            "VCB".to_string(),
            BankType::Vietcombank,
            StatementFormat::Excel,
            account_number,
            account_name,
            opening_balance,
            closing_balance,
            min_tx_date,
            max_tx_date,
            transactions,
            duration_ms,
        ))
    }
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            let rounded = (f.abs() + 0.5) as u64;
            if (f.abs() - rounded as f64).abs() < 1e-4 {
                format!("{rounded}")
            } else {
                f.to_string()
            }
        }
        Data::DateTime(d) => format!("{d}"),
        Data::Bool(b) => b.to_string(),
        Data::Empty => String::new(),
        _ => String::new(),
    }
}

fn cell_to_amount(cell: &Data) -> Option<u64> {
    match cell {
        Data::Int(i) => {
            if *i >= 0 {
                Some(*i as u64)
            } else {
                Some((-*i) as u64)
            }
        }
        Data::Float(f) => {
            let abs_f = f.abs();
            Some((abs_f + 0.5) as u64)
        }
        Data::String(s) => parse_vietnamese_amount(s),
        _ => None,
    }
}

fn extract_account_number(line: &str) -> Option<String> {
    let parts: Vec<&str> = line
        .split(|c: char| c == ':' || c == '-' || c == '/')
        .collect();
    for p in parts {
        let clean: String = p.chars().filter(|c| c.is_ascii_digit()).collect();
        if clean.len() >= 9 && clean.len() <= 16 {
            return Some(clean);
        }
    }
    None
}

fn extract_account_name(line: &str) -> Option<String> {
    if let Some(pos) = line.find(':') {
        let after = line[pos + 1..].trim();
        if !after.is_empty() {
            return Some(after.to_string());
        }
    }
    None
}

fn extract_balance_from_row(row: &[Data]) -> Option<u64> {
    // Pass 1: Prioritize cells containing balance keywords
    for cell in row {
        let s = cell_to_string(cell);
        let sl = s.to_lowercase();
        if sl.contains("số dư")
            || sl.contains("closing")
            || sl.contains("dư cuối")
            || sl.contains("opening")
            || sl.contains("đầu kỳ")
        {
            if let Some(pos) = s.rfind(':') {
                let after = s[pos + 1..].trim();
                if let Some(amt) = parse_vietnamese_amount(after).filter(|&amt| amt > 0) {
                    return Some(amt);
                }
            }
            if let Some(amt) = cell_to_amount(cell).filter(|&amt| amt > 0) {
                return Some(amt);
            }
        }
    }
    // Pass 2: Inspect cells containing a colon `:` and parse substring after colon
    for cell in row {
        let s = cell_to_string(cell);
        if let Some(pos) = s.rfind(':') {
            let after = s[pos + 1..].trim();
            if let Some(amt) = parse_vietnamese_amount(after).filter(|&amt| amt > 0) {
                return Some(amt);
            }
        }
    }
    // Pass 3: Fallback to any positive amount
    for cell in row {
        if let Some(amt) = cell_to_amount(cell).filter(|&amt| amt > 0) {
            return Some(amt);
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
