//! VietinBank (CTG) Statement Parser.
//!
//! Supports:
//! - Excel statements (.xlsx / .xls) via `calamine`.
//! - CSV statements via `csv::ReaderBuilder` with quote-aware delimiter detection and Windows-1258.
//! - PDF statements via `lopdf` with spatial clustering ($\Delta y \le 2.5\text{ pt}$) and column intervals.

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use lopdf::content::Content;
use lopdf::{Document, Object};
use std::io::Cursor;
use unicode_normalization::UnicodeNormalization;

use super::{BankStatementParser, ParserError};
use crate::banking::models::{
    BankStatement, BankType, StatementFormat, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};

pub struct VietinBankParser;

#[derive(Debug, Clone)]
struct TextElement {
    x: f32,
    y: f32,
    text: String,
}

impl BankStatementParser for VietinBankParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("ctg") || lower.contains("vietinbank") || lower.contains("vietin") {
            return true;
        }

        // PDF check
        if bytes.starts_with(b"%PDF-") {
            if let Ok(doc) = Document::load_mem(bytes) {
                for (&page_num, _) in doc.get_pages().iter().take(2) {
                    if let Ok(text) = doc.extract_text(&[page_num]) {
                        let t_lower = text.to_lowercase();
                        if t_lower.contains("vietinbank")
                            || t_lower.contains("công thương")
                            || t_lower.contains("ctg")
                        {
                            return true;
                        }
                    }
                }
            }
            return false;
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
                            if s.contains("vietinbank")
                                || s.contains("công thương")
                                || s.contains("ctg")
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
        if sample.contains("vietinbank") || sample.contains("công thương") || sample.contains("ctg")
        {
            return true;
        }

        false
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
        let start_time = std::time::Instant::now();

        // 1. Detect format by magic bytes
        if bytes.starts_with(b"%PDF-") {
            return parse_ctg_pdf(bytes, start_time);
        }

        if bytes.len() >= 4
            && (bytes[0..4] == [0x50, 0x4B, 0x03, 0x04] || bytes[0..4] == [0xD0, 0xCF, 0x11, 0xE0])
        {
            return parse_ctg_excel(bytes, filename, start_time);
        }

        // Default to CSV
        parse_ctg_csv(bytes, filename, start_time)
    }
}

