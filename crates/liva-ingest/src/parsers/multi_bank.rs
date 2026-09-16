//! Multi-bank tabular statement parser for VietinBank, MBBank, Agribank, and generic CSV/Excel.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use crate::models::{ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use liva_normalize::{parse_monetary_amount, BankIdentifier};
use std::io::Cursor;

pub struct MultiBankTableParser;

impl StatementParser for MultiBankTableParser {
    fn can_parse(&self, container: ContainerFormat, bank: BankIdentifier) -> bool {
        match container {
            ContainerFormat::TextCsv | ContainerFormat::ExcelZip | ContainerFormat::ExcelOle | ContainerFormat::HtmlTable => {
                matches!(
                    bank,
                    BankIdentifier::VietinBank
                        | BankIdentifier::MbBank
                        | BankIdentifier::Agribank
                        | BankIdentifier::Unknown
                )
            }
            _ => false,
        }
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        let text_attempt = String::from_utf8_lossy(bytes);

        // Check if plain text delimited (CSV)
        if text_attempt.contains(',') || text_attempt.contains(';') || text_attempt.contains('\t') {
            if let Ok(record) = parse_delimited_statement(&text_attempt, filename) {
                return Ok(record);
            }
        }

        // Try Excel workbook
        if let Ok(mut workbook) = open_workbook_auto_from_rs(Cursor::new(bytes)) {
            if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
                return parse_excel_range(&range, filename);
            }
        }

        Err(IngestError::UnsupportedFormat {
            filename: filename.to_string(),
        })
    }
}

