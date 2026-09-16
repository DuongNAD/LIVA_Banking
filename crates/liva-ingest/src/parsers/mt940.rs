//! SWIFT MT940 Customer Statement Message Parser.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::error::IngestError;
use crate::models::{ContainerFormat, RawStatementRecord, RawTransactionRecord, StatementParser};
use liva_normalize::{parse_monetary_amount, BankIdentifier};

pub struct Mt940Parser;

impl StatementParser for Mt940Parser {
    fn can_parse(&self, container: ContainerFormat, _bank: BankIdentifier) -> bool {
        container == ContainerFormat::SwiftMt || container == ContainerFormat::Unknown
    }

    fn parse(&self, bytes: &[u8], filename: &str) -> Result<RawStatementRecord, IngestError> {
        let text = String::from_utf8_lossy(bytes);
        let mut bank = BankIdentifier::Unknown;
        let mut account_no: Option<String> = None;
        let mut opening_balance: Option<u64> = None;
        let mut closing_balance: Option<u64> = None;

        let mut transactions = Vec::new();
        let mut current_tx: Option<RawTransactionRecord> = None;
        let mut tx_row_id = 0;

        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() {
                i += 1;
                continue;
            }

            // Check SWIFT header block for Bank BIC
            if line.starts_with("{1:F01") || line.starts_with("{2:I940") {
                let upper = line.to_uppercase();
                if upper.contains("BFTV") {
                    bank = BankIdentifier::Vietcombank;
                } else if upper.contains("TCBK") {
                    bank = BankIdentifier::Techcombank;
                } else if upper.contains("BIDV") {
                    bank = BankIdentifier::Bidv;
                } else if upper.contains("ICBV") {
                    bank = BankIdentifier::VietinBank;
                } else if upper.contains("MSCB") {
                    bank = BankIdentifier::MbBank;
                } else if upper.contains("VBAA") {
                    bank = BankIdentifier::Agribank;
                }
            }

            if let Some(acc_raw) = line.strip_prefix(":25:") {
                let acc = acc_raw.trim();
                if !acc.is_empty() {
                    account_no = Some(acc.to_string());
                }
            } else if line.starts_with(":60F:") || line.starts_with(":60M:") {
                let payload = line[5..].trim();
                // Format: C/D (1 char) + YYMMDD (6 chars) + CCY (3 chars) + Amount (comma decimal)
                if payload.len() >= 10 {
                    let amt_part = &payload[10..];
                    if let Ok(parsed) = parse_monetary_amount(amt_part) {
                        opening_balance = Some(parsed.minor_units);
                    }
                }
            } else if line.starts_with(":62F:") || line.starts_with(":62M:") {
                let payload = line[5..].trim();
                if payload.len() >= 10 {
                    let amt_part = &payload[10..];
                    if let Ok(parsed) = parse_monetary_amount(amt_part) {
                        closing_balance = Some(parsed.minor_units);
                    }
                }
            } else if let Some(stripped_61) = line.strip_prefix(":61:") {
                // Finalize previous tx if any
                if let Some(tx) = current_tx.take() {
                    transactions.push(tx);
                }

                tx_row_id += 1;
                let payload = stripped_61.trim();
                // Format :61: YYMMDD[MMDD]C/D/RC/RD Amount, NTRF Ref//ServicerRef
                let mut tx = RawTransactionRecord {
                    row_id: tx_row_id,
                    ..Default::default()
                };

                if payload.len() >= 6 {
                    let yymmdd = &payload[..6];
                    tx.date_str = yymmdd.to_string();

                    let rest = &payload[6..];
                    let (is_credit, amt_str, after_amt) = parse_field_61_amount(rest);
                    tx.is_credit = is_credit;

                    if let Ok(parsed) = parse_monetary_amount(&amt_str) {
                        tx.amount_cents = Some(parsed.minor_units);
                    }

                    // Extract doc_ref from after_amt if present (e.g. NTRFFT2621400001//NONREF)
                    if let Some(sep_idx) = after_amt.find("//") {
                        let ref_candidate = &after_amt[..sep_idx];
                        let clean_ref = if ref_candidate.len() > 4 && (ref_candidate.starts_with('N') || ref_candidate.starts_with('F')) {
                            &ref_candidate[4..]
                        } else {
                            ref_candidate
                        };
                        tx.doc_ref = Some(clean_ref.trim().to_string());
                    } else if !after_amt.trim().is_empty() {
                        let clean_ref = if after_amt.len() > 4 && (after_amt.starts_with('N') || after_amt.starts_with('F')) {
                            &after_amt[4..]
                        } else {
                            &after_amt
                        };
                        tx.doc_ref = Some(clean_ref.trim().to_string());
                    }
                }

                current_tx = Some(tx);
            } else if let Some(stripped_86) = line.strip_prefix(":86:") {
                let mut narration = stripped_86.trim().to_string();
                // Collect multi-line narration until next tag or end
                while i + 1 < lines.len() && !lines[i + 1].trim().starts_with(':') && !lines[i + 1].trim().starts_with('-') && !lines[i + 1].trim().starts_with('{') {
                    i += 1;
                    let next_line = lines[i].trim();
                    if !next_line.is_empty() {
                        narration.push(' ');
                        narration.push_str(next_line);
                    }
                }

                if let Some(ref mut tx) = current_tx {
                    // Extract counterparty name from ?32 or ?33 or generic text
                    if let Some(pos32) = narration.find("?32") {
                        let after32 = &narration[pos32 + 3..];
                        let end_pos = after32.find('?').unwrap_or(after32.len());
                        let p32 = after32[..end_pos].trim();

                        let mut full_partner = p32.to_string();
                        if let Some(pos33) = narration.find("?33") {
                            let after33 = &narration[pos33 + 3..];
                            let end_pos33 = after33.find('?').unwrap_or(after33.len());
                            let p33 = after33[..end_pos33].trim();
                            if !p33.is_empty() {
                                full_partner.push(' ');
                                full_partner.push_str(p33);
                            }
                        }
                        tx.counterparty_name = Some(full_partner);
                    }

                    // Clean subfields delimiters (?00, ?20, ?21, etc.)
                    let clean_narr = narration
                        .split('?')
                        .filter(|part| part.len() > 2)
                        .map(|part| &part[2..])
                        .collect::<Vec<_>>()
                        .join(" ");

                    tx.narration = if clean_narr.is_empty() { narration } else { clean_narr };
                }
            }

            i += 1;
        }

