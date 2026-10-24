//! Comprehensive tests for HITL Quarantine Governance and Decree 13 PII Sanitizer.

use liva_compliance::maker_checker::{ApprovalAction, ApprovalDecision};
use liva_compliance::quarantine::{
    HitlQuarantineGovernance, QuarantineRiskTier, QuarantineStatus,
};
use liva_compliance::sanitizer::sanitize_decree13_pii;

#[test]
fn test_quarantine_risk_classification_tiers() {
    // 1. Low: < 10M VND
    assert_eq!(
        HitlQuarantineGovernance::classify_risk(8_000_000, false),
        QuarantineRiskTier::Low
    );

    // 2. Medium: 10M - 100M VND
    assert_eq!(
        HitlQuarantineGovernance::classify_risk(45_000_000, false),
        QuarantineRiskTier::Medium
    );

    // 3. High: 100M - 500M VND
    assert_eq!(
        HitlQuarantineGovernance::classify_risk(250_000_000, false),
        QuarantineRiskTier::High
    );

    // 4. Critical: >= 500M VND
    assert_eq!(
        HitlQuarantineGovernance::classify_risk(600_000_000, false),
        QuarantineRiskTier::Critical
    );

    // 5. Critical: >= 400M VND statutory Decision 11
    assert_eq!(
        HitlQuarantineGovernance::classify_risk(420_000_000, false),
        QuarantineRiskTier::Critical
    );

    // 6. Critical: active AML alert regardless of amount
    assert_eq!(
        HitlQuarantineGovernance::classify_risk(5_000_000, true),
        QuarantineRiskTier::Critical
    );
}

#[test]
fn test_quarantine_governance_workflow_and_constant_time_token() {
    let mut gov = HitlQuarantineGovernance::with_default_policy();
    let now = 1726300000;

    // Place a medium-risk item in quarantine (50M VND)
    let item = gov
        .quarantine_item(
            "Q-ITEM-01",
            "TX-BANK-001",
            50_000_000,
            "Chenh lech so du chua khop",
            false,
            "MAKER_JANE",
            "OPERATOR_MAKER",
            now,
        )
        .expect("Must quarantine item");

    assert_eq!(item.risk_tier, QuarantineRiskTier::Medium);
    assert_eq!(item.status, QuarantineStatus::PendingReview);
    let token = item.current_token.expect("Must have single-use token");

    // Verify token validity
    assert!(gov.is_token_valid("Q-ITEM-01", &token, now + 10));
    assert!(!gov.is_token_valid("Q-ITEM-01", "WRONG_TOKEN_VALUE", now + 10));
    assert!(!gov.is_token_valid("Q-ITEM-01", &token, now + 1000)); // Expired

    // Checker approves item
    let action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_DAVE".to_string(),
        checker_role: "OPERATOR_CHECKER".to_string(),
        token,
        notes: "Giai trinh chenh lech da duoc chap thuan".to_string(),
    };

    let released_item = gov
        .review_quarantined_item("Q-ITEM-01", action, now + 20)
        .expect("Must approve and release");

    assert_eq!(released_item.status, QuarantineStatus::Released);
    assert!(released_item.current_token.is_none());
}

#[test]
fn test_decree13_pii_sanitization_and_corporate_preservation() {
    // Personal citizen ID (CCCD: 12 digits starting with 0)
    let cccd_text = "Thanh toan tien hang cho CCCD 031092008451 tai VCB";
    let sanitized_cccd = sanitize_decree13_pii(cccd_text);
    assert!(sanitized_cccd.contains("[REDACTED_CCCD]"));
    assert!(!sanitized_cccd.contains("031092008451"));

    // Personal mobile phone number
    let phone_text = "Lien he anh Nam 0908123456 de nhan hang";
    let sanitized_phone = sanitize_decree13_pii(phone_text);
    assert!(sanitized_phone.contains("[REDACTED_PHONE]"));
    assert!(!sanitized_phone.contains("0908123456"));

    // Bank account with keyword
    let acc_text = "Chuyen khoan vao stk 0071000982123 ngan hang Vietcombank";
    let sanitized_acc = sanitize_decree13_pii(acc_text);
    assert!(sanitized_acc.contains("[REDACTED_ACCOUNT]"));
    assert!(!sanitized_acc.contains("0071000982123"));

    // Natural personal names
    let name_text = "NGUYEN VAN BINH";
    assert_eq!(sanitize_decree13_pii(name_text), "[REDACTED_NAME]");

    // Corporate entity preservation: MUST NOT be masked
    let corp_name_1 = "CONG TY TNHH PHAT TRIEN CONG NGHE NGUYEN HOANG";
    assert_eq!(sanitize_decree13_pii(corp_name_1), corp_name_1);

    let corp_name_2 = "NGAN HANG TMCP NGOAI THUONG VIET NAM";
    assert_eq!(sanitize_decree13_pii(corp_name_2), corp_name_2);

    let corp_name_3 = "TRUONG DAI HOC BACH KHOA";
    assert_eq!(sanitize_decree13_pii(corp_name_3), corp_name_3);
}
