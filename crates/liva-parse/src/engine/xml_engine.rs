//! Generic XML (ISO 20022 CAMT.053) statement parser driven by StatementTemplate DSL.
//! Uses `quick-xml` streaming events and strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::engine::date_normalizer::parse_banking_date;
use crate::engine::invariant::verify_statement_invariants;
use crate::engine::number_normalizer::parse_dsl_amount;
use crate::error::ParseError;
use crate::template::schema::StatementTemplate;
use crate::{CanonicalStatement, CanonicalTx};
use liva_ledger::PostingType;
use liva_money::{Currency, Money};

/// Parses raw XML (CAMT.053) bytes into a CanonicalStatement using the provided DSL template.
pub fn parse_xml_statement(
    raw: &[u8],
    _filename: &str,
    template: &StatementTemplate,
) -> Result<CanonicalStatement, ParseError> {
    let text = std::str::from_utf8(raw)
        .map_err(|e| ParseError::EncodingError(e.to_string()))?;

    let currency = template
        .currency
        .parse::<Currency>()
        .unwrap_or(Currency::VND);

    let entry_tag = template
        .container_options
        .xml
        .entry_path
        .as_deref()
        .unwrap_or("Ntry");

    let mut reader = quick_xml::Reader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut current_tag = String::new();
    let mut tag_stack: Vec<String> = Vec::new();

    let mut account_number: Option<String> = None;
    let mut opening_balance: Option<Money> = None;
    let mut closing_balance: Option<Money> = None;

    // State within Bal node
    let mut in_bal = false;
    let mut bal_type = String::new();
    let mut bal_amt_str = String::new();
    let mut bal_ind = String::new();

    // State within Ntry node
    let mut in_entry = false;
    let mut entry_amount: Option<Money> = None;
    let mut entry_indicator: Option<PostingType> = None;
    let mut entry_ref = String::new();
    let mut entry_narration = String::new();
    let mut entry_cp_name = String::new();
    let mut entry_date: Option<i64> = None;

    let mut transactions = Vec::new();
    let mut row_counter = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_tag = name.clone();
                tag_stack.push(name.clone());

                if name == "Bal" {
                    in_bal = true;
                    bal_type.clear();
                    bal_amt_str.clear();
                    bal_ind.clear();
                } else if name == entry_tag {
                    in_entry = true;
                    entry_amount = None;
                    entry_indicator = None;
                    entry_ref.clear();
                    entry_narration.clear();
                    entry_cp_name.clear();
                    entry_date = None;
                }
            }
            Ok(quick_xml::events::Event::Text(e)) => {
                let decoded = e.decode().map_err(|err| ParseError::XmlError(err.to_string()))?;
                let val = quick_xml::escape::unescape(&decoded).map_err(|err| ParseError::XmlError(err.to_string()))?;
                let val_str = val.trim();

                if in_bal {
                    match current_tag.as_str() {
                        "Cd" | "Prtry" => {
                            if bal_type.is_empty() {
                                bal_type = val_str.to_string();
                            }
                        }
                        "Amt" => bal_amt_str = val_str.to_string(),
                        "CdtDbtInd" => bal_ind = val_str.to_string(),
                        _ => {}
                    }
                } else if in_entry {
                    match current_tag.as_str() {
                        "Amt" => {
                            if let Ok(m) = parse_dsl_amount(
                                val_str,
                                &template.formatting.number.thousand_separator,
                                &template.formatting.number.decimal_separator,
                                template.formatting.number.decimal_places,
                                &template.formatting.number.strip_tokens,
                                currency,
                            ) {
                                entry_amount = Some(m);
                            }
                        }
                        "CdtDbtInd" => {
                            if val_str == "CRDT" {
                                entry_indicator = Some(PostingType::Credit);
                            } else if val_str == "DBIT" {
                                entry_indicator = Some(PostingType::Debit);
                            }
                        }
                        "AcctSvcrRef" | "EndToEndId" | "Ref" => {
                            if entry_ref.is_empty() {
                                entry_ref = val_str.to_string();
                            }
                        }
                        "Ustrd" | "AddtlNtryInf" => {
                            if !val_str.is_empty() {
                                if !entry_narration.is_empty() {
                                    entry_narration.push(' ');
                                }
                                entry_narration.push_str(val_str);
                            }
                        }
                        "Nm" => {
                            // Counterparty name if under Cdtr or Dbtr
                            let parent = tag_stack.get(tag_stack.len().saturating_sub(2)).map(|s| s.as_str());
                            if (parent == Some("Cdtr") || parent == Some("Dbtr") || parent == Some("RltdPties")) && entry_cp_name.is_empty() {
                                entry_cp_name = val_str.to_string();
                            }
                        }
                        "DtTm" | "Dt" if entry_date.is_none() => {
                            if let Ok(d) = parse_banking_date(val_str, &template.formatting.date.formats) {
                                entry_date = Some(d);
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Global metadata
                    if current_tag == "IBAN" || (current_tag == "Id" && account_number.is_none()) {
                        account_number = Some(val_str.to_string());
                    }
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                tag_stack.pop();

                if name == "Bal" && in_bal {
                    let parsed_amt = parse_dsl_amount(
                        &bal_amt_str,
                        &template.formatting.number.thousand_separator,
                        &template.formatting.number.decimal_separator,
                        template.formatting.number.decimal_places,
                        &template.formatting.number.strip_tokens,
                        currency,
                    ).ok();

                    if let Some(mut amt) = parsed_amt {
                        if bal_ind == "DBIT" {
                            amt = Money::from_minor(-amt.amount(), currency);
                        }
                        if bal_type == "OPBD" || bal_type == "PRCD" {
                            opening_balance = Some(amt);
                        } else if bal_type == "CLBD" || bal_type == "ITBD" {
                            closing_balance = Some(amt);
                        }
                    }
                    in_bal = false;
                } else if name == entry_tag && in_entry {
                    if let (Some(amt), Some(ind)) = (entry_amount, entry_indicator) {
                        row_counter += 1;
                        let ref_code = if entry_ref.is_empty() {
                            format!("{}-{:04}", template.bank_code, row_counter)
                        } else {
                            entry_ref.clone()
                        };

                        let tx_timestamp = entry_date.unwrap_or(1726400000 + (row_counter as i64 * 3600));

                        transactions.push(CanonicalTx {
                            row_id: row_counter,
                            bank_code: template.bank_code.clone(),
                            ref_code,
                            posting_type: ind,
                            amount: amt,
                            balance_after: None,
                            counterparty_name: if entry_cp_name.is_empty() { None } else { Some(entry_cp_name.clone()) },
                            counterparty_account: None,
                            narration: entry_narration.clone(),
                            tx_timestamp,
                        });
                    }
                    in_entry = false;
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(ParseError::XmlError(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    let mut stmt = CanonicalStatement {
        bank_code: template.bank_code.clone(),
        account_number: account_number.or_else(|| Some(format!("{}-XML-ACC", template.bank_code))),
        opening_balance,
        closing_balance,
        transactions,
    };

    // Verify invariants
    verify_statement_invariants(&mut stmt, template)?;

    Ok(stmt)
}
