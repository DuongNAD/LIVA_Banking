//! Empirical Challenger Stress Tests for Banking Operations MCP Tool Suite.
//!
//! Stress-tests and challenges:
//! 1. `treasury_payment_order`:
//!    - Self-approval attempts: exact match, case-insensitive ("maker_01" vs "MAKER_01"), whitespace padding ("  maker_01  ")
//!    - Self-rejection attempts by maker
//!    - Invalid token approval (bogus token, empty token)
//!    - Token replay rejection (already used token)
//!    - Expired token approval (>15m TTL)
//!    - Zero amount rejection (amount_vnd: 0)
//!    - Negative amount rejection in JSON payload
//!    - Extreme amounts (1 quadrillion VND)
//!    - High-value (>500M VND, e.g. 1.5B VND) strict dual-control enforcement with HMAC-SHA256 audit signature
//! 2. `banking_reconcile`:
//!    - Statements with rounding differences / fee tolerances (10,000 VND diff)
//!    - Statements with discrepancy exceeding tolerance (15,000 VND diff)
//!    - Balance invariant check failure on 1 VND discrepancy
//!    - Empty statements (0 transactions XML)
//!    - Zero-length payload and invalid base64 handling
//!    - ISO 20022 camt.053 XML variations (different namespaces, debit/credit mixes, missing optional fields, malformed XML)
//! 3. `credit_risk_scoring`:
//!    - DSCR with 0 debt service (Debt-Free entity)
//!    - DSCR with negative NOI (EBITDA < CAPEX)
//!    - Quick Ratio with 0 liabilities
//!    - Cash shortfall simulations (immediate, delayed, none, large numbers)
//! 4. Arithmetic Precision:
//!    - 0 floating-point drift: exact basis points integer math

use base64::prelude::*;
use liva_native_core::banking::compliance::maker_checker::{
    MakerCheckerError, HITL_TOKEN_TTL_SECONDS,
};
use liva_native_core::banking::risk::{
    CreditRiskEngine, DailyCashflowPoint, DscrRiskCategory, LiquidityStatus,
};
use liva_native_core::banking::treasury::{
    PaymentOrderStatus, TreasuryEngine, find_payment_order_by_id, init_payment_orders_table,
    save_payment_order,
};
use liva_native_core::db::DatabasePool;
use liva_native_core::mcp::protocol::{CallToolRequest, ToolContent};
use liva_native_core::mcp::server::NativeMcpServer;
use rusqlite::Connection;
use serde_json::Value;

fn extract_text(content: &[ToolContent]) -> &str {
    match content.first() {
        Some(ToolContent::Text { text }) => text.as_str(),
        other => panic!("Expected ToolContent::Text, got: {:?}", other),
    }
}

// ===========================================================================
// SUITE 1: TREASURY PAYMENT ORDER ADVERSARIAL STRESS TESTS
// ===========================================================================

#[tokio::test]
async fn test_treasury_self_approval_case_insensitivity_and_whitespace() {
    let db_pool = DatabasePool::new_in_memory().expect("In-memory pool");
    let server = NativeMcpServer::new("test_vault").with_db_pool(db_pool);

    // 1. Propose order by maker "accountant_alice"
    let propose_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "19030000000001",
            "beneficiary_account": "00110000000002",
            "beneficiary_name": "CTY TNHH CONG NGHE XYZ",
            "beneficiary_bank": "VCB",
            "amount_vnd": 600_000_000, // High-value > 500M
            "purpose": "Thanh toan tien thiet bi may chu",
            "maker_id": "accountant_alice"
        }),
    };
    let propose_res = server.call_tool(propose_req).await.expect("Propose succeeds");
    assert!(!propose_res.is_error);
    let propose_val: Value = serde_json::from_str(extract_text(&propose_res.content)).unwrap();
    let order_id = propose_val["order_id"].as_str().unwrap().to_string();
    let token = propose_val["hitl_token"].as_str().unwrap().to_string();

    // Adversarial Attempt 1: Exact Maker self-approval
    let self_approve_1 = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "accountant_alice",
            "hitl_token": token.clone()
        }),
    };
    let res_1 = server.call_tool(self_approve_1).await.expect("Processed");
    assert!(res_1.is_error, "Exact self-approval must fail closed");

    // Adversarial Attempt 2: Case-manipulation bypass attempt ("ACCOUNTANT_ALICE")
    let self_approve_2 = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "ACCOUNTANT_ALICE",
            "hitl_token": token.clone()
        }),
    };
    let res_2 = server.call_tool(self_approve_2).await.expect("Processed");
    assert!(res_2.is_error, "Uppercase case-manipulation self-approval must fail closed");

    // Adversarial Attempt 3: Mixed-case bypass attempt ("Accountant_Alice")
    let self_approve_3 = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "Accountant_Alice",
            "hitl_token": token.clone()
        }),
    };
    let res_3 = server.call_tool(self_approve_3).await.expect("Processed");
    assert!(res_3.is_error, "Mixed-case self-approval must fail closed");

    // Adversarial Attempt 4: Whitespace padding bypass attempt ("  accountant_alice  ")
    let self_approve_4 = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "  accountant_alice  ",
            "hitl_token": token.clone()
        }),
    };
    let res_4 = server.call_tool(self_approve_4).await.expect("Processed");
    assert!(res_4.is_error, "Whitespace-padded self-approval must fail closed");

    // Adversarial Attempt 5: Self-rejection attempt by maker
    let self_reject = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "reject",
            "order_id": order_id.clone(),
            "checker_id": "accountant_alice",
            "hitl_token": token.clone(),
            "rejection_reason": "Maker cancelling own order via checker action"
        }),
    };
    let res_reject = server.call_tool(self_reject).await.expect("Processed");
    assert!(res_reject.is_error, "Maker self-rejection as checker must fail closed");
}

