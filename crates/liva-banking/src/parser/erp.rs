//! ERP Document / Invoice Parser for SAP, MISA, Fast, Bravo, and Generic ERP exports.
//!
//! Supports CSV and Excel (.xlsx) formats.

use super::ParserError;
use crate::models::{ErpDocument, TransactionType, parse_banking_date, parse_vietnamese_amount};
use calamine::{Data, Reader, open_workbook_auto_from_rs};
use std::io::Cursor;

pub struct ErpInvoiceParser;

impl ErpInvoiceParser {
    /// Parses an ERP CSV export into structured `ErpDocument` records.
    pub fn parse_csv(raw_bytes: &[u8]) -> Result<Vec<ErpDocument>, ParserError> {
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
            .has_headers(true)
            .flexible(true)
            .from_reader(Cursor::new(bytes));

        let headers = rdr
            .headers()
            .map_err(|e| ParserError::CsvError(e.to_string()))?
            .clone();

        let mut voucher_col = None;
        let mut invoice_col = None;
        let mut partner_code_col = None;
        let mut partner_name_col = None;
        let mut date_col = None;
        let mut due_date_col = None;
        let mut total_amt_col = None;
        let mut open_amt_col = None;
        let mut type_col = None;
        let mut desc_col = None;

        for (i, h) in headers.iter().enumerate() {
            let l = h.to_lowercase();
            if l.contains("chứng từ") || l.contains("số ct") || l.contains("voucher") {
                voucher_col = Some(i);
            } else if l.contains("hóa đơn") || l.contains("số hđ") || l.contains("invoice") {
                invoice_col = Some(i);
            } else if l.contains("mã kh") || l.contains("mã ncc") || l.contains("mã đối tác") || l.contains("partner") {
                partner_code_col = Some(i);
            } else if l.contains("tên kh") || l.contains("tên ncc") || l.contains("tên đối tác") || l.contains("customer") {
                partner_name_col = Some(i);
            } else if l.contains("ngày hđ") || l.contains("ngày ct") || l.contains("date") {
                date_col = Some(i);
            } else if l.contains("hạn tt") || l.contains("due") {
                due_date_col = Some(i);
            } else if l.contains("tổng tiền") || l.contains("nguyên tệ") || l.contains("total") {
                total_amt_col = Some(i);
            } else if l.contains("còn lại") || l.contains("còn nợ") || l.contains("open") {
                open_amt_col = Some(i);
            } else if l.contains("loại") || l.contains("chiều") || l.contains("ar/ap") {
                type_col = Some(i);
            } else if l.contains("diễn giải") || l.contains("mô tả") || l.contains("nội dung") {
                desc_col = Some(i);
            }
        }

        let mut docs = Vec::new();
        let mut row_idx = 0;

        for result in rdr.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => return Err(ParserError::CsvError(e.to_string())),
            };

            row_idx += 1;
            let voucher_no = voucher_col
                .and_then(|c| record.get(c))
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(&format!("VOUCHER-{row_idx}"))
                .trim()
                .to_string();

            let invoice_no = invoice_col
                .and_then(|c| record.get(c))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let partner_code = partner_code_col
                .and_then(|c| record.get(c))
                .unwrap_or("CUSTOMER_DEFAULT")
                .trim()
                .to_string();

            let partner_name = partner_name_col
                .and_then(|c| record.get(c))
                .unwrap_or(&partner_code)
                .trim()
                .to_string();

            let doc_date = date_col
                .and_then(|c| record.get(c))
                .and_then(|s| parse_banking_date(s))
                .unwrap_or(0);

            let due_date = due_date_col
                .and_then(|c| record.get(c))
                .and_then(|s| parse_banking_date(s));

            let total_amount = total_amt_col
                .and_then(|c| record.get(c))
                .and_then(|s| parse_vietnamese_amount(s))
                .unwrap_or(0);

            let open_amount = open_amt_col
                .and_then(|c| record.get(c))
                .and_then(|s| parse_vietnamese_amount(s))
                .unwrap_or(total_amount);

            let doc_type = if let Some(tc) = type_col {
                let val = record.get(tc).unwrap_or("").to_lowercase();
                if val.contains("phải trả") || val.contains("chi") || val.contains("ap") || val.contains("debit") {
                    TransactionType::Debit
                } else {
                    TransactionType::Credit
                }
            } else {
                TransactionType::Credit // Default AR (Receivable inflow)
            };

