//! Adversarial Stress and Edge-Case Test Suite for LIVA Banking Local AI.
//!
//! Evaluates:
//! 1. Tampered model artifact / corrupted SHA-256 hash detection and loading guard fail-closed.
//! 2. Exact smurfing boundary edge values (379.999M VND vs 380.0M VND vs 399.999M VND vs 400.0M VND).
//! 3. Off-hours transaction surveillance and high-velocity micro-burst evasion.
//! 4. Decree 13 PII adversarial injection: natural person CCCD/phone/patronymic masked while corporate entities preserved.
//! 5. Concurrency stress across MCP handlers under strict Zero Data Egress containment.

use liva_native_core::{embedded_model_hash, verify_model_artifact, verify_trusted_file};
use liva_native_core::banking::compliance::sanitizer::{is_legal_entity, sanitize_counterparty_name, sanitize_pii};
use liva_native_core::banking::compliance::security::{
    ZeroEgressNetfilter, GLOBAL_EGRESS_TRACKER,
};
use liva_native_core::banking::mcp::{
    ComplianceAmlScreenArgs, ScreeningTxInput, handle_compliance_aml_screen,
};
use liva_native_core::mcp::protocol::CallToolRequest;
use liva_native_core::mcp::server::NativeMcpServer;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

fn create_temp_dir(prefix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "liva_adv_{}_{}_{}",
        prefix,
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&dir).expect("Create temp dir");
    dir
}

// ---------------------------------------------------------------------------
// 1. Adversarial Tampered Model File & Invalid SHA-256 Hash Rejection
// ---------------------------------------------------------------------------

#[test]
fn test_adversarial_tampered_model_sha256_rejection() {
    const OFFICIAL_QWEN3_HASH: &str =
        "bf19bf5c77c530762012f71215b62aa8f0bae08111ee0d537b8496479e17cfa3";
    let temp_root = create_temp_dir("tampered_model");

    // Attack vector 1: Bit flip / tampered content in model file
    let fake_model = temp_root.join("Qwen_Qwen3-14B-Q5_K_M.gguf");
    let tampered_payload = b"TAMPERED_MALICIOUS_WEIGHTS_INJECTION_PAYLOAD";
    fs::write(&fake_model, tampered_payload).expect("Write tampered file");

    let verify_res = verify_trusted_file(&temp_root, &fake_model, OFFICIAL_QWEN3_HASH);
    assert!(
        verify_res.is_err(),
        "verify_trusted_file MUST reject tampered model file"
    );
    let err_msg = verify_res.unwrap_err();
    assert!(
        err_msg.contains("SHA-256 artifact không khớp"),
        "Error must explicitly cite SHA-256 mismatch: {err_msg}"
    );

    // Attack vector 2: Path traversal attack inside candidate path
    let traversal_candidate = temp_root.join("..").join("outside_exploit.gguf");
    let traversal_res = verify_trusted_file(&temp_root, &traversal_candidate, OFFICIAL_QWEN3_HASH);
    assert!(
        traversal_res.is_err(),
        "verify_trusted_file MUST reject path traversal containing '..'"
    );

    // Attack vector 3: Model extension evasion (attempting to load executable or script as model)
    let bad_ext = temp_root.join("exploit.dll");
    fs::write(&bad_ext, b"MZ_HEADER_EXPLOIT").expect("Write bad ext");
    let ext_res = verify_model_artifact(&temp_root, &bad_ext);
    assert!(
        ext_res.is_err(),
        "verify_model_artifact MUST reject non-gguf file extensions"
    );

    // Attack vector 4: Unregistered model name not in models-manifest.json
    let unknown_res = embedded_model_hash("unregistered_trojan_model.gguf");
    assert!(
        unknown_res.is_err(),
        "embedded_model_hash MUST reject models not whitelisted in trust manifest"
    );

    let _ = fs::remove_dir_all(temp_root);
}

// ---------------------------------------------------------------------------
// 2. Smurfing Edge Values (379.999M vs 380.0M vs 399.999M vs 400.0M VND)
// ---------------------------------------------------------------------------

