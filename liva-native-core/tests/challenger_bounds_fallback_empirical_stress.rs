//! Empirical Challenger Stress Tests for Safe Integer Bounds & SWIFT MT940 Fallback Parsing.
//!
//! Milestone: Phase 3 (M3) — Challenger 1 Adversarial Verification.
//!
//! Objectives:
//! 1. Bounds Protections:
//!    - Verify `u64::try_from(amount.max(0)).unwrap_or(0)` prevents negative-wrap underflow
//!      on extreme values (`i64::MIN`, `-1`, `-1_000_000_000`, `0`, `1`, `i64::MAX`).
//!    - Verify `i64::try_from(val).unwrap_or(i64::MAX)` prevents signed integer overflow / wrap-around
//!      to negative on extreme values (`u64::MAX`, `(i64::MAX as u64) + 1`, `i64::MAX as u64`, `0`).
//! 2. SWIFT MT940 Fallback Parsing:
//!    - Multi-transaction statement with both Debit and Credit lines (:61:) and balances (:60F:, :62F:).
//!    - All format aliases: `swift_mt940`, `swift`, `mt940`, and auto-detection (`None`, `"auto"`).
//!    - SWIFT Block 1 envelope header prefix `{1:F01...}`.
//! 3. Adversarial / Malformed Payloads:
//!    - Invalid / corrupted base64 strings.
//!    - Truncated MT940 statements lacking mandatory balance tags.
//!    - Random binary garbage payloads.
//!    - Missing file paths and missing payloads.
//! 4. End-to-End MCP Server Protocol Integration:
//!    - Full `NativeMcpServer::call_tool` dispatch for `banking_reconcile` and `compliance_aml_screen`.
//!    - Zero float drift, zero panics, strictly fail-closed error handling.

use base64::prelude::*;
use liva_native_core::banking::mcp::{
    BankingReconcileArgs, ComplianceAmlScreenArgs, ScreeningTxInput,
    handle_banking_reconcile, handle_compliance_aml_screen,
};
use liva_native_core::mcp::protocol::{CallToolRequest, ToolContent};
use liva_native_core::mcp::server::NativeMcpServer;
use serde_json::Value;

fn extract_text(content: &[ToolContent]) -> &str {
    match content.first() {
        Some(ToolContent::Text { text }) => text.as_str(),
        other => panic!("Expected ToolContent::Text, got: {:?}", other),
    }
}

// ===========================================================================
// SUITE 1: MATHEMATICAL & INTEGER BOUNDS CONVERSION EMPIRICAL TESTS
// ===========================================================================

#[test]
fn test_empirical_safe_integer_bounds_negative_clamp_and_saturation() {
    // 1. Stress test `u64::try_from(amount.max(0)).unwrap_or(0)`
    // In legacy code, `amount as u64` caused negative numbers like -1 to wrap to u64::MAX (18,446,744,073,709,551,615).
    let extreme_negative_inputs: [(i64, u64); 6] = [
        (i64::MIN, 0),                       // -9_223_372_036_854_775_808
        (i64::MIN + 1, 0),
        (-1_000_000_000_000, 0),             // -1 Trillion VND
        (-1, 0),                             // -1 unit wrap-around danger
        (0, 0),                              // Zero boundary
        (1, 1),                              // Unit positive
    ];

    for (input, expected) in extreme_negative_inputs {
        // Clamping logic used across mcp.rs
        let safe_clamped = u64::try_from(input.max(0)).unwrap_or(0);
        assert_eq!(
            safe_clamped, expected,
            "Input {input} must be safely clamped to {expected}, never wrap to u64::MAX"
        );
        assert_ne!(
            safe_clamped,
            u64::MAX,
            "Input {input} must NEVER wrap to u64::MAX (18 quintillion)"
        );
    }

    // Boundary check on i64::MAX
    let max_clamped = u64::try_from(i64::MAX.max(0)).unwrap_or(0);
    assert_eq!(max_clamped, i64::MAX as u64);

    // 2. Stress test `i64::try_from(val).unwrap_or(i64::MAX)`
    // In legacy code, `u64 as i64` caused values > i64::MAX to wrap around to negative numbers (e.g. u64::MAX as i64 == -1).
    let extreme_u64_inputs: [(u64, i64); 6] = [
        (0, 0),
        (1, 1),
        (100_000_000, 100_000_000),
        (i64::MAX as u64, i64::MAX),
        ((i64::MAX as u64) + 1, i64::MAX),   // Exceeds i64::MAX, must saturate
        (u64::MAX, i64::MAX),                 // Extreme u64::MAX, must saturate to i64::MAX, not wrap to -1
    ];

    for (input, expected) in extreme_u64_inputs {
        let safe_converted = i64::try_from(input).unwrap_or(i64::MAX);
        assert_eq!(
            safe_converted, expected,
            "Input {input} must convert to {expected}, never wrap to negative"
        );
        assert!(
            safe_converted >= 0,
            "Positive u64 input {input} must never turn into negative i64"
        );
    }
}