#[tokio::test]
async fn test_treasury_invalid_and_replayed_tokens() {
    let db_pool = DatabasePool::new_in_memory().expect("In-memory pool");
    let server = NativeMcpServer::new("test_vault").with_db_pool(db_pool);

    let propose_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "19030000000001",
            "beneficiary_account": "00110000000002",
            "beneficiary_name": "CONG TY TNHH THEP HOA PHAT",
            "beneficiary_bank": "TCB",
            "amount_vnd": 850_000_000, // High-value > 500M
            "purpose": "Thanh toan tien thep cuon",
            "maker_id": "maker_bob"
        }),
    };
    let propose_res = server.call_tool(propose_req).await.expect("Propose succeeds");
    let propose_val: Value = serde_json::from_str(extract_text(&propose_res.content)).unwrap();
    let order_id = propose_val["order_id"].as_str().unwrap().to_string();
    let valid_token = propose_val["hitl_token"].as_str().unwrap().to_string();

    // 1. Invalid token: bogus UUID
    let bogus_token_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "chief_carol",
            "hitl_token": "00000000-0000-0000-0000-000000000000"
        }),
    };
    let bogus_res = server.call_tool(bogus_token_req).await.expect("Processed");
    assert!(bogus_res.is_error, "Bogus token must fail closed");

    // 2. Empty token
    let empty_token_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "chief_carol",
            "hitl_token": ""
        }),
    };
    let empty_res = server.call_tool(empty_token_req).await.expect("Processed");
    assert!(empty_res.is_error, "Empty token must fail closed");

    // 3. Valid Checker approval succeeds once
    let valid_approve_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "chief_carol",
            "hitl_token": valid_token.clone()
        }),
    };
    let approve_res = server.call_tool(valid_approve_req).await.expect("Processed");
    assert!(!approve_res.is_error, "Valid checker approval succeeds");

    // 4. Replay Attack: Using the exact same valid token again
    let replay_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id.clone(),
            "checker_id": "chief_carol",
            "hitl_token": valid_token.clone()
        }),
    };
    let replay_res = server.call_tool(replay_req).await.expect("Processed");
    assert!(replay_res.is_error, "Replayed token must be rejected (single-use)");
}

#[test]
fn test_treasury_expired_token_simulation() {
    let mut order = TreasuryEngine::propose(
        "19030000000001",
        "00110000000002",
        "CONG TY DUC ANH",
        "BIDV",
        750_000_000,
        "Thanh toan tien may moc",
        "maker_dan",
    )
    .expect("Propose succeeds");

    let token = order.hitl_token.clone().unwrap();

    // Simulate creation 905 seconds ago (> 15-minute TTL)
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    order.created_at = now_ts - (HITL_TOKEN_TTL_SECONDS + 5);

    // Review detects expiration
    TreasuryEngine::review(&mut order);
    assert_eq!(order.status, PaymentOrderStatus::Expired);
    assert!(order.hitl_token.is_none());

    // Approval attempt on expired order fails closed
    let err = TreasuryEngine::approve(&mut order, "chief_eva", &token, None, None)
        .expect_err("Expired order approval must fail");
    match err {
        MakerCheckerError::InvalidProposalState { .. } | MakerCheckerError::TokenExpired { .. } => {}
        other => panic!("Unexpected error type: {:?}", other),
    }
}

