//! Integration tests for Banking Operations MCP Tool Suite (Milestone M1).
//!
//! Tests tool discovery (`list_tools`), category classification (`list_skills`),
//! and end-to-end dispatch (`call_tool`) for:
//! 1. `banking_reconcile`
//! 2. `treasury_payment_order` (Maker-Checker dual control lifecycle & self-approval fail-closed)
//! 3. `compliance_aml_screen` (Decision 11 AML rules, Decree 13 PII masking, Zero-Egress assertion)
//! 4. `credit_risk_scoring` (Zero-float-drift DSCR basis points, Quick Ratio, and cashflow simulation)

use base64::prelude::*;
use liva_native_core::db::DatabasePool;
use liva_native_core::mcp::protocol::{CallToolRequest, ToolContent};
use liva_native_core::mcp::server::NativeMcpServer;
use serde_json::Value;

fn extract_text(content: &[ToolContent]) -> &str {
    match content.first() {
        Some(ToolContent::Text { text }) => text.as_str(),
        other => panic!("Expected ToolContent::Text, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_banking_mcp_tool_listing_and_skills() {
    let server = NativeMcpServer::new("test_vault");
    let tool_list = server.list_tools();

    let banking_tools = [
        "banking_reconcile",
        "treasury_payment_order",
        "compliance_aml_screen",
        "credit_risk_scoring",
    ];

    for tool_name in &banking_tools {
        let tool = tool_list
            .tools
            .iter()
            .find(|t| t.name == *tool_name)
            .unwrap_or_else(|| panic!("Tool '{}' must be registered in list_tools", tool_name));

        assert!(
            !tool.description.is_empty(),
            "Tool '{}' description must not be empty",
            tool_name
        );
        let schema_val = serde_json::to_value(&tool.input_schema)
            .expect("Input schema must serialize to JSON Value");
        assert!(
            schema_val.get("properties").is_some() || schema_val.get("type").is_some(),
            "Tool '{}' must declare valid schema properties",
            tool_name
        );
    }

    // Verify list_skills category classification
    let skills = server.list_skills();
    for tool_name in &banking_tools {
        let skill = skills
            .iter()
            .find(|s| s["name"].as_str() == Some(tool_name))
            .unwrap_or_else(|| panic!("Skill '{}' must be found in list_skills", tool_name));

        assert_eq!(
            skill["category"].as_str(),
            Some("banking"),
            "Tool '{}' category must be 'banking'",
            tool_name
        );
    }
}

#[tokio::test]
async fn test_banking_reconcile_tool_execution() {
    let server = NativeMcpServer::new("test_vault");

    let camt053_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>MSG-20260914-001</MsgId>
      <CreDtTm>2026-09-14T08:30:00Z</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-2026-09</Id>
      <Acct>
        <Id><Othr><Id>0011000123456</Id></Othr></Id>
      </Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">500000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-09-01</Dt></Dt>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">550000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-09-14</Dt></Dt>
      </Bal>
      <Ntry>
        <Amt Ccy="VND">50000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Sts>BOOK</Sts>
        <BookgDt><Dt>2026-09-10</Dt></BookgDt>
        <ValDt><Dt>2026-09-10</Dt></ValDt>
        <AcctSvcrRef>REF-2026-001</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs><EndToEndId>INV-2026-001</EndToEndId></Refs>
            <RltdPties>
              <Dbtr><Nm>CTY CP DAU TU ABC</Nm></Dbtr>
            </RltdPties>
            <RmtInf><Ustrd>Thanh toan hoa don INV-2026-001</Ustrd></RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

    let b64 = BASE64_STANDARD.encode(camt053_xml.as_bytes());

    let req = CallToolRequest {
        name: "banking_reconcile".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64,
            "statement_format": "iso20022_xml"
        }),
    };

    let result = server.call_tool(req).await.expect("Call tool must succeed");
    assert!(!result.is_error, "Reconciliation must not return error flag");

    let text = extract_text(&result.content);
    let parsed: Value = serde_json::from_str(text).expect("Result text must be valid JSON");

    assert_eq!(parsed["statement_summary"]["opening_balance"], 500_000_000);
    assert_eq!(parsed["statement_summary"]["closing_balance"], 550_000_000);
    assert_eq!(parsed["statement_summary"]["total_credit"], 50_000_000);
    assert_eq!(parsed["balance_invariant_valid"], true);
}