        if let Some(tx) = current_tx.take() {
            transactions.push(tx);
        }

        if transactions.is_empty() {
            return Err(IngestError::InvalidStructure(format!(
                "No :61: transaction lines parsed from SWIFT MT940 file {filename}"
            )));
        }

        Ok(RawStatementRecord {
            bank,
            format: ContainerFormat::SwiftMt,
            account_no,
            account_name: None,
            opening_balance,
            closing_balance,
            transactions,
        })
    }
}

/// Parses C/D/RC/RD indicator and monetary amount from line 61 payload.
fn parse_field_61_amount(rest: &str) -> (bool, String, String) {
    let mut is_credit = true;
    let mut amt_start = 0;

    // Optional 4 chars entry date (MMDD) if followed by C or D
    if rest.len() >= 5 && (rest[4..].starts_with('C') || rest[4..].starts_with('D') || rest[4..].starts_with("RC") || rest[4..].starts_with("RD")) {
        // Skip entry date
        let after_entry = &rest[4..];
        if after_entry.starts_with('C') {
            is_credit = true;
            amt_start = 5;
        } else if after_entry.starts_with('D') {
            is_credit = false;
            amt_start = 5;
        } else if after_entry.starts_with("RC") {
            is_credit = false;
            amt_start = 6;
        } else if after_entry.starts_with("RD") {
            is_credit = true;
            amt_start = 6;
        }
    } else if rest.starts_with('C') {
        is_credit = true;
        amt_start = 1;
    } else if rest.starts_with('D') {
        is_credit = false;
        amt_start = 1;
    } else if rest.starts_with("RC") {
        is_credit = false;
        amt_start = 2;
    } else if rest.starts_with("RD") {
        is_credit = true;
        amt_start = 2;
    }

    let amt_slice = &rest[amt_start..];
    // Find where amount ends (at transaction type letter: S, N, F, etc. followed by 3 letters)
    let mut amt_end = amt_slice.len();
    for (idx, c) in amt_slice.char_indices() {
        if c.is_alphabetic() && c != 'N' && c != 'S' && c != 'F' {
            amt_end = idx;
            break;
        } else if (c == 'N' || c == 'S' || c == 'F') && idx > 0 {
            // Check if preceded by comma or digit
            amt_end = idx;
            break;
        }
    }

    let amt_str = amt_slice[..amt_end].trim_end_matches(',').to_string();
    let remainder = amt_slice[amt_end..].to_string();

    (is_credit, amt_str, remainder)
}