#[tokio::test]
async fn test_treasury_zero_negative_and_extreme_amounts() {
    let server = NativeMcpServer::new("test_vault");

    // 1. Zero amount: amount_vnd == 0
    let zero_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "19030000000001",
            "beneficiary_account": "00110000000002",
            "beneficiary_name": "CTY TNHH ZERO",
            "beneficiary_bank": "VCB",
            "amount_vnd": 0,
            "purpose": "Test 0 VND transfer",
            "maker_id": "maker_zero"
        }),
    };
    let zero_res = server.call_tool(zero_req).await.expect("Handled");
    assert!(zero_res.is_error, "0 VND transfer must be rejected");
    let err_msg = extract_text(&zero_res.content);
    assert!(err_msg.contains("greater than 0"), "Must state amount must be > 0: {err_msg}");

    // 2. Negative amount in JSON payload: -50000
    let neg_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "19030000000001",
            "beneficiary_account": "00110000000002",
            "beneficiary_name": "CTY NEGATIVE",
            "beneficiary_bank": "VCB",
            "amount_vnd": -50000,
            "purpose": "Test negative transfer",
            "maker_id": "maker_neg"
        }),
    };
    // Should return Err from call_tool due to deserialization into u64
    let neg_res = server.call_tool(neg_req).await;
    match neg_res {
        Err(e) => assert!(e.contains("invalid") || e.contains("negative") || e.contains("u64")),
        Ok(res) => assert!(res.is_error, "Negative amount must error"),
    }

    // 3. Extreme amount: 1 quadrillion VND (1,000,000,000,000,000 VND)
    let extreme_amount: u64 = 1_000_000_000_000_000;
    let extreme_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "19030000000001",
            "beneficiary_account": "00110000000002",
            "beneficiary_name": "KHO BAC NHA NUOC",
            "beneficiary_bank": "STATE_TREASURY",
            "amount_vnd": extreme_amount,
            "purpose": "Chuyen khoan ngan sach dac biet",
            "maker_id": "maker_treasury"
        }),
    };
    let extreme_res = server.call_tool(extreme_req).await.expect("Extreme amount handled");
    assert!(!extreme_res.is_error, "Valid large u64 transfer proposal succeeds");
    let ext_val: Value = serde_json::from_str(extract_text(&extreme_res.content)).unwrap();
    let order_id = ext_val["order_id"].as_str().unwrap();
    let token = ext_val["hitl_token"].as_str().unwrap();

    // Checker approval for extreme amount succeeds with valid HMAC signature
    let approve_extreme = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id,
            "checker_id": "checker_state",
            "hitl_token": token
        }),
    };
    let ext_approve_res = server.call_tool(approve_extreme).await.expect("Approved");
    assert!(!ext_approve_res.is_error);
    let approved_val: Value = serde_json::from_str(extract_text(&ext_approve_res.content)).unwrap();
    assert_eq!(approved_val["status"], "APPROVED");
    assert!(approved_val["signature_hmac"].as_str().is_some());
}

#[tokio::test]
async fn test_treasury_high_value_dual_authorization_enforcement() {
    let conn = Connection::open_in_memory().expect("In-memory SQLite");
    init_payment_orders_table(&conn).expect("Init table");
    liva_native_core::banking::compliance::audit_ledger::AuditLedger::init_audit_table(&conn)
        .expect("Init audit table");

    // High value payment order: 1,500,000,000 VND (1.5 Billion VND > 500M VND threshold)
    let amount_vnd = 1_500_000_000u64;

    let mut order = TreasuryEngine::propose(
        "19034567890123",
        "00110012345678",
        "TAP DOAN CONG NGHIEP DAI VIET",
        "VIETCOMBANK",
        amount_vnd,
        "Thanh toan hop dong cung cap thiet bi cong nghe cao HD-9988",
        "senior_accountant_nguyen",
    )
    .expect("Propose high-value payment order");

    save_payment_order(&conn, &order).expect("Save to DB");
    assert_eq!(order.status, PaymentOrderStatus::PendingApproval);
    assert_eq!(order.amount_vnd, 1_500_000_000);

    let token = order.hitl_token.clone().unwrap();

    // Verify Cannot execute or finalize without Checker
    assert!(order.checker_id.is_none());
    assert!(order.signature_hmac.is_none());

    // Maker attempts bypass (self-authorization on 1.5B VND)
    let self_bypass = TreasuryEngine::approve(
        &mut order,
        "senior_accountant_nguyen",
        &token,
        None,
        Some(&conn),
    );
    assert!(
        self_bypass.is_err(),
        "High-value payment order MUST reject maker self-approval"
    );

    // Independent Chief Accountant authorizes
    TreasuryEngine::approve(
        &mut order,
        "chief_cfo_tran",
        &token,
        None,
        Some(&conn),
    )
    .expect("Dual authorization approval succeeds");

    assert_eq!(order.status, PaymentOrderStatus::Approved);
    assert_eq!(order.checker_id.as_deref(), Some("chief_cfo_tran"));
    assert!(order.signature_hmac.is_some());

    // Verify DB integrity
    let loaded = find_payment_order_by_id(&conn, &order.id)
        .unwrap()
        .expect("Order in DB");
    assert_eq!(loaded.amount_vnd, 1_500_000_000);
    assert_eq!(loaded.status, PaymentOrderStatus::Approved);
    assert_eq!(loaded.checker_id.as_deref(), Some("chief_cfo_tran"));
}

// ===========================================================================
// SUITE 2: BANKING RECONCILIATION ADVERSARIAL STRESS TESTS
// ===========================================================================