fn parse_delimited_statement(text: &str, filename: &str) -> Result<RawStatementRecord, IngestError> {
    let mut delimiter = b',';
    for line in text.lines().take(10) {
        if line.matches(';').count() > line.matches(',').count() {
            delimiter = b';';
            break;
        } else if line.matches('\t').count() > line.matches(',').count() {
            delimiter = b'\t';
            break;
        }
    }

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .has_headers(false)
        .from_reader(text.as_bytes());

    let mut bank = BankIdentifier::Unknown;
    let mut account_no = None;
    let opening_balance = None;
    let closing_balance = None;

    let mut header_idx: Option<usize> = None;
    let mut col_date = None;
    let mut col_credit = None;
    let mut col_debit = None;
    let mut col_amount = None;
    let mut col_balance = None;
    let mut col_ref = None;
    let mut col_narration = None;
    let mut col_party = None;

    let mut rows_raw = Vec::new();
    for rec in rdr.records().flatten() {
        let row_vec: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
        rows_raw.push(row_vec);
    }

    for (r_idx, row) in rows_raw.iter().enumerate() {
        let full_row = row.join(" ");
        let lower = full_row.to_lowercase();

        if lower.contains("vietinbank") || lower.contains("icbv") {
            bank = BankIdentifier::VietinBank;
        } else if lower.contains("mbbank") || lower.contains("mscb") {
            bank = BankIdentifier::MbBank;
        } else if lower.contains("agribank") || lower.contains("vbaa") {
            bank = BankIdentifier::Agribank;
        }

        if (lower.contains("số tài khoản") || lower.contains("so tai khoan") || lower.contains("account no")) && account_no.is_none() {
            for cell in row {
                let digits: String = cell.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() >= 8 {
                    account_no = Some(digits);
                    break;
                }
            }
        }

        // Header detection
        if (lower.contains("ngày") || lower.contains("date"))
            && (lower.contains("tiền") || lower.contains("amount") || lower.contains("phát sinh") || lower.contains("nợ") || lower.contains("có"))
        {
            header_idx = Some(r_idx);
            for (c_idx, cell) in row.iter().enumerate() {
                let h_lower = cell.to_lowercase();
                if h_lower.contains("ngày") || h_lower.contains("date") {
                    col_date = Some(c_idx);
                } else if h_lower.contains("có") || h_lower.contains("credit") {
                    col_credit = Some(c_idx);
                } else if h_lower.contains("nợ") || h_lower.contains("debit") {
                    col_debit = Some(c_idx);
                } else if h_lower.contains("số tiền") || h_lower.contains("amount") {
                    col_amount = Some(c_idx);
                } else if h_lower.contains("số dư") || h_lower.contains("balance") {
                    col_balance = Some(c_idx);
                } else if h_lower.contains("tham chiếu") || h_lower.contains("số gd") || h_lower.contains("ref") {
                    col_ref = Some(c_idx);
                } else if h_lower.contains("nội dung") || h_lower.contains("diễn giải") || h_lower.contains("narration") || h_lower.contains("description") {
                    col_narration = Some(c_idx);
                } else if h_lower.contains("đối tác") || h_lower.contains("người nhận") || h_lower.contains("người chuyển") {
                    col_party = Some(c_idx);
                }
            }
            break;
        }
    }

    let start_row = header_idx.map(|idx| idx + 1).unwrap_or(0);
    let mut transactions = Vec::new();
    let mut row_id = 0;

    for row in rows_raw.iter().skip(start_row) {
        if row.is_empty() {
            continue;
        }

        let date_str = col_date.and_then(|c| row.get(c)).cloned().unwrap_or_default();
        if date_str.is_empty() || date_str.chars().all(|c| !c.is_ascii_digit()) {
            continue;
        }

        let mut credit_cents = None;
        let mut debit_cents = None;

        if let Some(c) = col_credit.and_then(|idx| row.get(idx)) {
            if let Ok(parsed) = parse_monetary_amount(c) {
                if parsed.minor_units > 0 {
                    credit_cents = Some(parsed.minor_units);
                }
            }
        }

        if let Some(d) = col_debit.and_then(|idx| row.get(idx)) {
            if let Ok(parsed) = parse_monetary_amount(d) {
                if parsed.minor_units > 0 {
                    debit_cents = Some(parsed.minor_units);
                }
            }
        }

        let (is_credit, amount_cents) = if let Some(cr) = credit_cents {
            (true, cr)
        } else if let Some(db) = debit_cents {
            (false, db)
        } else if let Some(amt_str) = col_amount.and_then(|idx| row.get(idx)) {
            if let Ok(parsed) = parse_monetary_amount(amt_str) {
                (true, parsed.minor_units)
            } else {
                continue;
            }
        } else {
            continue;
        };

        let narration = col_narration.and_then(|idx| row.get(idx)).cloned().unwrap_or_default();
        let doc_ref = col_ref.and_then(|idx| row.get(idx)).cloned();
        let counterparty = col_party.and_then(|idx| row.get(idx)).cloned();

        let balance_cents = col_balance
            .and_then(|idx| row.get(idx))
            .and_then(|s| parse_monetary_amount(s).ok())
            .map(|a| a.minor_units);

        row_id += 1;
        transactions.push(RawTransactionRecord {
            row_id,
            date_str,
            val_date_str: None,
            doc_ref,
            debit_amt_str: None,
            credit_amt_str: None,
            amount_cents: Some(amount_cents),
            is_credit,
            balance_str: None,
            balance_cents,
            narration,
            counterparty_name: counterparty,
        });
    }

    if transactions.is_empty() {
        return Err(IngestError::InvalidStructure(format!(
            "Failed to parse any transaction records from delimited file {filename}"
        )));
    }

    Ok(RawStatementRecord {
        bank,
        format: ContainerFormat::TextCsv,
        account_no,
        account_name: None,
        opening_balance,
        closing_balance,
        transactions,
    })
}

fn parse_excel_range(
    range: &calamine::Range<Data>,
    filename: &str,
) -> Result<RawStatementRecord, IngestError> {
    let mut rows_raw: Vec<Vec<String>> = Vec::new();
    for row in range.rows() {
        let row_vec: Vec<String> = row.iter().map(cell_to_string).collect();
        rows_raw.push(row_vec);
    }

    let fake_csv = rows_raw
        .into_iter()
        .map(|r| r.join("\t"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut record = parse_delimited_statement(&fake_csv, filename)?;
    record.format = ContainerFormat::ExcelZip;
    Ok(record)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            // Note: float format only for reading raw string cell, not arithmetic
            format!("{:.0}", f)
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => format!("{}", dt),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{:?}", e),
    }
}