            let description = desc_col
                .and_then(|c| record.get(c))
                .unwrap_or("")
                .trim()
                .to_string();

            docs.push(ErpDocument {
                id: format!("ERP-DOC-{row_idx}"),
                voucher_no,
                invoice_no,
                partner_code,
                partner_name,
                doc_date,
                due_date,
                doc_type,
                total_amount,
                open_amount,
                currency: "VND".to_string(),
                description,
                version: 1,
            });
        }

        Ok(docs)
    }

    /// Parses an ERP Excel (.xlsx) export into structured `ErpDocument` records.
    pub fn parse_excel(bytes: &[u8]) -> Result<Vec<ErpDocument>, ParserError> {
        let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes))
            .map_err(|e| ParserError::ExcelError(format!("Failed to open Excel: {e}")))?;

        let range = workbook
            .worksheet_range_at(0)
            .ok_or_else(|| ParserError::ExcelError("Workbook has no sheets".to_string()))?
            .map_err(|e| ParserError::ExcelError(format!("Failed to read sheet 0: {e}")))?;

        let mut docs = Vec::new();
        let mut header_found = false;
        let mut voucher_col = 0;
        let mut invoice_col = None;
        let mut partner_name_col = None;
        let mut date_col = None;
        let mut total_amt_col = None;
        let mut open_amt_col = None;
        let mut desc_col = None;
        let mut row_idx = 0;

        for row in range.rows() {
            let row_str: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    Data::String(s) => s.trim().to_string(),
                    Data::Float(f) => format!("{:.0}", f),
                    Data::Int(i) => i.to_string(),
                    _ => String::new(),
                })
                .collect();

            if row_str.iter().all(|s| s.is_empty()) {
                continue;
            }

            if !header_found {
                let is_header = row_str.iter().any(|c| {
                    let l = c.to_lowercase();
                    l.contains("chứng từ") || l.contains("hóa đơn") || l.contains("số tiền")
                });

                if is_header {
                    header_found = true;
                    for (i, col) in row_str.iter().enumerate() {
                        let l = col.to_lowercase();
                        if l.contains("chứng từ") || l.contains("voucher") {
                            voucher_col = i;
                        } else if l.contains("hóa đơn") || l.contains("invoice") {
                            invoice_col = Some(i);
                        } else if l.contains("khách hàng") || l.contains("đối tác") {
                            partner_name_col = Some(i);
                        } else if l.contains("ngày") || l.contains("date") {
                            date_col = Some(i);
                        } else if l.contains("tổng tiền") || l.contains("số tiền") {
                            total_amt_col = Some(i);
                        } else if l.contains("còn lại") {
                            open_amt_col = Some(i);
                        } else if l.contains("diễn giải") || l.contains("nội dung") {
                            desc_col = Some(i);
                        }
                    }
                    continue;
                }
            }

            if !header_found {
                continue;
            }

            row_idx += 1;
            let voucher_no = row_str.get(voucher_col).cloned().unwrap_or(format!("VOUCHER-{row_idx}"));
            let invoice_no = invoice_col.and_then(|c| row_str.get(c)).cloned();
            let partner_name = partner_name_col.and_then(|c| row_str.get(c)).cloned().unwrap_or("CUSTOMER".to_string());
            let doc_date = date_col
                .and_then(|c| row_str.get(c))
                .and_then(|s| parse_banking_date(s))
                .unwrap_or(0);
            let total_amount = total_amt_col
                .and_then(|c| row_str.get(c))
                .and_then(|s| parse_vietnamese_amount(s))
                .unwrap_or(0);
            let open_amount = open_amt_col
                .and_then(|c| row_str.get(c))
                .and_then(|s| parse_vietnamese_amount(s))
                .unwrap_or(total_amount);
            let description = desc_col.and_then(|c| row_str.get(c)).cloned().unwrap_or_default();

            if total_amount == 0 {
                continue;
            }

            docs.push(ErpDocument {
                id: format!("ERP-DOC-{row_idx}"),
                voucher_no,
                invoice_no,
                partner_code: partner_name.clone(),
                partner_name,
                doc_date,
                due_date: None,
                doc_type: TransactionType::Credit,
                total_amount,
                open_amount,
                currency: "VND".to_string(),
                description,
                version: 1,
            });
        }

        Ok(docs)
    }
}