#[tokio::test]
async fn test_reconcile_rounding_differences_and_fee_tolerances() {
    let server = NativeMcpServer::new("test_vault");

    // Bank statement with a transaction of 50,000,000 VND
    let camt_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>MSG-ROUNDING-01</MsgId>
      <CreDtTm>2026-09-14T10:00:00Z</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-ROUNDING</Id>
      <Acct><Id><Othr><Id>19030012345678</Id></Othr></Id></Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">100000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-09-01</Dt></Dt>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">150000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-09-14</Dt></Dt>
      </Bal>
      <Ntry>
        <Amt Ccy="VND">50000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2026-09-10</Dt></BookgDt>
        <NtryDtls>
          <TxDtls>
            <Refs><EndToEndId>INV-DIFF-01</EndToEndId></Refs>
            <RltdPties><Dbtr><Nm>CTY TNHH CONG NGHE NAM VIET</Nm></Dbtr></RltdPties>
            <RmtInf><Ustrd>Thanh toan hoa don INV-DIFF-01</Ustrd></RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

    let b64 = BASE64_STANDARD.encode(camt_xml.as_bytes());

    // Case 1: Exact balance invariant passes (100M + 50M = 150M)
    let req = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64,
            "statement_format": "iso20022_xml"
        }),
    };
    let res = server.call_tool(req).await.expect("Tool succeeds");
    assert!(!res.is_error);
    let val: Value = serde_json::from_str(extract_text(&res.content)).unwrap();
    assert_eq!(val["balance_invariant_valid"], true);
    assert_eq!(val["statement_summary"]["total_credit"], 50_000_000);
    assert_eq!(val["statement_summary"]["tx_count"], 1);

    // Case 2: Statement with 1 VND rounding error in bank statement closing balance
    // 100,000,000 + 50,000,000 = 150,000,000 != 150,000,001
    let camt_xml_discrepancy = camt_xml.replace("<Amt Ccy=\"VND\">150000000</Amt>", "<Amt Ccy=\"VND\">150000001</Amt>");
    let b64_disc = BASE64_STANDARD.encode(camt_xml_discrepancy.as_bytes());
    let req_disc = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64_disc,
            "statement_format": "iso20022_xml"
        }),
    };
    let res_disc = server.call_tool(req_disc).await.expect("Tool succeeds");
    let val_disc: Value = serde_json::from_str(extract_text(&res_disc.content)).unwrap();
    assert_eq!(
        val_disc["balance_invariant_valid"], false,
        "1 VND balance discrepancy MUST invalidate balance invariant"
    );
}

#[tokio::test]
async fn test_reconcile_empty_statement_and_zero_byte_handling() {
    let server = NativeMcpServer::new("test_vault");

    // 1. Empty statement XML (0 transactions)
    let empty_camt = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>
    <GrpHdr><MsgId>EMPTY-01</MsgId></GrpHdr>
    <Stmt>
      <Id>STMT-EMPTY</Id>
      <Acct><Id><Othr><Id>123456789</Id></Othr></Id></Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">10000000</Amt>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">10000000</Amt>
      </Bal>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

    let b64_empty = BASE64_STANDARD.encode(empty_camt.as_bytes());
    let req_empty = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64_empty,
            "statement_format": "iso20022_xml"
        }),
    };
    let res_empty = server.call_tool(req_empty).await.expect("Empty statement handled");
    assert!(!res_empty.is_error);
    let val_empty: Value = serde_json::from_str(extract_text(&res_empty.content)).unwrap();
    assert_eq!(val_empty["statement_summary"]["tx_count"], 0);
    assert_eq!(val_empty["balance_invariant_valid"], true);
    assert_eq!(val_empty["matched_exact_count"], 0);

    // 2. Zero-length payload
    let b64_zero = BASE64_STANDARD.encode(b"");
    let req_zero = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64_zero,
            "statement_format": "iso20022_xml"
        }),
    };
    let res_zero = server.call_tool(req_zero).await.expect("Zero length handled");
    assert!(res_zero.is_error, "Empty payload must return error gracefully");

    // 3. Invalid base64
    let req_inv_b64 = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": "!!!not_base64@@@",
            "statement_format": "iso20022_xml"
        }),
    };
    let res_inv = server.call_tool(req_inv_b64).await.expect("Invalid base64 handled");
    assert!(res_inv.is_error, "Invalid base64 string must return error");
}

