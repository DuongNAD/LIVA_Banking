use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

/// Normalizes Vietnamese accented characters to ASCII equivalents for robust matching.
/// Employs Unicode NFC composition first to handle decomposed NFD strings,
/// strips combining diacritical marks, and folds characters to lowercase ASCII.
pub fn normalize_vietnamese_text(text: &str) -> String {
    // 1. NFC normalization to compose combining characters
    let nfc_text: String = text.nfc().collect();
    let mut out = String::with_capacity(nfc_text.len());
    for c in nfc_text.chars() {
        // Skip any leftover combining diacritical marks (\u{0300}..=\u{036F})
        if ('\u{0300}'..='\u{036F}').contains(&c) {
            continue;
        }
        let mapped = match c {
            'a' | 'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă' | 'ằ'
            | 'ắ' | 'ặ' | 'ẳ' | 'ẵ' => 'a',
            'A' | 'À' | 'Á' | 'Ạ' | 'Ả' | 'Ã' | 'Â' | 'Ầ' | 'Ấ' | 'Ậ' | 'Ẩ' | 'Ẫ' | 'Ă' | 'Ằ'
            | 'Ắ' | 'Ặ' | 'Ẳ' | 'Ẵ' => 'a',
            'e' | 'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => 'e',
            'E' | 'È' | 'É' | 'Ẹ' | 'Ẻ' | 'Ẽ' | 'Ê' | 'Ề' | 'Ế' | 'Ệ' | 'Ể' | 'Ễ' => 'e',
            'i' | 'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
            'I' | 'Ì' | 'Í' | 'Ị' | 'Ỉ' | 'Ĩ' => 'i',
            'o' | 'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ'
            | 'ớ' | 'ợ' | 'ở' | 'ỡ' => 'o',
            'O' | 'Ò' | 'Ó' | 'Ọ' | 'Ỏ' | 'Õ' | 'Ô' | 'Ồ' | 'Ố' | 'Ộ' | 'Ổ' | 'Ỗ' | 'Ơ' | 'Ờ'
            | 'Ớ' | 'Ợ' | 'Ở' | 'Ỡ' => 'o',
            'u' | 'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => 'u',
            'U' | 'Ù' | 'Ú' | 'Ụ' | 'Ủ' | 'Ũ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ự' | 'Ử' | 'Ữ' => 'u',
            'y' | 'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
            'Y' | 'Ỳ' | 'Ý' | 'Ỵ' | 'Ỷ' | 'Ỹ' => 'y',
            'd' | 'đ' => 'd',
            'D' | 'Đ' => 'd',
            _ => c.to_ascii_lowercase(),
        };
        if mapped.is_alphanumeric() {
            out.push(mapped);
        } else {
            out.push(' ');
        }
    }
    // Collapse multi-spaces
    let words: Vec<&str> = out.split_whitespace().collect();
    words.join(" ")
}

