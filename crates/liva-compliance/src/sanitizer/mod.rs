//! Decree 13/2023/NĐ-CP Personal Data Protection Sanitizer.
//!
//! Masks Personally Identifiable Information (CCCD, personal phone numbers, bank accounts, personal names)
//! of natural persons while strictly preserving legal corporate entity identities for auditability.

/// Checks if a text represents a legal corporate/institutional entity.
pub fn is_corporate_entity(text: &str) -> bool {
    let upper = text.to_uppercase();
    upper.contains("CONG TY")
        || upper.contains("CÔNG TY")
        || upper.contains("TNHH")
        || upper.contains("CO PHAN")
        || upper.contains("CỔ PHẦN")
        || upper.contains("DOANH NGHIEP")
        || upper.contains("DOANH NGHIỆP")
        || upper.contains("NGAN HANG")
        || upper.contains("NGÂN HÀNG")
        || upper.contains("TRUONG DAI HOC")
        || upper.contains("TRƯỜNG ĐẠI HỌC")
        || upper.contains("BENH VIEN")
        || upper.contains("BỆNH VIỆN")
        || upper.contains("DNTN")
        || upper.contains("CHI NHANH")
        || upper.contains("CHI NHÁNH")
        || upper.contains("TAP DOAN")
        || upper.contains("TẬP ĐOÀN")
        || upper.contains("TONG CONG TY")
        || upper.contains("TỔNG CÔNG TY")
}

/// Sanitizes a counterparty name: preserves corporate entities and redacts natural person names.
pub fn sanitize_counterparty_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if is_corporate_entity(trimmed) {
        return trimmed.to_string();
    }

    let patronymics = [
        "NGUYEN", "NGUYỄN", "TRAN", "TRẦN", "LE", "LÊ", "PHAM", "PHẠM", "HOANG", "HOÀNG",
        "VU", "VŨ", "DANG", "ĐẶNG", "BUI", "BÙI", "DO", "ĐỖ", "TRUONG", "TRƯƠNG",
        "DINH", "ĐINH", "NGO", "NGÔ", "PHAN", "HUYNH", "HUỲNH", "VO", "VÕ", "DUONG", "DƯƠNG",
        "LY", "LÝ", "HA", "HÀ", "DOAN", "ĐOÀN",
    ];
    let upper = trimmed.to_uppercase();
    for p in patronymics {
        if upper.starts_with(p) {
            return "[REDACTED_NAME]".to_string();
        }
    }

    sanitize_decree13_pii(trimmed)
}

/// Sanitizes text to remove Decree 13 natural person PII while preserving legal corporate entities.
pub fn sanitize_decree13_pii(text: &str) -> String {
    let mut result = text.trim().to_string();

    // Check if whole text is a corporate entity declaration
    let is_corporate = is_corporate_entity(&result);

    // 1. Redact CCCD: 12-digit number starting with '0'
    let words: Vec<&str> = result.split_whitespace().collect();
    let mut cleaned_words = Vec::new();

    for w in words {
        let clean = w.trim_matches(|c: char| !c.is_ascii_digit());
        if clean.len() == 12 && clean.starts_with('0') {
            cleaned_words.push("[REDACTED_CCCD]");
        } else if (clean.len() == 10 || clean.len() == 11)
            && (clean.starts_with("03")
                || clean.starts_with("05")
                || clean.starts_with("07")
                || clean.starts_with("08")
                || clean.starts_with("09"))
        {
            cleaned_words.push("[REDACTED_PHONE]");
        } else {
            cleaned_words.push(w);
        }
    }
    result = cleaned_words.join(" ");

    // 2. Redact Bank Account when preceded by keywords (tk, stk, account, so tai khoan)
    let lower = result.to_lowercase();
    for kw in &["stk ", "tk ", "account ", "so tk ", "so tai khoan "] {
        if let Some(pos) = lower.find(kw) {
            let after_kw = &result[pos + kw.len()..];
            if let Some(space_idx) = after_kw.find(' ') {
                let token = &after_kw[..space_idx].trim_matches(|c: char| !c.is_ascii_alphanumeric());
                if token.len() >= 8 && token.len() <= 16 && token.chars().all(|c| c.is_ascii_digit()) {
                    result = format!("{}[REDACTED_ACCOUNT]{}", &result[..pos + kw.len()], &after_kw[space_idx..]);
                }
            } else {
                let token = after_kw.trim_matches(|c: char| !c.is_ascii_alphanumeric());
                if token.len() >= 8 && token.len() <= 16 && token.chars().all(|c| c.is_ascii_digit()) {
                    result = format!("{}[REDACTED_ACCOUNT]", &result[..pos + kw.len()]);
                }
            }
        }
    }

    // 3. Mask natural person names if not corporate entity
    if !is_corporate {
        let patronymics = [
            "NGUYEN", "NGUYỄN", "TRAN", "TRẦN", "LE", "LÊ", "PHAM", "PHẠM", "HOANG", "HOÀNG",
            "VU", "VŨ", "DANG", "ĐẶNG", "BUI", "BÙI", "DO", "ĐỖ", "TRUONG", "TRƯƠNG",
            "DINH", "ĐINH", "NGO", "NGÔ", "PHAN", "HUYNH", "HUỲNH", "VO", "VÕ", "DUONG", "DƯƠNG",
            "LY", "LÝ", "HA", "HÀ", "DOAN", "ĐOÀN",
        ];
        let upper = result.to_uppercase();
        for p in patronymics {
            if upper.starts_with(p) {
                return "[REDACTED_NAME]".to_string();
            }
        }
    }

    result
}

