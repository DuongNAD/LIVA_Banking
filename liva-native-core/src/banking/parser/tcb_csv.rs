//! Techcombank (TCB) CSV Statement Parser.
//!
//! Features:
//! - Stripping UTF-8 BOM (`\u{feff}`).
//! - Delimiter auto-detection (comma `,`, semicolon `;`, or tab `\t`).
//! - Zero-copy streaming via `csv::ReaderBuilder`.
//! - Parsing Napas reference codes and VietQR payment descriptions.

use std::io::Cursor;
use unicode_normalization::UnicodeNormalization;

use super::{BankStatementParser, ParserError};
use crate::banking::models::{
    BankStatement, BankType, StatementFormat, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};

pub struct TcbCsvParser;

impl BankStatementParser for TcbCsvParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("tcb") || lower.contains("techcombank") {
            return true;
        }

        let slice_len = bytes.len().min(1024);
        let sample = String::from_utf8_lossy(&bytes[..slice_len]).to_lowercase();
        if sample.contains("techcombank")
            || sample.contains("tcb")
            || sample.contains("napas")
            || sample.contains("vietqr")
        {
            return true;
        }

        // Generic CSV with banking terms
        if (lower.ends_with(".csv") || lower.ends_with(".txt"))
            && (sample.contains("ngày giao dịch")
                || sample.contains("mã giao dịch")
                || sample.contains("số tiền"))
        {
            return true;
        }

        false
    }

    fn parse(&self, raw_bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
        let start_time = std::time::Instant::now();

        // 1. Strip UTF-8 BOM or decode UTF-16 / Windows-1258 if present
        let text = if raw_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            match std::str::from_utf8(&raw_bytes[3..]) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    let (cow, _, _) = encoding_rs::WINDOWS_1258.decode(&raw_bytes[3..]);
                    cow.nfc().collect()
                }
            }
        } else if raw_bytes.starts_with(&[0xFF, 0xFE]) {
            let (cow, _, _) = encoding_rs::UTF_16LE.decode(&raw_bytes[2..]);
            cow.into_owned()
        } else if raw_bytes.starts_with(&[0xFE, 0xFF]) {
            let (cow, _, _) = encoding_rs::UTF_16BE.decode(&raw_bytes[2..]);
            cow.into_owned()
        } else {
            match std::str::from_utf8(raw_bytes) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    let (cow, _, _) = encoding_rs::WINDOWS_1258.decode(raw_bytes);
                    cow.nfc().collect()
                }
            }
        };

        // 2. Auto-detect delimiter with quote awareness
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
        let mut col_debit: Option<usize> = None;
        let mut col_credit: Option<usize> = None;
        let mut col_amount: Option<usize> = None;
        let mut col_balance: Option<usize> = None;
        let mut col_narration: Option<usize> = None;
        let mut col_counterparty_acc: Option<usize> = None;
        let mut col_counterparty_name: Option<usize> = None;
        let mut col_counterparty_bank: Option<usize> = None;

        // 3. Scan metadata & find table header
        for (r_idx, record) in records.iter().enumerate() {
            let line = record.iter().collect::<Vec<_>>().join(" ");
            let line_lower = line.to_lowercase();

            if r_idx < 20 {
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

            if (line_lower.contains("ngày")
                || line_lower.contains("date")
                || line_lower.contains("thời gian"))
                && (line_lower.contains("số tiền")
                    || line_lower.contains("tiền")
                    || line_lower.contains("nợ")
                    || line_lower.contains("có")
                    || line_lower.contains("amount")
                    || line_lower.contains("giao dịch"))
            {
                header_row_idx = Some(r_idx);
                for (c_idx, field) in record.iter().enumerate() {
                    let h = field.trim().to_lowercase();
                    if h.contains("ngày giao dịch")
                        || h.contains("ngày gd")
                        || h == "ngày"
                        || h == "date"
                        || h.contains("thời gian")
                    {
                        col_date = Some(c_idx);
                    } else if h.contains("ngày giá trị") || h.contains("value date") {
                        col_val_date = Some(c_idx);
                    } else if h.contains("mã giao dịch")
                        || h.contains("mã gd")
                        || h.contains("ft")
                        || h.contains("số tham chiếu")
                        || h.contains("ref")
                        || h.contains("chứng từ")
                    {
                        col_ref = Some(c_idx);
                    } else if h.contains("ghi nợ")
                        || h == "nợ"
                        || h.contains("debit")
                        || h.contains("tiền ra")
                    {
                        col_debit = Some(c_idx);
                    } else if h.contains("ghi có")
                        || h == "có"
                        || h.contains("credit")
                        || h.contains("tiền vào")
                    {
                        col_credit = Some(c_idx);
                    } else if h.contains("số tiền") || h == "tiền" || h == "amount" {
                        col_amount = Some(c_idx);
                    } else if h.contains("số dư") || h == "balance" {
                        col_balance = Some(c_idx);
                    } else if h.contains("nội dung")
                        || h.contains("diễn giải")
                        || h.contains("description")
                        || h.contains("chi tiết")
                    {
                        col_narration = Some(c_idx);
                    } else if h.contains("tài khoản đối ứng")
                        || h.contains("tk đối ứng")
                        || h.contains("số tk đối ứng")
                    {
                        col_counterparty_acc = Some(c_idx);
                    } else if h.contains("tên đối ứng")
                        || h.contains("người nhận")
                        || h.contains("người gửi")
                        || h.contains("tên người")
                    {
                        col_counterparty_name = Some(c_idx);
                    } else if h.contains("ngân hàng đối ứng")
                        || h.contains("ngân hàng")
                        || h.contains("bank")
                    {
                        col_counterparty_bank = Some(c_idx);
                    }
                }
                break;
            }
        }

        let header_idx = header_row_idx.ok_or_else(|| {
            ParserError::InvalidStructure(format!(
                "Could not find CSV header row in TCB statement: {filename}"
            ))
        })?;

        let c_date = col_date.unwrap_or(0);
        let c_narration =
            col_narration.unwrap_or(record_len(&records, header_idx).saturating_sub(1));

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
                None => continue, // Skip empty or summary rows
            };

            let val_date = col_val_date
                .and_then(|idx| record.get(idx))
                .and_then(parse_banking_date)
                .unwrap_or(tx_date);

            let mut doc_ref = col_ref
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            // Amounts
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
            } else if let Some(idx) = col_amount {
                let amt_str = record.get(idx).unwrap_or("");
                let is_neg =
                    amt_str.starts_with('-') || amt_str.contains("DB") || amt_str.contains("Nợ");
                let amt = parse_vietnamese_amount(amt_str).unwrap_or(0);
                if amt == 0 {
                    continue;
                }
                if is_neg {
                    (TransactionType::Debit, amt)
                } else {
                    (TransactionType::Credit, amt)
                }
            } else {
                continue;
            };

            let balance_after = col_balance
                .and_then(|idx| record.get(idx))
                .and_then(parse_vietnamese_amount);

            let raw_narration = record.get(c_narration).unwrap_or("").trim().to_string();

            // Extract Napas / VietQR metadata from narration if doc_ref is empty
            if doc_ref.is_none() {
                doc_ref = extract_napas_or_vietqr_ref(&raw_narration);
            }

            let counterparty_account = col_counterparty_acc
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| extract_counterparty_account(&raw_narration));

            let counterparty_name = col_counterparty_name
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| extract_counterparty_name(&raw_narration));

            let counterparty_bank = col_counterparty_bank
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let ft_number = doc_ref
                .as_ref()
                .filter(|r| r.starts_with("FT") || r.starts_with("VN"))
                .cloned()
                .or_else(|| extract_ft_ref(&raw_narration));

            let trace_id = doc_ref
                .as_ref()
                .filter(|r| r.starts_with("NPS"))
                .cloned()
                .or_else(|| extract_trace_id(&raw_narration));

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
                counterparty_account,
                counterparty_name,
                counterparty_bank,
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
            "TCB".to_string(),
            BankType::Techcombank,
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
}