fn parse_ctg_excel(
    bytes: &[u8],
    filename: &str,
    start_time: std::time::Instant,
) -> Result<BankStatement, ParserError> {
    let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes))
        .map_err(|e| ParserError::ExcelError(format!("Failed to open CTG Excel workbook: {e}")))?;

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
    let mut col_amount: Option<usize> = None;
    let mut col_direction: Option<usize> = None;
    let mut col_debit: Option<usize> = None;
    let mut col_credit: Option<usize> = None;
    let mut col_balance: Option<usize> = None;
    let mut col_narration: Option<usize> = None;

    for (r_idx, row) in rows.iter().enumerate() {
        let row_str = row.iter().map(cell_to_string).collect::<Vec<_>>().join(" ");
        let row_lower = row_str.to_lowercase();

        if r_idx < 25 {
            if row_lower.contains("số tài khoản") || row_lower.contains("account no") {
                account_number = extract_account_number(&row_str);
            }
            if row_lower.contains("tên tài khoản") || row_lower.contains("account name") {
                account_name = extract_account_name(&row_str);
            }
            if row_lower.contains("số dư đầu kỳ") || row_lower.contains("opening balance") {
                opening_balance = extract_balance_from_row(row);
            }
            if row_lower.contains("số dư cuối kỳ") || row_lower.contains("closing balance") {
                closing_balance = extract_balance_from_row(row);
            }
        }

        if (row_lower.contains("ngày") || row_lower.contains("date"))
            && (row_lower.contains("chứng từ")
                || row_lower.contains("phiếu")
                || row_lower.contains("số tiền")
                || row_lower.contains("nợ")
                || row_lower.contains("có"))
        {
            header_row_idx = Some(r_idx);
            for (c_idx, cell) in row.iter().enumerate() {
                let h = cell_to_string(cell).trim().to_lowercase();
                if (h.contains("ngày") || h.contains("date"))
                    && !h.contains("giá trị")
                    && !h.contains("value")
                {
                    col_date = Some(c_idx);
                } else if h.contains("giá trị") || h.contains("value date") {
                    col_val_date = Some(c_idx);
                } else if h.contains("phiếu")
                    || h.contains("chứng từ")
                    || h.contains("doc no")
                    || h.contains("ref")
                    || h.contains("mã gd")
                {
                    col_ref = Some(c_idx);
                } else if h.contains("nợ/có")
                    || h.contains("nợ / có")
                    || h.contains("d/c")
                    || h == "loại gd"
                {
                    col_direction = Some(c_idx);
                } else if h.contains("ghi nợ") || h == "nợ" || h.contains("debit") {
                    col_debit = Some(c_idx);
                } else if h.contains("ghi có") || h == "có" || h.contains("credit") {
                    col_credit = Some(c_idx);
                } else if h.contains("số tiền") || h.contains("amount") {
                    col_amount = Some(c_idx);
                } else if h.contains("dư") || h.contains("balance") {
                    col_balance = Some(c_idx);
                } else if h.contains("nội dung")
                    || h.contains("diễn giải")
                    || h.contains("description")
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
            "Could not find transaction header row in CTG statement: {filename}"
        ))
    })?;

    let c_date = col_date.unwrap_or(0);
    let c_narration = col_narration.unwrap_or(
        rows.get(header_idx)
            .map(|r| r.len())
            .unwrap_or(8)
            .saturating_sub(1),
    );

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

        let debit_amt = col_debit
            .and_then(|idx| row.get(idx))
            .and_then(cell_to_amount)
            .unwrap_or(0);
        let credit_amt = col_credit
            .and_then(|idx| row.get(idx))
            .and_then(cell_to_amount)
            .unwrap_or(0);

        let (tx_type, amount) = if let (Some(c_amt), Some(c_dir)) = (col_amount, col_direction) {
            let amt = row.get(c_amt).and_then(cell_to_amount).unwrap_or(0);
            let dir_str = row
                .get(c_dir)
                .map(cell_to_string)
                .unwrap_or_default()
                .trim()
                .to_uppercase();
            let is_debit =
                dir_str.starts_with('N') || dir_str.starts_with('D') || dir_str.contains("NỢ");
            let t_type = if is_debit {
                TransactionType::Debit
            } else {
                TransactionType::Credit
            };
            (t_type, amt)
        } else if let Some(c_amt) = col_amount {
            let amt = row.get(c_amt).and_then(cell_to_amount).unwrap_or(0);
            (TransactionType::Credit, amt)
        } else if debit_amt > 0 {
            (TransactionType::Debit, debit_amt)
        } else if credit_amt > 0 {
            (TransactionType::Credit, credit_amt)
        } else {
            continue;
        };

        if amount == 0 {
            continue;
        }

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

        let balance_after = col_balance
            .and_then(|idx| row.get(idx))
            .and_then(cell_to_amount);
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
            .filter(|r| r.starts_with("FT") || r.starts_with("TF") || r.starts_with("GD"))
            .cloned()
            .or_else(|| extract_ft_reference(&narration));

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
        "CTG".to_string(),
        BankType::VietinBank,
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