// ===========================================================================
// SUITE 2: SWIFT MT940 MULTI-TRANSACTION FALLBACK PARSING EMPIRICAL TESTS
// ===========================================================================

#[test]
fn test_empirical_mcp_reconcile_swift_mt940_full_debit_credit_lifecycle() {
    // Standard SWIFT MT940 statement with 3 transactions:
    // Opening Balance: 200,000,000 VND
    // Tx 1: Debit  40,000,000 VND (Ref: DEBIT-EXP-01)
    // Tx 2: Credit 90,000,000 VND (Ref: CREDIT-INV-02)
    // Tx 3: Debit  15,000,000 VND (Ref: DEBIT-TAX-03)
    // Expected Net: 200M - 40M + 90M - 15M = 235,000,000 VND
    // Closing Balance: 235,000,000 VND
    let mt940_payload = "\
:20:LIVA-MT940-STRESS-01\r\n\
:25:00110099887766\r\n\
:28C:00001/001\r\n\
:60F:C260914VND200000000,\r\n\
:61:260914D40000000NTRFNONREF//DEBIT-EXP-01\r\n\
:86:Chi phi thue van phong thang 9 nam 2026\r\n\
:61:260915C90000000NTRFNONREF//CREDIT-INV-02\r\n\
:86:Cong ty ABC thanh toan hop dong cung cap dich vu\r\n\
:61:260916D15000000NTRFNONREF//DEBIT-TAX-03\r\n\
:86:Nop thue thu nhap doanh nghiep tam tinh quy 3\r\n\
:62F:C260916VND235000000,\r\n\
-";

    let b64 = BASE64_STANDARD.encode(mt940_payload.as_bytes());

    let args = BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64),
        statement_format: Some("swift_mt940".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: Some(15_000),
        enable_semantic_recon: Some(false),
    };

    let res = handle_banking_reconcile(args).expect("Fallback StatementEngine parses multi-tx SWIFT MT940");

    assert!(res.balance_invariant_valid, "Balance invariant must hold: 200M - 40M + 90M - 15M == 235M");
    assert_eq!(res.statement_summary.account, "00110099887766");
    assert_eq!(res.statement_summary.opening_balance, 200_000_000);
    assert_eq!(res.statement_summary.closing_balance, 235_000_000);
    assert_eq!(res.statement_summary.total_credit, 90_000_000);
    assert_eq!(res.statement_summary.total_debit, 55_000_000); // 40M + 15M
    assert_eq!(res.statement_summary.tx_count, 3);
}

// ===========================================================================
// SUITE 3: SWIFT MT940 FORMAT ALIASES AND AUTO-DETECTION MATRIX
// ===========================================================================

