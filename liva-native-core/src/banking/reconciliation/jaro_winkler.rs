//! Deterministic Jaro-Winkler String Similarity Metric.
//!
//! Includes Vietnamese diacritics normalization for party names and token set overlap.

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
            'e' | 'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => {
                'e'
            }
            'E' | 'È' | 'É' | 'Ẹ' | 'Ẻ' | 'Ẽ' | 'Ê' | 'Ề' | 'Ế' | 'Ệ' | 'Ể' | 'Ễ' => {
                'e'
            }
            'i' | 'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
            'I' | 'Ì' | 'Í' | 'Ị' | 'Ỉ' | 'Ĩ' => 'i',
            'o' | 'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ'
            | 'ớ' | 'ợ' | 'ở' | 'ỡ' => 'o',
            'O' | 'Ò' | 'Ó' | 'Ọ' | 'Ỏ' | 'Õ' | 'Ô' | 'Ồ' | 'Ố' | 'Ộ' | 'Ổ' | 'Ỗ' | 'Ơ' | 'Ờ'
            | 'Ớ' | 'Ợ' | 'Ở' | 'Ỡ' => 'o',
            'u' | 'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => {
                'u'
            }
            'U' | 'Ù' | 'Ú' | 'Ụ' | 'Ủ' | 'Ũ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ự' | 'Ử' | 'Ữ' => {
                'u'
            }
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

