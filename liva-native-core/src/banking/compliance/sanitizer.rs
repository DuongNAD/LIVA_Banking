//! Real-time PII and sensitive data sanitizer for Decree 13/2023/NĐ-CP compliance.
//!
//! Redacts:
//! - CCCD (Vietnamese 12-digit Citizen Identification Number).
//! - Bank account numbers (8-16 digits with mandatory keyword prefix).
//! - Vietnamese 10-digit mobile numbers (03, 05, 07, 08, 09 or +84...).
//! - Natural person full names in transaction narrations and counterparties.
//! - Secrets, bearer tokens, API keys, and passwords.
//!
//! Excludes and preserves:
//! - Legal entity names (CÔNG TY, TNHH, CP, DOANH NGHIEP, NGAN HANG, etc.).
//! - Standalone transaction amounts (e.g. 1450230000, 15000000).
//! - Standard banking dates, transaction vouchers, and non-sensitive identifiers.

use regex::Regex;
use std::sync::LazyLock;

static RE_CCCD: LazyLock<Regex> = LazyLock::new(|| {
    // Exactly 12 digits, often starting with 0 (e.g. 001, 079)
    Regex::new(r"\b0\d{11}\b").expect("Valid regex for CCCD")
});

static RE_ACCOUNT: LazyLock<Regex> = LazyLock::new(|| {
    // Bank account numbers (8-16 digits) preceded by account prefixes:
    // "stk", "tk", "acc", "account", "so tk", "so tai khoan" (with optional colon ':' or whitespace).
    // Mandatory prefix prevents over-redaction of standalone numerical amounts or phone numbers.
    Regex::new(r"(?i)\b(?:so\s+tai\s+khoan|số\s+tài\s+khoản|so\s+tk|số\s+tk|account|stk|acc|tk)[:\s]+([0-9]{8,16})\b")
        .expect("Valid regex for account numbers")
});

static RE_PHONE: LazyLock<Regex> = LazyLock::new(|| {
    // Vietnamese 10-digit mobile numbers starting with 03, 05, 07, 08, 09 or +84...
    // Supports continuous digits or separated by '.', '-', or spaces (e.g. 0912.345.678)
    Regex::new(r"(?:\+84[.\-\s]?|\b0)[35789](?:[.\-\s]?\d){8}\b").expect("Valid regex for phone numbers")
});

static RE_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:password|secret|token|api[_-]?key|bearer)\s*[:=]\s*['\x22]?([a-zA-Z0-9_\-\.]{8,})['\x22]?")
        .expect("Valid regex for secrets")
});

static RE_OPENAI_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"sk-[a-zA-Z0-9]{20,}").expect("Valid regex for OpenAI API keys"));

static RE_NAME_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:chu\s+tk|chủ\s+tk|nguoi\s+gui|người\s+gửi|nguoi\s+nhan|người\s+nhận|khach\s+hang|khách\s+hàng|chu\s+tai\s+khoan|chủ\s+tài\s+khoản|chu\s+so\s+huu|chủ\s+sở\s+hữu|ten\s+kh|tên\s+kh|doi\s+tac|đối\s+tác|ben\s+huong|bên\s+hưởng|ben\s+gui|bên\s+gửi|nguoi\s+nop|người\s+nộp)[:\s]+([A-Za-zÀ-ỹ\s]{2,30})\b")
        .expect("Valid regex for name prefix")
});

