//! Comprehensive Automated Test Suite for LIVA Banking Local AI (Happy-Path Integration).
//!
//! Verifies:
//! 1. Artifact trust check and model loading guard for Qwen3 14B Q5_K_M (`bf19bf5c77c530762012f71215b62aa8f0bae08111ee0d537b8496479e17cfa3`).
//! 2. Multi-dimensional anomaly detection via MCP `compliance_aml_screen` (temporal velocity, near-threshold smurfing, semantic narration).
//! 3. Tier 2 hybrid semantic reconciliation via MCP `reconcile_statement_batch` (and alias `banking_reconcile`) with acronym recognition ("HAGL", "VCS").
//! 4. Credit risk qualitative summary via MCP `credit_risk_scoring` (zero-float DSCR, Quick Ratio, Vietnamese qualitative summary).
//! 5. Deterministic fallback when AI models are disabled or unallocated.
//! 6. Zero Cloud Egress assertion (100% loopback containment).

use base64::prelude::*;
use liva_native_core::{
    embedded_model_hash, verify_model_artifact, verify_trusted_file,
    DEFAULT_EXPERT_MODEL,
};
use liva_native_core::banking::compliance::security::{
    is_egress_permitted, verify_zero_egress, EgressTrafficTracker,
};
use liva_native_core::banking::mcp::{
    BankingReconcileResult, ComplianceAmlScreenResult, CreditRiskScoringResult,
};
use liva_native_core::mcp::protocol::{CallToolRequest, ToolContent};
use liva_native_core::mcp::server::NativeMcpServer;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn extract_text(content: &[ToolContent]) -> &str {
    match content.first() {
        Some(ToolContent::Text { text }) => text.as_str(),
        other => panic!("Expected ToolContent::Text, got: {:?}", other),
    }
}

fn create_temp_dir(prefix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "liva_test_{}_{}_{}",
        prefix,
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&dir).expect("Create temp dir must succeed");
    dir
}

fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// 1. Artifact Trust Check and Model Loading Guard for Qwen3 14B Q5_K_M
// ---------------------------------------------------------------------------

#[test]
fn test_qwen3_14b_artifact_trust_anchor_and_model_loading_guard() {
    const EXPECTED_QWEN3_HASH: &str =
        "bf19bf5c77c530762012f71215b62aa8f0bae08111ee0d537b8496479e17cfa3";
    const EXPECTED_MODEL_NAME: &str = "Qwen_Qwen3-14B-Q5_K_M.gguf";

    // A. Verify embedded trust manifest hash
    let hash = embedded_model_hash(EXPECTED_MODEL_NAME)
        .expect("Qwen3 14B Q5_K_M must be registered in embedded manifest");
    assert_eq!(
        hash, EXPECTED_QWEN3_HASH,
        "Embedded SHA-256 hash must match official trust anchor"
    );

    // B. Verify default expert model constant
    assert_eq!(
        DEFAULT_EXPERT_MODEL, EXPECTED_MODEL_NAME,
        "DEFAULT_EXPERT_MODEL in paths.rs must be Qwen_Qwen3-14B-Q5_K_M.gguf"
    );

    // C. Verify model loading guard on synthetic trusted root
    let temp_root = create_temp_dir("trust_guard");
    let dummy_content = b"TEST_QWEN3_TRUST_CONTENT_STREAM";
    let dummy_hash = compute_sha256(dummy_content);

    let dummy_file = temp_root.join("test_model.gguf");
    fs::write(&dummy_file, dummy_content).expect("Write dummy model");

    let verified = verify_trusted_file(&temp_root, &dummy_file, &dummy_hash)
        .expect("verify_trusted_file must succeed with exact SHA-256");
    assert_eq!(verified, dummy_file.canonicalize().unwrap());

    // D. Loading guard must reject non-GGUF and path traversal
    let non_gguf = temp_root.join("model.bin");
    fs::write(&non_gguf, dummy_content).expect("Write non-gguf file");
    assert!(
        verify_model_artifact(&temp_root, &non_gguf).is_err(),
        "Loading guard must reject non-.gguf extensions"
    );

    let traversal_path = PathBuf::from("../outside.gguf");
    assert!(
        verify_model_artifact(&temp_root, &traversal_path).is_err(),
        "Loading guard must reject path traversal components '..'"
    );

    // E. Physical model inspection if mounted on host
    let host_model_path = PathBuf::from("E:\\AI_Models\\Qwen_Qwen3-14B-Q5_K_M.gguf");
    if host_model_path.exists() {
        let meta = fs::metadata(&host_model_path).expect("Read metadata");
        assert_eq!(
            meta.len(),
            10_514_570_272,
            "Host Qwen3 14B GGUF file size must be exactly 10,514,570,272 bytes"
        );
    }

    let _ = fs::remove_dir_all(temp_root);
}

