//! Dynamic Bank Profile statement parser for Excel (calamine) and CSV.
//! Supports customizable column mappings, delimiter sniffing/configuration,
//! merged cell forward-filling, and BOM stripping.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use crate::models::{ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser};
use crate::profile::BankProfile;
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use csv::ReaderBuilder;
use encoding_rs::WINDOWS_1258;
use liva_normalize::{normalize_datetime, parse_monetary_amount, BankIdentifier};
use std::io::Cursor;

/// Dynamic profile-driven statement parser.
pub struct ProfileStatementParser {
    pub profile: BankProfile,
}

impl ProfileStatementParser {
    /// Creates a new parser with the given dynamic bank profile.
    pub fn new(profile: BankProfile) -> Self {
        Self { profile }
    }

    /// Parses raw statement record from bytes using the associated bank profile.
    pub fn parse_bytes(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        match self.profile.format {
            ContainerFormat::ExcelZip | ContainerFormat::ExcelOle => {
                Self::parse_excel(bytes, filename, &self.profile)
            }
            ContainerFormat::TextCsv => Self::parse_csv(bytes, filename, &self.profile),
            _ => Err(IngestError::UnsupportedFormat {
                filename: filename.to_string(),
            }),
        }
    }

    /// Excel (.xlsx / .xls) parser with dynamic column mappings and merged cell forward fill.
    pub fn parse_excel(
        bytes: &[u8],
        filename: &str,
        profile: &BankProfile,
    ) -> Result<RawStatementRecord, IngestError> {
        let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes))
            .map_err(|e| IngestError::Excel(format!("Failed to open Excel workbook: {e}")))?;

        let range = workbook
            .worksheet_range_at(0)
            .ok_or_else(|| IngestError::InvalidStructure("Workbook contains no sheets".to_string()))?
            .map_err(|e| IngestError::Excel(format!("Failed to read sheet 0: {e}")))?;

        let rows: Vec<&[Data]> = range.rows().collect();
        if rows.is_empty() {
            return Err(IngestError::InvalidStructure(
                "Excel sheet contains no data".to_string(),
            ));
        }

        let mut account_number: Option<String> = None;
        let mut account_name: Option<String> = None;
        let mut opening_balance: Option<u64> = None;
        let mut closing_balance: Option<u64> = None;

        // Scan metadata rows (rows before data start)
        let meta_limit = profile.header_row_index.min(rows.len());
        for row in rows.iter().take(meta_limit) {
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
                    if let Some(amt) = parse_amount_with_profile(
                        val_str,
                        profile.decimal_separator,
                        profile.thousands_separator,
                    ) {
                        opening_balance = Some(amt);
                    }
                }
            }

            if lower.contains("số dư cuối kỳ:") || lower.contains("closing balance:") {
                if let Some(pos) = row_str.find(':') {
                    let val_str = &row_str[pos + 1..];
                    if let Some(amt) = parse_amount_with_profile(
                        val_str,
                        profile.decimal_separator,
                        profile.thousands_separator,
                    ) {
                        closing_balance = Some(amt);
                    }
                }
            }
        }

        // Header resolution
        if profile.header_row_index >= rows.len() {
            return Err(IngestError::InvalidStructure(format!(
                "Header row index {} exceeds total rows {}",
                profile.header_row_index,
                rows.len()
            )));
        }

        let header_row = rows[profile.header_row_index];
        let headers: Vec<String> = header_row.iter().map(cell_to_string).collect();
        let resolved = profile.resolve_columns(&headers);

        let c_date = resolved.date.ok_or_else(|| {
            IngestError::InvalidStructure(format!(
                "Could not resolve date column for profile '{}' in file {}",
                profile.profile_name, filename
            ))
        })?;

        let c_narration = resolved.narration.ok_or_else(|| {
            IngestError::InvalidStructure(format!(
                "Could not resolve narration column for profile '{}' in file {}",
                profile.profile_name, filename
            ))
        })?;

        let end_row_limit = rows.len().saturating_sub(profile.footer_skip_rows);
        let mut transactions = Vec::new();
        let mut last_valid_date: Option<String> = None;

        for (row_offset, row) in rows
            .iter()
            .enumerate()
            .skip(profile.data_start_row_index)
            .take(end_row_limit.saturating_sub(profile.data_start_row_index))
        {
            if row.is_empty() {
                continue;
            }

            let first_cell = row.first().map(cell_to_string).unwrap_or_default();
            let first_lower = first_cell.to_lowercase();
            if first_lower.contains("tổng")
                || first_lower.contains("tổng cộng")
                || first_lower.contains("total")
                || first_lower.contains("số dư cuối kỳ")
            {
                break;
            }

            // Resolve date with merged cell forward-filling
            let date_cell = row.get(c_date);
            let raw_date = date_cell.and_then(cell_to_date_string).unwrap_or_default();
            let date_str = if !raw_date.trim().is_empty() && normalize_datetime(&raw_date).is_ok() {
                last_valid_date = Some(raw_date.clone());
                raw_date
            } else if let Some(ref lvd) = last_valid_date {
                lvd.clone()
            } else {
                continue; // Skip row if no valid date can be established
            };

            // Resolve amount, debit, credit
            let (is_credit, amount_cents, debit_str, credit_str) =
                if let (Some(d_idx), Some(c_idx)) = (resolved.debit, resolved.credit) {
                    let debit_amt = row
                        .get(d_idx)
                        .and_then(|c| cell_to_amount(c, profile.decimal_separator, profile.thousands_separator))
                        .unwrap_or(0);
                    let credit_amt = row
                        .get(c_idx)
                        .and_then(|c| cell_to_amount(c, profile.decimal_separator, profile.thousands_separator))
                        .unwrap_or(0);

                    if credit_amt > 0 {
                        (true, credit_amt, None, Some(credit_amt.to_string()))
                    } else if debit_amt > 0 {
                        (false, debit_amt, Some(debit_amt.to_string()), None)
                    } else {
                        continue; // Skip spacer or 0-amount row
                    }
                } else if let Some(a_idx) = resolved.amount {
                    let amt = row
                        .get(a_idx)
                        .and_then(|c| cell_to_amount(c, profile.decimal_separator, profile.thousands_separator))
                        .unwrap_or(0);

                    if amt == 0 {
                        continue;
                    }

                    let is_cr = if let Some(flag_idx) = resolved.is_credit_flag {
                        let flag = row.get(flag_idx).map(cell_to_string).unwrap_or_default();
                        is_credit_flag_value(&flag)
                    } else {
                        true
                    };

                    let d_str = if !is_cr { Some(amt.to_string()) } else { None };
                    let c_str = if is_cr { Some(amt.to_string()) } else { None };
                    (is_cr, amt, d_str, c_str)
                } else {
                    continue;
                };

            let balance_cents = resolved
                .balance
                .and_then(|idx| row.get(idx))
                .and_then(|c| cell_to_amount(c, profile.decimal_separator, profile.thousands_separator));

            let doc_ref = resolved
                .doc_ref
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .filter(|s| !s.is_empty());

            let narration = row
                .get(c_narration)
                .map(cell_to_string)
                .unwrap_or_default();

            let val_date_str = resolved
                .val_date
                .and_then(|idx| row.get(idx))
                .and_then(cell_to_date_string)
                .filter(|s| !s.is_empty());

            let counterparty_name = resolved
                .counterparty
                .and_then(|idx| row.get(idx))
                .map(cell_to_string)
                .filter(|s| !s.is_empty());

            transactions.push(RawTransactionRecord {
                row_id: row_offset + 1,
                date_str,
                val_date_str,
                doc_ref,
                debit_amt_str: debit_str,
                credit_amt_str: credit_str,
                amount_cents: Some(amount_cents),
                is_credit,
                balance_str: balance_cents.map(|b| b.to_string()),
                balance_cents,
                narration,
                counterparty_name,
            });
        }

        // Bottom-up scan for closing balance if not yet located
        if closing_balance.is_none() {
            for row in rows.iter().rev().take(30) {
                let row_str = row
                    .iter()
                    .map(cell_to_string)
                    .collect::<Vec<_>>()
                    .join(" ");
                let lower = row_str.to_lowercase();
                if lower.contains("số dư cuối kỳ") || lower.contains("closing balance") {
                    if let Some(pos) = row_str.find(':') {
                        let val_str = &row_str[pos + 1..];
                        if let Some(amt) = parse_amount_with_profile(
                            val_str,
                            profile.decimal_separator,
                            profile.thousands_separator,
                        ) {
                            closing_balance = Some(amt);
                            break;
                        }
                    }
                    for cell in row.iter().rev() {
                        if let Some(amt) = cell_to_amount(cell, profile.decimal_separator, profile.thousands_separator) {
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

        Ok(RawStatementRecord {
            bank: profile.bank_identifier(),
            format: profile.format,
            account_no: account_number,
            account_name,
            opening_balance,
            closing_balance,
            transactions,
        })
    }

    /// CSV parser with dynamic delimiter, BOM strip, and column mappings.
    pub fn parse_csv(
        bytes: &[u8],
        filename: &str,
        profile: &BankProfile,
    ) -> Result<RawStatementRecord, IngestError> {
        let text = decode_text_with_bom(bytes);
        let delimiter = profile.csv_delimiter.map(|c| c as u8).unwrap_or(b',');

        let mut rdr = ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .flexible(true)
            .from_reader(text.as_bytes());

        let records: Vec<csv::StringRecord> = rdr
            .records()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| IngestError::Csv(format!("CSV parse error: {e}")))?;

        if records.is_empty() {
            return Err(IngestError::InvalidStructure(
                "CSV file contains no data".to_string(),
            ));
        }

        let mut account_number: Option<String> = None;
        let mut account_name: Option<String> = None;
        let mut opening_balance: Option<u64> = None;
        let mut closing_balance: Option<u64> = None;

        // Scan metadata rows before header row
        let meta_limit = profile.header_row_index.min(records.len());
        for record in records.iter().take(meta_limit) {
            let row_line = record.iter().collect::<Vec<_>>().join(" ");
            let row_lower = row_line.to_lowercase();

            if row_lower.contains("số tài khoản:") || row_lower.contains("số tk:") || row_lower.contains("account no") {
                if let Some(pos) = row_line.find(':') {
                    let digits: String = row_line[pos + 1..].chars().filter(|c| c.is_ascii_digit()).collect();
                    if !digits.is_empty() {
                        account_number = Some(digits);
                    }
                }
                if account_number.is_none() {
                    for cell in record.iter().skip(1) {
                        let digits: String = cell.chars().filter(|c| c.is_ascii_digit()).collect();
                        if !digits.is_empty() {
                            account_number = Some(digits);
                            break;
                        }
                    }
                }
            } else if row_lower.contains("tên tài khoản:") || row_lower.contains("tên tk:") || row_lower.contains("account name") {
                if let Some(pos) = row_line.find(':') {
                    let name = row_line[pos + 1..].trim().to_string();
                    if !name.is_empty() {
                        account_name = Some(name);
                    }
                }
                if account_name.is_none() {
                    for cell in record.iter().skip(1) {
                        let trimmed = cell.trim();
                        if !trimmed.is_empty() {
                            account_name = Some(trimmed.to_string());
                            break;
                        }
                    }
                }
            } else if row_lower.contains("số dư đầu kỳ:") || row_lower.contains("opening balance") {
                if let Some(pos) = row_line.find(':') {
                    let val_str = &row_line[pos + 1..];
                    if let Some(amt) = parse_amount_with_profile(
                        val_str,
                        profile.decimal_separator,
                        profile.thousands_separator,
                    ) {
                        opening_balance = Some(amt);
                    }
                }
                if opening_balance.is_none() {
                    for cell in record.iter().skip(1) {
                        if let Some(amt) = parse_amount_with_profile(
                            cell,
                            profile.decimal_separator,
                            profile.thousands_separator,
                        ) {
                            opening_balance = Some(amt);
                            break;
                        }
                    }
                }
            } else if row_lower.contains("số dư cuối kỳ:") || row_lower.contains("closing balance") {
                if let Some(pos) = row_line.find(':') {
                    let val_str = &row_line[pos + 1..];
                    if let Some(amt) = parse_amount_with_profile(
                        val_str,
                        profile.decimal_separator,
                        profile.thousands_separator,
                    ) {
                        closing_balance = Some(amt);
                    }
                }
                if closing_balance.is_none() {
                    for cell in record.iter().skip(1) {
                        if let Some(amt) = parse_amount_with_profile(
                            cell,
                            profile.decimal_separator,
                            profile.thousands_separator,
                        ) {
                            closing_balance = Some(amt);
                            break;
                        }
                    }
                }
            }
        }

        // Header resolution
        if profile.header_row_index >= records.len() {
            return Err(IngestError::InvalidStructure(format!(
                "Header row index {} exceeds CSV total rows {}",
                profile.header_row_index,
                records.len()
            )));
        }

        let header_record = &records[profile.header_row_index];
        let headers: Vec<String> = header_record.iter().map(|s| s.trim().to_string()).collect();
        let resolved = profile.resolve_columns(&headers);

        let c_date = resolved.date.ok_or_else(|| {
            IngestError::InvalidStructure(format!(
                "Could not resolve date column for profile '{}' in file {}",
                profile.profile_name, filename
            ))
        })?;

        let c_narration = resolved.narration.ok_or_else(|| {
            IngestError::InvalidStructure(format!(
                "Could not resolve narration column for profile '{}' in file {}",
                profile.profile_name, filename
            ))
        })?;

        let end_row_limit = records.len().saturating_sub(profile.footer_skip_rows);
        let mut transactions = Vec::new();
        let mut last_valid_date: Option<String> = None;

        for (row_offset, record) in records
            .iter()
            .enumerate()
            .skip(profile.data_start_row_index)
            .take(end_row_limit.saturating_sub(profile.data_start_row_index))
        {
            if record.is_empty() {
                continue;
            }

            let first_cell = record.get(0).unwrap_or("").trim();
            let first_lower = first_cell.to_lowercase();
            if first_lower.contains("tổng")
                || first_lower.contains("tổng cộng")
                || first_lower.contains("total")
                || first_lower.contains("số dư cuối kỳ")
            {
                break;
            }

            // Resolve date with merged cell forward-filling
            let date_cell = record.get(c_date).unwrap_or("").trim();
            let date_str = if !date_cell.is_empty() && normalize_datetime(date_cell).is_ok() {
                last_valid_date = Some(date_cell.to_string());
                date_cell.to_string()
            } else if let Some(ref lvd) = last_valid_date {
                lvd.clone()
            } else {
                continue;
            };

            // Amount, debit, credit
            let (is_credit, amount_cents, debit_str, credit_str) =
                if let (Some(d_idx), Some(c_idx)) = (resolved.debit, resolved.credit) {
                    let debit_amt = record
                        .get(d_idx)
                        .and_then(|s| parse_amount_with_profile(s, profile.decimal_separator, profile.thousands_separator))
                        .unwrap_or(0);
                    let credit_amt = record
                        .get(c_idx)
                        .and_then(|s| parse_amount_with_profile(s, profile.decimal_separator, profile.thousands_separator))
                        .unwrap_or(0);

                    if credit_amt > 0 {
                        (true, credit_amt, None, Some(credit_amt.to_string()))
                    } else if debit_amt > 0 {
                        (false, debit_amt, Some(debit_amt.to_string()), None)
                    } else {
                        continue;
                    }
                } else if let Some(a_idx) = resolved.amount {
                    let amt = record
                        .get(a_idx)
                        .and_then(|s| parse_amount_with_profile(s, profile.decimal_separator, profile.thousands_separator))
                        .unwrap_or(0);

                    if amt == 0 {
                        continue;
                    }

                    let is_cr = if let Some(flag_idx) = resolved.is_credit_flag {
                        let flag = record.get(flag_idx).unwrap_or("").trim();
                        is_credit_flag_value(flag)
                    } else {
                        true
                    };

                    let d_str = if !is_cr { Some(amt.to_string()) } else { None };
                    let c_str = if is_cr { Some(amt.to_string()) } else { None };
                    (is_cr, amt, d_str, c_str)
                } else {
                    continue;
                };

            let balance_cents = resolved
                .balance
                .and_then(|idx| record.get(idx))
                .and_then(|s| parse_amount_with_profile(s, profile.decimal_separator, profile.thousands_separator));

            let doc_ref = resolved
                .doc_ref
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let narration = record
                .get(c_narration)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let val_date_str = resolved
                .val_date
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let counterparty_name = resolved
                .counterparty
                .and_then(|idx| record.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            transactions.push(RawTransactionRecord {
                row_id: row_offset + 1,
                date_str,
                val_date_str,
                doc_ref,
                debit_amt_str: debit_str,
                credit_amt_str: credit_str,
                amount_cents: Some(amount_cents),
                is_credit,
                balance_str: balance_cents.map(|b| b.to_string()),
                balance_cents,
                narration,
                counterparty_name,
            });
        }

        // Bottom-up scan for closing balance if not yet located
        if closing_balance.is_none() {
            for record in records.iter().rev().take(30) {
                let row_str = record.iter().collect::<Vec<_>>().join(" ");
                let lower = row_str.to_lowercase();
                if lower.contains("số dư cuối kỳ") || lower.contains("closing balance") {
                    if let Some(pos) = row_str.find(':') {
                        let val_str = &row_str[pos + 1..];
                        if let Some(amt) = parse_amount_with_profile(
                            val_str,
                            profile.decimal_separator,
                            profile.thousands_separator,
                        ) {
                            closing_balance = Some(amt);
                            break;
                        }
                    }
                    for cell in record.iter().rev() {
                        if let Some(amt) = parse_amount_with_profile(
                            cell,
                            profile.decimal_separator,
                            profile.thousands_separator,
                        ) {
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

        Ok(RawStatementRecord {
            bank: profile.bank_identifier(),
            format: ContainerFormat::TextCsv,
            account_no: account_number,
            account_name,
            opening_balance,
            closing_balance,
            transactions,
        })
    }
}

impl StatementParser for ProfileStatementParser {
    fn can_parse(&self, container: ContainerFormat, _bank: BankIdentifier) -> bool {
        self.profile.format == container
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        self.parse_bytes(bytes, filename)
    }
}

/// Helper function to convert cell to string.
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Int(v) => v.to_string(),
        Data::Float(v) => {
            let int_part = *v as i64;
            let s = v.to_string();
            if s.ends_with(".0") {
                int_part.to_string()
            } else {
                s
            }
        }
        Data::String(s) => s.trim().to_string(),
        Data::DateTime(d) => d.to_string(),
        Data::DateTimeIso(s) => s.to_string(),
        Data::DurationIso(s) => s.to_string(),
        Data::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// Helper to convert cell to formatted date string.
fn cell_to_date_string(cell: &Data) -> Option<String> {
    let s = cell_to_string(cell);
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Helper to extract numeric amount from calamine cell.
fn cell_to_amount(
    cell: &Data,
    dec_sep: Option<char>,
    thou_sep: Option<char>,
) -> Option<u64> {
    match cell {
        Data::Int(v) if *v >= 0 => Some(*v as u64),
        Data::Float(v) if *v >= 0.0 => {
            let s = v.to_string();
            parse_amount_with_profile(&s, dec_sep, thou_sep)
        }
        Data::String(s) => parse_amount_with_profile(s, dec_sep, thou_sep),
        _ => None,
    }
}

/// Parses amount string using custom decimal and thousands separators if specified.
pub fn parse_amount_with_profile(
    input: &str,
    decimal_sep: Option<char>,
    thousands_sep: Option<char>,
) -> Option<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let (Some(dec), Some(thou)) = (decimal_sep, thousands_sep) {
        let mut s = String::with_capacity(trimmed.len());
        for c in trimmed.chars() {
            if c == thou {
                continue;
            } else if c == dec {
                s.push('.');
            } else {
                s.push(c);
            }
        }
        if let Ok(parsed) = parse_monetary_amount(&s) {
            return Some(parsed.minor_units);
        }
    }

    parse_monetary_amount(trimmed).ok().map(|a| a.minor_units)
}

/// Decodes bytes into UTF-8, stripping BOM and falling back to Windows-1258.
pub fn decode_text_with_bom(bytes: &[u8]) -> String {
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (decoded, _, _) = WINDOWS_1258.decode(bytes);
            decoded.into_owned()
        }
    }
}

/// Determines whether a flag indicates Credit or Debit.
fn is_credit_flag_value(flag: &str) -> bool {
    let upper = flag.trim().to_uppercase();
    match upper.as_str() {
        "CR" | "C" | "+" | "1" | "CREDIT" | "GHI CÓ" | "CÓ" => true,
        "DR" | "D" | "-" | "0" | "DEBIT" | "GHI NỢ" | "NỢ" => false,
        _ => true,
    }
}
