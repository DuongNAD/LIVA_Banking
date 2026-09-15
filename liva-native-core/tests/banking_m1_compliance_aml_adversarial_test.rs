//! Empirical Adversarial Challenge Test Suite: Compliance, AML/CTF, and Zero-Egress
//!
//! Milestone M1 Verification for Challenger 2.
//! Covers:
//! 1. AML High-Value Threshold (>= 400M VND, 399.999.999 VND boundary, critical >= 1B VND, 500B VND treasury).
//! 2. AML Structuring / Smurfing (split transactions just below 400M, sum >= 800M, 72h window, sub-50M micro-structuring).
//! 3. AML Sudden Velocity Surge (>300% historical baseline, 0 baseline, and MCP tool parameter inspection).
//! 4. AML Pass-Through Account (>95% dispersal in <15m, time boundary, percentage boundary, minimum threshold).
//! 5. Suspicious Transaction Report (STR) risk scoring and statutory escalation actions.
//! 6. Decree 13 PII Sanitizer: Formatted Vietnamese phone numbers (dots, hyphens, spaces, +84 international).
//! 7. Decree 13 PII Sanitizer: Personal names vs legal entities (preservation of corporate names, masking of individuals).
//! 8. Zero Data Egress: Active interceptor, socket validator, environment proxy detection, and air-gapped assertion.
//! 9. Direct in-process handler `handle_compliance_aml_screen` verification.
//! 10. End-to-End MCP Tool `compliance_aml_screen` integration under adversarial scenarios.

use liva_native_core::banking::compliance::aml::{
    AmlAlert, AmlRuleCode, AmlScreeningEngine, AmlSeverity,
    TransactionScreeningItem,
};
use liva_native_core::banking::compliance::sanitizer::{
    is_legal_entity, sanitize_counterparty_name, sanitize_pii,
};
use liva_native_core::banking::compliance::security::{
    EgressTrafficTracker, ZeroEgressNetfilter, is_egress_permitted,
    verify_statement_processing_zero_egress, verify_zero_egress_from,
};
use liva_native_core::banking::mcp::{
    ComplianceAmlScreenArgs, ScreeningTxInput, handle_compliance_aml_screen,
};
use liva_native_core::mcp::protocol::{CallToolRequest, ToolContent};
use liva_native_core::mcp::server::NativeMcpServer;
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

// ===========================================================================
// Helpers
// ===========================================================================

fn extract_text(content: &[ToolContent]) -> &str {
    match content.first() {
        Some(ToolContent::Text { text }) => text.as_str(),
        other => panic!("Expected ToolContent::Text, got: {:?}", other),
    }
}

// ===========================================================================
// 1. AML High-Value Transaction Threshold (Decision 11/2023/QĐ-TTg)
// ===========================================================================

#[test]
fn test_adversarial_aml_high_value_boundaries() {
    let base_ts = 1726300000;

    let txs = vec![
        // Exactly at threshold (400M)
        TransactionScreeningItem {
            tx_id: "HV-EXACT-400M".to_string(),
            account_number: Some("00110001".to_string()),
            counterparty_name: Some("Nguyen Van A".to_string()),
            counterparty_account: None,
            amount_vnd: 400_000_000,
            timestamp: base_ts,
            narration: "Thanh toan dung 400M".to_string(),
            is_credit: Some(true),
        },
        // Just 1 VND below threshold (399,999,999) -> MUST NOT trigger AML_HIGH_VALUE
        TransactionScreeningItem {
            tx_id: "HV-BELOW-1VND".to_string(),
            account_number: Some("00110001".to_string()),
            counterparty_name: Some("Tran Thi B".to_string()),
            counterparty_account: None,
            amount_vnd: 399_999_999,
            timestamp: base_ts + 10,
            narration: "Thanh toan sat nut duoi 400M".to_string(),
            is_credit: Some(true),
        },
        // Just 1 VND above threshold (400,000,001) -> MUST trigger AML_HIGH_VALUE
        TransactionScreeningItem {
            tx_id: "HV-ABOVE-1VND".to_string(),
            account_number: Some("00110001".to_string()),
            counterparty_name: Some("Le Van C".to_string()),
            counterparty_account: None,
            amount_vnd: 400_000_001,
            timestamp: base_ts + 20,
            narration: "Thanh toan vuot 1 dong".to_string(),
            is_credit: Some(true),
        },
        // Critical threshold boundary (1,000,000,000) -> MUST trigger CRITICAL severity
        TransactionScreeningItem {
            tx_id: "HV-CRITICAL-1B".to_string(),
            account_number: Some("00110001".to_string()),
            counterparty_name: Some("Pham Van D".to_string()),
            counterparty_account: None,
            amount_vnd: 1_000_000_000,
            timestamp: base_ts + 30,
            narration: "Thanh toan 1 ty VND".to_string(),
            is_credit: Some(true),
        },
        // Massive Treasury scale: 500 Billion VND
        TransactionScreeningItem {
            tx_id: "HV-TREASURY-500B".to_string(),
            account_number: Some("00110001".to_string()),
            counterparty_name: Some("CONG TY CP DAU TU".to_string()),
            counterparty_account: None,
            amount_vnd: 500_000_000_000,
            timestamp: base_ts + 40,
            narration: "Chuyen tien kho bac 500 ty".to_string(),
            is_credit: Some(true),
        },
    ];

    let alerts = AmlScreeningEngine::screen_transactions(&txs, None);

    let hv_alerts: Vec<&AmlAlert> = alerts
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlHighValue)
        .collect();

    // 4 out of 5 transactions must trigger high value (HV-BELOW-1VND must NOT trigger)
    assert_eq!(hv_alerts.len(), 4, "Only 4 transactions exceed or equal 400M VND");

    let exact = hv_alerts.iter().find(|a| a.tx_id == "HV-EXACT-400M").unwrap();
    assert_eq!(exact.severity, AmlSeverity::High);

    let below = hv_alerts.iter().find(|a| a.tx_id == "HV-BELOW-1VND");
    assert!(below.is_none(), "399,999,999 VND must NOT trigger high value alert");

    let above = hv_alerts.iter().find(|a| a.tx_id == "HV-ABOVE-1VND").unwrap();
    assert_eq!(above.severity, AmlSeverity::High);

    let crit = hv_alerts.iter().find(|a| a.tx_id == "HV-CRITICAL-1B").unwrap();
    assert_eq!(crit.severity, AmlSeverity::Critical, ">= 1 Billion must be CRITICAL severity");

    let treasury = hv_alerts.iter().find(|a| a.tx_id == "HV-TREASURY-500B").unwrap();
    assert_eq!(treasury.severity, AmlSeverity::Critical);
    assert_eq!(treasury.amount_vnd, 500_000_000_000);
}