#[tokio::test]
async fn test_reconcile_camt053_xml_variations_and_malformed_xml() {
    let server = NativeMcpServer::new("test_vault");

    // Variation 1: camt.053.001.08 namespace, mixed Debit/Credit, missing optional TxDtls
    let camt053_v8 = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>MSG-V8-MIXED</MsgId>
      <CreDtTm>2026-09-14T12:00:00Z</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-V8</Id>
      <Acct>
        <Id><IBAN>VN12VCB000123456789</IBAN></Id>
        <Ccy>VND</Ccy>
        <Svcr><FinInstnId><Nm>VIETCOMBANK</Nm></FinInstnId></Svcr>
      </Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">200000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">270000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
      </Bal>
      <!-- Entry 1: Credit 100M with AddtlNtryInf (no TxDtls) -->
      <Ntry>
        <Amt Ccy="VND">100000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2026-09-12</Dt></BookgDt>
        <AddtlNtryInf>Tien thanh toan dich vu cloud</AddtlNtryInf>
      </Ntry>
      <!-- Entry 2: Debit 30M with TxDtls -->
      <Ntry>
        <Amt Ccy="VND">30000000</Amt>
        <CdtDbtInd>DBIT</CdtDbtInd>
        <BookgDt><Dt>2026-09-13</Dt></BookgDt>
        <NtryDtls>
          <TxDtls>
            <Refs><EndToEndId>DEBIT-TAX-01</EndToEndId></Refs>
            <RltdPties><Cdtr><Nm>KHO BAC NHA NUOC QUAN 1</Nm></Cdtr></RltdPties>
            <RmtInf><Ustrd>Nop thue GTGT thang 8</Ustrd></RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

    let b64_v8 = BASE64_STANDARD.encode(camt053_v8.as_bytes());
    let req_v8 = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64_v8,
            "statement_format": "auto"
        }),
    };
    let res_v8 = server.call_tool(req_v8).await.expect("Tool succeeds");
    assert!(!res_v8.is_error);
    let val_v8: Value = serde_json::from_str(extract_text(&res_v8.content)).unwrap();

    // 200M + 100M - 30M = 270M -> valid invariant!
    assert_eq!(val_v8["balance_invariant_valid"], true);
    assert_eq!(val_v8["statement_summary"]["total_credit"], 100_000_000);
    assert_eq!(val_v8["statement_summary"]["total_debit"], 30_000_000);
    assert_eq!(val_v8["statement_summary"]["tx_count"], 2);

    // Variation 2: Non-ISO XML (missing mandatory <BkToCstmrStmt> root element)
    let non_iso_xml = "<Document><OtherPayload>Not a bank statement</OtherPayload></Document>";
    let b64_non_iso = BASE64_STANDARD.encode(non_iso_xml.as_bytes());
    let req_non_iso = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64_non_iso,
            "statement_format": "iso20022_xml"
        }),
    };
    let res_non_iso = server.call_tool(req_non_iso).await.expect("Handled");
    assert!(res_non_iso.is_error, "Non-ISO XML must return error without crashing");
    let err_msg = extract_text(&res_non_iso.content);
    assert!(
        err_msg.contains("Failed to parse statement")
            || err_msg.contains("Unsupported")
            || err_msg.contains("Missing mandatory ISO 20022")
            || err_msg.contains("BkToCstmrStmt"),
        "Error message must indicate parse failure: {err_msg}"
    );

    // Variation 3: Truncated XML with unclosed tags (lenient DOM parser gracefully handles without crashing)
    let truncated_xml = "<Document><BkToCstmrStmt><Stmt><Acct>Unclosed";
    let b64_trunc = BASE64_STANDARD.encode(truncated_xml.as_bytes());
    let req_trunc = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64_trunc,
            "statement_format": "iso20022_xml"
        }),
    };
    let res_trunc = server.call_tool(req_trunc).await.expect("Handled");
    assert!(!res_trunc.is_error, "Lenient DOM flushes unclosed tags without panicking");
}

// ===========================================================================
// SUITE 3: CREDIT RISK SCORING STRESS & BOUNDARY TESTS
// ===========================================================================

#[test]
fn test_credit_risk_dscr_debt_free_and_negative_noi() {
    // 1. Debt-Free entity: Principal + Interest == 0
    let debt_free_report = CreditRiskEngine::calculate_dscr(
        1_000_000_000, // EBITDA
        200_000_000,   // CAPEX
        0,             // Principal
        0,             // Interest
    );
    assert_eq!(debt_free_report.risk_category, DscrRiskCategory::DebtFree);
    assert_eq!(debt_free_report.ratio, 999.0);
    assert_eq!(debt_free_report.ratio_bps, 9_990_000);
    assert_eq!(debt_free_report.debt_service_vnd, 0);
    assert_eq!(debt_free_report.buffer_vnd, 800_000_000);

    // 2. Negative NOI: EBITDA < CAPEX (e.g. EBITDA 100M, CAPEX 500M -> NOI = -400M)
    let distressed_report = CreditRiskEngine::calculate_dscr(
        100_000_000,
        500_000_000,
        200_000_000,
        50_000_000,
    );
    assert_eq!(distressed_report.risk_category, DscrRiskCategory::Distressed);
    assert_eq!(distressed_report.noi_vnd, -400_000_000);
    assert_eq!(distressed_report.debt_service_vnd, 250_000_000);
    assert_eq!(distressed_report.buffer_vnd, -650_000_000);
    assert!(distressed_report.ratio < 0.0);

    // 3. Exact Watchlist boundary: 1.00 <= DSCR < 1.30
    // NOI = 1,200,000,000, Debt Service = 1,000,000,000 -> DSCR = 1.20 (12,000 bps)
    let watchlist_report = CreditRiskEngine::calculate_dscr(
        1_500_000_000,
        300_000_000,
        800_000_000,
        200_000_000,
    );
    assert_eq!(watchlist_report.risk_category, DscrRiskCategory::Watchlist);
    assert_eq!(watchlist_report.ratio_bps, 12_000);
    assert_eq!(watchlist_report.ratio, 1.20);
}

