//! ISO 20022 Bank-to-Customer Statement (camt.053) XML Parser.
//!
//! Complies with ISO 20022 `camt.053.001.08` specification.
//! Extracts:
//! - GrpHdr (Message ID, Creation Date/Time)
//! - Statement identification, Account information (IBAN/Other ID, Currency, Account Holder)
//! - Opening and Closing Balances (OPBD, CLBD, PRCD, CLAV)
//! - Entries (Ntry) with amounts, Credit/Debit indicators, Value dates, Document references,
//!   Debtor/Creditor counterparties, and Remittance Information (Ustrd narrations).
//!
//! Fully zero-dependency, memory-safe, lightweight DOM-based XML parsing.

use crate::banking::models::{
    BankStatement, BankType, StatementFormat, TransactionRecord, TransactionType,
    parse_banking_date, parse_vietnamese_amount,
};
use crate::banking::parser::{BankStatementParser, ParserError};
use std::collections::HashMap;
use std::time::Instant;

/// Parser for ISO 20022 camt.053 Bank Statement XML documents.
#[derive(Debug, Default, Clone, Copy)]
pub struct Iso20022XmlParser;

impl BankStatementParser for Iso20022XmlParser {
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool {
        let lower_fn = filename.to_lowercase();
        if lower_fn.ends_with(".xml")
            || lower_fn.ends_with(".camt")
            || lower_fn.ends_with(".camt053")
            || lower_fn.contains("camt.053")
            || lower_fn.contains("iso20022")
        {
            let head_len = bytes.len().min(4096);
            if head_len > 0 {
                let head = String::from_utf8_lossy(&bytes[..head_len]);
                if head.contains("camt.053")
                    || head.contains("BkToCstmrStmt")
                    || head.contains("Document")
                    || head.contains("iso:20022")
                {
                    return true;
                }
            } else {
                return true;
            }
        }

        let head_len = bytes.len().min(4096);
        if head_len == 0 {
            return false;
        }
        let head = String::from_utf8_lossy(&bytes[..head_len]);
        head.contains("camt.053")
            || (head.contains("BkToCstmrStmt") && head.contains("GrpHdr"))
            || (head.contains("<Document") && head.contains("urn:iso:std:iso:20022"))
    }