static RE_FULL_NAME: LazyLock<Regex> = LazyLock::new(|| {
    // Standard Vietnamese surnames followed by 1 to 3 given/middle names
    Regex::new(r"(?i)\b(Nguyen|Tran|Le|Pham|Hoang|Huynh|Phan|Vu|Vo|Dang|Bui|Do|Ho|Ngo|Duong|Ly|Dinh|Dao|Doan|Mai|Trinh|Cao|Luong|Truong|Ha|Ta|Chu|Phung|Dinh|Van|Trieu|Quach|Bach|Luu|Tong|Kieu|Lam|Diep|Nguyễn|Trần|Lê|Phạm|Hoàng|Huỳnh|Phan|Vũ|Võ|Đặng|Bùi|Đỗ|Hồ|Ngô|Dương|Lý|Đinh|Đào|Đoàn|Mai|Trịnh|Cao|Lương|Trương|Hà|Tạ|Chu|Phùng|Đình|Văn|Triệu|Quách|Bạch|Lưu|Tống|Kiều|Lâm|Diệp)\s+([A-Za-zÀ-ỹ]+(?:\s+[A-Za-zÀ-ỹ]+){1,3})\b")
        .expect("Valid regex for full names")
});

/// Determines if a string represents an enterprise or legal entity rather than a natural person.
pub fn is_legal_entity(text: &str) -> bool {
    let s = text.to_uppercase();
    s.contains("CONG TY")
        || s.contains("CÔNG TY")
        || s.contains("TNHH")
        || s.contains(" CỔ PHẦN")
        || s.contains(" CO PHAN")
        || s.contains(" CP ")
        || s.ends_with(" CP")
        || s.starts_with("CP ")
        || s.contains("DOANH NGHIEP")
        || s.contains("DOANH NGHIỆP")
        || s.contains("DNTN")
        || s.contains("CHI NHANH")
        || s.contains("CHI NHÁNH")
        || s.contains("NGAN HANG")
        || s.contains("NGÂN HÀNG")
        || s.contains("UBND")
        || s.contains("TRUONG HOC")
        || s.contains("TRƯỜNG HỌC")
        || s.contains("TRUONG DAI HOC")
        || s.contains("TRƯỜNG ĐẠI HỌC")
        || s.contains("TRUONG CAO DANG")
        || s.contains("TRƯỜNG CAO ĐẲNG")
        || s.contains("TRUONG THPT")
        || s.contains("TRƯỜNG THPT")
        || s.contains("TRUONG THCS")
        || s.contains("TRƯỜNG THCS")
        || s.contains("TRUONG TIEU HOC")
        || s.contains("TRƯỜNG TIỂU HỌC")
        || s.contains("TRUONG MAM NON")
        || s.contains("TRƯỜNG MẦM NON")
        || s.contains("BENH VIEN")
        || s.contains("BỆNH VIỆN")
        || s.contains("TAP DOAN")
        || s.contains("TẬP ĐOÀN")
        || s.contains("VIETCOMBANK")
        || s.contains("TECHCOMBANK")
        || s.contains("BIDV")
        || s.contains("AGRIBANK")
        || s.contains("VIETINBANK")
        || s.contains("MBBANK")
        || s.contains("JSC")
        || s.contains("LTD")
        || s.contains("CORP")
        || s.contains("INC")
}

/// Sanitizes a counterparty name: preserves corporate legal entities, masks natural persons.
pub fn sanitize_counterparty_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if is_legal_entity(trimmed) {
        trimmed.to_string()
    } else {
        "[REDACTED_NAME]".to_string()
    }
}

