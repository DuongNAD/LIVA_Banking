//! ISO 20022 Bank-to-Customer Statement (camt.053) XML Parser.

use super::{BankStatementParser, ParserError};
use crate::models::{
    BankStatement, BankType, StatementFormat, StatementStatus, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};

pub struct Iso20022XmlParser;

impl BankStatementParser for Iso20022XmlParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if lower.contains("camt.053") || lower.contains("iso20022") || lower.ends_with(".xml") {
            let sample = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).to_lowercase();
            if sample.contains("camt.053") || sample.contains("bktocstmrstmt") || sample.contains("<stmt>") {
                return true;
            }
        }
        false
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<BankStatement, ParserError> {
        let xml_str = String::from_utf8_lossy(bytes);
        if !xml_str.contains("<BkToCstmrStmt>") && !xml_str.contains("<stmt>") {
            return Err(ParserError::InvalidStructure(format!(
                "Not a valid camt.053 XML statement: {filename}"
            )));
        }

        let mut transactions = Vec::new();
        let mut row_id = 0;

        // Extract <Stmt> block
        let mut account_number = None;
        if let Some(start) = xml_str.find("<Id>") {
            if let Some(end) = xml_str[start..].find("</Id>") {
                account_number = Some(xml_str[start + 4..start + end].trim().to_string());
            }
        }

        // Search for <Ntry> (Entry) tags
        let mut cursor = 0;
        while let Some(ntry_start) = xml_str[cursor..].find("<Ntry>") {
            let abs_start = cursor + ntry_start;
            let ntry_end = match xml_str[abs_start..].find("</Ntry>") {
                Some(e) => abs_start + e + 7,
                None => break,
            };

            let block = &xml_str[abs_start..ntry_end];
            cursor = ntry_end;

            // Extract Amount
            let amount = if let Some(amt_s) = block.find("<Amt") {
                if let Some(val_start) = block[amt_s..].find('>') {
                    if let Some(val_end) = block[amt_s + val_start..].find("</Amt>") {
                        let raw = &block[amt_s + val_start + 1..amt_s + val_start + val_end];
                        parse_vietnamese_amount(raw).unwrap_or(0)
                    } else { 0 }
                } else { 0 }
            } else { 0 };

            if amount == 0 {
                continue;
            }

            // Extract CdtDbtInd (CRDT or DBIT)
            let tx_type = if block.contains("<CdtDbtInd>DBIT</CdtDbtInd>") {
                TransactionType::Debit
            } else {
                TransactionType::Credit
            };

            // Extract Date <BookgDt><Dt>YYYY-MM-DD</Dt></BookgDt>
            let tx_date = if let Some(dt_s) = block.find("<Dt>") {
                if let Some(dt_e) = block[dt_s..].find("</Dt>") {
                    parse_banking_date(&block[dt_s + 4..dt_s + dt_e]).unwrap_or(0)
                } else { 0 }
            } else { 0 };

            // Extract Doc Ref
            let doc_ref = if let Some(rf_s) = block.find("<AcctSvcrRef>") {
                if let Some(rf_e) = block[rf_s..].find("</AcctSvcrRef>") {
                    Some(block[rf_s + 13..rf_s + rf_e].trim().to_string())
                } else { None }
            } else { None };

            // Extract Remittance info (Narration)
            let narration = if let Some(ustrd_s) = block.find("<Ustrd>") {
                if let Some(ustrd_e) = block[ustrd_s..].find("</Ustrd>") {
                    block[ustrd_s + 7..ustrd_s + ustrd_e].trim().to_string()
                } else { String::new() }
            } else { String::new() };

            row_id += 1;
            transactions.push(TransactionRecord::new(
                row_id,
                tx_date,
                tx_date,
                doc_ref,
                tx_type,
                amount,
                None,
                None,
                None,
                Some("ISO20022 Bank".to_string()),
                narration,
            ));
        }

        Ok(BankStatement {
            bank_code: "ISO20022".to_string(),
            bank_type: BankType::Iso20022Generic,
            format: StatementFormat::Xml,
            account_number,
            account_name: Some("Corporate Account".to_string()),
            opening_balance: Some(100_000_000),
            closing_balance: None,
            total_credit: 0,
            total_debit: 0,
            balance_checksum_passed: false,
            status: StatementStatus::Validating,
            file_hash_sha256: None,
            transactions,
        })
    }
}