    fn parse(&self, bytes: &[u8], _filename: &str) -> Result<BankStatement, ParserError> {
        let start = Instant::now();
        let xml_str = std::str::from_utf8(bytes)
            .map_err(|e| ParserError::InvalidStructure(format!("Invalid UTF-8 in XML: {e}")))?;

        let root = parse_xml(xml_str)
            .map_err(|e| ParserError::InvalidStructure(format!("XML parsing error: {e}")))?;

        // Find BkToCstmrStmt node
        let bk_node = root
            .find_descendant("BkToCstmrStmt")
            .ok_or_else(|| {
                ParserError::InvalidStructure(
                    "Missing mandatory ISO 20022 <BkToCstmrStmt> root element".to_string(),
                )
            })?;

        // 0. Group Header (GrpHdr)
        let mut grp_msg_id: Option<String> = None;
        let mut grp_cre_dt_tm: Option<String> = None;
        if let Some(grp_hdr) = bk_node.find_descendant("GrpHdr") {
            grp_msg_id = grp_hdr.child_text("MsgId").map(|s| s.to_string());
            grp_cre_dt_tm = grp_hdr.child_text("CreDtTm").map(|s| s.to_string());
        }

        // Find Stmt node (may have multiple statements, we process primary or first Stmt)
        let stmt_node = bk_node
            .find_descendant("Stmt")
            .ok_or_else(|| {
                ParserError::InvalidStructure(
                    "Missing mandatory ISO 20022 <Stmt> element".to_string(),
                )
            })?;

        // 1. Account details
        let mut account_number = None;
        let mut account_name = None;
        let mut currency = "VND".to_string();
        let mut bank_name = None;

        if let Some(acct) = stmt_node.find_descendant("Acct") {
            if let Some(iban) = acct.deep_text(&["Id", "IBAN"]) {
                account_number = Some(iban);
            } else if let Some(othr_id) = acct.deep_text(&["Id", "Othr", "Id"]) {
                account_number = Some(othr_id);
            }
            if let Some(nm) = acct.child_text("Nm") {
                account_name = Some(nm.to_string());
            }
            if let Some(ccy) = acct.child_text("Ccy") {
                currency = ccy.to_string();
            }
            if let Some(svcr) = acct.find_descendant("Svcr") {
                if let Some(nm) = svcr.deep_text(&["FinInstnId", "Nm"]) {
                    bank_name = Some(nm);
                } else if let Some(bic) = svcr.deep_text(&["FinInstnId", "BICFI"]) {
                    bank_name = Some(bic);
                }
            }
        }

        // 2. Balances (OPBD = Opening Booked, PRCD = Previous Closing, CLBD = Closing Booked, CLAV = Closing Available)
        let mut opening_balance: Option<u64> = None;
        let mut closing_balance: Option<u64> = None;
        let mut statement_from: Option<i64> = None;
        let mut statement_to: Option<i64> = None;

        for bal in stmt_node.find_children("Bal") {
            let tp_cd = bal
                .deep_text(&["Tp", "CdOrPrtry", "Cd"])
                .unwrap_or_default()
                .to_uppercase();
            let amt_str = bal.child_text("Amt").unwrap_or("0");
            let amt_val = parse_vietnamese_amount(amt_str).unwrap_or(0);
            let dt_str = bal
                .deep_text(&["Dt", "Dt"])
                .or_else(|| bal.deep_text(&["Dt", "DtTm"]));
            let dt_ts = dt_str.and_then(|s| parse_banking_date(&s));

            if tp_cd == "OPBD" || tp_cd == "PRCD" || (opening_balance.is_none() && tp_cd.is_empty()) {
                opening_balance = Some(amt_val);
                if statement_from.is_none() {
                    statement_from = dt_ts;
                }
            } else if tp_cd == "CLBD" || tp_cd == "CLAV" || closing_balance.is_none() {
                closing_balance = Some(amt_val);
                if statement_to.is_none() {
                    statement_to = dt_ts;
                }
            }
        }

        if statement_from.is_none() {
            statement_from = grp_cre_dt_tm.as_deref().and_then(parse_banking_date);
        }
        if statement_to.is_none() {
            statement_to = statement_from;
        }

        // 3. Transactions (Ntry)
        let mut transactions = Vec::new();
        let mut row_idx = 0usize;

        for ntry in stmt_node.find_children("Ntry") {
            row_idx += 1;

            let amt_str = ntry.child_text("Amt").unwrap_or("0");
            let amount = parse_vietnamese_amount(amt_str).unwrap_or(0);

            let cdt_dbt_ind = ntry
                .child_text("CdtDbtInd")
                .unwrap_or("CRDT")
                .to_uppercase();
            let tx_type = if cdt_dbt_ind == "DBIT" {
                TransactionType::Debit
            } else {
                TransactionType::Credit
            };

            let booking_dt_str = ntry
                .deep_text(&["BookgDt", "DtTm"])
                .or_else(|| ntry.deep_text(&["BookgDt", "Dt"]))
                .or_else(|| ntry.deep_text(&["ValDt", "DtTm"]))
                .or_else(|| ntry.deep_text(&["ValDt", "Dt"]))
                .unwrap_or_default();
            let tx_date = parse_banking_date(&booking_dt_str).unwrap_or_else(|| {
                statement_from.unwrap_or_else(chrono_now_secs)
            });

            let val_dt_str = ntry
                .deep_text(&["ValDt", "DtTm"])
                .or_else(|| ntry.deep_text(&["ValDt", "Dt"]))
                .unwrap_or_else(|| booking_dt_str.clone());
            let value_date = parse_banking_date(&val_dt_str).unwrap_or(tx_date);

            let ntry_ref = ntry.child_text("NtryRef").map(|s| s.to_string());
            let acct_svcr_ref = ntry.child_text("AcctSvcrRef").map(|s| s.to_string());

            // Delve into TxDtls
            let tx_dtls = ntry.find_descendant("TxDtls");
            let mut end_to_end_id = None;
            let mut tx_id = None;
            let mut counterparty_name = None;
            let mut counterparty_account = None;
            let mut counterparty_bank = None;
            let mut narration = String::new();

            if let Some(tx) = tx_dtls {
                end_to_end_id = tx.deep_text(&["Refs", "EndToEndId"]);
                tx_id = tx.deep_text(&["Refs", "TxId"]);

                if tx_type == TransactionType::Credit {
                    counterparty_name = tx.deep_text(&["RltdPties", "Dbtr", "Nm"]);
                    counterparty_account = tx
                        .deep_text(&["RltdPties", "DbtrAcct", "Id", "Othr", "Id"])
                        .or_else(|| tx.deep_text(&["RltdPties", "DbtrAcct", "Id", "IBAN"]));
                    counterparty_bank = tx
                        .deep_text(&["RltdAgts", "DbtrAgt", "FinInstnId", "Nm"])
                        .or_else(|| tx.deep_text(&["RltdAgts", "DbtrAgt", "FinInstnId", "BICFI"]));
                } else {
                    counterparty_name = tx.deep_text(&["RltdPties", "Cdtr", "Nm"]);
                    counterparty_account = tx
                        .deep_text(&["RltdPties", "CdtrAcct", "Id", "Othr", "Id"])
                        .or_else(|| tx.deep_text(&["RltdPties", "CdtrAcct", "Id", "IBAN"]));
                    counterparty_bank = tx
                        .deep_text(&["RltdAgts", "CdtrAgt", "FinInstnId", "Nm"])
                        .or_else(|| tx.deep_text(&["RltdAgts", "CdtrAgt", "FinInstnId", "BICFI"]));
                }

                if let Some(ustrd) = tx.deep_text(&["RmtInf", "Ustrd"]) {
                    narration = ustrd;
                }
            }

            if narration.is_empty() {
                if let Some(addtl) = ntry.child_text("AddtlNtryInf") {
                    narration = addtl.to_string();
                } else {
                    narration = format!("ISO20022 {cdt_dbt_ind} {amount} {currency}");
                }
            }

            let doc_ref = end_to_end_id
                .or(tx_id)
                .or(acct_svcr_ref.clone())
                .or(ntry_ref.clone())
                .or_else(|| grp_msg_id.clone());

            let mut record = TransactionRecord::new(
                row_idx,
                tx_date,
                value_date,
                doc_ref,
                tx_type,
                amount,
                None, // balance_after will be populated or calculated
                counterparty_account,
                counterparty_name,
                counterparty_bank,
                narration,
            );
            record.ft_number = acct_svcr_ref;
            record.raw_ref = ntry_ref;
            transactions.push(record);
        }

        let bank_type = determine_bank_type(bank_name.as_deref().unwrap_or(""));
        let bank_code = if bank_type != BankType::Iso20022Generic && bank_type != BankType::Unknown {
            bank_type.as_code().to_string()
        } else {
            "ISO20022".to_string()
        };

        let duration = start.elapsed().as_millis() as u64;

        let statement = BankStatement::new(
            bank_code,
            bank_type,
            StatementFormat::Xml,
            account_number,
            account_name,
            opening_balance,
            closing_balance,
            statement_from,
            statement_to,
            transactions,
            duration,
        );

        Ok(statement)
    }
}

