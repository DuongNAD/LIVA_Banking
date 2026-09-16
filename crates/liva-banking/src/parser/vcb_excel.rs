//! Vietcombank (VCB) Excel Statement Parser.

use super::{BankStatementParser, ParserError};
use crate::models::{
    BankStatement, BankType, StatementFormat, StatementStatus, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};
use calamine::{Data, Reader, open_workbook_auto_from_rs};
use std::io::Cursor;

pub struct VcbExcelParser;

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => format!("{:.0}", f),
        Data::Int(i) => i.to_string(),
        Data::DateTime(dt) => format!("{:.0}", dt),
        _ => String::new(),
    }
}

impl BankStatementParser for VcbExcelParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("vcb") || lower.contains("vietcombank") {
            return true;
        }

        if bytes.len() >= 4 {
            let is_xlsx = bytes[0..4] == [0x50, 0x4B, 0x03, 0x04];
            let is_xls = bytes[0..4] == [0xD0, 0xCF, 0x11, 0xE0];
            if is_xlsx || is_xls {
                if let Ok(mut workbook) = open_workbook_auto_from_rs(Cursor::new(bytes)) {
                    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
                        for row in range.rows().take(20) {
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
        let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes))
            .map_err(|e| ParserError::ExcelError(format!("Failed to open Excel: {e}")))?;

        let range = workbook
            .worksheet_range_at(0)
            .ok_or_else(|| ParserError::ExcelError("Workbook has no sheets".to_string()))?
            .map_err(|e| ParserError::ExcelError(format!("Failed to read sheet 0: {e}")))?;

        let mut account_number = None;
        let mut account_name = None;
        let mut opening_balance = None;
        let mut closing_balance = None;
        let mut transactions = Vec::new();

        let mut header_found = false;
        let mut date_col = 0;
        let mut doc_col = 1;
        let mut debit_col = None;
        let mut credit_col = None;
        let mut balance_col = None;
        let mut narration_col = 4;
        let mut counterparty_col = None;
        let mut row_idx = 0;

        for row in range.rows() {
            let row_str: Vec<String> = row.iter().map(cell_to_string).collect();
            if row_str.iter().all(|s| s.is_empty()) {
                continue;
            }

            let full_row_text = row_str.join(" ").to_lowercase();

            // Metadata discovery
            if full_row_text.contains("số tài khoản") || full_row_text.contains("tài khoản số") {
                for cell in &row_str {
                    let c = cell.trim();
                    if c.chars().all(|ch| ch.is_ascii_digit()) && c.len() >= 9 {
                        account_number = Some(c.to_string());
                    }
                }
            }
            if full_row_text.contains("tên tài khoản") || full_row_text.contains("chủ tài khoản") {
                for (i, cell) in row_str.iter().enumerate() {
                    if (cell.to_lowercase().contains("tên tài khoản") || cell.to_lowercase().contains("chủ tài khoản"))
                        && i + 1 < row_str.len()
                    {
                        let next_val = &row_str[i + 1];
                        if !next_val.is_empty() {
                            account_name = Some(next_val.clone());
                        }
                    }
                }
            }
            if full_row_text.contains("số dư đầu") || full_row_text.contains("số dư ban đầu") {
                for cell in &row_str {
                    if let Some(amt) = parse_vietnamese_amount(cell) {
                        if opening_balance.is_none() && amt > 0 {
                            opening_balance = Some(amt);
                        }
                    }
                }
            }
            if full_row_text.contains("số dư cuối") {
                for cell in &row_str {
                    if let Some(amt) = parse_vietnamese_amount(cell) {
                        if closing_balance.is_none() && amt > 0 {
                            closing_balance = Some(amt);
                        }
                    }
                }
            }

            if !header_found {
                let is_header = row_str.iter().any(|c| {
                    let l = c.to_lowercase();
                    l.contains("ngày gd") || l.contains("ngày hạch toán") || l.contains("số tham chiếu")
                });

                if is_header {
                    header_found = true;
                    for (i, col) in row_str.iter().enumerate() {
                        let l = col.to_lowercase();
                        if l.contains("ngày") {
                            date_col = i;
                        } else if l.contains("tham chiếu") || l.contains("chứng từ") {
                            doc_col = i;
                        } else if l.contains("nợ") || l.contains("rút") {
                            debit_col = Some(i);
                        } else if l.contains("có") || l.contains("gửi") {
                            credit_col = Some(i);
                        } else if l.contains("số dư") {
                            balance_col = Some(i);
                        } else if l.contains("nội dung") || l.contains("diễn giải") {
                            narration_col = i;
                        } else if l.contains("đối ứng") || l.contains("người nhận") {
                            counterparty_col = Some(i);
                        }
                    }
                    continue;
                }
            }

            if !header_found {
                continue;
            }

            let date_str = row_str.get(date_col).map(|s| s.as_str()).unwrap_or("");
            let tx_date = match parse_banking_date(date_str) {
                Some(d) => d,
                None => continue,
            };

            let doc_ref = row_str.get(doc_col).filter(|s| !s.is_empty()).cloned();
            let narration = row_str.get(narration_col).cloned().unwrap_or_default();
            let balance_after = balance_col
                .and_then(|c| row_str.get(c))
                .and_then(|s| parse_vietnamese_amount(s));
            let counterparty_name = counterparty_col
                .and_then(|c| row_str.get(c))
                .filter(|s| !s.is_empty())
                .cloned();

            let (tx_type, amount) = if let (Some(db_c), Some(cr_c)) = (debit_col, credit_col) {
                let db_val = row_str.get(db_c).and_then(|s| parse_vietnamese_amount(s)).unwrap_or(0);
                let cr_val = row_str.get(cr_c).and_then(|s| parse_vietnamese_amount(s)).unwrap_or(0);
                if db_val > 0 {
                    (TransactionType::Debit, db_val)
                } else if cr_val > 0 {
                    (TransactionType::Credit, cr_val)
                } else {
                    continue;
                }
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
                Some("Vietcombank".to_string()),
                narration,
            ));
        }

        if transactions.is_empty() {
            return Err(ParserError::InvalidStructure(format!(
                "No valid transactions parsed from VCB Excel: {filename}"
            )));
        }

        if closing_balance.is_none() {
            closing_balance = transactions.last().and_then(|t| t.balance_after);
        }

        Ok(BankStatement {
            bank_code: "VCB".to_string(),
            bank_type: BankType::Vietcombank,
            format: StatementFormat::Excel,
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
