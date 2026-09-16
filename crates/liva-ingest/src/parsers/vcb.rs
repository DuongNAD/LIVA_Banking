//! Vietcombank (VCB) statement parser.
//! Handles merged date cells forward-fill, bottom-up closing balance scan, and inlineStr.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use crate::models::{ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use liva_normalize::{normalize_datetime, parse_monetary_amount, BankIdentifier};
use std::io::Cursor;

pub struct VcbParser;

impl StatementParser for VcbParser {
    fn can_parse(&self, container: ContainerFormat, bank: BankIdentifier) -> bool {
        (container == ContainerFormat::ExcelZip || container == ContainerFormat::ExcelOle)
            && (bank == BankIdentifier::Vietcombank || bank == BankIdentifier::Unknown)
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes))
            .map_err(|e| IngestError::Excel(format!("Failed to open Excel workbook: {e}")))?;

        let range = workbook
            .worksheet_range_at(0)
            .ok_or_else(|| IngestError::InvalidStructure("Workbook contains no sheets".to_string()))?
            .map_err(|e| IngestError::Excel(format!("Failed to read sheet 0: {e}")))?;

        let rows: Vec<&[Data]> = range.rows().collect();
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

        // 1. Scan metadata and locate table header
        for (r_idx, row) in rows.iter().enumerate() {
            let row_str = row
                .iter()
                .map(cell_to_string)
                .collect::<Vec<_>>()
                .join(" ");
            let lower = row_str.to_lowercase();

            if lower.contains("số tài khoản:") || lower.contains("số tk:") || lower.contains("account no:") {
                if let Some(pos) = row_str.find(':') {
                    let num = row_str[pos + 1..]
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>();
                    if !num.is_empty() {
                        account_number = Some(num);
                    }
                }
            }

            if lower.contains("tên tài khoản:") || lower.contains("tên tk:") || lower.contains("account name:") {
                if let Some(pos) = row_str.find(':') {
                    let name = row_str[pos + 1..].trim().to_string();
                    if !name.is_empty() {
                        account_name = Some(name);
                    }
                }
            }

            if lower.contains("số dư đầu kỳ:") || lower.contains("opening balance:") {
                if let Some(pos) = row_str.find(':') {
                    let val_str = &row_str[pos + 1..];
                    if let Ok(amt) = parse_monetary_amount(val_str) {
                        opening_balance = Some(amt.minor_units);
                    }
                }
            }

            if lower.contains("số dư cuối kỳ:") || lower.contains("closing balance:") {
                if let Some(pos) = row_str.find(':') {
                    let val_str = &row_str[pos + 1..];
                    if let Ok(amt) = parse_monetary_amount(val_str) {
                        closing_balance = Some(amt.minor_units);
                    }
                }
            }

            // Identify column header row
            if (lower.contains("ngày giao dịch") || lower.contains("ngày gd") || lower.contains("trans date"))
                && (lower.contains("nợ") || lower.contains("debit") || lower.contains("có") || lower.contains("credit"))
            {
                header_row_idx = Some(r_idx);
                for (c_idx, cell) in row.iter().enumerate() {
                    let col_name = cell_to_string(cell).to_lowercase();
                    if col_name.contains("ngày giao dịch") || col_name.contains("ngày gd") || col_name.contains("trans date") {
                        col_date = Some(c_idx);
                    } else if col_name.contains("ngày giá trị") || col_name.contains("value date") {
                        col_val_date = Some(c_idx);
                    } else if col_name.contains("chứng từ") || col_name.contains("voucher") || col_name.contains("doc no") {
                        col_ref = Some(c_idx);
                    } else if col_name.contains("ghi nợ") || col_name.contains("nợ") || col_name.contains("debit") {
                        col_debit = Some(c_idx);
                    } else if col_name.contains("ghi có") || col_name.contains("có") || col_name.contains("credit") {
                        col_credit = Some(c_idx);
                    } else if col_name.contains("số dư") || col_name.contains("balance") {
                        col_balance = Some(c_idx);
                    } else if col_name.contains("nội dung") || col_name.contains("diễn giải") || col_name.contains("narration") {
                        col_narration = Some(c_idx);
                    } else if col_name.contains("đối tác") || col_name.contains("counterparty") {
                        col_counterparty = Some(c_idx);
                    }
                }
                break;
            }
        }

        // Bottom-up scan for closing balance if not found in top rows
        if closing_balance.is_none() {
            for row in rows.iter().rev().take(30) {
                let row_str = row
                    .iter()
                    .map(cell_to_string)
                    .collect::<Vec<_>>()
                    .join(" ");
                let lower = row_str.to_lowercase();
                if lower.contains("số dư cuối kỳ") || lower.contains("closing balance") {
                    // Try parsing after colon or inspecting numeric cells in this row
                    if let Some(pos) = row_str.find(':') {
                        let val_str = &row_str[pos + 1..];
                        if let Ok(amt) = parse_monetary_amount(val_str) {
                            closing_balance = Some(amt.minor_units);
                            break;
                        }
                    }
                    for cell in row.iter().rev() {
                        if let Some(amt) = cell_to_amount(cell) {
                            closing_balance = Some(amt);
                            break;
                        }
                    }
                    if closing_balance.is_some() {
                        break;
                    }
                }
            }
        }

        let header_idx = header_row_idx.ok_or_else(|| {
            IngestError::InvalidStructure(format!(
                "Could not locate transaction header row in VCB statement: {filename}"
            ))
        })?;

        let c_date = col_date.unwrap_or(0);
        let c_debit = col_debit.unwrap_or(3);
        let c_credit = col_credit.unwrap_or(4);
        let c_balance = col_balance.unwrap_or(5);
        let c_narration = col_narration.unwrap_or(6);

        let mut transactions = Vec::new();
        let mut last_valid_date: Option<String> = None;

        for (row_offset, row) in rows.iter().skip(header_idx + 1).enumerate() {
            if row.is_empty() {
                continue;
            }

            let first_cell = row.first().map(cell_to_string).unwrap_or_default();
            let first_lower = first_cell.to_lowercase();
            if first_lower.contains("tổng")
                || first_lower.contains("số dư cuối kỳ")
                || first_lower.contains("total")
            {
                break;
            }

            let debit_amt = row.get(c_debit).and_then(cell_to_amount).unwrap_or(0);
            let credit_amt = row.get(c_credit).and_then(cell_to_amount).unwrap_or(0);

            let (is_credit, amount_cents) = if credit_amt > 0 {
                (true, credit_amt)
            } else if debit_amt > 0 {
                (false, debit_amt)
            } else {
                continue; // Skip spacer or zero rows
            };

            // Merged date cell forward-fill
            let date_cell = row.get(c_date).map(cell_to_string).unwrap_or_default();
            let date_str = if normalize_datetime(&date_cell).is_ok() {
                last_valid_date = Some(date_cell.clone());
                date_cell
            } else if let Some(ref lvd) = last_valid_date {
                lvd.clone()
            } else {
                continue; // Cannot determine transaction date
            };

            let val_date_str = col_val_date
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .filter(|s| !s.is_empty());

            let doc_ref = col_ref
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .filter(|s| !s.is_empty());

            let balance_cents = row.get(c_balance).and_then(cell_to_amount);

            let narration = row
                .get(c_narration)
                .map(cell_to_string)
                .unwrap_or_default();

            let counterparty_name = col_counterparty
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .filter(|s| !s.is_empty());

            transactions.push(RawTransactionRecord {
                row_id: row_offset + 1,
                date_str,
                val_date_str,
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

        Ok(RawStatementRecord {
            bank: BankIdentifier::Vietcombank,
            format: ContainerFormat::ExcelZip,
            account_no: account_number,
            account_name,
            opening_balance,
            closing_balance,
            transactions,
        })
    }
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Int(v) => v.to_string(),
        Data::Float(v) => {
            let u = *v as u64;
            u.to_string()
        }
        Data::String(s) => s.trim().to_string(),
        Data::DateTime(d) => d.to_string(),
        _ => String::new(),
    }
}

fn cell_to_amount(cell: &Data) -> Option<u64> {
    match cell {
        Data::Int(v) if *v >= 0 => Some(*v as u64),
        Data::Float(v) if *v >= 0.0 => Some(*v as u64),
        Data::String(s) => parse_monetary_amount(s).ok().map(|p| p.minor_units),
        _ => None,
    }
}