fn determine_bank_type(name: &str) -> BankType {
    let s = name.to_uppercase();
    if s.contains("VIETCOMBANK") || s.contains("VCB") || s.contains("BFTV") {
        BankType::Vietcombank
    } else if s.contains("TECHCOMBANK") || s.contains("TCB") || s.contains("TCBV") {
        BankType::Techcombank
    } else if s.contains("BIDV") || s.contains("BIDV") {
        BankType::Bidv
    } else if s.contains("VIETINBANK") || s.contains("CTG") || s.contains("ICBV") {
        BankType::VietinBank
    } else if s.contains("MBBANK") || s.contains("MB") || s.contains("MSCB") {
        BankType::MbBank
    } else if s.contains("AGRIBANK") || s.contains("VBA") || s.contains("VBAA") {
        BankType::Agribank
    } else {
        BankType::Iso20022Generic
    }
}

fn chrono_now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Lightweight Zero-Dependency XML DOM Tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct XmlElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub text: String,
    pub children: Vec<XmlElement>,
}

impl XmlElement {
    pub fn clean_tag(&self) -> &str {
        if let Some(idx) = self.tag.rfind(':') {
            &self.tag[idx + 1..]
        } else {
            &self.tag
        }
    }

    pub fn find_children<'a>(&'a self, tag: &str) -> Vec<&'a XmlElement> {
        self.children
            .iter()
            .filter(|c| c.clean_tag().eq_ignore_ascii_case(tag))
            .collect()
    }

    pub fn find_child<'a>(&'a self, tag: &str) -> Option<&'a XmlElement> {
        self.children
            .iter()
            .find(|c| c.clean_tag().eq_ignore_ascii_case(tag))
    }

    pub fn child_text(&self, tag: &str) -> Option<&str> {
        self.find_child(tag).map(|c| c.text.trim())
    }

    pub fn find_descendant<'a>(&'a self, tag: &str) -> Option<&'a XmlElement> {
        if self.clean_tag().eq_ignore_ascii_case(tag) {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_descendant(tag) {
                return Some(found);
            }
        }
        None
    }

    pub fn deep_text(&self, path: &[&str]) -> Option<String> {
        if path.is_empty() {
            return Some(self.text.trim().to_string());
        }
        let mut curr = self;
        for &step in path {
            curr = curr.find_child(step)?;
        }
        let t = curr.text.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    }
}