#[test]
fn test_adversarial_smurfing_boundary_edge_values() {
    let t_day = 1789527600i64; // Daytime 10:00 AM GMT+7

    // Boundary Test Cases:
    // A: 379,999,999 VND -> 1 VND below 380M smurfing band -> NO NearThresholdSmurfing, NO HighValue
    // B: 380,000,000 VND -> EXACT lower bound of smurfing band -> MUST trigger AML_NEAR_THRESHOLD_SMURFING
    // C: 390,000,000 VND -> Mid-band smurfing -> MUST trigger AML_NEAR_THRESHOLD_SMURFING
    // D: 399,999,999 VND -> 1 VND below statutory 400M threshold -> MUST trigger AML_NEAR_THRESHOLD_SMURFING
    // E: 400,000,000 VND -> EXACT Decision 11/2023 statutory threshold -> MUST trigger AML_HIGH_VALUE (NOT smurfing)
    // F: 400,000,001 VND -> Above statutory threshold -> MUST trigger AML_HIGH_VALUE

    let test_cases = [
        ("TX-CASE-A", 379_999_999u64, false, false),
        ("TX-CASE-B", 380_000_000u64, true, false),
        ("TX-CASE-C", 390_000_000u64, true, false),
        ("TX-CASE-D", 399_999_999u64, true, false),
        ("TX-CASE-E", 400_000_000u64, false, true),
        ("TX-CASE-F", 400_000_001u64, false, true),
    ];

    for (tx_id, amount_vnd, expect_smurf, expect_high_value) in test_cases {
        let args = ComplianceAmlScreenArgs {
            transactions: vec![ScreeningTxInput {
                tx_id: tx_id.to_string(),
                account_number: Some("ACC-BOUNDARY-01".to_string()),
                counterparty_name: Some("Cong ty TNHH Thuong Mai Alpha".to_string()),
                counterparty_account: Some("CP-BOUNDARY-01".to_string()),
                amount_vnd,
                timestamp: Some(t_day),
                narration: "Thanh toan tien hang hop le theo hop dong".to_string(),
                is_credit: Some(true),
            }],
            redact_pii: Some(true),
        };

        let res = handle_compliance_aml_screen(args)
            .unwrap_or_else(|e| panic!("Screening failed for {tx_id}: {e}"));

        let has_smurf = res
            .alerts
            .iter()
            .any(|a| a.rule_code == "AML_NEAR_THRESHOLD_SMURFING");
        let has_high_val = res
            .alerts
            .iter()
            .any(|a| a.rule_code == "AML_HIGH_VALUE");

        assert_eq!(
            has_smurf, expect_smurf,
            "Failed smurfing boundary assertion for {tx_id} (amount {amount_vnd} VND): expected {expect_smurf}, got {has_smurf}"
        );
        assert_eq!(
            has_high_val, expect_high_value,
            "Failed high value boundary assertion for {tx_id} (amount {amount_vnd} VND): expected {expect_high_value}, got {has_high_val}"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Off-Hours Micro-Bursts and High-Velocity Transit Evasion
// ---------------------------------------------------------------------------

#[test]
fn test_adversarial_off_hours_and_velocity_evasion() {
    let t_day = 1789527600i64;   // Daytime 10:00 AM GMT+7
    let t_night = 1789498800i64; // Night 02:00 AM GMT+7

    // A. Off-Hours Night Boundary:
    // Daytime transaction with 250M VND -> No night anomaly
    let day_args = ComplianceAmlScreenArgs {
        transactions: vec![ScreeningTxInput {
            tx_id: "TX-DAY-VALID".to_string(),
            account_number: Some("0011000123".to_string()),
            counterparty_name: Some("Doi Tac Ban Ngay".to_string()),
            counterparty_account: Some("0099887766".to_string()),
            amount_vnd: 250_000_000,
            timestamp: Some(t_day),
            narration: "Thanh toan tien hang ban ngay".to_string(),
            is_credit: Some(true),
        }],
        redact_pii: Some(true),
    };
    let day_res = handle_compliance_aml_screen(day_args).expect("Screen day tx");
    assert!(
        !day_res.alerts.iter().any(|a| a.rule_code == "AML_NIGHT_ANOMALY"),
        "Daytime transaction must NOT trigger AML_NIGHT_ANOMALY"
    );

    // Night transaction with 250M VND (> 200M night threshold) -> Must trigger AML_NIGHT_ANOMALY
    let night_args = ComplianceAmlScreenArgs {
        transactions: vec![ScreeningTxInput {
            tx_id: "TX-NIGHT-FLAGGED".to_string(),
            account_number: Some("0011000123".to_string()),
            counterparty_name: Some("Doi Tac Ban Dem".to_string()),
            counterparty_account: Some("0099887766".to_string()),
            amount_vnd: 250_000_000,
            timestamp: Some(t_night),
            narration: "Thanh toan tien hang dem khuya".to_string(),
            is_credit: Some(true),
        }],
        redact_pii: Some(true),
    };
    let night_res = handle_compliance_aml_screen(night_args).expect("Screen night tx");
    assert!(
        night_res.alerts.iter().any(|a| a.rule_code == "AML_NIGHT_ANOMALY"),
        "Night transaction at 02:00 AM with 250M VND MUST trigger AML_NIGHT_ANOMALY"
    );

    // B. Micro-Burst Automated Bot Attack:
    // 5 rapid transactions within 30 seconds (< 60s window) from the same account
    let mut burst_txs = Vec::new();
    for i in 1..=5 {
        burst_txs.push(ScreeningTxInput {
            tx_id: format!("TX-BURST-BOT-{i}"),
            account_number: Some("ACC-BOT-ATTACK".to_string()),
            counterparty_name: Some("Bot Operator Mule".to_string()),
            counterparty_account: Some("MULE-ACCT-01".to_string()),
            amount_vnd: 45_000_000,
            timestamp: Some(t_day + (i as i64 * 6)), // Every 6 seconds!
            narration: format!("Lenh chuyen tu dong burst {i}"),
            is_credit: Some(false),
        });
    }

    let burst_args = ComplianceAmlScreenArgs {
        transactions: burst_txs,
        redact_pii: Some(true),
    };
    let burst_res = handle_compliance_aml_screen(burst_args).expect("Screen burst txs");
    assert!(
        burst_res.alerts.iter().any(|a| a.rule_code == "AML_MICRO_BURST_VELOCITY" || a.rule_code == "AML_VELOCITY_SURGE"),
        "Micro-burst sequence (<60s) MUST trigger velocity surge anomaly"
    );

    // C. Pass-Through Transit Mule Evasion:
    // Large credit 300,000,000 VND followed by debit 295,000,000 VND (98.3%) within 120s
    let transit_txs = vec![
        ScreeningTxInput {
            tx_id: "TX-TRANSIT-IN".to_string(),
            account_number: Some("ACC-MULE-TRANSIT".to_string()),
            counterparty_name: Some("Cong ty Nguon".to_string()),
            counterparty_account: Some("SRC-01".to_string()),
            amount_vnd: 300_000_000,
            timestamp: Some(t_day),
            narration: "Chuyen tien dau vao".to_string(),
            is_credit: Some(true),
        },
        ScreeningTxInput {
            tx_id: "TX-TRANSIT-OUT".to_string(),
            account_number: Some("ACC-MULE-TRANSIT".to_string()),
            counterparty_name: Some("Dich Den Cuoi".to_string()),
            counterparty_account: Some("DST-01".to_string()),
            amount_vnd: 295_000_000,
            timestamp: Some(t_day + 120), // 2 minutes later
            narration: "Rut tien ngay lap tuc".to_string(),
            is_credit: Some(false),
        },
    ];

    let transit_args = ComplianceAmlScreenArgs {
        transactions: transit_txs,
        redact_pii: Some(true),
    };
    let transit_res = handle_compliance_aml_screen(transit_args).expect("Screen transit txs");
    assert!(
        transit_res.alerts.iter().any(|a| a.rule_code == "AML_PASS_THROUGH"),
        "Pass-through transit mule transfer within 2 minutes MUST trigger AML_PASS_THROUGH"
    );
}

// ---------------------------------------------------------------------------
// 4. Decree 13 PII Adversarial Injection vs Corporate Entity Preservation
// ---------------------------------------------------------------------------

#[test]
fn test_adversarial_decree13_pii_injection_vs_corporate_preservation() {
    // Adversarial narratives attempting to hide natural person PII inside corporate contexts
    let deceptive_samples = [
        // Case 1: Natural person CCCD & phone embedded inside corporate contract payment
        (
            "Hop dong so 88 CONG TY CP HOANG ANH GIA LAI chuyen tien cho ong Nguyen Van Nam CCCD 012345678901 so dt 0912345678",
            "CONG TY CP HOANG ANH GIA LAI", // Corporate name to PRESERVE
            "012345678901",                 // CCCD to MASK
            "0912345678",                   // Phone to MASK
        ),
        // Case 2: Corporate branch with multiple natural person identities
        (
            "CONG TY TNHH DIEN MAY XANH CHI NHANH 1 chi tien cho ba Le Thi Mai CCCD 079188001122 sdt 0988777666 de mua hang",
            "CONG TY TNHH DIEN MAY XANH",
            "079188001122",
            "0988777666",
        ),
        // Case 3: International corporate abbreviation with Vietnamese personal patronymic
        (
            "Thanh toan hoa don CONG TY CP PHAN MEM FPT chu tk Tran Van Binh stk 19034567890123 CCCD 036095001234",
            "CONG TY CP PHAN MEM FPT",
            "036095001234",
            "19034567890123", // Account to MASK
        ),
    ];

    for (raw_narration, corporate_entity, sensitive_id, sensitive_contact) in deceptive_samples {
        let sanitized = sanitize_pii(raw_narration);

        // Corporate legal entity MUST BE PRESERVED
        assert!(
            sanitized.contains(corporate_entity),
            "Decree 13 sanitizer FAILED to preserve corporate entity '{corporate_entity}' in: {sanitized}"
        );

        // Sensitive ID (CCCD or Account) MUST BE MASKED
        assert!(
            !sanitized.contains(sensitive_id),
            "Decree 13 sanitizer LEAKED sensitive ID '{sensitive_id}' in: {sanitized}"
        );

        // Sensitive contact or patronymic MUST BE MASKED
        assert!(
            !sanitized.contains(sensitive_contact),
            "Decree 13 sanitizer LEAKED sensitive contact '{sensitive_contact}' in: {sanitized}"
        );
    }

    // Direct testing of counterparty name sanitization
    assert!(is_legal_entity("CONG TY TNHH PHAT TRIEN NANG LUONG XANH"));
    assert!(is_legal_entity("NGAN HANG TMCP NGOAI THUONG VIET NAM"));
    assert!(is_legal_entity("TAP DOAN CONG NGHIEP THAN KHOANG SAN"));

    assert!(!is_legal_entity("Nguyen Van An"));
    assert!(!is_legal_entity("Tran Thi Bich Ngoc"));
    assert!(!is_legal_entity("Le Hoang Phuc"));

    let sanitized_corp = sanitize_counterparty_name("CONG TY TNHH VINCOM RETAIL");
    assert_eq!(
        sanitized_corp, "CONG TY TNHH VINCOM RETAIL",
        "Corporate legal entity counterparty must remain unchanged"
    );

    let sanitized_person = sanitize_counterparty_name("Nguyen Van An");
    assert_ne!(
        sanitized_person, "Nguyen Van An",
        "Natural person counterparty MUST be sanitized/redacted"
    );
}

// ---------------------------------------------------------------------------
// 5. Concurrency Stress Across MCP Handlers with Zero Data Egress
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_adversarial_concurrency_stress_zero_data_egress() {
    let server = Arc::new(NativeMcpServer::new("test_vault"));
    let mut handles = Vec::new();

    // Reset egress tracker before stress test
    GLOBAL_EGRESS_TRACKER.reset();

    // Launch 24 concurrent async tasks across various MCP banking tools
    for i in 0..24 {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                // Screening task
                let req = CallToolRequest {
                    name: "compliance_aml_screen".to_string(),
                    arguments: serde_json::json!({
                        "transactions": [
                            {
                                "tx_id": format!("STRESS-AML-{i}"),
                                "counterparty_name": "CONG TY TNHH TEST STRESS",
                                "counterparty_account": format!("ACC-STRESS-{i}"),
                                "amount_vnd": 385_000_000 + (i as u64 * 500_000),
                                "timestamp": 1789527600 + (i as i64 * 10),
                                "narration": "Thanh toan dot stress concurrency",
                                "is_credit": true
                            }
                        ],
                        "redact_pii": true
                    }),
                };
                let res = server_clone.call_tool(req).await.expect("Tool call");
                assert!(!res.is_error);
            } else {
                // Credit risk scoring task
                let req = CallToolRequest {
                    name: "credit_risk_scoring".to_string(),
                    arguments: serde_json::json!({
                        "ebitda_vnd": 1_800_000_000i64 + (i as i64 * 50_000_000),
                        "capex_vnd": 300_000_000i64,
                        "debt_service_principal_vnd": 500_000_000i64,
                        "debt_service_interest_vnd": 150_000_000i64,
                        "cash_and_equivalents_vnd": 400_000_000i64,
                        "marketable_securities_vnd": 100_000_000i64,
                        "accounts_receivable_vnd": 350_000_000i64,
                        "current_liabilities_vnd": 600_000_000i64,
                        "forecast_days": 30
                    }),
                };
                let res = server_clone.call_tool(req).await.expect("Tool call");
                assert!(!res.is_error);
            }
        });
        handles.push(handle);
    }

    // Await all concurrent tasks
    for handle in handles {
        handle.await.expect("Concurrent task failed");
    }

    // Critical assertion: Zero Cloud Egress invariant must hold under concurrency
    let egress_res = ZeroEgressNetfilter::assert_zero_egress();
    assert!(
        egress_res.is_ok(),
        "Zero Data Egress must strictly hold under concurrency stress (0 external bytes transmitted)"
    );

    let report = ZeroEgressNetfilter::get_traffic_report();
    assert!(report.is_zero_egress);
    assert_eq!(report.external_bytes_transmitted, 0);
}
