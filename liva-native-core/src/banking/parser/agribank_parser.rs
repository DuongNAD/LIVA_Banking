//! Agribank (VBA) Statement Parser.
//!
//! Features:
//! - Agribank Excel (.xlsx / .xls BIFF8) parsing via `calamine`.
//! - In-house streaming HTML table parser for Agribank `.xls` HTML table exports (`<table><tr><td>...</td></tr></table>`).
//! - Standard CSV parsing fallback with Windows-1258.

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use std::io::Cursor;
use unicode_normalization::UnicodeNormalization;

use super::{BankStatementParser, ParserError};
use crate::banking::models::{
    BankStatement, BankType, StatementFormat, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};

pub struct AgribankParser;

impl BankStatementParser for AgribankParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("agri") || lower.contains("agribank") || lower.contains("vba") {
            return true;
        }

        // Check HTML table export
        if is_html_table(bytes) {
            let sample = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).to_lowercase();
            if sample.contains("agribank")
                || sample.contains("nông nghiệp")
                || sample.contains("sổ phụ")
                || sample.contains("bảng kê")
            {
                return true;
            }
        }

        // Check Excel
        if bytes.len() >= 4
            && (bytes[0..4] == [0x50, 0x4B, 0x03, 0x04] || bytes[0..4] == [0xD0, 0xCF, 0x11, 0xE0])
        {
            if let Ok(mut wb) = open_workbook_auto_from_rs(Cursor::new(bytes)) {
                if let Some(Ok(range)) = wb.worksheet_range_at(0) {
                    for row in range.rows().take(15) {
                        for cell in row {
                            let s = cell_to_string(cell).to_lowercase();
                            if s.contains("agribank")
                                || s.contains("nông nghiệp")
                                || s.contains("vba")
                            {
                                return true;
                            }
                        }
                    }
                }
            }
            return false;
        }

        // Check CSV
        let slice_len = bytes.len().min(1024);
        let sample = String::from_utf8_lossy(&bytes[..slice_len]).to_lowercase();
        if sample.contains("agribank") || sample.contains("nông nghiệp") || sample.contains("vba")
        {
            return true;
        }

        false
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
        let start_time = std::time::Instant::now();

        // 1. Check if file is an HTML table disguised as .xls
        if is_html_table(bytes) {
            return parse_agribank_html(bytes, filename, start_time);
        }

        // 2. Check if Excel
        if bytes.len() >= 4
            && (bytes[0..4] == [0x50, 0x4B, 0x03, 0x04] || bytes[0..4] == [0xD0, 0xCF, 0x11, 0xE0])
        {
            return parse_agribank_excel(bytes, filename, start_time);
        }

        // 3. Fallback to CSV
        parse_agribank_csv(bytes, filename, start_time)
    }
}

fn is_html_table(bytes: &[u8]) -> bool {
    let check_len = bytes.len().min(1024);
    if check_len < 6 {
        return false;
    }
    let sample = String::from_utf8_lossy(&bytes[..check_len]).to_lowercase();
    sample.contains("<html")
        || sample.contains("<!doctype html")
        || sample.contains("<?xml")
        || sample.contains("<table")
}