/// Calculates the Jaro similarity between two string slices.
pub fn jaro_similarity(s1: &str, s2: &str) -> f64 {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches: usize = 0;

    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);

        for j in start..end {
            if s2_matches[j] {
                continue;
            }
            if s1_chars[i] == s2_chars[j] {
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions: usize = 0;
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    let t = (transpositions / 2) as f64;

    ((m / len1 as f64) + (m / len2 as f64) + ((m - t) / m)) / 3.0
}

/// Calculates the Jaro-Winkler similarity score (between 0.0 and 1.0).
/// Uses prefix scale p = 0.1, up to max 4 matching prefix characters.
pub fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    let jaro = jaro_similarity(s1, s2);
    if jaro < 0.7 {
        return jaro;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let prefix_limit = 4.min(s1_chars.len().min(s2_chars.len()));
    let mut prefix_len = 0;

    for i in 0..prefix_limit {
        if s1_chars[i] == s2_chars[i] {
            prefix_len += 1;
        } else {
            break;
        }
    }

    let p = 0.1;
    jaro + (prefix_len as f64 * p * (1.0 - jaro))
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

/// Compares two Vietnamese party names using Jaro-Winkler and token set containment
/// after stripping bank narration noise and corporate legal noise.
pub fn compare_party_names(name1: &str, name2: &str) -> f64 {
    let clean1 = strip_bank_narration_noise(name1);
    let clean2 = strip_bank_narration_noise(name2);
    let n1 = normalize_vietnamese_text(&clean1);
    let n2 = normalize_vietnamese_text(&clean2);

    if n1 == n2 {
        return 1.0;
    }
    if n1.is_empty() || n2.is_empty() {
        return 0.0;
    }

    // Extract core distinctive business tokens by stripping corporate legal noise
    let (tokens1, core1) = strip_corporate_legal_noise(&n1);
    let (tokens2, core2) = strip_corporate_legal_noise(&n2);

    // Direct match on core distinctive names
    if core1 == core2 {
        return 1.0;
    }

    // Token set overlap on core tokens
    let set1: HashSet<&str> = tokens1.iter().map(|s| s.as_str()).collect();
    let set2: HashSet<&str> = tokens2.iter().map(|s| s.as_str()).collect();
    let intersection = set1.intersection(&set2).count();
    let min_len = set1.len().min(set2.len());
    let max_len = set1.len().max(set2.len());

    let token_containment = if min_len > 0 {
        intersection as f64 / min_len as f64
    } else {
        0.0
    };

    // If all core tokens of the shorter name are present in the longer name
    if min_len > 0 && intersection == min_len {
        let ratio = min_len as f64 / max_len as f64;
        return (0.88 + 0.12 * ratio).min(1.0);
    }

    // Core string containment (e.g. "abc" inside "abc group" or narration)
    if !core1.is_empty() && !core2.is_empty() {
        let min_char_len = core1.len().min(core2.len());
        let max_char_len = core1.len().max(core2.len());
        if min_char_len >= 3 && (core1.contains(&core2) || core2.contains(&core1)) {
            let ratio = min_char_len as f64 / max_char_len as f64;
            return (0.88 + 0.12 * ratio).min(1.0);
        }
    }

    if token_containment >= 0.80 {
        return (0.88 + 0.12 * token_containment).min(1.0);
    }

    // Jaro-Winkler similarity computed on stripped core tokens
    let core_jw = jaro_winkler(&core1, &core2);

    // If stripped core tokens have zero token overlap:
    // Enforce low party similarity (< 0.70) so FuzzyMatcher will NOT auto-approve.
    if intersection == 0 {
        return core_jw.min(0.65);
    }

    let has_distinctive_token = tokens1
        .iter()
        .any(|t| t.len() >= 4 && set2.contains(t.as_str()));

    let mut score = core_jw.max(token_containment * 0.85);
    if has_distinctive_token && score < 0.75 {
        score = score.max(0.70 + 0.15 * token_containment);
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_vietnamese() {
        let norm = normalize_vietnamese_text("Công ty Cổ phần Thép Việt Nhật");
        assert_eq!(norm, "cong ty co phan thep viet nhat");
    }

    #[test]
    fn test_jaro_winkler_exact() {
        let score = jaro_winkler("MARTHA", "MARHTA");
        assert!((score - 0.961).abs() < 0.01);
    }

    #[test]
    fn test_party_name_fuzzy_match() {
        let score = compare_party_names("Công ty Cổ phần Thương mại ABC", "CONG TY ABC");
        assert!(score >= 0.85, "Expected score >= 0.85, got {score}");

        let score2 = compare_party_names("NGUYEN VAN A", "Nguyen Van A");
        assert_eq!(score2, 1.0);

        let score3 = compare_party_names("Công ty TNHH Giải pháp Phần mềm LIVA", "LIVA SOFTWARE");
        assert!(score3 >= 0.70);
    }

    #[test]
    fn test_unrelated_companies_sharing_legal_form_produce_low_score() {
        let score = compare_party_names("CONG TY TNHH MINH ANH", "CONG TY TNHH PHUONG DONG");
        assert!(score < 0.70, "Expected score < 0.70, got {score}");

        let score2 = compare_party_names(
            "CÔNG TY CỔ PHẦN THƯƠNG MẠI MINH ANH",
            "CÔNG TY CỔ PHẦN THƯƠNG MẠI PHƯƠNG ĐÔNG",
        );
        assert!(score2 < 0.70, "Expected score2 < 0.70, got {score2}");
    }

    #[test]
    fn test_unrelated_companies_sharing_industry_descriptors_produce_low_score() {
        // Unrelated entities sharing generic industry sector descriptors must score < 0.70
        let score_tech = compare_party_names(
            "CÔNG TY CỔ PHẦN CÔNG NGHỆ THÁI BÌNH",
            "CÔNG TY CỔ PHẦN CÔNG NGHỆ SÔNG HỒNG",
        );
        assert!(
            score_tech < 0.70,
            "Technology sector sharing must score < 0.70, got {score_tech}"
        );

        let score_const = compare_party_names(
            "CÔNG TY TNHH XÂY DỰNG BÌNH MINH",
            "CÔNG TY TNHH XÂY DỰNG RẠNG ĐÔNG",
        );
        assert!(
            score_const < 0.70,
            "Construction sector sharing must score < 0.70, got {score_const}"
        );

        let score_re = compare_party_names(
            "CÔNG TY CỔ PHẦN BẤT ĐỘNG SẢN AN KHANG",
            "CÔNG TY CỔ PHẦN BẤT ĐỘNG SẢN THỊNH VƯỢNG",
        );
        assert!(
            score_re < 0.70,
            "Real estate sector sharing must score < 0.70, got {score_re}"
        );

        let score_inv = compare_party_names(
            "CÔNG TY CỔ PHẦN ĐẦU TƯ THÁI BÌNH",
            "CÔNG TY CỔ PHẦN ĐẦU TƯ SÔNG HỒNG",
        );
        assert!(
            score_inv < 0.70,
            "Investment sector sharing must score < 0.70, got {score_inv}"
        );

        let score_mfg = compare_party_names(
            "CÔNG TY TNHH SẢN XUẤT AN BÌNH",
            "CÔNG TY TNHH SẢN XUẤT VIỆT THẮNG",
        );
        assert!(
            score_mfg < 0.70,
            "Manufacturing sector sharing must score < 0.70, got {score_mfg}"
        );

        let score_trans = compare_party_names(
            "CÔNG TY CỔ PHẦN VẬN TẢI HẢI PHÒNG",
            "CÔNG TY CỔ PHẦN VẬN TẢI SÀI GÒN",
        );
        assert!(
            score_trans < 0.70,
            "Transport sector sharing must score < 0.70, got {score_trans}"
        );

        let score_edu = compare_party_names(
            "CÔNG TY CỔ PHẦN GIÁO DỤC TOÀN CẦU",
            "CÔNG TY CỔ PHẦN GIÁO DỤC ĐẠI VIỆT",
        );
        assert!(
            score_edu < 0.70,
            "Education sector sharing must score < 0.70, got {score_edu}"
        );

        let score_med = compare_party_names(
            "CÔNG TY CỔ PHẦN Y TẾ VIỆT NHẬT",
            "CÔNG TY CỔ PHẦN Y TẾ HOÀNG GIA",
        );
        assert!(
            score_med < 0.70,
            "Healthcare sector sharing must score < 0.70, got {score_med}"
        );

        let score_pharma = compare_party_names(
            "CÔNG TY CỔ PHẦN DƯỢC PHẨM TRUNG ƯƠNG",
            "CÔNG TY CỔ PHẦN DƯỢC PHẨM ĐÔNG Á",
        );
        assert!(
            score_pharma < 0.70,
            "Pharma sector sharing must score < 0.70, got {score_pharma}"
        );

        let score_media = compare_party_names(
            "CÔNG TY CỔ PHẦN TRUYỀN THÔNG ĐẠI DƯƠNG",
            "CÔNG TY CỔ PHẦN TRUYỀN THÔNG BẠCH ĐẰNG",
        );
        assert!(
            score_media < 0.70,
            "Media sector sharing must score < 0.70, got {score_media}"
        );
    }

    #[test]
    fn test_genuine_company_variants_high_score() {
        let score1 = compare_party_names("CONG TY TNHH MINH ANH", "CTY TNHH MINH ANH");
        assert!(score1 >= 0.85, "Expected score1 >= 0.85, got {score1}");

        let score2 = compare_party_names("CONG TY TNHH MINH ANH", "MINH ANH CO LTD");
        assert!(score2 >= 0.85, "Expected score2 >= 0.85, got {score2}");

        let score3 = compare_party_names("CONG TY TNHH MINH ANH", "MINH ANH");
        assert!(score3 >= 0.85, "Expected score3 >= 0.85, got {score3}");
    }

    #[test]
    fn test_strip_bank_narration_noise() {
        let n1 = strip_bank_narration_noise("MBVCB.123456789.012345.CT TU NGUYEN VAN A");
        assert!(n1.contains("nguyen van a"), "Got: {n1}");

        let n2 = strip_bank_narration_noise("Napas VietQR TT HD102 Tu: CONG TY ABC");
        assert!(n2.contains("cong ty abc"), "Got: {n2}");

        let n3 = strip_bank_narration_noise("QRIBFT chuyen khoan hop dong HD200 CTY MINH ANH");
        assert!(n3.contains("minh anh"), "Got: {n3}");

        let score = compare_party_names("Công ty ABC", "Napas VietQR TT HD102 Tu: CONG TY ABC");
        assert!(score >= 0.85, "Expected score >= 0.85, got {score}");
    }

    #[test]
    fn test_nfd_combining_diacritics_normalization() {
        // NFD representation: 'e' + '\u{0302}' (circumflex) + '\u{0323}' (dot below) -> "ệ"
        let nfd_viet = "Vi\u{0065}\u{0302}\u{0323}t Nam";
        let norm = normalize_vietnamese_text(nfd_viet);
        assert_eq!(norm, "viet nam");
    }
}