// ---------------------------------------------------------------------------
// 2. ONNX Multi-Dimensional Anomaly Detection via MCP `compliance_aml_screen`
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_onnx_multidimensional_anomaly_detection_mcp() {
    let server = NativeMcpServer::new("test_vault");

    // Synthetic test transactions covering multi-dimensional vectors:
    // 1. Near-threshold structuring/smurfing: 388,500,000 VND (in [380M, 400M))
    // 2. Off-hours night anomaly: 250,000,000 VND at 02:00 AM GMT+7
    // 3. Semantic narration anomaly: crypto OTC / money mule keywords
    // 4. Benign normal transaction: 15,000,000 VND daytime electricity bill
    const REF_MIDNIGHT_GMT7_UTC: i64 = 1726333200;
    let t_day = REF_MIDNIGHT_GMT7_UTC + (10 * 3600); // Daytime 10:00 AM GMT+7
    let t_night = REF_MIDNIGHT_GMT7_UTC + 7200;      // Night 02:00 AM GMT+7

    let req = CallToolRequest {
        name: "compliance_aml_screen".to_string(),
        arguments: serde_json::json!({
            "transactions": [
                {
                    "tx_id": "TX-SMURF-01",
                    "account_number": "0071000999",
                    "counterparty_name": "Tran Van Binh",
                    "counterparty_account": "0071000999",
                    "amount_vnd": 388_500_000,
                    "timestamp": t_day,
                    "narration": "Chuyen tien thanh toan tien mua vat tu hop dong so 12",
                    "is_credit": true
                },
                {
                    "tx_id": "TX-NIGHT-02",
                    "account_number": "0011000888",
                    "counterparty_name": "Le Thi Thu",
                    "counterparty_account": "0011000888",
                    "amount_vnd": 250_000_000,
                    "timestamp": t_night,
                    "narration": "Chuyen khoan dem khuya thanh toan",
                    "is_credit": true
                },
                {
                    "tx_id": "TX-SEMANTIC-03",
                    "account_number": "0021000777",
                    "counterparty_name": "Pham Quoc Huy",
                    "counterparty_account": "0021000777",
                    "amount_vnd": 50_000_000,
                    "timestamp": t_day + 3600,
                    "narration": "Chuyen tien tay ba doi ngoai te chui chi ngoai so tai khoan muon",
                    "is_credit": true
                },
                {
                    "tx_id": "TX-BENIGN-04",
                    "counterparty_name": "CONG TY TNHH DIEN LUC HA NOI",
                    "counterparty_account": "0091000666",
                    "amount_vnd": 15_000_000,
                    "timestamp": t_day + 7200,
                    "narration": "Thanh toan tien dien thang 09 cho van phong",
                    "is_credit": false
                }
            ],
            "redact_pii": true
        }),
    };

    let result = server.call_tool(req).await.expect("Call tool must succeed");
    assert!(!result.is_error, "compliance_aml_screen must not return error");

    let text = extract_text(&result.content);
    let parsed: ComplianceAmlScreenResult =
        serde_json::from_str(text).expect("Must parse ComplianceAmlScreenResult");

    assert_eq!(parsed.total_screened, 4);
    assert!(parsed.zero_egress_verified, "Must verify Zero Data Egress");

    // Aggregate Local AI Anomaly Score must be present and elevated
    assert!(
        parsed.ai_anomaly_score.is_some(),
        "Aggregate AI anomaly score must be populated"
    );
    let score = parsed.ai_anomaly_score.unwrap();
    assert!(score > 0.0, "Aggregate anomaly score must be > 0.0");
    assert!(score <= 1.0, "Aggregate anomaly score must be <= 1.0");

    assert!(
        parsed.anomaly_reasons.is_some(),
        "Anomaly reasons list must be populated"
    );
    assert!(
        parsed.risk_profile.is_some(),
        "Risk profile category must be populated"
    );

    // Verify individual alerts triggered
    let smurf_alert = parsed
        .alerts
        .iter()
        .find(|a| a.tx_id == "TX-SMURF-01" && a.rule_code == "AML_NEAR_THRESHOLD_SMURFING");
    assert!(
        smurf_alert.is_some(),
        "TX-SMURF-01 must trigger AML_NEAR_THRESHOLD_SMURFING"
    );

    let night_alert = parsed
        .alerts
        .iter()
        .find(|a| a.tx_id == "TX-NIGHT-02" && a.rule_code == "AML_NIGHT_ANOMALY");
    assert!(
        night_alert.is_some(),
        "TX-NIGHT-02 must trigger AML_NIGHT_ANOMALY"
    );

    let semantic_alert = parsed
        .alerts
        .iter()
        .find(|a| a.tx_id == "TX-SEMANTIC-03" && a.rule_code == "AML_SEMANTIC_NARRATION_ANOMALY");
    assert!(
        semantic_alert.is_some(),
        "TX-SEMANTIC-03 must trigger AML_SEMANTIC_NARRATION_ANOMALY"
    );

    // Verify Vietnamese explanation presence and Decree 13 PII masking
    for alert in &parsed.alerts {
        assert!(alert.ai_anomaly_score.is_some());
        if let Some(ref exp) = alert.vietnamese_explanation {
            assert!(!exp.is_empty(), "Vietnamese explanation must not be empty");
            assert!(
                !exp.contains("Tran Van Binh") && !exp.contains("Le Thi Thu"),
                "Vietnamese explanation must not leak natural person names"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Tier 2 Hybrid Semantic Reconciliation via MCP `reconcile_statement_batch`
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_tier2_hybrid_semantic_reconciliation_mcp() {
    let server = NativeMcpServer::new("test_vault");
    let temp_dir = create_temp_dir("recon_mcp");

    // 1. Prepare ERP Ledger Open Invoices:
    // - Invoice 1: "Cong ty Co phan Hoang Anh Gia Lai" (doc: INV-HAGL-2026) -> Acronym: HAGL
    // - Invoice 2: "Tong Cong ty Than Vinacomin" (doc: INV-VCS-2026) -> Corporate alias: VCS
    // - Invoice 3: "CONG TY TNHH SAO MAI" (doc: INV-REG-2026) -> Exact doc_ref match
    let erp_invoices = serde_json::json!([
        {
            "id": "LEDGER-001",
            "account_id": "0011000123456",
            "doc_no": "INV-HAGL-2026",
            "entry_date": 1789516800,
            "entry_type": "CREDIT",
            "amount": 50_000_000,
            "partner_code": "HAGL",
            "partner_name": "Cong ty Co phan Hoang Anh Gia Lai",
            "description": "Tien ban hang nong san HAGL",
            "reconciled_status": "UNMATCHED",
            "created_at": 1789516800
        },
        {
            "id": "LEDGER-002",
            "account_id": "0011000123456",
            "doc_no": "INV-VCS-2026",
            "entry_date": 1789516800,
            "entry_type": "CREDIT",
            "amount": 120_000_000,
            "partner_code": "VCS",
            "partner_name": "Tong Cong ty Than Vinacomin",
            "description": "Cung cap nguyen lieu than",
            "reconciled_status": "UNMATCHED",
            "created_at": 1789516800
        },
        {
            "id": "LEDGER-003",
            "account_id": "0011000123456",
            "doc_no": "INV-REG-2026",
            "entry_date": 1789516800,
            "entry_type": "CREDIT",
            "amount": 30_000_000,
            "partner_code": "SAOMAI",
            "partner_name": "CONG TY TNHH SAO MAI",
            "description": "Dich vu van phong",
            "reconciled_status": "UNMATCHED",
            "created_at": 1789516800
        }
    ]);

    let ledger_path = temp_dir.join("erp_ledger.json");
    fs::write(&ledger_path, erp_invoices.to_string()).expect("Write erp ledger");

    // 2. Prepare ISO 20022 camt.053 XML Statement:
    // Opening balance: 200,000,000 VND
    // Total Credits: 50,000,000 + 120,000,000 + 30,000,000 = 200,000,000 VND
    // Closing balance: 400,000,000 VND (Invariant 100% Valid)
    let camt053_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>MSG-20260916-RECON</MsgId>
      <CreDtTm>2026-09-16T08:30:00Z</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-2026-09-16</Id>
      <Acct>
        <Id><Othr><Id>0011000123456</Id></Othr></Id>
      </Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">200000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-09-01</Dt></Dt>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">400000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-09-16</Dt></Dt>
      </Bal>
      <Ntry>
        <Amt Ccy="VND">50000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Sts>BOOK</Sts>
        <BookgDt><Dt>2026-09-16</Dt></BookgDt>
        <ValDt><Dt>2026-09-16</Dt></ValDt>
        <AcctSvcrRef>REF-HAGL-01</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs><EndToEndId>TX-HAGL-01</EndToEndId></Refs>
            <RltdPties>
              <Dbtr><Nm>HAGL</Nm></Dbtr>
            </RltdPties>
            <RmtInf><Ustrd>HAGL thanh toan dot 1 hop dong cung cap</Ustrd></RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
      <Ntry>
        <Amt Ccy="VND">120000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Sts>BOOK</Sts>
        <BookgDt><Dt>2026-09-16</Dt></BookgDt>
        <ValDt><Dt>2026-09-16</Dt></ValDt>
        <AcctSvcrRef>REF-VCS-02</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs><EndToEndId>TX-VCS-02</EndToEndId></Refs>
            <RltdPties>
              <Dbtr><Nm>VCS</Nm></Dbtr>
            </RltdPties>
            <RmtInf><Ustrd>VCS thanh toan tien than quy 3</Ustrd></RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
      <Ntry>
        <Amt Ccy="VND">30000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Sts>BOOK</Sts>
        <BookgDt><Dt>2026-09-16</Dt></BookgDt>
        <ValDt><Dt>2026-09-16</Dt></ValDt>
        <AcctSvcrRef>INV-REG-2026</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs><EndToEndId>INV-REG-2026</EndToEndId></Refs>
            <RltdPties>
              <Dbtr><Nm>CONG TY TNHH SAO MAI</Nm></Dbtr>
            </RltdPties>
            <RmtInf><Ustrd>Thanh toan hoa don INV-REG-2026</Ustrd></RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

    let b64 = BASE64_STANDARD.encode(camt053_xml.as_bytes());

    // 3. Dispatch tool `reconcile_statement_batch` with `enable_semantic_recon: true`
    let req = CallToolRequest {
        name: "reconcile_statement_batch".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64,
            "statement_format": "iso20022_xml",
            "erp_ledger_path": ledger_path.to_string_lossy(),
            "enable_semantic_recon": true
        }),
    };

    let result = server.call_tool(req).await.expect("Call tool must succeed");
    assert!(!result.is_error, "Reconciliation must not return error");

    let text = extract_text(&result.content);
    let parsed: BankingReconcileResult =
        serde_json::from_str(text).expect("Must parse BankingReconcileResult");

    // Invariant assertions
    assert!(
        parsed.balance_invariant_valid,
        "Mathematical balance invariant Opening + Inflows - Outflows == Closing must hold"
    );
    assert_eq!(parsed.statement_summary.opening_balance, 200_000_000);
    assert_eq!(parsed.statement_summary.closing_balance, 400_000_000);
    assert_eq!(parsed.statement_summary.total_credit, 200_000_000);
    assert_eq!(parsed.discrepancies_count, 0);

    // Semantic reconciliation telemetry:
    // HAGL and VCS elevated via Tier 2 Semantic Acronym matching
    assert!(
        parsed.matched_semantic_count >= 2,
        "Expected at least 2 semantic matches (HAGL & VCS), got: {}",
        parsed.matched_semantic_count
    );

    // INV-REG-2026 matched via exact doc_ref
    assert!(
        parsed.matched_exact_count >= 1,
        "Expected at least 1 exact match (INV-REG-2026), got: {}",
        parsed.matched_exact_count
    );

    let _ = fs::remove_dir_all(temp_dir);
}

// ---------------------------------------------------------------------------
// 4. Credit Risk Qualitative Summary via MCP `credit_risk_scoring`
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_credit_risk_qualitative_summary_mcp() {
    let server = NativeMcpServer::new("test_vault");

    let req = CallToolRequest {
        name: "credit_risk_scoring".to_string(),
        arguments: serde_json::json!({
            "ebitda_vnd": 2_400_000_000i64,
            "capex_vnd": 400_000_000i64,
            "debt_service_principal_vnd": 600_000_000i64,
            "debt_service_interest_vnd": 200_000_000i64,
            "cash_and_equivalents_vnd": 500_000_000i64,
            "marketable_securities_vnd": 150_000_000i64,
            "accounts_receivable_vnd": 450_000_000i64,
            "current_liabilities_vnd": 700_000_000i64,
            "forecast_days": 30,
            "historical_cashflow": [
                { "day_offset": 3, "net_inflow_vnd": 100_000_000i64 },
                { "day_offset": 10, "net_inflow_vnd": -200_000_000i64 }
            ]
        }),
    };

    let result = server.call_tool(req).await.expect("Call tool must succeed");
    assert!(!result.is_error, "credit_risk_scoring must succeed");

    let text = extract_text(&result.content);
    let parsed: CreditRiskScoringResult =
        serde_json::from_str(text).expect("Must parse CreditRiskScoringResult");

    // DSCR = (2.4B - 0.4B) / (0.6B + 0.2B) = 2.0B / 0.8B = 2.50 => HEALTHY
    assert_eq!(parsed.dscr.ratio, 2.5);
    assert_eq!(parsed.dscr.risk_category, "HEALTHY");
    assert_eq!(parsed.dscr.buffer_vnd, 1_200_000_000); // 2.0B - 0.8B = 1.2B

    // Quick Ratio = (500M + 150M + 450M) / 700M = 1.1B / 0.7B = 1.5714... => STRONG
    assert_eq!(parsed.quick_ratio.liquidity_status, "STRONG");
    assert!(
        parsed.quick_ratio.ratio > 1.57 && parsed.quick_ratio.ratio < 1.58,
        "Quick Ratio must be ~1.5714"
    );

    // Qualitative Risk Summary synthesis check
    let summary = &parsed.qualitative_risk_summary;
    assert!(!summary.is_empty(), "Qualitative risk summary must not be empty");
    assert!(
        summary.contains("DSCR") || summary.contains("khả năng"),
        "Summary must cite financial debt service capacity: {summary}"
    );
    assert!(
        summary.contains("thanh toán nhanh") || summary.contains("thanh khoản"),
        "Summary must evaluate liquidity position: {summary}"
    );
}

// ---------------------------------------------------------------------------
// 5. Deterministic Fallback when AI Models are Disabled or Unallocated
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_deterministic_fallback_when_ai_models_disabled() {
    let server = NativeMcpServer::new("test_vault");
    let temp_dir = create_temp_dir("fallback_test");

    // A. Reconcile with `enable_semantic_recon: false` -> Pure deterministic fallback
    let erp_invoices = serde_json::json!([
        {
            "id": "LEDGER-HAGL",
            "account_id": "0011000123456",
            "doc_no": "INV-DISTANT-01",
            "entry_date": 1726300800,
            "entry_type": "CREDIT",
            "amount": 50_000_000,
            "partner_code": "HAGL",
            "partner_name": "Cong ty Co phan Hoang Anh Gia Lai",
            "description": "Tien hang",
            "reconciled_status": "UNMATCHED",
            "created_at": 1726300800
        }
    ]);
    let ledger_path = temp_dir.join("fallback_ledger.json");
    fs::write(&ledger_path, erp_invoices.to_string()).expect("Write ledger");

    let camt053_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>
    <GrpHdr><MsgId>MSG-FALLBACK</MsgId><CreDtTm>2026-09-16T08:30:00Z</CreDtTm></GrpHdr>
    <Stmt>
      <Id>STMT-FALLBACK</Id>
      <Acct><Id><Othr><Id>0011000123456</Id></Othr></Id></Acct>
      <Bal><Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp><Amt Ccy="VND">100000000</Amt><CdtDbtInd>CRDT</CdtDbtInd><Dt><Dt>2026-09-01</Dt></Dt></Bal>
      <Bal><Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp><Amt Ccy="VND">150000000</Amt><CdtDbtInd>CRDT</CdtDbtInd><Dt><Dt>2026-09-16</Dt></Dt></Bal>
      <Ntry>
        <Amt Ccy="VND">50000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Sts>BOOK</Sts>
        <BookgDt><Dt>2026-09-16</Dt></BookgDt>
        <ValDt><Dt>2026-09-16</Dt></ValDt>
        <NtryDtls><TxDtls><Refs><EndToEndId>TX-ACRONYM-ONLY</EndToEndId></Refs><RmtInf><Ustrd>HAGL thanh toan nong san</Ustrd></RmtInf></TxDtls></NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

    let b64 = BASE64_STANDARD.encode(camt053_xml.as_bytes());

    let req_disabled = CallToolRequest {
        name: "reconcile_statement_batch".to_string(),
        arguments: serde_json::json!({
            "statement_content_base64": b64,
            "statement_format": "iso20022_xml",
            "erp_ledger_path": ledger_path.to_string_lossy(),
            "enable_semantic_recon": false // EXPLICITLY DISABLED!
        }),
    };

    let res_disabled = server.call_tool(req_disabled).await.expect("Call succeeds");
    assert!(!res_disabled.is_error);

    let text_disabled = extract_text(&res_disabled.content);
    let parsed_disabled: BankingReconcileResult =
        serde_json::from_str(text_disabled).expect("Parse result");

    // When semantic recon is disabled, candidate requiring acronym elevation must NOT match
    assert_eq!(
        parsed_disabled.matched_semantic_count, 0,
        "matched_semantic_count must be 0 when enable_semantic_recon is false"
    );
    assert!(
        parsed_disabled.balance_invariant_valid,
        "Mathematical balance invariant holds even during fallback"
    );

    // B. Compliance screening fallback when neural weights are unallocated
    let aml_req = CallToolRequest {
        name: "compliance_aml_screen".to_string(),
        arguments: serde_json::json!({
            "transactions": [
                {
                    "tx_id": "TX-FALLBACK-01",
                    "counterparty_name": "Doi Tac Fallback",
                    "counterparty_account": "00998877",
                    "amount_vnd": 450_000_000, // Statutory threshold rule
                    "timestamp": 1789527600,
                    "narration": "Thanh toan tien hang hop le",
                    "is_credit": true
                }
            ]
        }),
    };

    let aml_res = server.call_tool(aml_req).await.expect("Call succeeds");
    assert!(!aml_res.is_error);
    let aml_parsed: ComplianceAmlScreenResult =
        serde_json::from_str(extract_text(&aml_res.content)).expect("Parse AML result");
    assert_eq!(aml_parsed.total_screened, 1);
    assert!(aml_parsed.alerts.iter().any(|a| a.rule_code == "AML_HIGH_VALUE"));

    let _ = fs::remove_dir_all(temp_dir);
}

// ---------------------------------------------------------------------------
// 6. Zero Cloud Egress Assertion (100% Loopback Containment)
// ---------------------------------------------------------------------------

#[test]
fn test_zero_cloud_egress_100_percent_loopback_containment() {
    // 1. Assert host and gateway listeners are bound strictly to loopback
    let (is_isolated, listeners) = verify_zero_egress();
    assert!(
        is_isolated,
        "Zero Egress environment inspection must report 100% isolated loopback"
    );
    for listener in &listeners {
        assert!(
            listener.contains("127.0.0.1") || listener.contains("localhost") || listener.contains("::1"),
            "Listener '{listener}' must resolve to loopback interface"
        );
    }

    // 2. Test netfilter destination authorization
    assert!(is_egress_permitted("127.0.0.1:8002"));
    assert!(is_egress_permitted("http://localhost:8000/v1/status"));
    assert!(is_egress_permitted("[::1]:8080"));

    // Prohibited cloud domains must be strictly denied
    assert!(!is_egress_permitted("https://api.openai.com/v1/chat/completions"));
    assert!(!is_egress_permitted("http://aws.amazon.com/s3/vault"));
    assert!(!is_egress_permitted("8.8.8.8:53"));
    assert!(!is_egress_permitted("192.168.1.50:9000"));

    // 3. Assert global egress tracker reports 0 external bytes
    let tracker = EgressTrafficTracker::new();
    assert!(
        tracker.record_egress("127.0.0.1:8002", 4096).is_ok(),
        "Loopback traffic must be permitted"
    );
    assert!(
        tracker.assert_zero_external_egress().is_ok(),
        "External egress bytes must remain strictly 0"
    );

    let report = tracker.report();
    assert!(report.is_zero_egress);
    assert_eq!(report.external_bytes_transmitted, 0);
    assert_eq!(report.loopback_bytes_transmitted, 4096);
}
