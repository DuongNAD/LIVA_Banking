//! Reference tokenization: extracts banking trace references and invoice numbers.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use regex::Regex;
use std::sync::OnceLock;

static RE_BANK_REFS: OnceLock<Regex> = OnceLock::new();
static RE_INVOICE_SINGLE: OnceLock<Regex> = OnceLock::new();
static RE_CONSECUTIVE_HD: OnceLock<Regex> = OnceLock::new();

fn get_bank_ref_regex() -> &'static Regex {
    RE_BANK_REFS.get_or_init(|| {
        // FT references, Napas references, Debit order references, Bank fee codes
        Regex::new(r"\b(FT[0-9]{8,16}|NPS[0-9]{8,16}|VN[0-9]{4,10}[A-Z0-9]+|VCBFEE[0-9]+|TCBFEE[0-9]+|BIDVFEE[0-9]+)\b").unwrap()
    })
}

fn get_invoice_single_regex() -> &'static Regex {
    RE_INVOICE_SINGLE.get_or_init(|| {
        // Invoice patterns: HD131, INV-2026, HĐ 101, HDON_12345
        Regex::new(r"(?i)\b(?:HD|INV|HĐ|HDON)[\s\-_]*([0-9]{3,8})\b").unwrap()
    })
}

fn get_consecutive_hd_regex() -> &'static Regex {
    RE_CONSECUTIVE_HD.get_or_init(|| {
        // Matches tokens like HD208, HD209, HD210
        Regex::new(r"(?i)\bHD([0-9]{3,8})\b").unwrap()
    })
}

/// Extracts all unique banking reference codes and invoice tokens from a transaction narration and optional doc_ref.
pub fn extract_reference_codes(narration: &str, doc_ref: Option<&str>) -> Vec<String> {
    let mut tokens = Vec::new();

    // 1. Process explicit doc_ref if provided
    if let Some(dref) = doc_ref {
        let trimmed = dref.trim();
        if !trimmed.is_empty() && trimmed != "NONREF" && !tokens.contains(&trimmed.to_string()) {
            tokens.push(trimmed.to_string());
        }
    }

    // 2. Extract bank reference patterns (FT, NPS, VN, Fee)
    let re_bank = get_bank_ref_regex();
    for cap in re_bank.find_iter(narration) {
        let code = cap.as_str().to_string();
        if !tokens.contains(&code) {
            tokens.push(code);
        }
    }

    // 3. Extract single & multi invoice patterns (e.g. "HD 131" -> "HD131", "HD208 VA HD209", "HD210 HD211 HD212")
    let re_inv = get_invoice_single_regex();
    for cap in re_inv.captures_iter(narration) {
        if let Some(num_match) = cap.get(1) {
            let canon = format!("HD{}", num_match.as_str());
            if !tokens.contains(&canon) {
                tokens.push(canon);
            }
        }
    }

    let re_consec = get_consecutive_hd_regex();
    for cap in re_consec.captures_iter(narration) {
        if let Some(num_match) = cap.get(1) {
            let canon = format!("HD{}", num_match.as_str());
            if !tokens.contains(&canon) {
                tokens.push(canon);
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bank_references() {
        let narration = "Napas VietQR TT HD131 Tu Cty An Phat FT2624000001";
        let tokens = extract_reference_codes(narration, None);
        assert!(tokens.contains(&"FT2624000001".to_string()));
        assert!(tokens.contains(&"HD131".to_string()));
    }

    #[test]
    fn test_extract_napas_and_debit_order() {
        let narration = "Chuyen khoan lien ngan hang NPS26080001 lenh VN2624DEB01";
        let tokens = extract_reference_codes(narration, None);
        assert!(tokens.contains(&"NPS26080001".to_string()));
        assert!(tokens.contains(&"VN2624DEB01".to_string()));
    }

    #[test]
    fn test_extract_multi_invoice_batch() {
        let narration = "Napas VietQR TT HD208 VA HD209 TU CONG TY HOA BINH";
        let tokens = extract_reference_codes(narration, None);
        assert!(tokens.contains(&"HD208".to_string()));
        assert!(tokens.contains(&"HD209".to_string()));
    }

    #[test]
    fn test_extract_multi_invoice_list() {
        let narration = "Thanh toan hoa don HD210 HD211 HD212 THEP VIET NHAT";
        let tokens = extract_reference_codes(narration, None);
        assert!(tokens.contains(&"HD210".to_string()));
        assert!(tokens.contains(&"HD211".to_string()));
        assert!(tokens.contains(&"HD212".to_string()));
    }
}