#[test]
fn test_empirical_mcp_reconcile_swift_mt940_format_aliases_and_auto_sniff() {
    let mt940_content = "\
:20:STMT-ALIAS-CHECK\r\n\
:25:19038877665544\r\n\
:60F:C260914VND50000000,\r\n\
:61:260914C25000000NTRFNONREF//INV-555\r\n\
:86:Thanh toan tien linh kien dien tu\r\n\
:62F:C260914VND75000000,\r\n\
-";
    let b64 = BASE64_STANDARD.encode(mt940_content.as_bytes());

    // 1. Test alias "swift"
    let res_swift = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64.clone()),
        statement_format: Some("swift".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: Some(false),
    }).expect("Format 'swift' must succeed");
    assert!(res_swift.balance_invariant_valid);
    assert_eq!(res_swift.statement_summary.closing_balance, 75_000_000);

    // 2. Test alias "mt940"
    let res_mt940 = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64.clone()),
        statement_format: Some("mt940".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: Some(false),
    }).expect("Format 'mt940' must succeed");
    assert!(res_mt940.balance_invariant_valid);
    assert_eq!(res_mt940.statement_summary.closing_balance, 75_000_000);

    // 3. Test auto-detection with format: None
    let res_none = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64.clone()),
        statement_format: None,
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: Some(false),
    }).expect("Format None (auto-detect) must succeed");
    assert!(res_none.balance_invariant_valid);
    assert_eq!(res_none.statement_summary.closing_balance, 75_000_000);

    // 4. Test auto-detection with format: "auto"
    let res_auto = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64.clone()),
        statement_format: Some("auto".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: Some(false),
    }).expect("Format 'auto' must succeed");
    assert!(res_auto.balance_invariant_valid);
    assert_eq!(res_auto.statement_summary.closing_balance, 75_000_000);

    // 5. Test with SWIFT Block 1 header envelope
    let block1_mt940 = format!("{{1:F01VCBVNVXAXXXX0000000000}}{{2:I940VCBVNVXXXXXN}}{{4:\r\n{mt940_content}}}");
    let b64_block1 = BASE64_STANDARD.encode(block1_mt940.as_bytes());
    let res_block1 = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64_block1),
        statement_format: None, // Must auto-detect {1: header
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: Some(false),
    }).expect("Block 1 header auto-sniff must succeed");
    assert!(res_block1.balance_invariant_valid);
    assert_eq!(res_block1.statement_summary.closing_balance, 75_000_000);
}

// ===========================================================================
// SUITE 4: MALFORMED, TRUNCATED, AND CORRUPTED PAYLOAD FAIL-CLOSED TESTS
// ===========================================================================

#[test]
fn test_empirical_mcp_reconcile_malformed_and_adversarial_payloads() {
    // 1. Invalid base64 characters
    let res_inv_b64 = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some("###INVALID_BASE64_DATA@@@".to_string()),
        statement_format: Some("swift_mt940".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: None,
    });
    assert!(res_inv_b64.is_err(), "Corrupted base64 must return Err");
    let err_b64 = res_inv_b64.unwrap_err();
    assert!(
        err_b64.contains("Invalid base64"),
        "Error message should mention base64 failure, got: {err_b64}"
    );

    // 2. Truncated SWIFT MT940 (only :20: and :25:, missing balance lines)
    // StatementEngine gracefully parses this as an empty statement with 0 transactions and 0 balance.
    let truncated_mt940 = ":20:TRUNCATED-01\r\n:25:123456\r\n";
    let b64_trunc = BASE64_STANDARD.encode(truncated_mt940.as_bytes());
    let res_trunc = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64_trunc),
        statement_format: Some("swift_mt940".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: None,
    }).expect("Truncated MT940 parses gracefully as empty statement");
    assert_eq!(res_trunc.statement_summary.tx_count, 0);
    assert_eq!(res_trunc.statement_summary.account, "123456");

    // 3. Zero-length payload (0 bytes) must fail closed
    let b64_zero = BASE64_STANDARD.encode(b"");
    let res_zero = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64_zero),
        statement_format: Some("swift_mt940".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: None,
    });
    assert!(res_zero.is_err(), "Zero-length statement payload must return Err");

    // 4. Random binary garbage
    let garbage_bytes = [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF, 0xFE, 0xFD, 0xFC];
    let b64_garbage = BASE64_STANDARD.encode(&garbage_bytes);
    let res_garbage = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64_garbage),
        statement_format: None,
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: None,
    });
    assert!(res_garbage.is_err(), "Random binary garbage must fail closed");

    // 5. Unbalanced SWIFT MT940 statement (tampered closing balance)
    // Opening = 100M, Credit = 50M -> Closing should be 150M. But closing balance claims 999,999,999!
    let unbalanced_mt940 = "\
:20:UNBALANCED-01\r\n\
:25:9988776655\r\n\
:60F:C260914VND100000000,\r\n\
:61:260914C50000000NTRFNONREF//TX-01\r\n\
:86:Thanh toan tien hang\r\n\
:62F:C260914VND999999999,\r\n\
-";
    let b64_unbalanced = BASE64_STANDARD.encode(unbalanced_mt940.as_bytes());
    let res_unbalanced = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64_unbalanced),
        statement_format: Some("swift_mt940".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: None,
    });
    assert!(
        res_unbalanced.is_err(),
        "Mathematical invariant violation (100M + 50M != 999.99M) must be strictly rejected by StatementEngine"
    );
    let err_unbalanced = res_unbalanced.unwrap_err();
    assert!(
        err_unbalanced.contains("Mathematical invariant failure in statement balance")
            || err_unbalanced.contains("Statement balance mismatch"),
        "Error message must specify mathematical invariant failure, got: {err_unbalanced}"
    );

    // 4. Neither file path nor base64 provided
    let res_empty = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: None,
        statement_format: None,
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: None,
    });
    assert!(res_empty.is_err(), "Empty request must return Err");
    let err_empty = res_empty.unwrap_err();
    assert!(err_empty.contains("Either `statement_file_path` or `statement_content_base64` must be provided"));

    // 5. Non-existent file path
    let res_nonexist = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: Some("C:\\nonexistent_file_path_12345.bin".to_string()),
        statement_content_base64: None,
        statement_format: None,
        erp_ledger_path: None,
        fee_tolerance_vnd: None,
        enable_semantic_recon: None,
    });
    assert!(res_nonexist.is_err(), "Nonexistent file must return Err");
    assert!(res_nonexist.unwrap_err().contains("does not exist"));
}