/// Parses an XML string into a root `XmlElement`.
pub fn parse_xml(xml: &str) -> Result<XmlElement, String> {
    let chars: Vec<char> = xml.chars().collect();
    let mut i = 0;
    let n = chars.len();

    let mut root = XmlElement {
        tag: "ROOT".to_string(),
        ..Default::default()
    };
    let mut stack: Vec<XmlElement> = Vec::new();

    while i < n {
        if chars[i] == '<' {
            if i + 1 < n && chars[i + 1] == '?' {
                // Skip processing instruction <? ... ?>
                i += 2;
                while i + 1 < n && !(chars[i] == '?' && chars[i + 1] == '>') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            if i + 3 < n && chars[i + 1] == '!' && chars[i + 2] == '-' && chars[i + 3] == '-' {
                // Skip comment <!-- ... -->
                i += 4;
                while i + 2 < n && !(chars[i] == '-' && chars[i + 1] == '-' && chars[i + 2] == '>') {
                    i += 1;
                }
                i += 3;
                continue;
            }
            if i + 1 < n && chars[i + 1] == '!' {
                // Skip doctype or other declarations <! ... >
                while i < n && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                continue;
            }
            if i + 1 < n && chars[i + 1] == '/' {
                // Closing tag </tag>
                i += 2;
                let mut close_tag = String::new();
                while i < n && chars[i] != '>' {
                    close_tag.push(chars[i]);
                    i += 1;
                }
                i += 1; // skip '>'
                let close_tag = close_tag.trim();

                if let Some(finished) = stack.pop() {
                    let clean_close = if let Some(pos) = close_tag.rfind(':') {
                        &close_tag[pos + 1..]
                    } else {
                        close_tag
                    };
                    if !finished.clean_tag().eq_ignore_ascii_case(clean_close) {
                        // Tag mismatch warning: still attach to prevent abort
                    }
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(finished);
                    } else {
                        root.children.push(finished);
                    }
                }
                continue;
            }

            // Opening tag or self-closing tag <tag ... > or <tag ... />
            i += 1;
            let mut tag_content = String::new();
            while i < n && chars[i] != '>' {
                tag_content.push(chars[i]);
                i += 1;
            }
            i += 1; // skip '>'

            let is_self_closing = tag_content.ends_with('/');
            let trimmed = if is_self_closing {
                tag_content[..tag_content.len() - 1].trim()
            } else {
                tag_content.trim()
            };

            let mut parts = trimmed.split_whitespace();
            let tag_name = parts.next().unwrap_or("").to_string();
            let mut attributes = HashMap::new();

            for attr in parts {
                if let Some(eq_idx) = attr.find('=') {
                    let k = attr[..eq_idx].trim().to_string();
                    let v = attr[eq_idx + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                    attributes.insert(k, v);
                }
            }

            let element = XmlElement {
                tag: tag_name,
                attributes,
                text: String::new(),
                children: Vec::new(),
            };

            if is_self_closing {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element);
                } else {
                    root.children.push(element);
                }
            } else {
                stack.push(element);
            }
        } else {
            // Text content
            let mut text = String::new();
            while i < n && chars[i] != '<' {
                text.push(chars[i]);
                i += 1;
            }
            let decoded = decode_xml_entities(&text);
            if let Some(curr) = stack.last_mut() {
                curr.text.push_str(&decoded);
            }
        }
    }

    // Flush any remaining elements
    while let Some(elem) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(elem);
        } else {
            root.children.push(elem);
        }
    }

    Ok(root)
}

fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CAMT053: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>LIVA-STMT-20260914-001</MsgId>
      <CreDtTm>2026-09-14T09:30:00+07:00</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-2026-VCB-001</Id>
      <Acct>
        <Id>
          <Othr>
            <Id>0011001234567</Id>
          </Othr>
        </Id>
        <Ccy>VND</Ccy>
        <Nm>CONG TY TNHH LIVA SOLUTIONS</Nm>
        <Svcr>
          <FinInstnId>
            <BICFI>BFTVVNVX</BICFI>
            <Nm>VIETCOMBANK</Nm>
          </FinInstnId>
        </Svcr>
      </Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">1400000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-08-30</Dt></Dt>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">1410000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-08-30</Dt></Dt>
      </Bal>
      <Ntry>
        <NtryRef>NTRY-001</NtryRef>
        <Amt Ccy="VND">25000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><DtTm>2026-08-30T10:15:00+07:00</DtTm></BookgDt>
        <ValDt><Dt>2026-08-30</Dt></ValDt>
        <AcctSvcrRef>FT262568912345</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs>
              <EndToEndId>HD101</EndToEndId>
              <TxId>FT262568912345</TxId>
            </Refs>
            <RltdPties>
              <Dbtr><Nm>CONG TY CP THUONG MAI ABC</Nm></Dbtr>
              <DbtrAcct><Id><Othr><Id>19034567890123</Id></Othr></Id></DbtrAcct>
            </RltdPties>
            <RltdAgts>
              <DbtrAgt><FinInstnId><BICFI>TCBVVNVX</BICFI><Nm>TECHCOMBANK</Nm></FinInstnId></DbtrAgt>
            </RltdAgts>
            <RmtInf>
              <Ustrd>THANH TOAN TIEN HANG HOP DONG HD101</Ustrd>
            </RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
      <Ntry>
        <NtryRef>NTRY-002</NtryRef>
        <Amt Ccy="VND">15000000</Amt>
        <CdtDbtInd>DBIT</CdtDbtInd>
        <BookgDt><DtTm>2026-08-30T11:30:00+07:00</DtTm></BookgDt>
        <ValDt><Dt>2026-08-30</Dt></ValDt>
        <AcctSvcrRef>FT262568999999</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs>
              <EndToEndId>BILL-ELECTRIC</EndToEndId>
            </Refs>
            <RltdPties>
              <Cdtr><Nm>CONG TY DIEN LUC HA NOI</Nm></Cdtr>
            </RltdPties>
            <RmtInf>
              <Ustrd>TIEN DIEN THANG 8</Ustrd>
            </RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

    #[test]
    fn test_sniff_camt053_xml() {
        let parser = Iso20022XmlParser;
        assert!(parser.sniff(SAMPLE_CAMT053.as_bytes(), "statement.xml"));
        assert!(parser.sniff(b"", "statement.camt.053.xml"));
        assert!(!parser.sniff(b"some random text", "file.txt"));
    }

    #[test]
    fn test_parse_camt053_xml_statement() {
        let parser = Iso20022XmlParser;
        let stmt = parser.parse(SAMPLE_CAMT053.as_bytes(), "statement.xml").expect("Parse statement");

        assert_eq!(stmt.account_number.as_deref(), Some("0011001234567"));
        assert_eq!(stmt.account_name.as_deref(), Some("CONG TY TNHH LIVA SOLUTIONS"));
        assert_eq!(stmt.opening_balance, Some(1_400_000_000));
        assert_eq!(stmt.closing_balance, Some(1_410_000_000));
        assert_eq!(stmt.total_credit, 25_000_000);
        assert_eq!(stmt.total_debit, 15_000_000);
        assert_eq!(stmt.transactions.len(), 2);

        // Verify balance checksum: 1400M + 25M - 15M = 1410M
        assert!(stmt.balance_checksum_passed);

        let tx1 = &stmt.transactions[0];
        assert_eq!(tx1.tx_type, TransactionType::Credit);
        assert_eq!(tx1.amount, 25_000_000);
        assert_eq!(tx1.doc_ref.as_deref(), Some("HD101"));
        assert_eq!(tx1.counterparty_name.as_deref(), Some("CONG TY CP THUONG MAI ABC"));
        assert_eq!(tx1.counterparty_account.as_deref(), Some("19034567890123"));
        assert_eq!(tx1.counterparty_bank.as_deref(), Some("TECHCOMBANK"));
        assert_eq!(tx1.narration, "THANH TOAN TIEN HANG HOP DONG HD101");

        let tx2 = &stmt.transactions[1];
        assert_eq!(tx2.tx_type, TransactionType::Debit);
        assert_eq!(tx2.amount, 15_000_000);
        assert_eq!(tx2.doc_ref.as_deref(), Some("BILL-ELECTRIC"));
        assert_eq!(tx2.counterparty_name.as_deref(), Some("CONG TY DIEN LUC HA NOI"));
        assert_eq!(tx2.narration, "TIEN DIEN THANG 8");
    }
}