/// Quote-aware delimiter detector supporting comma, semicolon, tab, and pipe.
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

fn record_len(records: &[csv::StringRecord], idx: usize) -> usize {
    records.get(idx).map(|r| r.len()).unwrap_or(0)
}

fn extract_account_number(line: &str) -> Option<String> {
    let parts: Vec<&str> = line
        .split(|c: char| c == ':' || c == '-' || c == ',' || c == ';')
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

/// Extracts Napas reference or VietQR code (e.g. `FT262568912345`, `NPS...`, `VN26...`)
fn extract_napas_or_vietqr_ref(narration: &str) -> Option<String> {
    let tokens: Vec<&str> = narration.split_whitespace().collect();
    for token in tokens {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.starts_with("FT") && clean.len() >= 10 {
            return Some(clean.to_string());
        }
        if clean.starts_with("NPS") && clean.len() >= 8 {
            return Some(clean.to_string());
        }
        if clean.starts_with("VN") && clean.len() >= 10 {
            return Some(clean.to_string());
        }
        if clean.starts_with("MBVCB") || clean.starts_with("IBFT") {
            return Some(clean.to_string());
        }
    }
    None
}

fn extract_ft_ref(narration: &str) -> Option<String> {
    for token in narration.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if (clean.starts_with("FT") || clean.starts_with("VN")) && clean.len() >= 10 {
            return Some(clean.to_string());
        }
    }
    None
}

fn extract_trace_id(narration: &str) -> Option<String> {
    let tokens: Vec<&str> = narration.split_whitespace().collect();
    for (i, &token) in tokens.iter().enumerate() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.starts_with("NPS") && clean.len() >= 8 {
            return Some(clean.to_string());
        }
        if clean.eq_ignore_ascii_case("trace") || clean.eq_ignore_ascii_case("trace:") {
            if let Some(next_token) = tokens.get(i + 1) {
                let next_clean = next_token.trim_matches(|c: char| !c.is_alphanumeric());
                if !next_clean.is_empty() {
                    return Some(next_clean.to_string());
                }
            }
        }
    }
    None
}

fn extract_counterparty_account(narration: &str) -> Option<String> {
    // Looks for patterns like "TK 1903... ", "STK 1903...", "to 001100..."
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

fn extract_counterparty_name(narration: &str) -> Option<String> {
    // E.g. "Nguyen Van A chuyen tien" or "Tu: CONG TY ABC" or "Den: NGUYEN VAN B"
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
