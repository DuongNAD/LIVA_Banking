//! MBBank Statement Parser.
//!
//! Features:
//! - Dual-language headers (Vietnamese / English).
//! - Dedicated counterparty columns (Counterparty Account, Counterparty Name, Counterparty Bank).
//! - Excel (.xlsx/.xls) parsing via `calamine` with merged cell forward-fill.
//! - CSV parsing via `csv::ReaderBuilder` with quote-aware delimiter sniffing and Windows-1258.

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use std::io::Cursor;
use unicode_normalization::UnicodeNormalization;

use super::{BankStatementParser, ParserError};
use crate::banking::models::{
    BankStatement, BankType, StatementFormat, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};

pub struct MbBankParser;

impl BankStatementParser for MbBankParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("mbbank")
            || lower.contains("mb bank")
            || lower.contains("mbb")
            || lower.starts_with("mb_")
        {
            return true;
        }

        // Excel check
        if bytes.len() >= 4
            && (bytes[0..4] == [0x50, 0x4B, 0x03, 0x04] || bytes[0..4] == [0xD0, 0xCF, 0x11, 0xE0])
        {
            if let Ok(mut wb) = open_workbook_auto_from_rs(Cursor::new(bytes)) {
                if let Some(Ok(range)) = wb.worksheet_range_at(0) {
                    for row in range.rows().take(15) {
                        for cell in row {
                            let s = cell_to_string(cell).to_lowercase();
                            if s.contains("mbbank")
                                || s.contains("mb bank")
                                || s.contains("quân đội")
                            {
                                return true;
                            }
                        }
                    }
                }
            }
            return false;
        }

        // CSV/Text check
        let slice_len = bytes.len().min(1024);
        let sample = String::from_utf8_lossy(&bytes[..slice_len]).to_lowercase();
        if sample.contains("mbbank") || sample.contains("mb bank") || sample.contains("quân đội")
        {
            return true;
        }

        false
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
        let start_time = std::time::Instant::now();

        if bytes.len() >= 4
            && (bytes[0..4] == [0x50, 0x4B, 0x03, 0x04] || bytes[0..4] == [0xD0, 0xCF, 0x11, 0xE0])
        {
            return parse_mb_excel(bytes, filename, start_time);
        }

        parse_mb_csv(bytes, filename, start_time)
    }
}