/// Redacts sensitive PII from text for on-premise Decree 13 audit and log safety.
pub fn sanitize_pii(text: &str) -> String {
    let mut sanitized = text.to_string();

    // 1. Redact API keys and secrets
    sanitized = RE_SECRET
        .replace_all(&sanitized, |caps: &regex::Captures| {
            let full = &caps[0];
            let secret = &caps[1];
            full.replace(secret, "[REDACTED_SECRET]")
        })
        .to_string();

    sanitized = RE_OPENAI_KEY
        .replace_all(&sanitized, "[REDACTED_API_KEY]")
        .to_string();

    // 2. Redact 12-digit CCCD
    sanitized = RE_CCCD
        .replace_all(&sanitized, "[REDACTED_CCCD]")
        .to_string();

    // 3. Redact Bank Account numbers when identified by prefix keyword
    sanitized = RE_ACCOUNT
        .replace_all(&sanitized, |caps: &regex::Captures| {
            let full = &caps[0];
            let acc = &caps[1];
            full.replace(acc, "[REDACTED_ACCOUNT]")
        })
        .to_string();

    // 4. Redact Vietnamese 10-digit mobile numbers
    sanitized = RE_PHONE
        .replace_all(&sanitized, "[REDACTED_PHONE]")
        .to_string();

    // 5. Redact Vietnamese personal full names
    // A. Prefix-guided name masking (chu tk: Nguyen Van A)
    sanitized = RE_NAME_PREFIX
        .replace_all(&sanitized, |caps: &regex::Captures| {
            let full = &caps[0];
            let name_candidate = caps[1].trim();
            if !is_legal_entity(name_candidate) {
                full.replace(name_candidate, "[REDACTED_NAME]")
            } else {
                full.to_string()
            }
        })
        .to_string();

    // B. Standalone Vietnamese personal names with standard family names
    let mut result = String::new();
    let mut last_idx = 0;
    for mat in RE_FULL_NAME.find_iter(&sanitized) {
        result.push_str(&sanitized[last_idx..mat.start()]);
        let prefix = &sanitized[..mat.start()];
        let upper_prefix = prefix.trim_end().to_uppercase();
        let is_after_corporate = upper_prefix.ends_with("CONG TY")
            || upper_prefix.ends_with("CÔNG TY")
            || upper_prefix.ends_with("TNHH")
            || upper_prefix.ends_with("DOANH NGHIEP")
            || upper_prefix.ends_with("DOANH NGHIỆP")
            || upper_prefix.ends_with("CP")
            || upper_prefix.ends_with("CO PHAN")
            || upper_prefix.ends_with("CỔ PHẦN");

        let candidate = mat.as_str();
        if is_after_corporate || is_legal_entity(candidate) {
            result.push_str(candidate);
        } else {
            result.push_str("[REDACTED_NAME]");
        }
        last_idx = mat.end();
    }
    result.push_str(&sanitized[last_idx..]);
    sanitized = result;

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_cccd() {
        let input = "Khach hang Nguyen Van A, CCCD: 001095012345 yeu cau chuyen khoan";
        let output = sanitize_pii(input);
        assert!(!output.contains("001095012345"));
        assert!(output.contains("[REDACTED_CCCD]"));
    }

    #[test]
    fn test_sanitize_account() {
        let input = "Chuyen tien den so TK: 19034567890123 tai Techcombank";
        let output = sanitize_pii(input);
        assert!(!output.contains("19034567890123"));
        assert!(output.contains("[REDACTED_ACCOUNT]"));
    }

    #[test]
    fn test_sanitize_phone_numbers() {
        // Standard 10-digit mobile prefixes
        let input1 = "Lien he voi khach qua so 0912345678 hoac 0387654321";
        let output1 = sanitize_pii(input1);
        assert!(!output1.contains("0912345678"));
        assert!(!output1.contains("0387654321"));
        assert_eq!(output1.matches("[REDACTED_PHONE]").count(), 2);

        // International format +84
        let input2 = "SDT quoc te: +84987654321";
        let output2 = sanitize_pii(input2);
        assert!(!output2.contains("+84987654321"));
        assert!(output2.contains("[REDACTED_PHONE]"));
    }

    #[test]
    fn test_sanitize_personal_names() {
        let input1 = "Chuyen khoan chu TK: Nguyen Van An so tien 5000000";
        let output1 = sanitize_pii(input1);
        assert!(!output1.contains("Nguyen Van An"));
        assert!(output1.contains("[REDACTED_NAME]"));

        let input2 = "Khach hang Tran Thi Bich nop tien tai quay";
        let output2 = sanitize_pii(input2);
        assert!(!output2.contains("Tran Thi Bich"));
        assert!(output2.contains("[REDACTED_NAME]"));
    }

    #[test]
    fn test_preserve_corporate_legal_entities() {
        let corp1 = "CONG TY TNHH THEP VIET NHAT";
        let out1 = sanitize_counterparty_name(corp1);
        assert_eq!(out1, corp1);

        let corp2 = "CONG TY CP THUONG MAI ABC";
        let out2 = sanitize_counterparty_name(corp2);
        assert_eq!(out2, corp2);

        let person = "Nguyen Van A";
        let out_p = sanitize_counterparty_name(person);
        assert_eq!(out_p, "[REDACTED_NAME]");
    }

    #[test]
    fn test_preserve_transaction_amounts() {
        // Plain transaction amounts without account prefix must NOT be redacted
        let input1 = "So du hien tai: 1450230000 VND";
        let output1 = sanitize_pii(input1);
        assert!(output1.contains("1450230000"));
        assert!(!output1.contains("[REDACTED_ACCOUNT]"));
        assert!(!output1.contains("[REDACTED_PHONE]"));

        let input2 = "Chuyen khoan thanh toan hoa don 15000000";
        let output2 = sanitize_pii(input2);
        assert!(output2.contains("15000000"));
        assert!(!output2.contains("[REDACTED_ACCOUNT]"));
        assert!(!output2.contains("[REDACTED_PHONE]"));
    }

    #[test]
    fn test_sanitize_secret() {
        let input = "Bearer token=abc123xyz456secretkey for authentication";
        let output = sanitize_pii(input);
        assert!(!output.contains("abc123xyz456secretkey"));
        assert!(output.contains("[REDACTED_SECRET]"));
    }

    #[test]
    fn test_sanitize_openai_key() {
        let input = "API Key: sk-abcdefghijklmnopqrstuvwx1234567890";
        let output = sanitize_pii(input);
        assert!(!output.contains("sk-abcdefghijklmnopqrstuvwx1234567890"));
        assert!(output.contains("[REDACTED_API_KEY]"));
    }

    #[test]
    fn test_sanitize_phone_numbers_formatted() {
        let input = "Lien he: 0912.345.678 hoac 0987-654-321 hoac +84 901 234 567";
        let output = sanitize_pii(input);
        assert!(!output.contains("0912.345.678"));
        assert!(!output.contains("0987-654-321"));
        assert!(!output.contains("+84 901 234 567"));
        assert_eq!(output.matches("[REDACTED_PHONE]").count(), 3);
    }

    #[test]
    fn test_preserve_corporate_names_in_narration() {
        let input = "Thanh toan tien hang cho CONG TY TNHH NGUYEN PHAT so tien 20000000";
        let output = sanitize_pii(input);
        assert!(output.contains("CONG TY TNHH NGUYEN PHAT"));
        assert!(!output.contains("[REDACTED_NAME]"));

        let input_person = "Thanh toan tien luong cho Nguyen Van An thang 8";
        let output_person = sanitize_pii(input_person);
        assert!(!output_person.contains("Nguyen Van An"));
        assert!(output_person.contains("[REDACTED_NAME]"));
    }

    #[test]
    fn test_refine_truong_corporate_vs_personal() {
        // Institutional entities must be preserved
        let uni = "TRUONG DAI HOC BACH KHOA";
        assert_eq!(sanitize_counterparty_name(uni), uni);
        let school = "TRƯỜNG HỌC QUỐC TẾ";
        assert_eq!(sanitize_counterparty_name(school), school);

        // Individuals named Truong / Trương must be redacted
        let person1 = "Truong Van Nam";
        assert_eq!(sanitize_counterparty_name(person1), "[REDACTED_NAME]");
        let person2 = "Trương Gia Bình";
        assert_eq!(sanitize_counterparty_name(person2), "[REDACTED_NAME]");

        let narration = "Chuyen tien hoc phi cho Truong Van Nam lop 12";
        let out = sanitize_pii(narration);
        assert!(!out.contains("Truong Van Nam"));
        assert!(out.contains("[REDACTED_NAME]"));
    }
}
