//! Techcombank (TCB) CSV statement parser.
//! Handles dynamic delimiter sniffing (semicolon default), UTF-8 BOM stripping, and Napas/VietQR memos.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use crate::models::{ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser};
use csv::ReaderBuilder;
use encoding_rs::WINDOWS_1258;
use liva_normalize::{normalize_datetime, parse_monetary_amount, BankIdentifier};

pub struct TcbParser;

impl StatementParser for TcbParser {
    fn can_parse(&self, container: ContainerFormat, bank: BankIdentifier) -> bool {
        container == ContainerFormat::TextCsv
            && (bank == BankIdentifier::Techcombank || bank == BankIdentifier::Unknown)
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        let text = decode_text_with_bom(bytes);
        let delimiter = sniff_delimiter(&text);

        let mut rdr = ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .flexible(true)
            .from_reader(text.as_bytes());

        let mut account_number: Option<String> = None;
        let mut account_name: Option<String> = None;
        let mut opening_balance: Option<u64> = None;
        let mut closing_balance: Option<u64> = None;

        let mut header_found = false;
        let mut col_date: usize = 0;
        let mut col_ref: usize = 1;
        let mut col_debit: usize = 2;
        let mut col_credit: usize = 3;
        let mut col_balance: usize = 4;
        let mut col_narration: usize = 5;
        let mut col_counterparty: Option<usize> = Some(6);

        let mut transactions = Vec::new();
        let mut row_counter = 0;

        for result in rdr.records() {
            let record = result.map_err(|e| IngestError::Csv(format!("CSV read error: {e}")))?;
            if record.is_empty() {
                continue;
            }

            let first_col = record.get(0).unwrap_or("").trim();
            let row_line = record.iter().collect::<Vec<_>>().join(" ");
            let row_lower = row_line.to_lowercase();

            if !header_found {
                if row_lower.contains("số tài khoản:") || row_lower.contains("account no") {
                    for cell in record.iter().skip(1) {
                        let digits: String = cell.chars().filter(|c| c.is_ascii_digit()).collect();
                        if !digits.is_empty() {
                            account_number = Some(digits);
                            break;
                        }
                    }
                } else if row_lower.contains("tên tài khoản:") || row_lower.contains("account name") {
                    for cell in record.iter().skip(1) {
                        let trimmed = cell.trim();
                        if !trimmed.is_empty() {
                            account_name = Some(trimmed.to_string());
                            break;
                        }
                    }
                } else if row_lower.contains("số dư đầu kỳ:") || row_lower.contains("opening balance") {
                    for cell in record.iter().skip(1) {
                        if let Ok(amt) = parse_monetary_amount(cell) {
                            opening_balance = Some(amt.minor_units);
                            break;
                        }
                    }
                }

                // Check for column headers
                if (first_col.to_lowercase().contains("ngày") || row_lower.contains("ngày giao dịch"))
                    && (row_lower.contains("nợ") || row_lower.contains("có"))
                {
                    header_found = true;
                    for (c_idx, cell) in record.iter().enumerate() {
                        let c_lower = cell.to_lowercase();
                        if c_lower.contains("ngày") {
                            col_date = c_idx;
                        } else if c_lower.contains("mã giao dịch") || c_lower.contains("chứng từ") {
                            col_ref = c_idx;
                        } else if c_lower.contains("ghi nợ") || c_lower.contains("nợ") {
                            col_debit = c_idx;
                        } else if c_lower.contains("ghi có") || c_lower.contains("có") {
                            col_credit = c_idx;
                        } else if c_lower.contains("số dư") {
                            col_balance = c_idx;
                        } else if c_lower.contains("nội dung") || c_lower.contains("diễn giải") {
                            col_narration = c_idx;
                        } else if c_lower.contains("đối tác") {
                            col_counterparty = Some(c_idx);
                        }
                    }
                }
                continue;
            }

            // After header: check footer
            if row_lower.contains("tổng phát sinh") || row_lower.contains("số dư cuối kỳ") {
                // Look for closing balance in footer
                if let Some(pos) = row_line.find("Số dư cuối kỳ:") {
                    let part = &row_line[pos + "Số dư cuối kỳ:".len()..];
                    if let Ok(amt) = parse_monetary_amount(part) {
                        closing_balance = Some(amt.minor_units);
                    }
                } else if let Some(cell) = record.get(col_balance) {
                    if let Ok(amt) = parse_monetary_amount(cell) {
                        closing_balance = Some(amt.minor_units);
                    }
                }
                continue;
            }

            // Transaction row: must have a valid date
            let date_cell = record.get(col_date).unwrap_or("").trim();
            if normalize_datetime(date_cell).is_err() {
                continue;
            }

            let debit_cell = record.get(col_debit).unwrap_or("").trim();
            let credit_cell = record.get(col_credit).unwrap_or("").trim();

            let debit_amt = parse_monetary_amount(debit_cell)
                .map(|p| p.minor_units)
                .unwrap_or(0);
            let credit_amt = parse_monetary_amount(credit_cell)
                .map(|p| p.minor_units)
                .unwrap_or(0);

            let (is_credit, amount_cents) = if credit_amt > 0 {
                (true, credit_amt)
            } else if debit_amt > 0 {
                (false, debit_amt)
            } else {
                continue;
            };

            let doc_ref = record
                .get(col_ref)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let balance_cell = record.get(col_balance).unwrap_or("").trim();
            let balance_cents = parse_monetary_amount(balance_cell)
                .map(|p| p.minor_units)
                .ok();

            let narration = record
                .get(col_narration)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let counterparty_name = col_counterparty
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            row_counter += 1;
            transactions.push(RawTransactionRecord {
                row_id: row_counter,
                date_str: date_cell.to_string(),
                val_date_str: None,
                doc_ref,
                debit_amt_str: if debit_amt > 0 { Some(debit_amt.to_string()) } else { None },
                credit_amt_str: if credit_amt > 0 { Some(credit_amt.to_string()) } else { None },
                amount_cents: Some(amount_cents),
                is_credit,
                balance_str: balance_cents.map(|b| b.to_string()),
                balance_cents,
                narration,
                counterparty_name,
            });
        }

        if transactions.is_empty() {
            return Err(IngestError::InvalidStructure(format!(
                "No transactions extracted from Techcombank statement: {filename}"
            )));
        }

        Ok(RawStatementRecord {
            bank: BankIdentifier::Techcombank,
            format: ContainerFormat::TextCsv,
            account_no: account_number,
            account_name,
            opening_balance,
            closing_balance,
            transactions,
        })
    }
}

/// Decodes bytes into UTF-8, stripping BOM and falling back to Windows-1258.
fn decode_text_with_bom(bytes: &[u8]) -> String {
    // Check UTF-8 BOM
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // Windows-1258 fallback
            let (decoded, _, _) = WINDOWS_1258.decode(bytes);
            decoded.into_owned()
        }
    }
}

/// Dynamically sniffs whether CSV uses semicolon, comma, tab, or pipe.
fn sniff_delimiter(text: &str) -> u8 {
    let mut count_semi = 0;
    let mut count_comma = 0;
    let mut count_tab = 0;
    let mut count_pipe = 0;

    for line in text.lines().take(10) {
        count_semi += line.chars().filter(|&c| c == ';').count();
        count_comma += line.chars().filter(|&c| c == ',').count();
        count_tab += line.chars().filter(|&c| c == '\t').count();
        count_pipe += line.chars().filter(|&c| c == '|').count();
    }

    if count_semi >= count_comma && count_semi >= count_tab && count_semi >= count_pipe && count_semi > 0 {
        b';'
    } else if count_tab >= count_comma && count_tab > 0 {
        b'\t'
    } else if count_pipe >= count_comma && count_pipe > 0 {
        b'|'
    } else {
        b','
    }
}