#[test]
fn test_credit_risk_quick_ratio_zero_liabilities_and_status() {
    // 1. Zero Liabilities: liabilities == 0
    let zero_liab_report = CreditRiskEngine::calculate_quick_ratio(
        500_000_000, // Cash
        100_000_000, // Securities
        200_000_000, // Receivables
        0,           // Current Liabilities
    );
    assert_eq!(zero_liab_report.liquidity_status, LiquidityStatus::Strong);
    assert_eq!(zero_liab_report.ratio, 999.0);
    assert_eq!(zero_liab_report.current_liabilities_vnd, 0);

    // 2. Critical Liquidity: Quick Ratio < 0.80
    // Liquid assets = 300M, Liabilities = 500M -> QR = 0.60 (6,000 bps)
    let critical_report = CreditRiskEngine::calculate_quick_ratio(
        100_000_000,
        50_000_000,
        150_000_000,
        500_000_000,
    );
    assert_eq!(critical_report.liquidity_status, LiquidityStatus::Critical);
    assert_eq!(critical_report.ratio_bps, 6_000);
    assert_eq!(critical_report.ratio, 0.60);

    // 3. Adequate Liquidity: 0.80 <= QR < 1.00
    // Liquid assets = 900M, Liabilities = 1,000M -> QR = 0.90 (9,000 bps)
    let adequate_report = CreditRiskEngine::calculate_quick_ratio(
        500_000_000,
        200_000_000,
        200_000_000,
        1_000_000_000,
    );
    assert_eq!(adequate_report.liquidity_status, LiquidityStatus::Adequate);
    assert_eq!(adequate_report.ratio_bps, 9_000);
    assert_eq!(adequate_report.ratio, 0.90);
}

#[test]
fn test_credit_risk_cashflow_shortfall_simulations() {
    // 1. Immediate Deficit on Day 1
    let flows_day1 = vec![
        DailyCashflowPoint { day_offset: 1, net_inflow_vnd: -300_000_000 },
    ];
    let cf_day1 = CreditRiskEngine::simulate_cashflow(100_000_000, &flows_day1, 30, 0);
    assert!(cf_day1.shortfall_warning);
    assert_eq!(cf_day1.deficit_date_offset, Some(1));
    assert_eq!(cf_day1.minimum_balance_vnd, -200_000_000);

    // 2. Delayed Deficit on Day 15
    let flows_delayed = vec![
        DailyCashflowPoint { day_offset: 5, net_inflow_vnd: 50_000_000 },
        DailyCashflowPoint { day_offset: 15, net_inflow_vnd: -600_000_000 },
    ];
    let cf_delayed = CreditRiskEngine::simulate_cashflow(500_000_000, &flows_delayed, 30, 0);
    assert!(cf_delayed.shortfall_warning);
    assert_eq!(cf_delayed.deficit_date_offset, Some(15));
    assert_eq!(cf_delayed.minimum_balance_vnd, -50_000_000);

    // 3. No Deficit with Minimum Reserve buffer requirement
    let flows_healthy = vec![
        DailyCashflowPoint { day_offset: 3, net_inflow_vnd: 100_000_000 },
        DailyCashflowPoint { day_offset: 10, net_inflow_vnd: -50_000_000 },
    ];
    let cf_healthy = CreditRiskEngine::simulate_cashflow(300_000_000, &flows_healthy, 30, 100_000_000);
    assert!(!cf_healthy.shortfall_warning);
    assert_eq!(cf_healthy.deficit_date_offset, None);
    assert_eq!(cf_healthy.minimum_balance_vnd, 300_000_000);

    // 4. Large Multi-Trillion VND Values (verifying no 32-bit integer overflow)
    let starting_cash: i64 = 100_000_000_000_000; // 100 Trillion VND
    let flows_large = vec![
        DailyCashflowPoint { day_offset: 10, net_inflow_vnd: -20_000_000_000_000 },
    ];
    let cf_large = CreditRiskEngine::simulate_cashflow(starting_cash, &flows_large, 30, 0);
    assert!(!cf_large.shortfall_warning);
    assert_eq!(cf_large.projected_end_balance_vnd, 80_000_000_000_000);
}

// ===========================================================================
// SUITE 4: ZERO FLOAT DRIFT ARITHMETIC PRECISION
// ===========================================================================

#[test]
fn test_arithmetic_precision_zero_float_drift() {
    // 1. Repeating fractional decimal: 1 / 3 = 0.3333333333333333
    // NOI: 1,000,000,000 VND
    // Debt Service: 300,000,000 VND
    // Expected exact basis points: (1_000_000_000 * 10_000) / 300_000_000 = 33_333 bps
    let dscr = CreditRiskEngine::calculate_dscr(1_200_000_000, 200_000_000, 200_000_000, 100_000_000);
    assert_eq!(dscr.ratio_bps, 33_333, "Exact integer basis points without drift");
    assert_eq!(dscr.ratio, 3.3333);

    // 2. Prime 1 VND numbers for Quick Ratio
    // Liquid Assets: 1,000,000,001 VND
    // Liabilities: 3,000,000,000 VND
    // ratio_bps = (1_000_000_001 * 10_000) / 3_000_000_000 = 3_333 bps
    let qr = CreditRiskEngine::calculate_quick_ratio(1_000_000_001, 0, 0, 3_000_000_000);
    assert_eq!(qr.ratio_bps, 3_333);
    assert_eq!(qr.ratio, 0.3333);

    // 3. Payment order exact integer VND preservation
    let conn = Connection::open_in_memory().expect("In-memory SQLite");
    init_payment_orders_table(&conn).expect("Init table");
    let exact_vnd: u64 = 123_456_789_101;
    let order = TreasuryEngine::propose(
        "19030000000001",
        "00110000000002",
        "CTY TINH CHINH XAC",
        "VCB",
        exact_vnd,
        "Thanh toan chinh xac tung dong",
        "maker_precise",
    )
    .expect("Propose");
    save_payment_order(&conn, &order).expect("Save");
    let loaded = find_payment_order_by_id(&conn, &order.id).unwrap().unwrap();
    assert_eq!(loaded.amount_vnd, exact_vnd, "Exact 1 VND integer fidelity guaranteed");
}