/// Pure-Rust streaming HTML table parser for Agribank Corporate E-Banking `.xls` HTML exports.
fn parse_agribank_html(
    raw_bytes: &[u8],
    filename: &str,
    start_time: std::time::Instant,
) -> Result<BankStatement, ParserError> {
    let bytes = if raw_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw_bytes[3..]
    } else {
        raw_bytes
    };
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cow, _, _) = encoding_rs::WINDOWS_1258.decode(bytes);
            cow.nfc().collect()
        }
    };

    let table = extract_html_table_rows(&text);
    if table.is_empty() {
        return Err(ParserError::InvalidStructure(format!(
            "No HTML table rows found in Agribank statement: {filename}"
        )));
    }

    let mut account_number: Option<String> = None;
    let mut account_name: Option<String> = None;
    let mut opening_balance: Option<u64> = None;
    let mut closing_balance: Option<u64> = None;

    let mut header_row_idx: Option<usize> = None;
    let mut col_date: Option<usize> = None;
    let mut col_val_date: Option<usize> = None;
    let mut col_ref: Option<usize> = None;
    let mut col_narration: Option<usize> = None;
    let mut col_debit: Option<usize> = None;
    let mut col_credit: Option<usize> = None;
    let mut col_balance: Option<usize> = None;

    for (r_idx, row) in table.iter().enumerate() {
        let row_str = row.join(" ");
        let row_lower = row_str.to_lowercase();

        if r_idx < 25 {
            if row_lower.contains("số tài khoản") || row_lower.contains("account no") {
                account_number = extract_account_number(&row_str);
            }
            if row_lower.contains("tên tài khoản") || row_lower.contains("tên khách hàng") {
                account_name = extract_account_name(&row_str);
            }
            if row_lower.contains("số dư đầu kỳ") || row_lower.contains("dư đầu") {
                opening_balance = extract_balance_from_str_slice(row);
            }
            if row_lower.contains("số dư cuối kỳ") || row_lower.contains("dư cuối") {
                closing_balance = extract_balance_from_str_slice(row);
            }
        }

        if (row_lower.contains("ngày") || row_lower.contains("date"))
            && (row_lower.contains("chứng từ")
                || row_lower.contains("tiền")
                || row_lower.contains("nợ")
                || row_lower.contains("có"))
        {
            header_row_idx = Some(r_idx);
            for (c_idx, cell) in row.iter().enumerate() {
                let h = cell.trim().to_lowercase();
                if h.contains("ngày hạch toán")
                    || h.contains("ngày ht")
                    || h.contains("ngày gd")
                    || h == "ngày"
                {
                    col_date = Some(c_idx);
                } else if h.contains("ngày chứng từ")
                    || h.contains("ngày ct")
                    || h.contains("ngày giá trị")
                {
                    col_val_date = Some(c_idx);
                } else if h.contains("số chứng từ")
                    || h.contains("số ct")
                    || h.contains("chứng từ")
                    || h.contains("mã gd")
                {
                    col_ref = Some(c_idx);
                } else if h.contains("diễn giải")
                    || h.contains("nội dung")
                    || h.contains("chi tiết")
                {
                    col_narration = Some(c_idx);
                } else if h.contains("ghi nợ") || h == "nợ" || h.contains("phát sinh nợ") {
                    col_debit = Some(c_idx);
                } else if h.contains("ghi có") || h == "có" || h.contains("phát sinh có") {
                    col_credit = Some(c_idx);
                } else if h.contains("số dư") {
                    col_balance = Some(c_idx);
                }
            }
            break;
        }
    }

    if closing_balance.is_none() {
        for row in table.iter().rev().take(30) {
            let row_str = row.join(" ");
            let row_lower = row_str.to_lowercase();
            if row_lower.contains("số dư cuối kỳ") || row_lower.contains("dư cuối") {
                closing_balance = extract_balance_from_str_slice(row);
                if closing_balance.is_some() {
                    break;
                }
            }
        }
    }

    let header_idx = header_row_idx.ok_or_else(|| {
        ParserError::InvalidStructure(format!(
            "Could not find HTML table header in Agribank statement: {filename}"
        ))
    })?;

    let c_date = col_date.unwrap_or(0);
    let c_ref = col_ref.unwrap_or(2);
    let c_narration = col_narration.unwrap_or(3);
    let c_debit = col_debit.unwrap_or(4);
    let c_credit = col_credit.unwrap_or(5);
    let c_balance = col_balance.unwrap_or(6);

    let mut transactions = Vec::new();
    let mut min_tx_date: Option<i64> = None;
    let mut max_tx_date: Option<i64> = None;
    let mut last_valid_date: Option<i64> = None;

    for row in table.iter().skip(header_idx + 1) {
        if row.is_empty() {
            continue;
        }

        let first_col_str = row.first().map(|s| s.as_str()).unwrap_or_default();
        let lower_first = first_col_str.to_lowercase();
        if lower_first.contains("tổng")
            || lower_first.contains("số dư cuối kỳ")
            || lower_first.contains("total")
        {
            break;
        }

        let debit_amt = row
            .get(c_debit)
            .and_then(|s| parse_vietnamese_amount(s))
            .unwrap_or(0);
        let credit_amt = row
            .get(c_credit)
            .and_then(|s| parse_vietnamese_amount(s))
            .unwrap_or(0);

        let (tx_type, amount) = if debit_amt > 0 {
            (TransactionType::Debit, debit_amt)
        } else if credit_amt > 0 {
            (TransactionType::Credit, credit_amt)
        } else {
            continue;
        };

        let date_cell = row.get(c_date).map(|s| s.as_str()).unwrap_or_default();
        let tx_date = match parse_banking_date(date_cell) {
            Some(d) => {
                last_valid_date = Some(d);
                d
            }
            None => match last_valid_date {
                Some(d) => d,
                None => continue,
            },
        };

        let val_date = col_val_date
            .and_then(|idx| row.get(idx))
            .and_then(|s| parse_banking_date(s))
            .unwrap_or(tx_date);

        let balance_after = row.get(c_balance).and_then(|s| parse_vietnamese_amount(s));
        let narration = row
            .get(c_narration)
            .cloned()
            .unwrap_or_default()
            .trim()
            .to_string();

        let mut doc_ref = row
            .get(c_ref)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if doc_ref.is_none() {
            doc_ref = extract_agri_ref(&narration);
        }

        let ft_number = doc_ref
            .as_ref()
            .filter(|r| r.starts_with("FT") || r.starts_with("VB"))
            .cloned()
            .or_else(|| extract_agri_ref(&narration));

        let trace_id = extract_trace_reference(&narration);

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
            counterparty_name: None,
            counterparty_bank: None,
            narration,
            ft_number,
            trace_id,
            raw_ref: None,
        });
    }

    // Bottom-up search for closing balance if not parsed in header
    if closing_balance.is_none() {
        for row in table.iter().rev().take(15) {
            let line = row.join(" ").to_lowercase();
            if line.contains("số dư cuối kỳ")
                || line.contains("closing balance")
                || line.contains("dư cuối")
                || line.contains("tổng")
            {
                if let Some(col_b) = col_balance {
                    if let Some(val) = row.get(col_b).and_then(|s| parse_vietnamese_amount(s)) {
                        closing_balance = Some(val);
                        break;
                    }
                }
                for cell in row.iter().rev() {
                    if let Some(bal) = parse_vietnamese_amount(cell) {
                        if bal > 0 {
                            closing_balance = Some(bal);
                            break;
                        }
                    }
                }
                if closing_balance.is_some() {
                    break;
                }
            }
        }
    }
    if closing_balance.is_none() {
        closing_balance = transactions.last().and_then(|t| t.balance_after);
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(BankStatement::new(
        "VBA".to_string(),
        BankType::Agribank,
        StatementFormat::Html,
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

fn parse_agribank_excel(
    bytes: &[u8],
    filename: &str,
    start_time: std::time::Instant,
) -> Result<BankStatement, ParserError> {
    let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes)).map_err(|e| {
        ParserError::ExcelError(format!("Failed to open Agribank Excel workbook: {e}"))
    })?;

    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| ParserError::InvalidStructure("Workbook has no sheets".to_string()))?
        .map_err(|e| ParserError::ExcelError(format!("Failed to read sheet 0: {e}")))?;

    let rows: Vec<&[Data]> = range.rows().collect();

    let mut account_number: Option<String> = None;
    let mut account_name: Option<String> = None;
    let mut opening_balance: Option<u64> = None;
    let mut closing_balance: Option<u64> = None;

    let mut header_row_idx: Option<usize> = None;
    let mut col_date: Option<usize> = None;
    let mut col_val_date: Option<usize> = None;
    let mut col_ref: Option<usize> = None;
    let mut col_narration: Option<usize> = None;
    let mut col_debit: Option<usize> = None;
    let mut col_credit: Option<usize> = None;
    let mut col_balance: Option<usize> = None;

    for (r_idx, row) in rows.iter().enumerate() {
        let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
        let row_lower = row_str.to_lowercase();

        if r_idx < 25 {
            if row_lower.contains("số tài khoản") || row_lower.contains("account no") {
                account_number = extract_account_number(&row_str);
            }
            if row_lower.contains("tên tài khoản") || row_lower.contains("tên khách hàng") {
                account_name = extract_account_name(&row_str);
            }
            if row_lower.contains("số dư đầu kỳ") || row_lower.contains("dư đầu") {
                opening_balance = extract_balance_from_row(row);
            }
            if row_lower.contains("số dư cuối kỳ") || row_lower.contains("dư cuối") {
                closing_balance = extract_balance_from_row(row);
            }
        }

        if (row_lower.contains("ngày") || row_lower.contains("date"))
            && (row_lower.contains("chứng từ")
                || row_lower.contains("tiền")
                || row_lower.contains("nợ")
                || row_lower.contains("có"))
        {
            header_row_idx = Some(r_idx);
            for (c_idx, cell) in row.iter().enumerate() {
                let h = cell_to_string(cell).trim().to_lowercase();
                if h.contains("ngày hạch toán")
                    || h.contains("ngày ht")
                    || h.contains("ngày gd")
                    || h == "ngày"
                {
                    col_date = Some(c_idx);
                } else if h.contains("ngày chứng từ")
                    || h.contains("ngày ct")
                    || h.contains("ngày giá trị")
                {
                    col_val_date = Some(c_idx);
                } else if h.contains("số chứng từ")
                    || h.contains("số ct")
                    || h.contains("chứng từ")
                    || h.contains("mã gd")
                {
                    col_ref = Some(c_idx);
                } else if h.contains("diễn giải") || h.contains("nội dung") {
                    col_narration = Some(c_idx);
                } else if h.contains("ghi nợ") || h == "nợ" || h.contains("phát sinh nợ") {
                    col_debit = Some(c_idx);
                } else if h.contains("ghi có") || h == "có" || h.contains("phát sinh có") {
                    col_credit = Some(c_idx);
                } else if h.contains("số dư") {
                    col_balance = Some(c_idx);
                }
            }
            break;
        }
    }

    if closing_balance.is_none() {
        for row in rows.iter().rev().take(30) {
            let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
            let row_lower = row_str.to_lowercase();
            if row_lower.contains("số dư cuối kỳ") || row_lower.contains("dư cuối") {
                closing_balance = extract_balance_from_row(row);
                if closing_balance.is_some() {
                    break;
                }
            }
        }
    }

    let header_idx = header_row_idx.ok_or_else(|| {
        ParserError::InvalidStructure(format!(
            "Could not find transaction header row in Agribank statement: {filename}"
        ))
    })?;

    let c_date = col_date.unwrap_or(0);
    let c_narration = col_narration.unwrap_or(3);
    let c_debit = col_debit.unwrap_or(4);
    let c_credit = col_credit.unwrap_or(5);
    let c_balance = col_balance.unwrap_or(6);

    let mut transactions = Vec::new();
    let mut min_tx_date: Option<i64> = None;
    let mut max_tx_date: Option<i64> = None;
    let mut last_valid_date: Option<i64> = None;

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
            continue;
        };

        let date_cell = row.get(c_date).map(cell_to_string).unwrap_or_default();
        let tx_date = match parse_banking_date(&date_cell) {
            Some(d) => {
                last_valid_date = Some(d);
                d
            }
            None => match last_valid_date {
                Some(d) => d,
                None => continue,
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
            doc_ref = extract_agri_ref(&narration);
        }

        let ft_number = doc_ref
            .as_ref()
            .filter(|r| r.starts_with("FT") || r.starts_with("VB"))
            .cloned()
            .or_else(|| extract_agri_ref(&narration));

        let trace_id = extract_trace_reference(&narration);

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
            counterparty_name: None,
            counterparty_bank: None,
            narration,
            ft_number,
            trace_id,
            raw_ref: None,
        });
    }

    // Bottom-up search for closing balance
    if closing_balance.is_none() {
        for row in rows.iter().rev().take(15) {
            let line = row
                .iter()
                .map(cell_to_string)
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            if line.contains("số dư cuối kỳ")
                || line.contains("closing balance")
                || line.contains("dư cuối")
                || line.contains("tổng")
            {
                if let Some(col_b) = col_balance {
                    if let Some(val) = row.get(col_b).and_then(cell_to_amount) {
                        closing_balance = Some(val);
                        break;
                    }
                }
                for cell in row.iter().rev() {
                    if let Some(bal) = cell_to_amount(cell) {
                        if bal > 0 {
                            closing_balance = Some(bal);
                            break;
                        }
                    }
                }
                if closing_balance.is_some() {
                    break;
                }
            }
        }
    }
    if closing_balance.is_none() {
        closing_balance = transactions.last().and_then(|t| t.balance_after);
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(BankStatement::new(
        "VBA".to_string(),
        BankType::Agribank,
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

fn parse_agribank_csv(
    raw_bytes: &[u8],
    filename: &str,
    start_time: std::time::Instant,
) -> Result<BankStatement, ParserError> {
    let bytes = if raw_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw_bytes[3..]
    } else {
        raw_bytes
    };
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cow, _, _) = encoding_rs::WINDOWS_1258.decode(bytes);
            cow.nfc().collect()
        }
    };

    let delimiter = detect_delimiter(&text);
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(Cursor::new(text.as_bytes()));

    let mut records = Vec::new();
    for res in reader.records() {
        let record = res.map_err(|e| ParserError::CsvError(format!("CSV read error: {e}")))?;
        records.push(record);
    }

    let mut account_number: Option<String> = None;
    let mut account_name: Option<String> = None;
    let mut opening_balance: Option<u64> = None;
    let mut closing_balance: Option<u64> = None;

    let mut header_row_idx: Option<usize> = None;
    let mut col_date: Option<usize> = None;
    let mut col_ref: Option<usize> = None;
    let mut col_narration: Option<usize> = None;
    let mut col_debit: Option<usize> = None;
    let mut col_credit: Option<usize> = None;
    let mut col_balance: Option<usize> = None;

    for (r_idx, record) in records.iter().enumerate() {
        let line = record.iter().collect::<Vec<_>>().join(" ");
        let line_lower = line.to_lowercase();

        if r_idx < 25 {
            if line_lower.contains("số tài khoản") || line_lower.contains("account no") {
                account_number = extract_account_number(&line);
            }
            if line_lower.contains("tên tài khoản") || line_lower.contains("tên khách hàng")
            {
                account_name = extract_account_name(&line);
            }
            if line_lower.contains("số dư đầu kỳ") || line_lower.contains("dư đầu") {
                opening_balance = extract_balance_from_record(record);
            }
            if line_lower.contains("số dư cuối kỳ") || line_lower.contains("dư cuối") {
                closing_balance = extract_balance_from_record(record);
            }
        }

        if (line_lower.contains("ngày") || line_lower.contains("date"))
            && (line_lower.contains("chứng từ")
                || line_lower.contains("tiền")
                || line_lower.contains("nợ")
                || line_lower.contains("có"))
        {
            header_row_idx = Some(r_idx);
            for (c_idx, field) in record.iter().enumerate() {
                let h = field.trim().to_lowercase();
                if h.contains("ngày hạch toán")
                    || h.contains("ngày ht")
                    || h.contains("ngày gd")
                    || h == "ngày"
                {
                    col_date = Some(c_idx);
                } else if h.contains("số chứng từ") || h.contains("số ct") || h.contains("chứng từ")
                {
                    col_ref = Some(c_idx);
                } else if h.contains("diễn giải") || h.contains("nội dung") {
                    col_narration = Some(c_idx);
                } else if h.contains("ghi nợ") || h == "nợ" || h.contains("phát sinh nợ") {
                    col_debit = Some(c_idx);
                } else if h.contains("ghi có") || h == "có" || h.contains("phát sinh có") {
                    col_credit = Some(c_idx);
                } else if h.contains("số dư") {
                    col_balance = Some(c_idx);
                }
            }
            break;
        }
    }

    let header_idx = header_row_idx.ok_or_else(|| {
        ParserError::InvalidStructure(format!(
            "Could not find CSV header row in Agribank statement: {filename}"
        ))
    })?;

    let c_date = col_date.unwrap_or(0);
    let c_narration = col_narration.unwrap_or(3);

    let mut transactions = Vec::new();
    let mut min_tx_date: Option<i64> = None;
    let mut max_tx_date: Option<i64> = None;

    for record in records.iter().skip(header_idx + 1) {
        if record.is_empty() {
            continue;
        }

        let date_cell = record.get(c_date).unwrap_or("");
        let tx_date = match parse_banking_date(date_cell) {
            Some(d) => d,
            None => continue,
        };

        let debit_amt = col_debit
            .and_then(|idx| record.get(idx))
            .and_then(parse_vietnamese_amount)
            .unwrap_or(0);

        let credit_amt = col_credit
            .and_then(|idx| record.get(idx))
            .and_then(parse_vietnamese_amount)
            .unwrap_or(0);

        let (tx_type, amount) = if debit_amt > 0 {
            (TransactionType::Debit, debit_amt)
        } else if credit_amt > 0 {
            (TransactionType::Credit, credit_amt)
        } else {
            continue;
        };

        let balance_after = col_balance
            .and_then(|idx| record.get(idx))
            .and_then(parse_vietnamese_amount);

        let raw_narration = record.get(c_narration).unwrap_or("").trim().to_string();
        let mut doc_ref = col_ref
            .and_then(|idx| record.get(idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if doc_ref.is_none() {
            doc_ref = extract_agri_ref(&raw_narration);
        }

        let ft_number = doc_ref
            .as_ref()
            .filter(|r| r.starts_with("FT") || r.starts_with("VB"))
            .cloned()
            .or_else(|| extract_agri_ref(&raw_narration));

        let trace_id = extract_trace_reference(&raw_narration);

        min_tx_date = Some(min_tx_date.map_or(tx_date, |m| m.min(tx_date)));
        max_tx_date = Some(max_tx_date.map_or(tx_date, |m| m.max(tx_date)));

        transactions.push(TransactionRecord {
            row_id: transactions.len() + 1,
            tx_date,
            value_date: tx_date,
            doc_ref,
            tx_type,
            amount,
            balance_after,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: raw_narration,
            ft_number,
            trace_id,
            raw_ref: None,
        });
    }

    // Bottom-up search for closing balance
    if closing_balance.is_none() {
        for record in records.iter().rev().take(15) {
            let line = record.iter().collect::<Vec<_>>().join(" ").to_lowercase();
            if line.contains("số dư cuối kỳ")
                || line.contains("closing balance")
                || line.contains("dư cuối")
                || line.contains("tổng")
            {
                if let Some(col_b) = col_balance {
                    if let Some(val) = record.get(col_b).and_then(parse_vietnamese_amount) {
                        closing_balance = Some(val);
                        break;
                    }
                }
                for field in record.iter().rev() {
                    if let Some(bal) = parse_vietnamese_amount(field) {
                        if bal > 0 {
                            closing_balance = Some(bal);
                            break;
                        }
                    }
                }
                if closing_balance.is_some() {
                    break;
                }
            }
        }
    }
    if closing_balance.is_none() {
        closing_balance = transactions.last().and_then(|t| t.balance_after);
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(BankStatement::new(
        "VBA".to_string(),
        BankType::Agribank,
        StatementFormat::Csv,
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

/// Tokenizes HTML content to extract rows and cells `<tr><td>...</td></tr>`.
fn extract_html_table_rows(html: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let lower = html.to_lowercase();
    let mut pos = 0;

    while let Some(tr_start) = lower[pos..].find("<tr") {
        let abs_tr_start = pos + tr_start;
        let tr_end = match lower[abs_tr_start..].find("</tr>") {
            Some(e) => abs_tr_start + e + 5,
            None => html.len(),
        };

        let tr_chunk = &html[abs_tr_start..tr_end];
        let cells = extract_cells_from_row(tr_chunk);
        if !cells.is_empty() {
            rows.push(cells);
        }

        pos = tr_end;
        if pos >= html.len() {
            break;
        }
    }

    rows
}

fn extract_cells_from_row(row_html: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let lower = row_html.to_lowercase();
    let mut pos = 0;

    while pos < row_html.len() {
        let td_start = lower[pos..].find("<td");
        let th_start = lower[pos..].find("<th");

        let tag_start = match (td_start, th_start) {
            (Some(d), Some(h)) => Some(pos + d.min(h)),
            (Some(d), None) => Some(pos + d),
            (None, Some(h)) => Some(pos + h),
            (None, None) => None,
        };

        let abs_tag_start = match tag_start {
            Some(p) => p,
            None => break,
        };

        let close_tag_start = match lower[abs_tag_start..].find('>') {
            Some(c) => abs_tag_start + c + 1,
            None => break,
        };

        let end_tag = if lower[abs_tag_start..].starts_with("<th") {
            "</th>"
        } else {
            "</td>"
        };
        let abs_tag_end = match lower[close_tag_start..].find(end_tag) {
            Some(e) => close_tag_start + e,
            None => row_html.len(),
        };

        let inner_text = &row_html[close_tag_start..abs_tag_end];
        let cleaned = clean_html_cell(inner_text);
        cells.push(cleaned);

        pos = abs_tag_end + end_tag.len();
    }

    cells
}

fn clean_html_cell(cell: &str) -> String {
    let mut res = String::with_capacity(cell.len());
    let mut in_tag = false;

    for c in cell.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            res.push(c);
        }
    }

    res.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .trim()
        .to_string()
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

fn detect_delimiter(text: &str) -> u8 {
    let mut comma_count = 0;
    let mut semi_count = 0;
    let mut tab_count = 0;
    let mut pipe_count = 0;

    for line in text.lines().take(20) {
        let mut in_quotes = false;
        for b in line.bytes() {
            if b == b'"' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                match b {
                    b',' => comma_count += 1,
                    b';' => semi_count += 1,
                    b'\t' => tab_count += 1,
                    b'|' => pipe_count += 1,
                    _ => {}
                }
            }
        }
    }

    if semi_count > comma_count && semi_count > tab_count && semi_count > pipe_count {
        b';'
    } else if tab_count > comma_count && tab_count > semi_count && tab_count > pipe_count {
        b'\t'
    } else if pipe_count > comma_count && pipe_count > semi_count && pipe_count > tab_count {
        b'|'
    } else {
        b','
    }
}

fn extract_account_number(line: &str) -> Option<String> {
    let parts: Vec<&str> = line
        .split(|c: char| c == ':' || c == '-' || c == ',' || c == ';' || c == '/')
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

fn extract_balance_from_record(record: &csv::StringRecord) -> Option<u64> {
    for field in record {
        let fl = field.to_lowercase();
        if fl.contains("số dư")
            || fl.contains("closing")
            || fl.contains("dư cuối")
            || fl.contains("opening")
            || fl.contains("đầu kỳ")
        {
            if let Some(pos) = field.rfind(':') {
                let after = field[pos + 1..].trim();
                if let Some(amt) = parse_vietnamese_amount(after).filter(|&amt| amt > 0) {
                    return Some(amt);
                }
            }
            if let Some(amt) = parse_vietnamese_amount(field).filter(|&amt| amt > 0) {
                return Some(amt);
            }
        }
    }
    for field in record {
        if let Some(pos) = field.rfind(':') {
            let after = field[pos + 1..].trim();
            if let Some(amt) = parse_vietnamese_amount(after).filter(|&amt| amt > 0) {
                return Some(amt);
            }
        }
    }
    for field in record {
        if let Some(amt) = parse_vietnamese_amount(field).filter(|&amt| amt > 0) {
            return Some(amt);
        }
    }
    None
}

fn extract_balance_from_str_slice(row: &[String]) -> Option<u64> {
    for cell in row {
        if let Some(amt) = parse_vietnamese_amount(cell) {
            if amt > 0 {
                return Some(amt);
            }
        }
        if let Some(pos) = cell.rfind(':') {
            let after = cell[pos + 1..].trim();
            if let Some(amt) = parse_vietnamese_amount(after) {
                if amt > 0 {
                    return Some(amt);
                }
            }
        }
    }
    None
}

fn extract_agri_ref(narration: &str) -> Option<String> {
    for token in narration.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if (clean.starts_with("FT") || clean.starts_with("VB")) && clean.len() >= 8 {
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