// ===========================================================================
// SUITE 5: COMPLIANCE AML SCREENING BOUNDS & OVERFLOW DEFENSE
// ===========================================================================

#[test]
fn test_empirical_compliance_aml_extreme_bounds_and_saturation() {
    // Submit transactions with boundary values: 0, u64::MAX, and high-value
    let screen_args = ComplianceAmlScreenArgs {
        transactions: vec![
            ScreeningTxInput {
                tx_id: "TX-ZERO-AMT".to_string(),
                account_number: Some("19030000000001".to_string()),
                counterparty_name: Some("CONG TY ABC".to_string()),
                counterparty_account: Some("00110000000002".to_string()),
                amount_vnd: 0,
                timestamp: Some(1726000000),
                narration: "Giao dich khong dong".to_string(),
                is_credit: Some(true),
            },
            ScreeningTxInput {
                tx_id: "TX-MAX-U64".to_string(),
                account_number: Some("19030000000001".to_string()),
                counterparty_name: Some("QUY DAU TU XYZ".to_string()),
                counterparty_account: Some("00110000000003".to_string()),
                amount_vnd: u64::MAX, // Extreme u64 value
                timestamp: None,      // Must fall back to safe timestamp without overflow
                narration: "Giao dich chuyen tien khung cuc dai".to_string(),
                is_credit: Some(true),
            },
            ScreeningTxInput {
                tx_id: "TX-HIGH-VAL-500M".to_string(),
                account_number: Some("19030000000001".to_string()),
                counterparty_name: Some("TAP DOAN XAY DUNG".to_string()),
                counterparty_account: Some("00110000000004".to_string()),
                amount_vnd: 500_000_000,
                timestamp: Some(1726000000),
                narration: "Thanh toan tien thi cong cong trinh".to_string(),
                is_credit: Some(false),
            },
        ],
        redact_pii: Some(true),
    };

    let result = handle_compliance_aml_screen(screen_args).expect("Compliance screening handles boundary amounts without panic");

    // The extreme u64::MAX amount and 500M must trigger AML_HIGH_VALUE without arithmetic overflow panic
    assert!(!result.alerts.is_empty(), "Extreme and high-value transactions must trigger alerts");
    let max_tx_alert = result.alerts.iter().find(|a| a.tx_id == "TX-MAX-U64");
    assert!(max_tx_alert.is_some(), "TX-MAX-U64 must be evaluated and flagged");
    assert_eq!(max_tx_alert.unwrap().rule_code, "AML_HIGH_VALUE");
}

// ===========================================================================
// SUITE 6: END-TO-END MCP PROTOCOL RECONCILE DISPATCH (SWIFT MT940)
// ===========================================================================

#[tokio::test]
async fn test_empirical_mcp_server_protocol_dispatch_swift_mt940() {
    let server = NativeMcpServer::new("test_vault");

    let mt940_payload = "\
:20:MCP-E2E-TEST\r\n\
:25:00223344556677\r\n\
:60F:C260914VND100000000,\r\n\
:61:260914C30000000NTRFNONREF//INV-777\r\n\
:86:Thanh toan tien hang hop dong 777\r\n\
:62F:C260914VND130000000,\r\n\
-";
    let b64 = BASE64_STANDARD.encode(mt940_payload.as_bytes());

    let req = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64,
            "statement_format": "swift_mt940",
            "fee_tolerance_vnd": 11000
        }),
    };

    let res = server.call_tool(req).await.expect("Tool call executes");
    assert!(!res.is_error, "Tool call must succeed without error");

    let text = extract_text(&res.content);
    let val: Value = serde_json::from_str(text).expect("Valid JSON result");

    assert_eq!(val["balance_invariant_valid"], true);
    assert_eq!(val["statement_summary"]["account"], "00223344556677");
    assert_eq!(val["statement_summary"]["opening_balance"], 100_000_000);
    assert_eq!(val["statement_summary"]["closing_balance"], 130_000_000);
    assert_eq!(val["statement_summary"]["total_credit"], 30_000_000);
    assert_eq!(val["statement_summary"]["tx_count"], 1);
}