// ===========================================================================
// 2. AML Structuring / Smurfing Patterns
// ===========================================================================

#[test]
fn test_adversarial_aml_structuring_smurfing_patterns() {
    let base_ts = 1726300000;

    // Pattern A: Classic structuring (3 txs just under 400M: 390M, 390M, 390M within 48 hours = 1.17B VND)
    let txs_classic = vec![
        TransactionScreeningItem {
            tx_id: "SMURF-A1".to_string(),
            account_number: Some("ACCT-01".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 390_000_000,
            timestamp: base_ts,
            narration: "Split payment 1".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-A2".to_string(),
            account_number: Some("ACCT-01".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 390_000_000,
            timestamp: base_ts + 3600 * 20, // 20 hours later
            narration: "Split payment 2".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-A3".to_string(),
            account_number: Some("ACCT-01".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 390_000_000,
            timestamp: base_ts + 3600 * 40, // 40 hours later
            narration: "Split payment 3".to_string(),
            is_credit: Some(true),
        },
    ];

    let alerts_a = AmlScreeningEngine::screen_transactions(&txs_classic, None);
    let structuring_a: Vec<&AmlAlert> = alerts_a
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlStructuring)
        .collect();

    assert_eq!(structuring_a.len(), 3, "All 3 split transactions must be flagged for structuring");
    assert!(structuring_a.iter().all(|a| a.severity == AmlSeverity::Critical));

    // Pattern B: 4 transactions of 250M VND within 24 hours (sum = 1,000,000,000 VND >= 800M)
    let txs_quad = vec![
        TransactionScreeningItem {
            tx_id: "SMURF-B1".to_string(),
            account_number: Some("ACCT-02".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 250_000_000,
            timestamp: base_ts,
            narration: "Batch 1".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-B2".to_string(),
            account_number: Some("ACCT-02".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 250_000_000,
            timestamp: base_ts + 3600 * 4,
            narration: "Batch 2".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-B3".to_string(),
            account_number: Some("ACCT-02".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 250_000_000,
            timestamp: base_ts + 3600 * 8,
            narration: "Batch 3".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-B4".to_string(),
            account_number: Some("ACCT-02".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 250_000_000,
            timestamp: base_ts + 3600 * 12,
            narration: "Batch 4".to_string(),
            is_credit: Some(true),
        },
    ];

    let alerts_b = AmlScreeningEngine::screen_transactions(&txs_quad, None);
    let structuring_b: Vec<&AmlAlert> = alerts_b
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlStructuring)
        .collect();
    assert_eq!(structuring_b.len(), 4, "All 4 transactions must be flagged");

    // Pattern C: Boundary - Only 2 transactions of 399M VND (sum = 798M VND < 800M AND count < 3)
    let txs_under_boundary = vec![
        TransactionScreeningItem {
            tx_id: "SMURF-C1".to_string(),
            account_number: Some("ACCT-03".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 399_000_000,
            timestamp: base_ts,
            narration: "Part 1".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-C2".to_string(),
            account_number: Some("ACCT-03".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 399_000_000,
            timestamp: base_ts + 3600,
            narration: "Part 2".to_string(),
            is_credit: Some(true),
        },
    ];

    let alerts_c = AmlScreeningEngine::screen_transactions(&txs_under_boundary, None);
    let structuring_c: Vec<&AmlAlert> = alerts_c
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlStructuring)
        .collect();
    assert_eq!(structuring_c.len(), 0, "2 transactions do not meet count threshold (>=3)");

    // Pattern D: Time window expiry - 3 transactions of 350M, but spaced 73 hours apart
    let txs_expired_window = vec![
        TransactionScreeningItem {
            tx_id: "SMURF-D1".to_string(),
            account_number: Some("ACCT-04".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 350_000_000,
            timestamp: base_ts,
            narration: "Day 0".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-D2".to_string(),
            account_number: Some("ACCT-04".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 350_000_000,
            timestamp: base_ts + 3600 * 36, // 36 hours later
            narration: "Day 1.5".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SMURF-D3".to_string(),
            account_number: Some("ACCT-04".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 350_000_000,
            timestamp: base_ts + 3600 * 73, // 73 hours later (> 72 hours from D1)
            narration: "Day 3 + 1h".to_string(),
            is_credit: Some(true),
        },
    ];

    let alerts_d = AmlScreeningEngine::screen_transactions(&txs_expired_window, None);
    let structuring_d: Vec<&AmlAlert> = alerts_d
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlStructuring)
        .collect();
    assert_eq!(
        structuring_d.len(),
        0,
        "Transactions spanning > 72 hours must not trigger structuring (no 3 txs within 72h reach 800M)"
    );

    // Pattern E: Micro-structuring limitation test (< 50M each)
    let txs_micro = (0..20)
        .map(|i| TransactionScreeningItem {
            tx_id: format!("MICRO-{i}"),
            account_number: Some("ACCT-05".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 40_000_000, // 40M < 50M threshold
            timestamp: base_ts + i * 1800,
            narration: "Micro tx".to_string(),
            is_credit: Some(true),
        })
        .collect::<Vec<_>>();

    let alerts_e = AmlScreeningEngine::screen_transactions(&txs_micro, None);
    let structuring_e: Vec<&AmlAlert> = alerts_e
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlStructuring)
        .collect();
    // In current implementation, txs < 50M are filtered out from structuring to prevent false positives on payroll
    assert_eq!(
        structuring_e.len(),
        0,
        "Micro-transactions < 50M VND are filtered out of statutory structuring by design"
    );
}

// ===========================================================================
// 3. AML Sudden Velocity Surge (> 300% historical baseline)
// ===========================================================================

#[test]
fn test_adversarial_aml_velocity_surge() {
    let base_ts = 1726300000;

    // A. 500M volume against 100M historical baseline -> 500M > 3 * 100M = 300M (> 300% surge)
    let txs_surge = vec![
        TransactionScreeningItem {
            tx_id: "SURGE-TX-1".to_string(),
            account_number: Some("ACCT-SURGE".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 300_000_000,
            timestamp: base_ts,
            narration: "Surge 1".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "SURGE-TX-2".to_string(),
            account_number: Some("ACCT-SURGE".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 200_000_000,
            timestamp: base_ts + 60,
            narration: "Surge 2".to_string(),
            is_credit: Some(true),
        },
    ];

    let alerts = AmlScreeningEngine::screen_transactions(&txs_surge, Some(100_000_000));
    let surge_alerts: Vec<&AmlAlert> = alerts
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlVelocitySurge)
        .collect();

    assert_eq!(surge_alerts.len(), 1, "Velocity surge must trigger on >300% volume increase");
    assert_eq!(surge_alerts[0].severity, AmlSeverity::High);
    assert_eq!(surge_alerts[0].amount_vnd, 500_000_000);

    // B. Exactly 300M volume against 100M baseline -> 300M is NOT > 3 * 100M -> NO surge alert
    let txs_exact_300m = vec![TransactionScreeningItem {
        tx_id: "SURGE-EXACT-3X".to_string(),
        account_number: Some("ACCT-SURGE".to_string()),
        counterparty_name: None,
        counterparty_account: None,
        amount_vnd: 300_000_000,
        timestamp: base_ts,
        narration: "Exact 300M".to_string(),
        is_credit: Some(true),
    }];

    let alerts_exact = AmlScreeningEngine::screen_transactions(&txs_exact_300m, Some(100_000_000));
    let surge_exact: Vec<&AmlAlert> = alerts_exact
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlVelocitySurge)
        .collect();
    assert_eq!(surge_exact.len(), 0, "Exactly 300% does not exceed >300% threshold");

    // C. Zero historical baseline must not cause division by zero or false positive
    let alerts_zero_base = AmlScreeningEngine::screen_transactions(&txs_surge, Some(0));
    let surge_zero: Vec<&AmlAlert> = alerts_zero_base
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlVelocitySurge)
        .collect();
    assert_eq!(surge_zero.len(), 0, "Zero baseline must be ignored safely");

    // D. None baseline must not trigger velocity surge
    let alerts_none_base = AmlScreeningEngine::screen_transactions(&txs_surge, None);
    let surge_none: Vec<&AmlAlert> = alerts_none_base
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlVelocitySurge)
        .collect();
    assert_eq!(surge_none.len(), 0, "No baseline provided means surge cannot be calculated");
}

// ===========================================================================
// 4. AML Pass-Through / Transit Account Patterns (>95% dispersal in <15m)
// ===========================================================================

#[test]
fn test_adversarial_aml_pass_through_patterns() {
    let base_ts = 1726300000;

    // Case 1: Inflow 500M VND, Outflow 480M VND (96% > 95%) 8 minutes later (480s < 900s) -> TRIGGER
    let txs_valid_pt = vec![
        TransactionScreeningItem {
            tx_id: "IN-01".to_string(),
            account_number: Some("TRANSIT-01".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 500_000_000,
            timestamp: base_ts,
            narration: "Nhan chuyen khoan".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "OUT-01".to_string(),
            account_number: Some("TRANSIT-01".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 480_000_000, // 96%
            timestamp: base_ts + 480, // 8 minutes
            narration: "Chuyen ngay lap tuc".to_string(),
            is_credit: Some(false),
        },
    ];

    let alerts1 = AmlScreeningEngine::screen_transactions(&txs_valid_pt, None);
    let pt_alerts1: Vec<&AmlAlert> = alerts1
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlPassThrough)
        .collect();
    assert_eq!(pt_alerts1.len(), 1);
    assert_eq!(pt_alerts1[0].tx_id, "OUT-01");
    assert_eq!(pt_alerts1[0].severity, AmlSeverity::Critical);

    // Case 2: Time boundary - Outflow occurs 15 minutes and 1 second later (901s > 900s) -> NO TRIGGER
    let txs_time_expired = vec![
        TransactionScreeningItem {
            tx_id: "IN-02".to_string(),
            account_number: Some("TRANSIT-02".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 500_000_000,
            timestamp: base_ts,
            narration: "Nhan tien".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "OUT-02".to_string(),
            account_number: Some("TRANSIT-02".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 490_000_000,
            timestamp: base_ts + 901, // 15m 1s > 900s
            narration: "Chuyen sau 15 phut".to_string(),
            is_credit: Some(false),
        },
    ];

    let alerts2 = AmlScreeningEngine::screen_transactions(&txs_time_expired, None);
    let pt_alerts2: Vec<&AmlAlert> = alerts2
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlPassThrough)
        .collect();
    assert_eq!(pt_alerts2.len(), 0, "Outflow after 15m window must not trigger pass-through alert");

    // Case 3: Percentage boundary - Outflow is 94.9% (below 95%) in 5 minutes -> NO TRIGGER
    // 500M * 95% = 475M. 474M is 94.8%
    let txs_under_pct = vec![
        TransactionScreeningItem {
            tx_id: "IN-03".to_string(),
            account_number: Some("TRANSIT-03".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 500_000_000,
            timestamp: base_ts,
            narration: "Nhan tien".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "OUT-03".to_string(),
            account_number: Some("TRANSIT-03".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 474_000_000, // 94.8% < 95%
            timestamp: base_ts + 300,
            narration: "Chuyen 94.8%".to_string(),
            is_credit: Some(false),
        },
    ];

    let alerts3 = AmlScreeningEngine::screen_transactions(&txs_under_pct, None);
    let pt_alerts3: Vec<&AmlAlert> = alerts3
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlPassThrough)
        .collect();
    assert_eq!(pt_alerts3.len(), 0, "< 95% dispersal must not trigger pass-through alert");

    // Case 4: Inflow below minimum threshold (< 100M VND)
    let txs_small_inflow = vec![
        TransactionScreeningItem {
            tx_id: "IN-04".to_string(),
            account_number: Some("TRANSIT-04".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 80_000_000, // < 100M threshold
            timestamp: base_ts,
            narration: "Tien nho".to_string(),
            is_credit: Some(true),
        },
        TransactionScreeningItem {
            tx_id: "OUT-04".to_string(),
            account_number: Some("TRANSIT-04".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 79_000_000, // 98.75%
            timestamp: base_ts + 120,
            narration: "Chuyen het".to_string(),
            is_credit: Some(false),
        },
    ];

    let alerts4 = AmlScreeningEngine::screen_transactions(&txs_small_inflow, None);
    let pt_alerts4: Vec<&AmlAlert> = alerts4
        .iter()
        .filter(|a| a.rule_code == AmlRuleCode::AmlPassThrough)
        .collect();
    assert_eq!(pt_alerts4.len(), 0, "Inflow < 100M VND must not trigger pass-through alert");
}

// ===========================================================================
// 5. Suspicious Transaction Report (STR) Generator
// ===========================================================================

#[test]
fn test_adversarial_str_generation_and_scoring() {
    // 1 Critical + 1 High alert -> Score = 40 + 20 = 60
    let alerts = vec![
        AmlAlert {
            tx_id: "TX-1".to_string(),
            rule_code: AmlRuleCode::AmlPassThrough,
            severity: AmlSeverity::Critical,
            description: "Transit alert".to_string(),
            amount_vnd: 400_000_000,
            timestamp: 1726300000,
        },
        AmlAlert {
            tx_id: "TX-2".to_string(),
            rule_code: AmlRuleCode::AmlHighValue,
            severity: AmlSeverity::High,
            description: "High value".to_string(),
            amount_vnd: 450_000_000,
            timestamp: 1726300100,
        },
    ];

    let report = AmlScreeningEngine::generate_str(alerts, 10);
    assert_eq!(report.total_screened, 10);
    assert_eq!(report.alerts_count, 2);
    assert_eq!(report.risk_score, 60);
    // Any Critical alert mandates FILE_STR_WITH_SBV
    assert_eq!(report.recommended_action, "FILE_STR_WITH_SBV");
    assert!(report.report_id.starts_with("STR-"));
    assert_eq!(report.legal_references.len(), 3);
}

// ===========================================================================
// 6. Decree 13 PII Sanitizer: Formatted Vietnamese Phone Numbers
// ===========================================================================

#[test]
fn test_adversarial_phone_number_formats() {
    // Exact requested phone numbers
    let p1 = "So dien thoai khach hang: 0901.234.567 yeu cau tra cuu";
    let s1 = sanitize_pii(p1);
    assert!(!s1.contains("0901.234.567"), "0901.234.567 must be redacted");
    assert!(s1.contains("[REDACTED_PHONE]"));

    let p2 = "Lien he voi ong A so 098-765-4321 de xac nhan";
    let s2 = sanitize_pii(p2);
    assert!(!s2.contains("098-765-4321"), "098-765-4321 must be redacted");
    assert!(s2.contains("[REDACTED_PHONE]"));

    let p3 = "Hotline ho tro quoc te: +84 912 345 678 24/7";
    let s3 = sanitize_pii(p3);
    assert!(!s3.contains("+84 912 345 678"), "+84 912 345 678 must be redacted");
    assert!(s3.contains("[REDACTED_PHONE]"));

    // Multi-format combinations in single text
    let mixed = "Goi 0901.234.567 hoac 098-765-4321 hoac +84 912 345 678 hoac 038 123 4567";
    let s_mixed = sanitize_pii(mixed);
    assert!(!s_mixed.contains("0901.234.567"));
    assert!(!s_mixed.contains("098-765-4321"));
    assert!(!s_mixed.contains("+84 912 345 678"));
    assert!(!s_mixed.contains("038 123 4567"));
    assert_eq!(s_mixed.matches("[REDACTED_PHONE]").count(), 4);

    // Negative case: Standalone monetary amounts must NOT be redacted as phone numbers
    let amounts = "So du hien tai la 1450230000 VND va GD 15000000 dong";
    let s_amounts = sanitize_pii(amounts);
    assert!(s_amounts.contains("1450230000"));
    assert!(s_amounts.contains("15000000"));
    assert!(!s_amounts.contains("[REDACTED_PHONE]"));
}

// ===========================================================================
// 7. Decree 13 PII Sanitizer: Legal Entities vs Personal Names
// ===========================================================================

#[test]
fn test_adversarial_personal_names_vs_legal_entities() {
    // 1. Exact requirement: CONG TY TNHH ABC must not be redacted; individual names masked
    let legal1 = "CONG TY TNHH ABC";
    assert!(is_legal_entity(legal1));
    assert_eq!(sanitize_counterparty_name(legal1), legal1);

    let legal2 = "CONG TY CP DAU TU VA PHAT TRIEN CONG NGHE LIVA";
    assert!(is_legal_entity(legal2));
    assert_eq!(sanitize_counterparty_name(legal2), legal2);

    let individual1 = "Nguyen Van A";
    assert!(!is_legal_entity(individual1));
    assert_eq!(sanitize_counterparty_name(individual1), "[REDACTED_NAME]");

    let individual2 = "Tran Thi Mai Trang";
    assert!(!is_legal_entity(individual2));
    assert_eq!(sanitize_counterparty_name(individual2), "[REDACTED_NAME]");

    // 2. Narration containing both legal entity AND individual natural person
    let mixed_narration = "CONG TY TNHH ABC thanh toan tien luong thang 9 cho Nguyen Van An";
    let s_narr = sanitize_pii(mixed_narration);
    assert!(
        s_narr.contains("CONG TY TNHH ABC"),
        "CONG TY TNHH ABC must be preserved in narration"
    );
    assert!(
        !s_narr.contains("Nguyen Van An"),
        "Nguyen Van An must be masked in narration"
    );
    assert!(s_narr.contains("[REDACTED_NAME]"));

    // 3. Corporate entities with founder surnames (e.g. CONG TY TNHH NGUYEN PHAT)
    let corp_with_surname = "Chuyen khoan cho CONG TY TNHH NGUYEN PHAT hop dong so 45";
    let s_corp_sur = sanitize_pii(corp_with_surname);
    assert!(
        s_corp_sur.contains("CONG TY TNHH NGUYEN PHAT"),
        "Company named after surname must be preserved"
    );
    assert!(!s_corp_sur.contains("[REDACTED_NAME]"));

    // 4. Banking and government entities
    let bank_entity = "NGAN HANG TMCP NGOAI THUONG VIET NAM (VIETCOMBANK)";
    assert!(is_legal_entity(bank_entity));
    assert_eq!(sanitize_counterparty_name(bank_entity), bank_entity);
}

// ===========================================================================
// 8. Zero Data Egress Verification & Socket Interceptor
// ===========================================================================

#[test]
fn test_adversarial_zero_data_egress_security() {
    // 1. Loopback addresses permitted
    assert!(is_egress_permitted("127.0.0.1"));
    assert!(is_egress_permitted("127.0.0.1:8002"));
    assert!(is_egress_permitted("http://127.0.0.1:8002/api"));
    assert!(is_egress_permitted("localhost"));
    assert!(is_egress_permitted("http://localhost:8000/v1/chat"));
    assert!(is_egress_permitted("[::1]:8080"));

    // 2. Cloud and external addresses strictly blocked
    assert!(!is_egress_permitted("https://api.openai.com/v1/chat/completions"));
    assert!(!is_egress_permitted("https://aws.amazon.com/s3"));
    assert!(!is_egress_permitted("http://192.168.1.50:9000"));
    assert!(!is_egress_permitted("http://10.0.0.1:8080"));
    assert!(!is_egress_permitted("8.8.8.8"));
    assert!(!is_egress_permitted("https://telemetry.evil.com/collect"));

    // 3. Socket Address validation via ZeroEgressNetfilter
    let loopback_v4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let loopback_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8080);
    let external_v4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080);
    let public_dns = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);

    assert!(ZeroEgressNetfilter::validate_socket_addr(&loopback_v4).is_ok());
    assert!(ZeroEgressNetfilter::validate_socket_addr(&loopback_v6).is_ok());
    assert!(ZeroEgressNetfilter::validate_socket_addr(&external_v4).is_err());
    assert!(ZeroEgressNetfilter::validate_socket_addr(&public_dns).is_err());

    // 4. Active Interceptor test
    let tracker = EgressTrafficTracker::new();
    assert!(tracker.record_egress("127.0.0.1:8002", 2048).is_ok());
    assert!(tracker.record_egress("https://api.openai.com", 1024).is_err());

    let rep = tracker.report();
    assert_eq!(rep.external_bytes_transmitted, 1024);
    assert_eq!(rep.blocked_attempts_count, 1);
    assert_eq!(rep.loopback_bytes_transmitted, 2048);
    assert!(!rep.is_zero_egress);
    assert!(tracker.assert_zero_external_egress().is_err());

    // 5. Zero egress execution guard on statement processing
    let (tx_count, traffic_report) = verify_statement_processing_zero_egress(|| {
        let items: Vec<u64> = (0..50_000).collect();
        items.len()
    })
    .expect("Statement processing maintains zero external egress");

    assert_eq!(tx_count, 50_000);
    assert_eq!(traffic_report.external_bytes_transmitted, 0);
    assert!(traffic_report.is_zero_egress);
    assert_eq!(traffic_report.blocked_attempts_count, 0);

    // 6. Parameterized Environment Pollution Detection
    let (clean_ok, _) = verify_zero_egress_from(|var| match var {
        "LIVA_SERVER_HOST" => Some("127.0.0.1".to_string()),
        _ => None,
    });
    assert!(clean_ok, "Clean loopback environment must pass zero egress");

    let (dirty_proxy_ok, _) = verify_zero_egress_from(|var| match var {
        "HTTP_PROXY" => Some("http://proxy.external.com:8080".to_string()),
        _ => None,
    });
    assert!(!dirty_proxy_ok, "External HTTP_PROXY must invalidate zero egress");

    let (dirty_openai_ok, _) = verify_zero_egress_from(|var| match var {
        "OPENAI_API_BASE" => Some("https://api.openai.com/v1".to_string()),
        _ => None,
    });
    assert!(!dirty_openai_ok, "External OPENAI_API_BASE must invalidate zero egress");
}

// ===========================================================================
// 9. Direct in-process execution: `handle_compliance_aml_screen`
// ===========================================================================

#[test]
fn test_adversarial_compliance_aml_screen_direct_handler() {
    let args = ComplianceAmlScreenArgs {
        transactions: vec![
            ScreeningTxInput {
                tx_id: "DIRECT-01".to_string(),
                account_number: Some("00110001".to_string()),
                counterparty_name: Some("CONG TY TNHH ABC".to_string()),
                counterparty_account: Some("99887766".to_string()),
                amount_vnd: 450_000_000,
                timestamp: Some(1726300000),
                narration: "Thanh toan tien hang 0901.234.567 cho CONG TY TNHH ABC".to_string(),
                is_credit: Some(true),
            },
        ],
        redact_pii: Some(true),
    };

    let result = handle_compliance_aml_screen(args).expect("Handler must succeed");
    assert_eq!(result.total_screened, 1);
    assert!(result.zero_egress_verified);
    assert_eq!(result.alerts.len(), 1);
    assert_eq!(result.alerts[0].rule_code, "AML_HIGH_VALUE");
    assert_eq!(result.sanitized_transactions[0].masked_counterparty, "CONG TY TNHH ABC");
    assert!(result.sanitized_transactions[0].masked_narration.contains("[REDACTED_PHONE]"));
    assert!(result.sanitized_transactions[0].masked_narration.contains("CONG TY TNHH ABC"));
}

// ===========================================================================
// 10. End-to-End MCP Tool `compliance_aml_screen` Adversarial Integration
// ===========================================================================

#[tokio::test]
async fn test_adversarial_compliance_aml_screen_mcp_tool_end_to_end() {
    let server = NativeMcpServer::new("test_vault");
    let base_ts = 1726300000;

    // Craft an adversarial multi-transaction screening payload:
    // 1. Structuring cluster (3 txs of 350M VND within 48 hours = 1.05B VND)
    // 2. High-value tx (1.5 Billion VND >= 1B -> CRITICAL)
    // 3. Pass-through cluster (Inflow 600M VND, Outflow 580M VND 6 minutes later)
    // 4. Formatted phone numbers and mixed legal entity vs personal names in narrations
    let req = CallToolRequest {
        name: "compliance_aml_screen".to_string(),
        arguments: serde_json::json!({
            "transactions": [
                // Structuring Tx 1
                {
                    "tx_id": "ADV-SMURF-1",
                    "account_number": "19030001",
                    "counterparty_name": "Tran Van A",
                    "counterparty_account": "00112233",
                    "amount_vnd": 350000000,
                    "timestamp": base_ts,
                    "narration": "Chuyen khoan tach lo 1 lien he 0901.234.567",
                    "is_credit": true
                },
                // Structuring Tx 2
                {
                    "tx_id": "ADV-SMURF-2",
                    "account_number": "19030001",
                    "counterparty_name": "Tran Van A",
                    "counterparty_account": "00112233",
                    "amount_vnd": 350000000,
                    "timestamp": base_ts + 3600 * 24,
                    "narration": "Chuyen khoan tach lo 2 lien he 098-765-4321",
                    "is_credit": true
                },
                // Structuring Tx 3
                {
                    "tx_id": "ADV-SMURF-3",
                    "account_number": "19030001",
                    "counterparty_name": "Tran Van A",
                    "counterparty_account": "00112233",
                    "amount_vnd": 350000000,
                    "timestamp": base_ts + 3600 * 48,
                    "narration": "Chuyen khoan tach lo 3 lien he +84 912 345 678",
                    "is_credit": true
                },
                // High Value Critical Tx
                {
                    "tx_id": "ADV-HIGH-VAL",
                    "account_number": "19030001",
                    "counterparty_name": "CONG TY TNHH ABC",
                    "counterparty_account": "99887766",
                    "amount_vnd": 1500000000,
                    "timestamp": base_ts + 3600 * 50,
                    "narration": "Thanh toan hop dong 1.5 ty cho CONG TY TNHH ABC dai dien ong Le Van B",
                    "is_credit": true
                },
                // Pass-Through Inflow
                {
                    "tx_id": "ADV-PT-IN",
                    "account_number": "19030099",
                    "counterparty_name": "CONG TY CP XUAT NHAP KHAU",
                    "counterparty_account": "55443322",
                    "amount_vnd": 600000000,
                    "timestamp": base_ts + 3600 * 60,
                    "narration": "Nhan tien hang CCCD: 001095012345",
                    "is_credit": true
                },
                // Pass-Through Outflow (580M = 96.6% in 360s = 6 mins)
                {
                    "tx_id": "ADV-PT-OUT",
                    "account_number": "19030099",
                    "counterparty_name": "Pham Van C",
                    "counterparty_account": "11223344",
                    "amount_vnd": 580000000,
                    "timestamp": base_ts + 3600 * 60 + 360,
                    "narration": "Chuyen tiep ngay lap tuc",
                    "is_credit": false
                }
            ],
            "redact_pii": true
        }),
    };

    let res = server.call_tool(req).await.expect("Tool call must succeed");
    assert!(!res.is_error, "Tool call must not error");

    let text = extract_text(&res.content);
    let val: Value = serde_json::from_str(text).expect("Valid JSON response");

    assert_eq!(val["total_screened"], 6);
    assert_eq!(val["zero_egress_verified"], true);

    let alerts = val["alerts"].as_array().expect("alerts array");

    // 1. Verify Structuring alert triggered for SMURF txs
    let structuring_alerts: Vec<&Value> = alerts
        .iter()
        .filter(|a| a["rule_code"] == "AML_STRUCTURING")
        .collect();
    assert_eq!(
        structuring_alerts.len(),
        3,
        "Structuring must flag all 3 smurf transactions"
    );

    // 2. Verify High Value alert triggered for ADV-HIGH-VAL
    let hv_alerts: Vec<&Value> = alerts
        .iter()
        .filter(|a| a["rule_code"] == "AML_HIGH_VALUE")
        .collect();
    assert!(
        hv_alerts.iter().any(|a| a["tx_id"] == "ADV-HIGH-VAL"),
        "ADV-HIGH-VAL must trigger AML_HIGH_VALUE"
    );
    let high_val = hv_alerts.iter().find(|a| a["tx_id"] == "ADV-HIGH-VAL").unwrap();
    assert_eq!(high_val["severity"], "CRITICAL");

    // 3. Verify Pass-Through alert triggered for ADV-PT-OUT
    let pt_alerts: Vec<&Value> = alerts
        .iter()
        .filter(|a| a["rule_code"] == "AML_PASS_THROUGH")
        .collect();
    assert_eq!(pt_alerts.len(), 1);
    assert_eq!(pt_alerts[0]["tx_id"], "ADV-PT-OUT");
    assert_eq!(pt_alerts[0]["severity"], "CRITICAL");

    // 4. Verify Decree 13 PII sanitization across narrations and counterparties
    let sanitized = val["sanitized_transactions"].as_array().expect("sanitized array");

    // Phone numbers masked
    let s_smurf1 = sanitized.iter().find(|s| s["tx_id"] == "ADV-SMURF-1").unwrap();
    assert!(!s_smurf1["masked_narration"].as_str().unwrap().contains("0901.234.567"));
    assert!(s_smurf1["masked_narration"].as_str().unwrap().contains("[REDACTED_PHONE]"));

    let s_smurf2 = sanitized.iter().find(|s| s["tx_id"] == "ADV-SMURF-2").unwrap();
    assert!(!s_smurf2["masked_narration"].as_str().unwrap().contains("098-765-4321"));
    assert!(s_smurf2["masked_narration"].as_str().unwrap().contains("[REDACTED_PHONE]"));

    let s_smurf3 = sanitized.iter().find(|s| s["tx_id"] == "ADV-SMURF-3").unwrap();
    assert!(!s_smurf3["masked_narration"].as_str().unwrap().contains("+84 912 345 678"));
    assert!(s_smurf3["masked_narration"].as_str().unwrap().contains("[REDACTED_PHONE]"));

    // Corporate entity CONG TY TNHH ABC preserved; natural person Tran Van A & Le Van B masked
    assert_eq!(s_smurf1["masked_counterparty"].as_str().unwrap(), "[REDACTED_NAME]");

    let s_high = sanitized.iter().find(|s| s["tx_id"] == "ADV-HIGH-VAL").unwrap();
    assert_eq!(s_high["masked_counterparty"].as_str().unwrap(), "CONG TY TNHH ABC");
    let high_narr = s_high["masked_narration"].as_str().unwrap();
    assert!(high_narr.contains("CONG TY TNHH ABC"));
    assert!(!high_narr.contains("Le Van B"));
    assert!(high_narr.contains("[REDACTED_NAME]"));

    // CCCD masked
    let s_pt_in = sanitized.iter().find(|s| s["tx_id"] == "ADV-PT-IN").unwrap();
    assert!(!s_pt_in["masked_narration"].as_str().unwrap().contains("001095012345"));
    assert!(s_pt_in["masked_narration"].as_str().unwrap().contains("[REDACTED_CCCD]"));
}