fn parse_ctg_csv(
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
    let mut col_amount: Option<usize> = None;
    let mut col_direction: Option<usize> = None;
    let mut col_debit: Option<usize> = None;
    let mut col_credit: Option<usize> = None;
    let mut col_balance: Option<usize> = None;
    let mut col_narration: Option<usize> = None;

    for (r_idx, record) in records.iter().enumerate() {
        let line = record.iter().collect::<Vec<_>>().join(" ");
        let line_lower = line.to_lowercase();

        if r_idx < 25 {
            if line_lower.contains("số tài khoản") || line_lower.contains("account no") {
                account_number = extract_account_number(&line);
            }
            if line_lower.contains("tên tài khoản") || line_lower.contains("chủ tài khoản")
            {
                account_name = extract_account_name(&line);
            }
            if line_lower.contains("số dư đầu kỳ") || line_lower.contains("opening balance")
            {
                opening_balance = extract_balance_from_record(record);
            }
            if line_lower.contains("số dư cuối kỳ") || line_lower.contains("closing balance")
            {
                closing_balance = extract_balance_from_record(record);
            }
        }

        if (line_lower.contains("ngày") || line_lower.contains("date"))
            && (line_lower.contains("chứng từ")
                || line_lower.contains("phiếu")
                || line_lower.contains("số tiền")
                || line_lower.contains("nợ")
                || line_lower.contains("có")
                || line_lower.contains("amount")
                || line_lower.contains("credit")
                || line_lower.contains("debit")
                || line_lower.contains("doc no")
                || line_lower.contains("ref"))
        {
            header_row_idx = Some(r_idx);
            for (c_idx, field) in record.iter().enumerate() {
                let h = field.trim().to_lowercase();
                if (h.contains("ngày") || h.contains("date"))
                    && !h.contains("giá trị")
                    && !h.contains("value")
                {
                    col_date = Some(c_idx);
                } else if h.contains("giá trị") || h.contains("value date") {
                    col_val_date = Some(c_idx);
                } else if h.contains("phiếu")
                    || h.contains("chứng từ")
                    || h.contains("doc no")
                    || h.contains("ref")
                    || h.contains("mã gd")
                {
                    col_ref = Some(c_idx);
                } else if h.contains("nợ/có")
                    || h.contains("nợ / có")
                    || h.contains("d/c")
                    || h == "loại gd"
                {
                    col_direction = Some(c_idx);
                } else if h.contains("ghi nợ") || h == "nợ" || h.contains("debit") {
                    col_debit = Some(c_idx);
                } else if h.contains("ghi có") || h == "có" || h.contains("credit") {
                    col_credit = Some(c_idx);
                } else if h.contains("số tiền") || h.contains("amount") {
                    col_amount = Some(c_idx);
                } else if h.contains("dư") || h.contains("balance") {
                    col_balance = Some(c_idx);
                } else if h.contains("nội dung")
                    || h.contains("diễn giải")
                    || h.contains("description")
                {
                    col_narration = Some(c_idx);
                }
            }
            break;
        }
    }

    let header_idx = header_row_idx.ok_or_else(|| {
        ParserError::InvalidStructure(format!(
            "Could not find CSV header row in CTG statement: {filename}"
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
        } else if let (Some(c_amt), Some(c_dir)) = (col_amount, col_direction) {
            let amt = record
                .get(c_amt)
                .and_then(parse_vietnamese_amount)
                .unwrap_or(0);
            let dir_str = record.get(c_dir).unwrap_or("").trim().to_uppercase();
            let is_debit =
                dir_str.starts_with('N') || dir_str.starts_with('D') || dir_str.contains("NỢ");
            let t_type = if is_debit {
                TransactionType::Debit
            } else {
                TransactionType::Credit
            };
            (t_type, amt)
        } else if let Some(c_amt) = col_amount {
            let raw_val = record.get(c_amt).unwrap_or("").trim();
            let is_neg = raw_val.starts_with('-') || raw_val.starts_with('(');
            let amt = parse_vietnamese_amount(raw_val).unwrap_or(0);
            let t_type = if is_neg {
                TransactionType::Debit
            } else {
                TransactionType::Credit
            };
            (t_type, amt)
        } else {
            continue;
        };

        if amount == 0 {
            continue;
        }

        let balance_after = col_balance
            .and_then(|idx| record.get(idx))
            .and_then(parse_vietnamese_amount);

        let raw_narration = record.get(c_narration).unwrap_or("").trim().to_string();
        if doc_ref.is_none() {
            doc_ref = extract_ft_reference(&raw_narration);
        }

        let ft_number = doc_ref
            .as_ref()
            .filter(|r| r.starts_with("FT") || r.starts_with("TF") || r.starts_with("GD"))
            .cloned()
            .or_else(|| extract_ft_reference(&raw_narration));

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
        "CTG".to_string(),
        BankType::VietinBank,
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

fn parse_ctg_pdf(
    bytes: &[u8],
    start_time: std::time::Instant,
) -> Result<BankStatement, ParserError> {
    let doc = Document::load_mem(bytes)
        .map_err(|e| ParserError::PdfError(format!("Failed to load CTG PDF bytes: {e}")))?;

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

    for (&page_num, &page_id) in &pages {
        let mut elements = extract_page_elements(&doc, page_id);
        if elements.is_empty() {
            if let Ok(text) = doc.extract_text(&[page_num]) {
                elements = text_to_synthetic_elements(&text);
            }
        }

        let lines = group_elements_into_lines(elements);
        let mut in_table = false;

        for line in &lines {
            let line_text: String = line
                .iter()
                .map(|e| e.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let line_lower = line_text.to_lowercase();

            if account_number.is_none()
                && (line_lower.contains("số tài khoản") || line_lower.contains("account no"))
            {
                account_number = extract_account_number(&line_text);
            }
            if account_name.is_none()
                && (line_lower.contains("tên khách hàng") || line_lower.contains("chủ tài khoản"))
            {
                account_name = extract_account_name(&line_text);
            }
            if line_lower.contains("số dư đầu kỳ") || line_lower.contains("opening balance")
            {
                if let Some(amt) = extract_amount_from_line(line) {
                    global_opening_balance.get_or_insert(amt);
                }
            }
            if line_lower.contains("số dư cuối kỳ") || line_lower.contains("closing balance")
            {
                if let Some(amt) = extract_amount_from_line(line) {
                    global_closing_balance = Some(amt);
                }
            }

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
                    }
                }
                continue;
            }

            if !in_table {
                continue;
            }

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
                .or(global_opening_balance);

            if let Some(mut tx) =
                try_parse_ctg_pdf_row(line, x_debit, x_credit, x_balance, last_bal)
            {
                tx.row_id = all_transactions.len() + 1;
                min_tx_date = Some(min_tx_date.map_or(tx.tx_date, |m| m.min(tx.tx_date)));
                max_tx_date = Some(max_tx_date.map_or(tx.tx_date, |m| m.max(tx.tx_date)));
                all_transactions.push(tx);
            } else if let Some(last_tx) = all_transactions.last_mut() {
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
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(BankStatement::new(
        "CTG".to_string(),
        BankType::VietinBank,
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

fn try_parse_ctg_pdf_row(
    line: &[TextElement],
    x_debit: Option<f32>,
    x_credit: Option<f32>,
    x_balance: Option<f32>,
    last_balance: Option<u64>,
) -> Option<TransactionRecord> {
    if line.is_empty() {
        return None;
    }

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
                || token.starts_with("TF")
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
        .filter(|r| r.starts_with("FT") || r.starts_with("TF") || r.starts_with("GD"))
        .cloned()
        .or_else(|| extract_ft_reference(&narration));
    let trace_id = extract_trace_reference(&narration);

    Some(TransactionRecord {
        row_id: 0,
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

fn extract_ft_reference(narration: &str) -> Option<String> {
    for token in narration.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if (clean.starts_with("FT") || clean.starts_with("TF") || clean.starts_with("GD"))
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

fn is_noise_footer(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("ngân hàng tmcp công thương")
        || lower.contains("vietinbank.vn")
        || lower.contains("trang ")
        || lower.contains("page ")
        || lower.contains("hotline")
        || lower.contains("tổng phát sinh")
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

    let mut cur_x: f32 = 0.0;
    let mut cur_y: f32 = 0.0;

    for op in &content.operations {
        match op.operator.as_str() {
            "Tm" => {
                if op.operands.len() >= 6 {
                    if let (Ok(x), Ok(y)) = (op.operands[4].as_float(), op.operands[5].as_float()) {
                        cur_x = x;
                        cur_y = y;
                    }
                }
            }
            "Td" | "TD" => {
                if op.operands.len() >= 2 {
                    if let (Ok(dx), Ok(dy)) = (op.operands[0].as_float(), op.operands[1].as_float())
                    {
                        cur_x += dx;
                        cur_y += dy;
                    }
                }
            }
            "T*" => {
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

fn group_elements_into_lines(mut elements: Vec<TextElement>) -> Vec<Vec<TextElement>> {
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