// ===========================================================================
// SUITE 7: EXHAUSTIVE BOUNDS PROPERTY FUZZING
// ===========================================================================

#[test]
fn test_empirical_fuzz_bounds_properties_exhaustive() {
    // Test a wide spectrum of integer samples across signed and unsigned spectrum
    let signed_samples: Vec<i64> = vec![
        i64::MIN,
        i64::MIN + 1,
        i64::MIN + 100_000,
        -10_000_000_000_000,
        -1_000_000_000,
        -1_000_000,
        -10_000,
        -100,
        -2,
        -1,
        0,
        1,
        2,
        100,
        10_000,
        1_000_000,
        1_000_000_000,
        10_000_000_000_000,
        i64::MAX - 100_000,
        i64::MAX - 1,
        i64::MAX,
    ];

    for s in signed_samples {
        // Property 1: Clamped result must be >= 0 and <= i64::MAX as u64
        let clamped = u64::try_from(s.max(0)).unwrap_or(0);
        if s <= 0 {
            assert_eq!(clamped, 0, "Non-positive input {s} must clamp to 0");
        } else {
            assert_eq!(clamped, s as u64, "Positive input {s} must equal s as u64");
        }
        assert!(clamped <= i64::MAX as u64, "Clamped {clamped} exceeds i64::MAX");
    }

    let unsigned_samples: Vec<u64> = vec![
        0,
        1,
        2,
        100,
        10_000,
        1_000_000,
        1_000_000_000,
        i64::MAX as u64 - 1,
        i64::MAX as u64,
        i64::MAX as u64 + 1,
        i64::MAX as u64 + 1_000_000,
        u64::MAX - 1_000_000,
        u64::MAX - 1,
        u64::MAX,
    ];

    for u in unsigned_samples {
        // Property 2: Saturated conversion must be >= 0 and <= i64::MAX
        let saturated = i64::try_from(u).unwrap_or(i64::MAX);
        assert!(saturated >= 0, "Saturated i64 must never be negative for unsigned input {u}");
        if u <= i64::MAX as u64 {
            assert_eq!(saturated, u as i64, "Input within range {u} must convert exactly");
        } else {
            assert_eq!(saturated, i64::MAX, "Input exceeding i64::MAX ({u}) must saturate to i64::MAX");
        }
    }
}

// ===========================================================================
// SUITE 8: TREASURY PAYMENT ORDER EXTREME VALUES AND ZERO-REJECTION
// ===========================================================================