/// Strips bank transaction boilerplate, payment gateway prefixes, Napas/VietQR headers,
/// transaction trace tokens, and account identifiers to isolate party names and invoice references.
pub fn strip_bank_narration_noise(narration: &str) -> String {
    let norm = normalize_vietnamese_text(narration);
    let mut text = format!(" {norm} ");

    // Multi-word bank narration phrases (ordered from longest to shortest)
    let bank_phrases = [
        "napas vietqr tt",
        "napas vietqr",
        "vietqr tt",
        "vietqr",
        "napas 247",
        "napas",
        "qribft",
        "ibft",
        "mbvcb",
        "ibvcb",
        "chuyen tien tu tk",
        "chuyen tien den tk",
        "chuyen tien tu",
        "chuyen tien den",
        "chuyen tien",
        "chuyen khoan tu tk",
        "chuyen khoan den tk",
        "chuyen khoan tu",
        "chuyen khoan den",
        "chuyen khoan",
        "ct tu tk",
        "ct den tk",
        "ct tu",
        "ct den",
        "sang tk",
        "den tk",
        "tu tk",
        "thanh toan tien hang",
        "thanh toan hoa don",
        "thanh toan hd",
        "thanh toan",
        "tt tien hang",
        "tt hoa don",
        "tt hd",
        "ck tien hang",
        "ck hoa don",
        "ck hd",
        "ck",
        "noi dung",
    ];

    for phrase in &bank_phrases {
        let pattern = format!(" {phrase} ");
        while let Some(pos) = text.find(&pattern) {
            text.replace_range(pos..pos + pattern.len(), " ");
        }
    }

    // Filter out trace references, standalone codes, or pure long digit strings (account numbers, timestamps)
    let raw_words: Vec<&str> = text.split_whitespace().collect();
    let mut tokens = Vec::new();
    for (i, token) in raw_words.iter().enumerate() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.is_empty() {
            continue;
        }
        // Protect "van tai" (transportation) and "dau tu" (investment) from single stop word stripping
        let is_protected = (clean == "tai"
            && i > 0
            && raw_words[i - 1].trim_matches(|c: char| !c.is_alphanumeric()) == "van")
            || (clean == "tu"
                && i > 0
                && raw_words[i - 1].trim_matches(|c: char| !c.is_alphanumeric()) == "dau");

        // Ignore single stop tokens unless protected by preceding word
        if !is_protected
            && matches!(
                clean,
                "tu" | "den" | "tk" | "nd" | "so" | "tai" | "khoan" | "gd"
            )
        {
            continue;
        }
        // If it is a sequence of 6+ digits, it is likely an account number or sequence code
        if clean.len() >= 6 && clean.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        // If it starts with FT, NPS, VN and followed by digits (trace codes)
        if (clean.starts_with("ft")
            && clean.len() >= 6
            && clean.chars().skip(2).any(|c| c.is_ascii_digit()))
            || (clean.starts_with("nps") && clean.len() >= 6)
            || (clean.starts_with("vn")
                && clean.len() >= 6
                && clean.chars().skip(2).any(|c| c.is_ascii_digit()))
        {
            continue;
        }
        tokens.push(clean.to_string());
    }

    if tokens.is_empty() {
        norm
    } else {
        tokens.join(" ")
    }
}

/// Strips Vietnamese corporate legal form stop words and phrases to isolate
/// the core distinctive business tokens.
pub fn strip_corporate_legal_noise(normalized: &str) -> (Vec<String>, String) {
    let mut text = format!(" {normalized} ");

    // Multi-word phrases (ordered longest to shortest)
    let phrases = [
        "trach nhiem huu han",
        "thuong mai dich vu",
        "mot thanh vien",
        "bat dong san",
        "truyen thong",
        "thuong mai",
        "duoc pham",
        "cong nghe",
        "giai phap",
        "xay dung",
        "san xuat",
        "giao duc",
        "viet nam",
        "van tai",
        "dich vu",
        "co phan",
        "cong ty",
        "dau tu",
        "co ltd",
        "y te",
    ];

    for phrase in &phrases {
        let pattern = format!(" {phrase} ");
        while let Some(pos) = text.find(&pattern) {
            text.replace_range(pos..pos + pattern.len(), " ");
        }
    }

    let single_stop_tokens: HashSet<&str> = [
        "cty", "tnhh", "cp", "mtv", "corp", "inc", "ltd", "bds", "tmdv",
    ]
    .iter()
    .copied()
    .collect();

    let tokens: Vec<String> = text
        .split_whitespace()
        .filter(|t| !single_stop_tokens.contains(t))
        .map(|s| s.to_string())
        .collect();

    if tokens.is_empty() {
        let fallback_tokens: Vec<String> = normalized
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let fallback_str = fallback_tokens.join(" ");
        (fallback_tokens, fallback_str)
    } else {
        let core_str = tokens.join(" ");
        (tokens, core_str)
    }
}