fn parse_mb_excel(
    bytes: &[u8],
    filename: &str,
    start_time: std::time::Instant,
) -> Result<BankStatement, ParserError> {
    let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes)).map_err(|e| {
        ParserError::ExcelError(format!("Failed to open MBBank Excel workbook: {e}"))
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
    let mut col_cp_acc: Option<usize> = None;
    let mut col_cp_name: Option<usize> = None;
    let mut col_cp_bank: Option<usize> = None;
    let mut col_debit: Option<usize> = None;
    let mut col_credit: Option<usize> = None;
    let mut col_balance: Option<usize> = None;
    let mut col_narration: Option<usize> = None;

    for (r_idx, row) in rows.iter().enumerate() {
        let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
        let row_lower = row_str.to_lowercase();

        if r_idx < 25 {
            if row_lower.contains("tài khoản") || row_lower.contains("account") {
                if account_number.is_none() {
                    account_number = extract_account_number(&row_str);
                }
            }
            if row_lower.contains("tên tài khoản")
                || row_lower.contains("tên khách hàng")
                || row_lower.contains("chủ tài khoản")
                || row_lower.contains("account name")
            {
                if account_name.is_none() {
                    account_name = extract_account_name(&row_str);
                }
            }
            if row_lower.contains("số dư đầu kỳ")
                || row_lower.contains("số dư ban đầu")
                || row_lower.contains("opening balance")
            {
                if opening_balance.is_none() {
                    opening_balance = extract_balance_from_row(row);
                }
            }
            if row_lower.contains("số dư cuối kỳ") || row_lower.contains("closing balance") {
                if closing_balance.is_none() {
                    closing_balance = extract_balance_from_row(row);
                }
            }
        }

        // Dual-language header recognition
        if (row_lower.contains("ngày")
            || row_lower.contains("date")
            || row_lower.contains("trans date"))
            && (row_lower.contains("tiền")
                || row_lower.contains("nợ")
                || row_lower.contains("có")
                || row_lower.contains("debit")
                || row_lower.contains("credit"))
        {
            header_row_idx = Some(r_idx);
            for (c_idx, cell) in row.iter().enumerate() {
                let h = cell_to_string(cell).trim().to_lowercase();
                if (h.contains("ngày") || h.contains("date"))
                    && !h.contains("giá trị")
                    && !h.contains("value")
                {
                    col_date = Some(c_idx);
                } else if h.contains("ngày giá trị") || h.contains("value date") {
                    col_val_date = Some(c_idx);
                } else if h.contains("mã giao dịch")
                    || h.contains("trans no")
                    || h.contains("trans id")
                    || h.contains("ref no")
                    || h.contains("số gd")
                    || h.contains("chứng từ")
                {
                    col_ref = Some(c_idx);
                } else if h.contains("đối ứng")
                    || h.contains("thụ hưởng")
                    || h.contains("counterparty")
                    || h.contains("beneficiary")
                {
                    if h.contains("tên") || h.contains("name") {
                        col_cp_name = Some(c_idx);
                    } else if h.contains("ngân hàng") || h.contains("bank") {
                        col_cp_bank = Some(c_idx);
                    } else {
                        col_cp_acc = Some(c_idx);
                    }
                } else if h.contains("ghi nợ") || h == "nợ" || h.contains("debit") {
                    col_debit = Some(c_idx);
                } else if h.contains("ghi có") || h == "có" || h.contains("credit") {
                    col_credit = Some(c_idx);
                } else if h.contains("số dư") || h.contains("balance") {
                    col_balance = Some(c_idx);
                } else if h.contains("nội dung")
                    || h.contains("diễn giải")
                    || h.contains("description")
                    || h.contains("narration")
                {
                    col_narration = Some(c_idx);
                }
            }
            break;
        }
    }

    if closing_balance.is_none() {
        for row in rows.iter().rev().take(30) {
            let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
            let row_lower = row_str.to_lowercase();
            if row_lower.contains("số dư cuối kỳ") || row_lower.contains("closing balance") {
                closing_balance = extract_balance_from_row(row);
                if closing_balance.is_some() {
                    break;
                }
            }
        }
    }

    let header_idx = header_row_idx.ok_or_else(|| {
        ParserError::InvalidStructure(format!(
            "Could not find transaction header row in MBBank statement: {filename}"
        ))
    })?;

    let c_date = col_date.unwrap_or(0);
    let c_debit = col_debit.unwrap_or(4);
    let c_credit = col_credit.unwrap_or(5);
    let c_balance = col_balance.unwrap_or(6);
    let c_narration = col_narration.unwrap_or(7);

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
            doc_ref = extract_mb_ref(&narration);
        }

        let cp_acc = col_cp_acc
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| extract_cp_acc(&narration));

        let cp_name = col_cp_name
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| extract_cp_name(&narration));

        let cp_bank = col_cp_bank
            .and_then(|idx| row.get(idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let ft_number = doc_ref
            .as_ref()
            .filter(|r| r.starts_with("FT") || r.starts_with("MB"))
            .cloned()
            .or_else(|| extract_mb_ref(&narration));

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
            counterparty_account: cp_acc,
            counterparty_name: cp_name,
            counterparty_bank: cp_bank,
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
            {
                if let Some(bal) = extract_balance_from_row(row) {
                    closing_balance = Some(bal);
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
        "MB".to_string(),
        BankType::MbBank,
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

fn parse_mb_csv(
    raw_bytes: &[u8],
    filename: &str,
    start_time: std::time::Instant,
) -> Result<BankStatement, ParserError> {
    let text = if raw_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        match std::str::from_utf8(&raw_bytes[3..]) {
            Ok(s) => s.to_string(),
            Err(_) => {
                let (cow, _, _) = encoding_rs::WINDOWS_1258.decode(&raw_bytes[3..]);
                cow.nfc().collect()
            }
        }
    } else {
        match std::str::from_utf8(raw_bytes) {
            Ok(s) => s.to_string(),
            Err(_) => {
                let (cow, _, _) = encoding_rs::WINDOWS_1258.decode(raw_bytes);
                cow.nfc().collect()
            }
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
    let mut col_val_date: Option<usize> = None;
    let mut col_ref: Option<usize> = None;
    let mut col_cp_acc: Option<usize> = None;
    let mut col_cp_name: Option<usize> = None;
    let mut col_cp_bank: Option<usize> = None;
    let mut col_debit: Option<usize> = None;
    let mut col_credit: Option<usize> = None;
    let mut col_balance: Option<usize> = None;
    let mut col_narration: Option<usize> = None;

    for (r_idx, record) in records.iter().enumerate() {
        let line = record.iter().collect::<Vec<_>>().join(" ");
        let line_lower = line.to_lowercase();

        if r_idx < 25 {
            if line_lower.contains("tài khoản") || line_lower.contains("account") {
                if account_number.is_none() {
                    account_number = extract_account_number(&line);
                }
            }
            if line_lower.contains("tên tài khoản")
                || line_lower.contains("tên khách hàng")
                || line_lower.contains("chủ tài khoản")
                || line_lower.contains("account name")
            {
                if account_name.is_none() {
                    account_name = extract_account_name(&line);
                }
            }
            if line_lower.contains("số dư đầu kỳ")
                || line_lower.contains("số dư ban đầu")
                || line_lower.contains("opening balance")
            {
                if opening_balance.is_none() {
                    opening_balance = extract_balance_from_record(record);
                }
            }
            if line_lower.contains("số dư cuối kỳ") || line_lower.contains("closing balance")
            {
                if closing_balance.is_none() {
                    closing_balance = extract_balance_from_record(record);
                }
            }
        }

        if (line_lower.contains("ngày")
            || line_lower.contains("date")
            || line_lower.contains("trans date"))
            && (line_lower.contains("tiền")
                || line_lower.contains("nợ")
                || line_lower.contains("có")
                || line_lower.contains("debit")
                || line_lower.contains("credit"))
        {
            header_row_idx = Some(r_idx);
            for (c_idx, field) in record.iter().enumerate() {
                let h = field.trim().to_lowercase();
                if (h.contains("ngày") || h.contains("date"))
                    && !h.contains("giá trị")
                    && !h.contains("value")
                {
                    col_date = Some(c_idx);
                } else if h.contains("ngày giá trị") || h.contains("value date") {
                    col_val_date = Some(c_idx);
                } else if h.contains("mã giao dịch")
                    || h.contains("trans no")
                    || h.contains("trans id")
                    || h.contains("ref no")
                    || h.contains("số gd")
                    || h.contains("chứng từ")
                {
                    col_ref = Some(c_idx);
                } else if h.contains("đối ứng")
                    || h.contains("thụ hưởng")
                    || h.contains("counterparty")
                    || h.contains("beneficiary")
                {
                    if h.contains("tên") || h.contains("name") {
                        col_cp_name = Some(c_idx);
                    } else if h.contains("ngân hàng") || h.contains("bank") {
                        col_cp_bank = Some(c_idx);
                    } else {
                        col_cp_acc = Some(c_idx);
                    }
                } else if h.contains("ghi nợ") || h == "nợ" || h.contains("debit") {
                    col_debit = Some(c_idx);
                } else if h.contains("ghi có") || h == "có" || h.contains("credit") {
                    col_credit = Some(c_idx);
                } else if h.contains("số dư") || h.contains("balance") {
                    col_balance = Some(c_idx);
                } else if h.contains("nội dung")
                    || h.contains("diễn giải")
                    || h.contains("description")
                    || h.contains("narration")
                {
                    col_narration = Some(c_idx);
                }
            }
            break;
        }
    }

    let header_idx = header_row_idx.ok_or_else(|| {
        ParserError::InvalidStructure(format!(
            "Could not find CSV header row in MBBank statement: {filename}"
        ))
    })?;

    let c_date = col_date.unwrap_or(0);
    let c_narration = col_narration.unwrap_or(
        records
            .get(header_idx)
            .map(|r| r.len())
            .unwrap_or(8)
            .saturating_sub(1),
    );

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

        let val_date = col_val_date
            .and_then(|idx| record.get(idx))
            .and_then(parse_banking_date)
            .unwrap_or(tx_date);

        let mut doc_ref = col_ref
            .and_then(|idx| record.get(idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

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
        if doc_ref.is_none() {
            doc_ref = extract_mb_ref(&raw_narration);
        }

        let cp_acc = col_cp_acc
            .and_then(|idx| record.get(idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| extract_cp_acc(&raw_narration));

        let cp_name = col_cp_name
            .and_then(|idx| record.get(idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| extract_cp_name(&raw_narration));

        let cp_bank = col_cp_bank
            .and_then(|idx| record.get(idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let ft_number = doc_ref
            .as_ref()
            .filter(|r| r.starts_with("FT") || r.starts_with("MB"))
            .cloned()
            .or_else(|| extract_mb_ref(&raw_narration));

        let trace_id = extract_trace_reference(&raw_narration);

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
            counterparty_account: cp_acc,
            counterparty_name: cp_name,
            counterparty_bank: cp_bank,
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
            {
                if let Some(bal) = extract_balance_from_record(record) {
                    closing_balance = Some(bal);
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
        "MB".to_string(),
        BankType::MbBank,
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
        if clean.len() >= 8 && clean.len() <= 20 {
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

fn extract_mb_ref(narration: &str) -> Option<String> {
    for token in narration.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if (clean.starts_with("FT") || clean.starts_with("MB") || clean.starts_with("VN"))
            && clean.len() >= 8
        {
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

fn extract_cp_acc(narration: &str) -> Option<String> {
    let lower = narration.to_lowercase();
    for pattern in &["tk ", "stk ", "tai khoan ", "to "] {
        if let Some(pos) = lower.find(pattern) {
            let sub = &narration[pos + pattern.len()..];
            let num: String = sub.chars().take_while(|c| c.is_ascii_digit()).collect();
            if num.len() >= 8 && num.len() <= 16 {
                return Some(num);
            }
        }
    }
    None
}

fn extract_cp_name(narration: &str) -> Option<String> {
    if let Some(pos) = narration.find("Tu:") {
        let sub = narration[pos + 3..].split('-').next().unwrap_or("").trim();
        if !sub.is_empty() {
            return Some(sub.to_string());
        }
    }
    if let Some(pos) = narration.find("Den:") {
        let sub = narration[pos + 4..].split('-').next().unwrap_or("").trim();
        if !sub.is_empty() {
            return Some(sub.to_string());
        }
    }
    None
}
