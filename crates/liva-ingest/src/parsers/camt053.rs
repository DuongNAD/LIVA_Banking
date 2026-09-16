//! ISO 20022 CAMT.053 XML statement parser.
//! Parses Bank-to-Customer Statements (camt.053.001.02 and compatible).
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use crate::models::{ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser};
use liva_normalize::{parse_monetary_amount, BankIdentifier};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub struct Camt053Parser;

impl StatementParser for Camt053Parser {
    fn can_parse(&self, container: ContainerFormat, bank: BankIdentifier) -> bool {
        container == ContainerFormat::Xml
            || (container == ContainerFormat::Unknown
                && (bank == BankIdentifier::Unknown || bank == BankIdentifier::Vietcombank))
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        let mut reader = Reader::from_reader(bytes);
        reader.trim_text(true);

        let mut bank = BankIdentifier::Unknown;
        let mut account_no: Option<String> = None;
        let mut account_name: Option<String> = None;
        let mut opening_balance: Option<u64> = None;
        let mut closing_balance: Option<u64> = None;

        let mut transactions = Vec::new();

        let mut buf = Vec::new();
        let mut tag_stack: Vec<String> = Vec::new();

        let mut current_bal_tp: Option<String> = None;
        let mut current_bal_amt_str: Option<String> = None;

        let mut current_tx: Option<RawTransactionRecord> = None;
        let mut tx_row_id = 0;
        let mut current_party_type: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    tag_stack.push(name.clone());

                    if name == "Ntry" {
                        tx_row_id += 1;
                        current_tx = Some(RawTransactionRecord {
                            row_id: tx_row_id,
                            ..Default::default()
                        });
                    } else if name == "Bal" {
                        current_bal_tp = None;
                        current_bal_amt_str = None;
                    } else if name == "Cdtr" || name == "Dbtr" {
                        current_party_type = Some(name);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    if name == "Ntry" {
                        if let Some(tx) = current_tx.take() {
                            transactions.push(tx);
                        }
                    } else if name == "Bal" {
                        if let (Some(tp), Some(amt_str)) = (&current_bal_tp, &current_bal_amt_str) {
                            if let Ok(parsed) = parse_monetary_amount(amt_str) {
                                if tp == "OPBD" || tp == "PRCD" {
                                    opening_balance = Some(parsed.minor_units);
                                } else if tp == "CLBD" || tp == "ITBD" {
                                    closing_balance = Some(parsed.minor_units);
                                }
                            }
                        }
                    } else if name == "Cdtr" || name == "Dbtr" {
                        current_party_type = None;
                    }

                    tag_stack.pop();
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().map_err(|err| IngestError::Xml(err.to_string()))?;
                    let text = text.trim();
                    if text.is_empty() {
                        buf.clear();
                        continue;
                    }

                    let current_tag = tag_stack.last().map(|s| s.as_str()).unwrap_or("");
                    let parent_tag = if tag_stack.len() >= 2 {
                        tag_stack[tag_stack.len() - 2].as_str()
                    } else {
                        ""
                    };

                    // Detect bank identifier
                    if (current_tag == "Nm" || current_tag == "BICFI") && parent_tag == "FinInstnId" {
                        let t_upper = text.to_uppercase();
                        if t_upper.contains("VIETCOMBANK") || t_upper.contains("BFTV") {
                            bank = BankIdentifier::Vietcombank;
                        } else if t_upper.contains("TECHCOMBANK") || t_upper.contains("TCBK") {
                            bank = BankIdentifier::Techcombank;
                        } else if t_upper.contains("BIDV") {
                            bank = BankIdentifier::Bidv;
                        } else if t_upper.contains("VIETINBANK") || t_upper.contains("ICBV") {
                            bank = BankIdentifier::VietinBank;
                        } else if t_upper.contains("MBBANK") || t_upper.contains("MSCB") {
                            bank = BankIdentifier::MbBank;
                        } else if t_upper.contains("AGRIBANK") || t_upper.contains("VBAA") {
                            bank = BankIdentifier::Agribank;
                        }
                    }

                    // Account Number & Name
                    if (current_tag == "Id" && parent_tag == "Othr" || current_tag == "IBAN") && account_no.is_none() {
                        account_no = Some(text.to_string());
                    } else if current_tag == "Nm" && parent_tag == "Acct" && account_name.is_none() {
                        account_name = Some(text.to_string());
                    }

                    // Balance Fields
                    if tag_stack.contains(&"Bal".to_string()) {
                        if current_tag == "Cd" && parent_tag == "CdOrPrtry" {
                            current_bal_tp = Some(text.to_string());
                        } else if current_tag == "Amt" {
                            current_bal_amt_str = Some(text.to_string());
                        }
                    }

                    // Transaction Entry Fields
                    if let Some(ref mut tx) = current_tx {
                        if current_tag == "Amt" && parent_tag == "Ntry" {
                            if let Ok(parsed) = parse_monetary_amount(text) {
                                tx.amount_cents = Some(parsed.minor_units);
                            }
                        } else if current_tag == "CdtDbtInd" && parent_tag == "Ntry" {
                            tx.is_credit = text.eq_ignore_ascii_case("CRDT");
                        } else if current_tag == "Dt" && (parent_tag == "BookgDt" || parent_tag == "Dt") {
                            if tx.date_str.is_empty() {
                                tx.date_str = text.to_string();
                            }
                        } else if current_tag == "Dt" && parent_tag == "ValDt" {
                            tx.val_date_str = Some(text.to_string());
                        } else if current_tag == "AcctSvcrRef" || current_tag == "EndToEndId" {
                            if tx.doc_ref.is_none() || tx.doc_ref.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                                tx.doc_ref = Some(text.to_string());
                            }
                        } else if current_tag == "Ustrd" {
                            if !tx.narration.is_empty() {
                                tx.narration.push(' ');
                            }
                            tx.narration.push_str(text);
                        } else if current_tag == "Nm"
                            && (current_party_type.as_deref() == Some("Cdtr") || current_party_type.as_deref() == Some("Dbtr"))
                            && tx.counterparty_name.is_none()
                        {
                            tx.counterparty_name = Some(text.to_string());
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(IngestError::Xml(format!("XML parser error at pos {}: {:?}", reader.buffer_position(), e))),
                _ => {}
            }
            buf.clear();
        }

        if transactions.is_empty() {
            return Err(IngestError::InvalidStructure(format!(
                "No transaction entries (<Ntry>) parsed from CAMT.053 in {filename}"
            )));
        }

        Ok(RawStatementRecord {
            bank,
            format: ContainerFormat::Xml,
            account_no,
            account_name,
            opening_balance,
            closing_balance,
            transactions,
        })
    }
}