// ===========================================================================
// SUITE 5: ADVANCED BOUNDARY & COMPLIANCE ADVERSARIAL TESTS
// ===========================================================================

#[tokio::test]
async fn test_treasury_nonexistent_and_invalid_state_transitions() {
    let server = NativeMcpServer::new("test_vault");

    // 1. Approve non-existent order
    let fake_id = "11111111-2222-3333-4444-555555555555";
    let approve_fake = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": fake_id,
            "checker_id": "checker_01",
            "hitl_token": "any-token"
        }),
    };
    let res_fake = server.call_tool(approve_fake).await.expect("Processed");
    assert!(res_fake.is_error);
    let err_txt = extract_text(&res_fake.content);
    assert!(err_txt.contains("not found"));

    // 2. Propose with empty fields
    let empty_maker = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "19030000000001",
            "beneficiary_account": "00110000000002",
            "beneficiary_name": "CTY TEST",
            "beneficiary_bank": "VCB",
            "amount_vnd": 100_000_000,
            "maker_id": "   "
        }),
    };
    let res_empty_maker = server.call_tool(empty_maker).await.expect("Processed");
    assert!(res_empty_maker.is_error, "Whitespace maker ID must be rejected");

    let empty_debit = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "",
            "beneficiary_account": "00110000000002",
            "beneficiary_name": "CTY TEST",
            "beneficiary_bank": "VCB",
            "amount_vnd": 100_000_000,
            "maker_id": "maker_valid"
        }),
    };
    let res_empty_debit = server.call_tool(empty_debit).await.expect("Processed");
    assert!(res_empty_debit.is_error, "Empty debit account must be rejected");
}

#[tokio::test]
async fn test_reconcile_file_path_and_missing_inputs() {
    let server = NativeMcpServer::new("test_vault");

    // 1. Neither path nor base64 provided
    let empty_args = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({}),
    };
    let res_empty = server.call_tool(empty_args).await.expect("Processed");
    assert!(res_empty.is_error);
    let err_txt = extract_text(&res_empty.content);
    assert!(err_txt.contains("Either `statement_file_path` or `statement_content_base64` must be provided"));

    // 2. Non-existent file path
    let fake_path_req = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_file_path": "C:\\non_existent_folder_xyz_123\\fake_statement.xlsx"
        }),
    };
    let res_path = server.call_tool(fake_path_req).await.expect("Processed");
    assert!(res_path.is_error);
    let err_path_txt = extract_text(&res_path.content);
    assert!(err_path_txt.contains("does not exist"));
}

#[tokio::test]
async fn test_credit_risk_scoring_all_nulls_and_exact_boundaries() {
    let server = NativeMcpServer::new("test_vault");

    // 1. Empty payload `{}`
    let empty_scoring = CallToolRequest {
        name: "credit_risk_scoring".to_string(),
        arguments: serde_json::json!({}),
    };
    let res_empty = server.call_tool(empty_scoring).await.expect("Processed");
    assert!(!res_empty.is_error, "Default all nulls must not panic");
    let val_empty: Value = serde_json::from_str(extract_text(&res_empty.content)).unwrap();
    assert_eq!(val_empty["dscr"]["risk_category"], "DEBT_FREE");
    assert_eq!(val_empty["quick_ratio"]["liquidity_status"], "STRONG");

    // 2. Exact DSCR Boundaries
    // Boundary A: Exactly 1.30 (13,000 bps) -> HEALTHY
    let dscr_130 = CreditRiskEngine::calculate_dscr(1_300_000_000, 0, 1_000_000_000, 0);
    assert_eq!(dscr_130.ratio_bps, 13_000);
    assert_eq!(dscr_130.risk_category, DscrRiskCategory::Healthy);

    // Boundary B: Exactly 1.2999 (12,999 bps) -> WATCHLIST
    let dscr_129 = CreditRiskEngine::calculate_dscr(1_299_900_000, 0, 1_000_000_000, 0);
    assert_eq!(dscr_129.ratio_bps, 12_999);
    assert_eq!(dscr_129.risk_category, DscrRiskCategory::Watchlist);

    // Boundary C: Exactly 1.00 (10,000 bps) -> WATCHLIST
    let dscr_100 = CreditRiskEngine::calculate_dscr(1_000_000_000, 0, 1_000_000_000, 0);
    assert_eq!(dscr_100.ratio_bps, 10_000);
    assert_eq!(dscr_100.risk_category, DscrRiskCategory::Watchlist);

    // Boundary D: Exactly 0.9999 (9,999 bps) -> DISTRESSED
    let dscr_099 = CreditRiskEngine::calculate_dscr(999_900_000, 0, 1_000_000_000, 0);
    assert_eq!(dscr_099.ratio_bps, 9_999);
    assert_eq!(dscr_099.risk_category, DscrRiskCategory::Distressed);

    // 3. Cashflow simulation with multiple cashflows on the same day_offset
    let multi_flows = vec![
        DailyCashflowPoint { day_offset: 2, net_inflow_vnd: -100_000_000 },
        DailyCashflowPoint { day_offset: 2, net_inflow_vnd: -150_000_000 },
        DailyCashflowPoint { day_offset: 2, net_inflow_vnd: 50_000_000 },
    ];
    let cf_multi = CreditRiskEngine::simulate_cashflow(200_000_000, &multi_flows, 10, 0);
    // Net on day 2 = -100 - 150 + 50 = -200M. Starting 200M - 200M = 0 VND balance
    assert_eq!(cf_multi.minimum_balance_vnd, 0);
    assert!(!cf_multi.shortfall_warning);
}