#[tokio::test]
async fn test_treasury_payment_order_maker_checker_lifecycle_with_db() {
    let db_pool = DatabasePool::new_in_memory().expect("In-memory database pool creates");
    let server = NativeMcpServer::new("test_vault").with_db_pool(db_pool);

    // 1. Propose payment order (Maker)
    let propose_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "propose",
            "debit_account": "0011000123456",
            "beneficiary_account": "0071000987654",
            "beneficiary_name": "CTY TNHH CONG NGHE PHAN MEM XYZ",
            "beneficiary_bank": "VCB",
            "amount_vnd": 250000000,
            "purpose": "Thanh toan tien phan mem ERP thang 9",
            "maker_id": "accountant_maker_01"
        }),
    };

    let propose_res = server.call_tool(propose_req).await.expect("Call must succeed");
    assert!(!propose_res.is_error);
    let text = extract_text(&propose_res.content);
    let propose_val: Value = serde_json::from_str(text).expect("JSON response");

    assert_eq!(propose_val["status"], "PENDING_APPROVAL");
    let order_id = propose_val["order_id"].as_str().expect("order_id exists").to_string();
    let hitl_token = propose_val["hitl_token"].as_str().expect("hitl_token exists").to_string();

    // 2. Strict Self-Approval Prevention: Maker attempts to approve their own order
    let self_approve_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id,
            "checker_id": "accountant_maker_01", // SAME AS MAKER!
            "hitl_token": hitl_token
        }),
    };

    let self_approve_res = server.call_tool(self_approve_req).await.expect("Call completed");
    assert!(
        self_approve_res.is_error,
        "Self-approval MUST fail closed under Circular 09 Dual Control"
    );
    let err_text = extract_text(&self_approve_res.content);
    assert!(
        err_text.contains("Self-approval") || err_text.contains("SelfApprovalProhibited"),
        "Error message must explicitly cite self-approval restriction: {err_text}"
    );

    // 3. Legitimate Checker Approval
    let valid_approve_req = CallToolRequest {
        name: "treasury_payment_order".to_string(),
        arguments: serde_json::json!({
            "action": "approve",
            "order_id": order_id,
            "checker_id": "chief_accountant_checker_02", // DIFFERENT CHECKER!
            "hitl_token": hitl_token
        }),
    };

    let valid_approve_res = server.call_tool(valid_approve_req).await.expect("Call completed");
    assert!(!valid_approve_res.is_error, "Checker approval must succeed");
    let approve_text = extract_text(&valid_approve_res.content);
    let approve_val: Value = serde_json::from_str(approve_text).expect("JSON response");

    assert_eq!(approve_val["status"], "APPROVED");
    assert_eq!(approve_val["checker_id"], "chief_accountant_checker_02");
    assert!(
        approve_val["signature_hmac"].as_str().is_some(),
        "Approved payment order must possess cryptographic HMAC-SHA256 signature"
    );
}

#[tokio::test]
async fn test_compliance_aml_screen_tool_execution() {
    let server = NativeMcpServer::new("test_vault");

    let req = CallToolRequest {
        name: "compliance_aml_screen".to_string(),
        arguments: serde_json::json!({
            "transactions": [
                {
                    "tx_id": "TX-01",
                    "counterparty_name": "Nguyen Van A",
                    "counterparty_account": "0987654321",
                    "amount_vnd": 500000000, // Exceeds 400M threshold
                    "timestamp": 1726300800,
                    "narration": "Chuyen tien thanh toan sdt 0912345678 CCCD: 001095012345 cho CTY TNHH KINH DOANH ABC",
                    "is_credit": true
                }
            ],
            "redact_pii": true
        }),
    };

    let result = server.call_tool(req).await.expect("Call must succeed");
    assert!(!result.is_error);

    let text = extract_text(&result.content);
    let parsed: Value = serde_json::from_str(text).expect("Valid JSON");

    assert_eq!(parsed["total_screened"], 1);
    assert_eq!(parsed["zero_egress_verified"], true);

    // Verify AML threshold alert triggered
    let alerts = parsed["alerts"].as_array().expect("alerts array");
    assert!(
        alerts.iter().any(|a| a["rule_code"] == "AML_HIGH_VALUE"),
        "Must trigger Decision 11/2023 threshold rule for >= 400M VND (AML_HIGH_VALUE)"
    );

    // Verify Decree 13 PII sanitization
    let sanitized = parsed["sanitized_transactions"].as_array().expect("sanitized txs");
    assert_eq!(sanitized.len(), 1);
    let narr = sanitized[0]["masked_narration"].as_str().unwrap();
    assert!(!narr.contains("0912345678"), "Phone number must be masked");
    assert!(!narr.contains("001095012345"), "CCCD must be masked");
    assert!(
        narr.contains("CTY TNHH KINH DOANH ABC"),
        "Legal entity company name must be preserved in narration"
    );
}

#[tokio::test]
async fn test_credit_risk_scoring_tool_execution() {
    let server = NativeMcpServer::new("test_vault");

    let req = CallToolRequest {
        name: "credit_risk_scoring".to_string(),
        arguments: serde_json::json!({
            "ebitda_vnd": 1500000000,
            "capex_vnd": 200000000,
            "debt_service_principal_vnd": 600000000,
            "debt_service_interest_vnd": 200000000,
            "cash_and_equivalents_vnd": 400000000,
            "marketable_securities_vnd": 100000000,
            "accounts_receivable_vnd": 500000000,
            "current_liabilities_vnd": 800000000,
            "forecast_days": 30,
            "historical_cashflow": [
                { "day_offset": 5, "net_inflow_vnd": -600000000 }
            ]
        }),
    };

    let result = server.call_tool(req).await.expect("Call must succeed");
    assert!(!result.is_error);

    let text = extract_text(&result.content);
    let parsed: Value = serde_json::from_str(text).expect("Valid JSON");

    // DSCR = (1.5B - 0.2B) / (0.6B + 0.2B) = 1.3B / 0.8B = 1.625 => HEALTHY
    assert_eq!(parsed["dscr"]["risk_category"], "HEALTHY");
    assert_eq!(parsed["dscr"]["ratio"], 1.625);

    // Quick Ratio = (400M + 100M + 500M) / 800M = 1.0B / 0.8B = 1.25 => STRONG
    assert_eq!(parsed["quick_ratio"]["liquidity_status"], "STRONG");
    assert_eq!(parsed["quick_ratio"]["ratio"], 1.25);

    // Deficit simulation: deficit on day offset 5
    assert_eq!(parsed["cashflow_forecast"]["shortfall_warning"], true);
    assert_eq!(parsed["cashflow_forecast"]["deficit_date_offset"], 5);
}
