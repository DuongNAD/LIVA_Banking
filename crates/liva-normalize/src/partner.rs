//! Partner name normalization: unaccented uppercase folding and company prefix standardization.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use unicode_normalization::UnicodeNormalization;

/// Normalizes a counterparty or partner name into deterministic, unaccented, uppercase ASCII.
///
/// Steps:
/// 1. NFC Canonical Decomposition / Vietnamese diacritic stripping.
/// 2. Punctuation removal: replaces non-alphanumeric characters with spaces.
/// 3. Whitespace condensation & uppercase ASCII conversion.
/// 4. Legal company prefix standardization (`CTY` -> `CONG TY`, `CP` -> `CO PHAN`, etc.).
pub fn normalize_partner_name(input: &str) -> String {
    let unaccented = strip_vietnamese_diacritics(input);

    // Filter to ASCII alphanumeric and space
    let mut filtered = String::with_capacity(unaccented.len());
    for ch in unaccented.chars() {
        if ch.is_ascii_alphanumeric() {
            filtered.push(ch.to_ascii_uppercase());
        } else {
            filtered.push(' ');
        }
    }

    // Standardize corporate tokens
    let words: Vec<&str> = filtered.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let mut result_words = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        if w == "CTY" {
            result_words.push("CONG");
            result_words.push("TY");
            i += 1;
        } else if w == "CP" {
            result_words.push("CO");
            result_words.push("PHAN");
            i += 1;
        } else if w == "TMDV" {
            result_words.push("THUONG");
            result_words.push("MAI");
            result_words.push("DICH");
            result_words.push("VU");
            i += 1;
        } else if w == "TM" && i + 1 < words.len() && words[i + 1] == "DV" {
            result_words.push("THUONG");
            result_words.push("MAI");
            result_words.push("DICH");
            result_words.push("VU");
            i += 2;
        } else if w == "TRACH"
            && i + 3 < words.len()
            && words[i + 1] == "NHIEM"
            && words[i + 2] == "HUU"
            && words[i + 3] == "HAN"
        {
            result_words.push("TNHH");
            i += 4;
        } else if w == "XNK" {
            result_words.push("XUAT");
            result_words.push("NHAP");
            result_words.push("KHAU");
            i += 1;
        } else {
            result_words.push(w);
            i += 1;
        }
    }

    result_words.join(" ")
}

/// Strips all Vietnamese accents and special characters (e.g. đ, Đ, ư, ơ, ê, â, ă) into ASCII base equivalents.
pub fn strip_vietnamese_diacritics(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    // Normalize to NFD so base characters and combining diacritics are split
    for ch in s.nfd() {
        match ch {
            // Vietnamese D with stroke
            'đ' => out.push('d'),
            'Đ' => out.push('D'),
            // Skip combining diacritical marks (U+0300 through U+036F)
            '\u{0300}'..='\u{036f}' => {}
            // Horn and breve modifications in case they don't decompose
            'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => out.push('u'),
            'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => out.push('U'),
            'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' => out.push('o'),
            'Ơ' | 'Ờ' | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => out.push('O'),
            'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => out.push('e'),
            'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => out.push('E'),
            'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => out.push('a'),
            'Â' | 'Ầ' | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => out.push('A'),
            'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' => out.push('a'),
            'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' => out.push('A'),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_partner_an_phat() {
        let input = "Công ty TNHH Thương mại Dịch vụ An Phát";
        let res = normalize_partner_name(input);
        assert_eq!(res, "CONG TY TNHH THUONG MAI DICH VU AN PHAT");
    }

    #[test]
    fn test_normalize_partner_hoa_binh() {
        let input = "Công ty Cổ phần Xây dựng và Địa ốc Hòa Bình";
        let res = normalize_partner_name(input);
        assert_eq!(res, "CONG TY CO PHAN XAY DUNG VA DIA OC HOA BINH");
    }

    #[test]
    fn test_normalize_company_abbreviations() {
        let input = "Cty CP TM DV An Binh";
        let res = normalize_partner_name(input);
        assert_eq!(res, "CONG TY CO PHAN THUONG MAI DICH VU AN BINH");
    }

    #[test]
    fn test_strip_punctuation_and_whitespace() {
        let input = "  (CONG TY TNHH) - DICH VU & DU LICH...  ";
        let res = normalize_partner_name(input);
        assert_eq!(res, "CONG TY TNHH DICH VU DU LICH");
    }
}
