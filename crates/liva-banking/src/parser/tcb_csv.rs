//! Techcombank (TCB) CSV Statement Parser.

use super::{BankStatementParser, ParserError};
use crate::models::{
    BankStatement, BankType, StatementFormat, StatementStatus, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};
use std::io::Cursor;

pub struct TcbCsvParser;

impl BankStatementParser for TcbCsvParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("tcb") || lower.contains("techcombank") {
            return true;
        }

        let slice_len = bytes.len().min(1024);
        let sample = String::from_utf8_lossy(&bytes[..slice_len]).to_lowercase();
        sample.contains("techcombank")
            || sample.contains("tcb")
            || sample.contains("napas")
            || sample.contains("vietqr")
            || ((lower.ends_with(".csv") || lower.ends_with(".txt"))
                && (sample.contains("ngày giao dịch")
                    || sample.contains("mã giao dịch")
                    || sample.contains("số tiền")))
    }

    fn parse(&self, raw_bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
        // Strip BOM if present
        let bytes = if raw_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            &raw_bytes[3..]
        } else {
            raw_bytes
        };

        let sample = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]);
        let delimiter = if sample.contains(';') && sample.matches(';').count() > sample.matches(',').count() {
            b';'
        } else if sample.contains('\t') {
            b'\t'
        } else {
            b','
        };

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .flexible(true)
            .from_reader(Cursor::new(bytes));

        let mut transactions = Vec::new();
        let mut account_number = None;
        let mut account_name = None;
        let mut opening_balance = None;
        let mut closing_balance = None;

        let mut header_found = false;
        let mut date_col = 0;
        let mut ref_col = 1;
        let mut debit_col = None;
        let mut credit_col = None;
        let mut amount_col = None;
        let mut type_col = None;
        let mut balance_col = None;
        let mut narration_col = 2;
        let mut counterparty_col = None;

        let mut row_idx = 0;

        for result in rdr.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => return Err(ParserError::CsvError(e.to_string())),
            };

            let row: Vec<String> = record.iter().map(|s| s.trim().to_string()).collect();
            if row.is_empty() || row.iter().all(|s| s.is_empty()) {
                continue;
            }

            // Detect metadata in header rows
            let row_str = row.join(" ").to_lowercase();
            if row_str.contains("số tài khoản") || row_str.contains("tài khoản số") {
                for cell in &row {
                    let c = cell.trim();
                    if c.chars().all(|ch| ch.is_ascii_digit()) && c.len() >= 9 {
                        account_number = Some(c.to_string());
                    }
                }
            }
            if row_str.contains("chủ tài khoản") || row_str.contains("tên tài khoản") {
                for (i, cell) in row.iter().enumerate() {
                    if (cell.to_lowercase().contains("chủ tài khoản") || cell.to_lowercase().contains("tên tài khoản")) && i + 1 < row.len() {
                        account_name = Some(row[i + 1].clone());
                    }
                }
            }
            if row_str.contains("số dư đầu") || row_str.contains("opening balance") {
                for cell in &row {
                    if let Some(amt) = parse_vietnamese_amount(cell) {
                        if opening_balance.is_none() && amt > 0 {
                            opening_balance = Some(amt);
                        }
                    }
                }
            }
            if row_str.contains("số dư cuối") || row_str.contains("closing balance") {
                for cell in &row {
                    if let Some(amt) = parse_vietnamese_amount(cell) {
                        if closing_balance.is_none() && amt > 0 {
                            closing_balance = Some(amt);
                        }
                    }
                }
            }

            if !header_found {
                // Check if this row looks like column headers
                let is_header = row.iter().any(|c| {
                    let l = c.to_lowercase();
                    l.contains("ngày") || l.contains("date") || l.contains("mô tả") || l.contains("diễn giải")
                });

                if is_header {
                    header_found = true;
                    for (i, col) in row.iter().enumerate() {
                        let l = col.to_lowercase();
                        if l.contains("ngày") || l.contains("date") {
                            date_col = i;
                        } else if l.contains("số tham chiếu") || l.contains("mã gd") || l.contains("ref") {
                            ref_col = i;
                        } else if l.contains("nợ") || l.contains("rút") || l.contains("debit") {
                            debit_col = Some(i);
                        } else if l.contains("có") || l.contains("gửi") || l.contains("credit") {
                            credit_col = Some(i);
                        } else if l.contains("số tiền") || l.contains("amount") {
                            amount_col = Some(i);
                        } else if l.contains("loại gd") || l.contains("type") {
                            type_col = Some(i);
                        } else if l.contains("số dư") || l.contains("balance") {
                            balance_col = Some(i);
                        } else if l.contains("diễn giải") || l.contains("mô tả") || l.contains("nội dung") || l.contains("description") {
                            narration_col = i;
                        } else if l.contains("đối tác") || l.contains("người nhận") || l.contains("counterparty") {
                            counterparty_col = Some(i);
                        }
                    }
                    continue;
                }
            }

            if !header_found {
                continue;
            }

            // Parse transaction row
            let date_str = row.get(date_col).map(|s| s.as_str()).unwrap_or("");
            let tx_date = match parse_banking_date(date_str) {
                Some(d) => d,
                None => continue, // Not a valid data row
            };

            let doc_ref = row.get(ref_col).filter(|s| !s.is_empty()).cloned();
            let narration = row.get(narration_col).cloned().unwrap_or_default();
            let balance_after = balance_col.and_then(|c| row.get(c)).and_then(|s| parse_vietnamese_amount(s));
            let counterparty_name = counterparty_col.and_then(|c| row.get(c)).filter(|s| !s.is_empty()).cloned();

            let (tx_type, amount) = if let (Some(db_c), Some(cr_c)) = (debit_col, credit_col) {
                let db_val = row.get(db_c).and_then(|s| parse_vietnamese_amount(s)).unwrap_or(0);
                let cr_val = row.get(cr_c).and_then(|s| parse_vietnamese_amount(s)).unwrap_or(0);
                if db_val > 0 {
                    (TransactionType::Debit, db_val)
                } else if cr_val > 0 {
                    (TransactionType::Credit, cr_val)
                } else {
                    continue;
                }
            } else if let Some(amt_c) = amount_col {
                let amt = row.get(amt_c).and_then(|s| parse_vietnamese_amount(s)).unwrap_or(0);
                if amt == 0 {
                    continue;
                }
                let t_type = if let Some(tc) = type_col {
                    row.get(tc).and_then(|s| s.parse().ok()).unwrap_or(TransactionType::Credit)
                } else {
                    TransactionType::Credit
                };
                (t_type, amt)
            } else {
                continue;
            };

            row_idx += 1;
            transactions.push(TransactionRecord::new(
                row_idx,
                tx_date,
                tx_date,
                doc_ref,
                tx_type,
                amount,
                balance_after,
                None,
                counterparty_name,
                Some("Techcombank".to_string()),
                narration,
            ));
        }

        if transactions.is_empty() {
            return Err(ParserError::InvalidStructure(format!(
                "No valid transaction records found in TCB statement: {filename}"
            )));
        }

        // If closing balance missing from header, infer from last transaction's balance_after
        if closing_balance.is_none() {
            closing_balance = transactions.last().and_then(|t| t.balance_after);
        }

        Ok(BankStatement {
            bank_code: "TCB".to_string(),
            bank_type: BankType::Techcombank,
            format: StatementFormat::Csv,
            account_number,
            account_name,
            opening_balance,
            closing_balance,
            total_credit: 0,
            total_debit: 0,
            balance_checksum_passed: false,
            status: StatementStatus::Validating,
            file_hash_sha256: None,
            transactions,
        })
    }
}