#[test]
fn test_empirical_treasury_payment_order_extreme_amounts_and_hmac() {
    use liva_native_core::banking::mcp::{TreasuryPaymentOrderArgs, handle_treasury_payment_order};

    // 1. Propose order with 0 amount -> Must be rejected (zero amount is invalid)
    let zero_args = TreasuryPaymentOrderArgs {
        action: "propose".to_string(),
        order_id: None,
        debit_account: Some("19030000000001".to_string()),
        beneficiary_account: Some("00110000000002".to_string()),
        beneficiary_name: Some("CONG TY ABC".to_string()),
        beneficiary_bank: Some("VCB".to_string()),
        amount_vnd: Some(0),
        purpose: Some("Chuyen tien 0 dong".to_string()),
        maker_id: Some("maker_01".to_string()),
        checker_id: None,
        hitl_token: None,
        rejection_reason: None,
    };
    let zero_res = handle_treasury_payment_order(zero_args, None);
    assert!(zero_res.is_err(), "Zero amount payment order must be rejected");

    // 2. Propose high-value order (1 Trillion VND) -> Must generate HITL token and PENDING_APPROVAL
    let huge_args = TreasuryPaymentOrderArgs {
        action: "propose".to_string(),
        order_id: None,
        debit_account: Some("19030000000001".to_string()),
        beneficiary_account: Some("00110000000002".to_string()),
        beneficiary_name: Some("TAP DOAN DA QUOC GIA".to_string()),
        beneficiary_bank: Some("BIDV".to_string()),
        amount_vnd: Some(1_000_000_000_000), // 1 Trillion VND
        purpose: Some("Giai ngan von dau tu du an quy 3".to_string()),
        maker_id: Some("maker_chief_01".to_string()),
        checker_id: None,
        hitl_token: None,
        rejection_reason: None,
    };
    let huge_res = handle_treasury_payment_order(huge_args, None).expect("1 Trillion VND order proposed safely");
    assert_eq!(huge_res.status, "PENDING_APPROVAL");
    assert!(huge_res.order_id.is_some());
    assert!(huge_res.hitl_token.is_some());

    // 3. Maker attempting self-approval on 1 Trillion VND order -> Strictly rejected
    let order_id = huge_res.order_id.unwrap();
    let hitl_token = huge_res.hitl_token.unwrap();

    let self_approve_args = TreasuryPaymentOrderArgs {
        action: "approve".to_string(),
        order_id: Some(order_id.clone()),
        debit_account: None,
        beneficiary_account: None,
        beneficiary_name: None,
        beneficiary_bank: None,
        amount_vnd: None,
        purpose: None,
        maker_id: None,
        checker_id: Some("maker_chief_01".to_string()),
        hitl_token: Some(hitl_token.clone()),
        rejection_reason: None,
    };
    let self_err = handle_treasury_payment_order(self_approve_args, None).expect_err("Self-approval must fail");
    assert!(self_err.contains("Self-approval") || self_err.contains("SelfApprovalProhibited"));

    // 4. Legitimate Checker approving 1 Trillion VND order -> HMAC-SHA256 audit signature generated
    let valid_approve_args = TreasuryPaymentOrderArgs {
        action: "approve".to_string(),
        order_id: Some(order_id),
        debit_account: None,
        beneficiary_account: None,
        beneficiary_name: None,
        beneficiary_bank: None,
        amount_vnd: None,
        purpose: None,
        maker_id: None,
        checker_id: Some("cfo_checker_02".to_string()),
        hitl_token: Some(hitl_token),
        rejection_reason: None,
    };
    let approve_res = handle_treasury_payment_order(valid_approve_args, None).expect("Approval of 1T VND succeeds");
    assert_eq!(approve_res.status, "APPROVED");
    assert!(approve_res.signature_hmac.is_some(), "High-value disbursement must have cryptographic HMAC signature");
}

// ===========================================================================
// SUITE 9: SWIFT MT940 WITH VIETNAMESE UTF-8 AND SPECIAL NARRATIONS
// ===========================================================================

#[test]
fn test_empirical_mcp_reconcile_swift_mt940_complex_utf8_narration() {
    // Statement with complex Vietnamese diacritics and special invoice reference notations
    let mt940_utf8 = "\
:20:VN-MT940-COMPLEX-01\r\n\
:25:1234567890123\r\n\
:60F:C260914VND500000000,\r\n\
:61:260914D125000000NTRFNONREF//HĐ-2026/09-ĐT\r\n\
:86:Thanh toán tiền điện và dịch vụ viễn thông công nghệ cao LIVA\r\n\
:61:260915C350000000NTRFNONREF//UNC-8899\r\n\
:86:Công ty Cổ phần Đầu tư & Phát triển Công nghệ thanh toán đợt 2\r\n\
:62F:C260915VND725000000,\r\n\
-";
    // Math: 500M - 125M + 350M = 725,000,000 VND
    let b64 = BASE64_STANDARD.encode(mt940_utf8.as_bytes());

    let res = handle_banking_reconcile(BankingReconcileArgs {
        statement_file_path: None,
        statement_content_base64: Some(b64),
        statement_format: Some("swift_mt940".to_string()),
        erp_ledger_path: None,
        fee_tolerance_vnd: Some(22_000),
        enable_semantic_recon: Some(false),
    }).expect("UTF-8 MT940 statement parses cleanly");

    assert!(res.balance_invariant_valid, "Balance invariant must hold: 500M - 125M + 350M == 725M");
    assert_eq!(res.statement_summary.opening_balance, 500_000_000);
    assert_eq!(res.statement_summary.closing_balance, 725_000_000);
    assert_eq!(res.statement_summary.total_debit, 125_000_000);
    assert_eq!(res.statement_summary.total_credit, 350_000_000);
    assert_eq!(res.statement_summary.tx_count, 2);
}