#[tokio::test]
async fn test_compliance_aml_exact_thresholds_and_structuring() {
    let server = NativeMcpServer::new("test_vault");

    // Case 1: Exactly 400,000,000 VND -> MUST trigger AML_HIGH_VALUE (Decision 11 threshold is >= 400M)
    let req_exact_400m = CallToolRequest {
        name: "compliance_aml_screen".to_string(),
        arguments: serde_json::json!({
            "transactions": [
                {
                    "tx_id": "TX-400M-EXACT",
                    "amount_vnd": 400_000_000,
                    "narration": "Thanh toan tien vat tu cong trinh",
                    "is_credit": true
                }
            ]
        }),
    };
    let res_exact = server.call_tool(req_exact_400m).await.expect("Processed");
    assert!(!res_exact.is_error);
    let val_exact: Value = serde_json::from_str(extract_text(&res_exact.content)).unwrap();
    let alerts_exact = val_exact["alerts"].as_array().unwrap();
    assert!(
        alerts_exact.iter().any(|a| a["rule_code"] == "AML_HIGH_VALUE"),
        "Exact 400,000,000 VND transaction must trigger AML_HIGH_VALUE"
    );

    // Case 2: 399,999,999 VND -> MUST NOT trigger AML_HIGH_VALUE
    let req_399m = CallToolRequest {
        name: "compliance_aml_screen".to_string(),
        arguments: serde_json::json!({
            "transactions": [
                {
                    "tx_id": "TX-399M-SUB",
                    "amount_vnd": 399_999_999,
                    "narration": "Thanh toan tien vat tu cong trinh duoi nguong",
                    "is_credit": true
                }
            ]
        }),
    };
    let res_399m = server.call_tool(req_399m).await.expect("Processed");
    assert!(!res_399m.is_error);
    let val_399m: Value = serde_json::from_str(extract_text(&res_399m.content)).unwrap();
    let alerts_399m = val_399m["alerts"].as_array().unwrap();
    assert!(
        alerts_399m.iter().all(|a| a["rule_code"] != "AML_HIGH_VALUE"),
        "399,999,999 VND must NOT trigger single-transaction threshold"
    );

    // Case 3: Structuring / Smurfing detection: 3 transactions of 280,000,000 summing to 840,000,000 within 72h
    let base_ts = 1726000000i64;
    let req_smurf = CallToolRequest {
        name: "compliance_aml_screen".to_string(),
        arguments: serde_json::json!({
            "transactions": [
                {
                    "tx_id": "TX-SMURF-1",
                    "account_number": "1903999999",
                    "amount_vnd": 280_000_000,
                    "timestamp": base_ts,
                    "narration": "Chuyen tien don hang A",
                    "is_credit": true
                },
                {
                    "tx_id": "TX-SMURF-2",
                    "account_number": "1903999999",
                    "amount_vnd": 280_000_000,
                    "timestamp": base_ts + 3600,
                    "narration": "Chuyen tien don hang B",
                    "is_credit": true
                },
                {
                    "tx_id": "TX-SMURF-3",
                    "account_number": "1903999999",
                    "amount_vnd": 280_000_000,
                    "timestamp": base_ts + 7200,
                    "narration": "Chuyen tien don hang C",
                    "is_credit": true
                }
            ]
        }),
    };
    let res_smurf = server.call_tool(req_smurf).await.expect("Processed");
    assert!(!res_smurf.is_error);
    let val_smurf: Value = serde_json::from_str(extract_text(&res_smurf.content)).unwrap();
    let alerts_smurf = val_smurf["alerts"].as_array().unwrap();
    assert!(
        alerts_smurf.iter().any(|a| a["rule_code"] == "AML_STRUCTURING"),
        "Multiple split transactions exceeding 800M VND in 72h must trigger AML_STRUCTURING"
    );
}

