//! Banking MCP Tool Suite definitions and execution handlers.
//!
//! Provides 4 standardized Model Context Protocol (MCP) tools for LIVA Banking Harness:
//! 1. `banking_reconcile`: Bank statement ingestion (VCB, TCB, BIDV, ISO 20022 XML) and 3-Tier deterministic reconciliation.
//! 2. `treasury_payment_order`: Circular 09 Dual Control payment order proposal, review, and approval lifecycle.
//! 3. `compliance_aml_screen`: Decision 11/2023/QĐ-TTg AML/CTF rules screening and Decree 13 PII sanitization.
//! 4. `credit_risk_scoring`: Zero-float-drift DSCR solvency, Quick Ratio liquidity, and rolling deficit forecasting.

use crate::banking::compliance::aml::{
    AmlScreeningEngine, AmlSeverity, TransactionScreeningItem,
};
use crate::banking::compliance::sanitizer::{sanitize_counterparty_name, sanitize_pii};
use crate::banking::compliance::security::verify_zero_egress;
use crate::banking::models::{
    BankStatement, BankTransactionRow, BankType, InternalLedgerEntry, MatchType,
    ReconciliationStatus, StatementFormat, TransactionRecord, TransactionType,
};
use crate::banking::parser::sniff_and_parse;
use crate::banking::reconciliation::ReconciliationEngine;
use crate::banking::risk::{CreditRiskEngine, DailyCashflowPoint};
use crate::banking::treasury::{
    find_payment_order_by_id, init_payment_orders_table, save_payment_order, update_payment_order,
    PaymentOrder, TreasuryEngine,
};
use liva_money::Money;
use liva_parse::{CanonicalTx, StatementEngine};
use liva_recon::dsl::FeeScheduleConfig;
use liva_recon::{DeclarativeReconEngine, InternalInvoice};
use base64::prelude::*;
use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// 1. `banking_reconcile` / `reconcile_statement_batch`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct BankingReconcileArgs {
    /// Absolute path to bank statement file (XLSX, CSV, PDF, XML).
    pub statement_file_path: Option<String>,
    /// Optional base64-encoded statement content.
    pub statement_content_base64: Option<String>,
    /// Statement format hint ("vcb_excel", "tcb_csv", "bidv_pdf", "iso20022_xml", "auto").
    pub statement_format: Option<String>,
    /// Optional path to ERP ledger open invoices JSON.
    pub erp_ledger_path: Option<String>,
    /// Wire fee deduction tolerance in VND (default: 11000 VND).
    #[serde(default = "default_fee_tolerance")]
    pub fee_tolerance_vnd: Option<u64>,
    /// Flag to enable Tier 2 hybrid semantic reconciliation with acronym elevation (default: true).
    #[serde(default = "default_true")]
    pub enable_semantic_recon: Option<bool>,
}

fn default_fee_tolerance() -> Option<u64> {
    Some(11_000)
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct StatementSummaryOutput {
    pub bank: String,
    pub account: String,
    pub opening_balance: u64,
    pub closing_balance: u64,
    pub total_credit: u64,
    pub total_debit: u64,
    pub tx_count: usize,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct HitlQuarantinedTx {
    pub tx_id: String,
    pub amount: u64,
    pub reason: String,
    pub hitl_token: String,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct BankingReconcileResult {
    pub statement_summary: StatementSummaryOutput,
    pub balance_invariant_valid: bool,
    pub matched_exact_count: usize,
    pub matched_fuzzy_count: usize,
    pub matched_split_count: usize,
    pub discrepancies_count: usize,
    pub hitl_quarantined: Vec<HitlQuarantinedTx>,
    /// Number of transactions matched via Tier 2 Semantic Elevation (acronyms, informal abbreviations).
    #[serde(default)]
    pub matched_semantic_count: usize,
}

// ---------------------------------------------------------------------------
// 2. `treasury_payment_order`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct TreasuryPaymentOrderArgs {
    /// Lifecycle action ("propose", "review", "approve", "reject", "get_status").
    pub action: String,
    /// Order UUID (required for review, approve, reject, get_status).
    pub order_id: Option<String>,
    /// Payer debit account number.
    pub debit_account: Option<String>,
    /// Beneficiary receiving account number.
    pub beneficiary_account: Option<String>,
    /// Beneficiary natural person or company legal name.
    pub beneficiary_name: Option<String>,
    /// Beneficiary bank name or code (e.g. "Vietcombank", "TCB").
    pub beneficiary_bank: Option<String>,
    /// Transfer amount in VND (must be > 0).
    pub amount_vnd: Option<u64>,
    /// Payment description / narration.
    pub purpose: Option<String>,
    /// Principal ID of initiator / accountant.
    pub maker_id: Option<String>,
    /// Principal ID of authorizer / chief accountant (must != maker_id).
    pub checker_id: Option<String>,
    /// Single-use UUIDv4 HITL token issued upon proposal (required for approve/reject).
    pub hitl_token: Option<String>,
    /// Optional justification required when rejecting an order.
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct TreasuryPaymentOrderResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checker_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hitl_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_hmac: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// 3. `compliance_aml_screen`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct ScreeningTxInput {
    pub tx_id: String,
    #[serde(default)]
    pub account_number: Option<String>,
    #[serde(default)]
    pub counterparty_name: Option<String>,
    #[serde(default)]
    pub counterparty_account: Option<String>,
    pub amount_vnd: u64,
    #[serde(default)]
    pub timestamp: Option<i64>,
    pub narration: String,
    #[serde(default)]
    pub is_credit: Option<bool>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct ComplianceAmlScreenArgs {
    pub transactions: Vec<ScreeningTxInput>,
    #[serde(default = "default_true")]
    pub redact_pii: Option<bool>,
}

fn default_true() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct AmlAlertOutput {
    pub tx_id: String,
    pub rule_code: String,
    pub severity: String,
    pub description: String,
    /// Local AI Anomaly Score (0.0 to 1.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_anomaly_score: Option<f64>,
    /// Multi-dimensional anomaly explanation generated by local SLM (Vietnamese, PII-redacted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anomaly_reasons: Option<Vec<String>>,
    /// Multi-dimensional risk category profile (e.g. CRITICAL_SUSPICIOUS, HIGH_RISK)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_profile: Option<String>,
    /// Enhanced alert including Vietnamese explanation generated by local SLM
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vietnamese_explanation: Option<String>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct SanitizedTxOutput {
    pub tx_id: String,
    pub masked_narration: String,
    pub masked_counterparty: String,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct ComplianceAmlScreenResult {
    pub total_screened: usize,
    pub alerts: Vec<AmlAlertOutput>,
    pub sanitized_transactions: Vec<SanitizedTxOutput>,
    pub zero_egress_verified: bool,
    /// Aggregate Local AI Anomaly Score (0.0 to 1.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_anomaly_score: Option<f64>,
    /// All multi-dimensional anomaly reasons
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anomaly_reasons: Option<Vec<String>>,
    /// Highest risk profile among screened transactions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_profile: Option<String>,
}

// ---------------------------------------------------------------------------
// 4. `credit_risk_scoring`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct CashflowDayInput {
    pub day_offset: i32,
    pub net_inflow_vnd: i64,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct CreditRiskScoringArgs {
    #[serde(default)]
    pub ebitda_vnd: Option<i64>,
    #[serde(default)]
    pub capex_vnd: Option<i64>,
    #[serde(default)]
    pub debt_service_principal_vnd: Option<i64>,
    #[serde(default)]
    pub debt_service_interest_vnd: Option<i64>,
    #[serde(default)]
    pub cash_and_equivalents_vnd: Option<i64>,
    #[serde(default)]
    pub marketable_securities_vnd: Option<i64>,
    #[serde(default)]
    pub accounts_receivable_vnd: Option<i64>,
    #[serde(default)]
    pub current_liabilities_vnd: Option<i64>,
    #[serde(default)]
    pub historical_cashflow: Option<Vec<CashflowDayInput>>,
    #[serde(default = "default_forecast_days")]
    pub forecast_days: Option<usize>,
}

fn default_forecast_days() -> Option<usize> {
    Some(30)
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct DscrOutput {
    pub ratio: f64,
    pub risk_category: String,
    pub buffer_vnd: i64,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct QuickRatioOutput {
    pub ratio: f64,
    pub liquidity_status: String,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct CashflowForecastOutput {
    pub projected_end_balance_vnd: i64,
    pub minimum_balance_vnd: i64,
    pub deficit_date_offset: Option<i32>,
    pub shortfall_warning: bool,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
pub struct CreditRiskScoringResult {
    pub dscr: DscrOutput,
    pub quick_ratio: QuickRatioOutput,
    pub cashflow_forecast: CashflowForecastOutput,
    /// Qualitative risk analysis summary synthesized by local SLM / deterministic fallback (Vietnamese)
    pub qualitative_risk_summary: String,
}

// ---------------------------------------------------------------------------
// Global In-Memory Fallback State for Payment Orders
// ---------------------------------------------------------------------------

static MEMORY_PAYMENT_ORDERS: std::sync::LazyLock<Arc<Mutex<HashMap<String, PaymentOrder>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

// ---------------------------------------------------------------------------
// Execution Handlers
// ---------------------------------------------------------------------------

/// Execution handler for `banking_reconcile` tool.
pub fn handle_banking_reconcile(args: BankingReconcileArgs) -> Result<BankingReconcileResult, String> {
    let (bytes, filename) = if let Some(ref b64) = args.statement_content_base64 {
        let decoded = BASE64_STANDARD
            .decode(b64.trim())
            .map_err(|e| format!("Invalid base64 in statement_content_base64: {e}"))?;
        let ext = match args.statement_format.as_deref().unwrap_or("auto") {
            "vcb_excel" => "statement.xlsx",
            "tcb_csv" => "statement.csv",
            "bidv_pdf" => "statement.pdf",
            "iso20022_xml" => "statement.xml",
            "swift_mt940" | "swift" | "mt940" => "statement.940",
            _ => {
                if decoded.starts_with(b"%PDF-") {
                    "statement.pdf"
                } else if decoded.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
                    "statement.xlsx"
                } else if decoded.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
                    "statement.xls"
                } else if decoded.starts_with(b":20:")
                    || decoded.starts_with(b"{1:")
                    || decoded.windows(4).any(|w| w == b":20:")
                {
                    "statement.940"
                } else {
                    let prefix_len = decoded.len().min(512);
                    let prefix = String::from_utf8_lossy(&decoded[..prefix_len]).to_lowercase();
                    if prefix.contains("<?xml")
                        || prefix.contains("<document")
                        || prefix.contains("<bktocstmrstmt")
                    {
                        "statement.xml"
                    } else if prefix.contains("<html") || prefix.contains("<table") {
                        "statement.html"
                    } else {
                        "statement.csv"
                    }
                }
            }
        };
        (decoded, ext.to_string())
    } else if let Some(ref path_str) = args.statement_file_path {
        let p = Path::new(path_str);
        if !p.exists() {
            return Err(format!("Statement file does not exist at path: {path_str}"));
        }
        let b = std::fs::read(p).map_err(|e| format!("Failed to read statement file: {e}"))?;
        let fn_str = p
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("statement.bin")
            .to_string();
        (b, fn_str)
    } else {
        return Err("Either `statement_file_path` or `statement_content_base64` must be provided".to_string());
    };

    let statement: BankStatement = match sniff_and_parse(&bytes, &filename) {
        Ok(s) => s,
        Err(native_err) => {
            // Gracefully fall back to dynamic StatementEngine in crates/liva-parse
            // supporting SWIFT MT940 and dynamic YAML templates
            match StatementEngine::new() {
                Ok(engine) => match engine.parse(&bytes, &filename) {
                    Ok(canonical) => {
                        let b_type = BankType::from_code(&canonical.bank_code);
                        let open_u64 = canonical
                            .opening_balance
                            .map(|m| u64::try_from(m.amount().max(0)).unwrap_or(0));
                        let close_u64 = canonical
                            .closing_balance
                            .map(|m| u64::try_from(m.amount().max(0)).unwrap_or(0));

                        let tx_records: Vec<TransactionRecord> = canonical
                            .transactions
                            .iter()
                            .enumerate()
                            .map(|(idx, tx)| {
                                let t_type = match tx.posting_type {
                                    liva_ledger::PostingType::Credit => TransactionType::Credit,
                                    liva_ledger::PostingType::Debit => TransactionType::Debit,
                                };
                                let amt_u64 = u64::try_from(tx.amount.amount().max(0)).unwrap_or(0);
                                let bal_after_u64 = tx
                                    .balance_after
                                    .map(|b| u64::try_from(b.amount().max(0)).unwrap_or(0));

                                TransactionRecord::new(
                                    idx + 1,
                                    tx.tx_timestamp,
                                    tx.tx_timestamp,
                                    Some(tx.ref_code.clone()),
                                    t_type,
                                    amt_u64,
                                    bal_after_u64,
                                    tx.counterparty_account.clone(),
                                    tx.counterparty_name.clone(),
                                    None,
                                    tx.narration.clone(),
                                )
                            })
                            .collect();

                        let stmt_from = canonical.transactions.first().map(|t| t.tx_timestamp);
                        let stmt_to = canonical.transactions.last().map(|t| t.tx_timestamp);
                        let fmt = if filename.to_lowercase().ends_with(".940")
                            || filename.to_lowercase().ends_with(".sta")
                            || filename.to_lowercase().ends_with(".mt940")
                        {
                            StatementFormat::SwiftMt
                        } else {
                            StatementFormat::Unknown
                        };

                        BankStatement::new(
                            canonical.bank_code,
                            b_type,
                            fmt,
                            canonical.account_number,
                            None,
                            open_u64,
                            close_u64,
                            stmt_from,
                            stmt_to,
                            tx_records,
                            0,
                        )
                    }
                    Err(engine_err) => {
                        return Err(format!(
                            "Failed to parse statement: native sniffer failed ({native_err}), fallback StatementEngine failed ({engine_err})"
                        ));
                    }
                },
                Err(init_err) => {
                    return Err(format!(
                        "Failed to parse statement: {native_err} (failed to initialize fallback StatementEngine: {init_err})"
                    ));
                }
            }
        }
    };

    // Load ledger if provided, otherwise create mock or empty ledger
    let ledger_entries: Vec<InternalLedgerEntry> = if let Some(ref ledger_path) = args.erp_ledger_path {
        let lp = Path::new(ledger_path);
        if lp.exists() {
            let s = std::fs::read_to_string(lp)
                .map_err(|e| format!("Failed to read erp_ledger_path: {e}"))?;
            serde_json::from_str(&s)
                .map_err(|e| format!("Failed to deserialize erp_ledger_path JSON: {e}"))?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Convert transactions to BankTransactionRow
    let bank_rows: Vec<BankTransactionRow> = statement
        .transactions
        .iter()
        .map(|tx| BankTransactionRow {
            id: tx.doc_ref.clone().unwrap_or_else(|| format!("TX-{}", tx.row_id)),
            statement_id: "STMT-CURRENT".to_string(),
            account_id: statement.account_number.clone().unwrap_or_default(),
            bank_code: statement.bank_code.clone(),
            tx_date: tx.tx_date,
            value_date: tx.value_date,
            doc_ref: tx.doc_ref.clone(),
            tx_type: tx.tx_type,
            amount: tx.amount,
            balance_after: tx.balance_after,
            counterparty_account: tx.counterparty_account.clone(),
            counterparty_name: tx.counterparty_name.clone(),
            counterparty_bank: tx.counterparty_bank.clone(),
            narration: tx.narration.clone(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: tx.tx_date,
        })
        .collect();

    let (reconciliation_matches, reconciliation_summary) =
        ReconciliationEngine::reconcile(&bank_rows, &ledger_entries);

    // Map to CanonicalTx and InternalInvoice for crates/liva-recon Declarative Engine
    let canonical_txs: Vec<CanonicalTx> = statement
        .transactions
        .iter()
        .enumerate()
        .map(|(idx, tx)| {
            let p_type = match tx.tx_type {
                TransactionType::Credit => liva_ledger::PostingType::Credit,
                TransactionType::Debit => liva_ledger::PostingType::Debit,
            };
            CanonicalTx {
                row_id: idx + 1,
                bank_code: statement.bank_code.clone(),
                ref_code: tx.doc_ref.clone().unwrap_or_else(|| format!("TX-{}", tx.row_id)),
                posting_type: p_type,
                amount: Money::vnd(i64::try_from(tx.amount).unwrap_or(i64::MAX)),
                balance_after: tx
                    .balance_after
                    .map(|b| Money::vnd(i64::try_from(b).unwrap_or(i64::MAX))),
                counterparty_name: tx.counterparty_name.clone(),
                counterparty_account: tx.counterparty_account.clone(),
                narration: tx.narration.clone(),
                tx_timestamp: tx.tx_date,
            }
        })
        .collect();

    let recon_invoices: Vec<InternalInvoice> = ledger_entries
        .iter()
        .map(|entry| {
            InternalInvoice {
                invoice_id: entry.id.clone(),
                doc_ref: entry.doc_no.clone(),
                counterparty_name: entry.partner_name.clone().unwrap_or_default(),
                amount: Money::vnd(i64::try_from(entry.amount).unwrap_or(i64::MAX)),
                timestamp: entry.entry_date,
            }
        })
        .collect();

    let mut recon_engine = DeclarativeReconEngine::default_rules();
    let enable_semantic = args.enable_semantic_recon.unwrap_or(true);
    recon_engine.config.tier2.enable_semantic_recon = enable_semantic;
    if let Some(fee) = args.fee_tolerance_vnd {
        recon_engine.config.tier2.fee_schedules.push(FeeScheduleConfig::Range {
            name: "dynamic_fee_tolerance".to_string(),
            min_vnd: 0,
            max_vnd: i64::try_from(fee).unwrap_or(i64::MAX),
        });
    }

    let recon_batch_res = recon_engine.reconcile_batch_with_semantic(&canonical_txs, &recon_invoices, None);

    let (matched_exact_count, matched_fuzzy_count, matched_semantic_count, matched_split_count, discrepancies_count) =
        if recon_batch_res.total_matched_count > 0 {
            (
                recon_batch_res.matched_exact_count,
                recon_batch_res.matched_fuzzy_count,
                recon_batch_res.matched_semantic_count,
                recon_batch_res.matched_split_count,
                recon_batch_res.unmatched_tx_count,
            )
        } else {
            (
                reconciliation_summary.matched_exact_count,
                reconciliation_summary.matched_fuzzy_count,
                0,
                reconciliation_summary.matched_split_count,
                reconciliation_summary.discrepancy_count,
            )
        };

    let summary = StatementSummaryOutput {
        bank: statement.bank_code,
        account: statement.account_number.unwrap_or_else(|| "N/A".to_string()),
        opening_balance: statement.opening_balance.unwrap_or(0),
        closing_balance: statement.closing_balance.unwrap_or(0),
        total_credit: statement.total_credit,
        total_debit: statement.total_debit,
        tx_count: statement.transactions.len(),
    };

    let hitl_items = if !recon_batch_res.hitl_quarantine.is_empty() {
        recon_batch_res
            .hitl_quarantine
            .iter()
            .map(|h| HitlQuarantinedTx {
                tx_id: h.bank_tx.ref_code.clone(),
                amount: u64::try_from(h.bank_tx.amount.amount().max(0)).unwrap_or(0),
                reason: format!("Chờ phê duyệt ngoại lệ HITL: {}", h.bank_tx.narration),
                hitl_token: h.token.clone(),
            })
            .collect()
    } else {
        reconciliation_matches
            .iter()
            .filter(|m| m.match_type == MatchType::ManualHitl || m.status == "PENDING_HITL")
            .map(|m| HitlQuarantinedTx {
                tx_id: m.bank_tx_id.clone(),
                amount: m.matched_amount,
                reason: m.notes.clone().unwrap_or_else(|| "Pending human-in-the-loop review".to_string()),
                hitl_token: m.hitl_token.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
            })
            .collect()
    };

    Ok(BankingReconcileResult {
        statement_summary: summary,
        balance_invariant_valid: statement.balance_checksum_passed,
        matched_exact_count,
        matched_fuzzy_count,
        matched_semantic_count,
        matched_split_count,
        discrepancies_count,
        hitl_quarantined: hitl_items,
    })
}

/// Execution handler for `reconcile_statement_batch` tool (alias to `handle_banking_reconcile`).
pub fn handle_reconcile_statement_batch(args: BankingReconcileArgs) -> Result<BankingReconcileResult, String> {
    handle_banking_reconcile(args)
}

/// Execution handler for `treasury_payment_order` tool.
pub fn handle_treasury_payment_order(
    args: TreasuryPaymentOrderArgs,
    conn: Option<&Connection>,
) -> Result<TreasuryPaymentOrderResult, String> {
    let action = args.action.trim().to_lowercase();

    match action.as_str() {
        "propose" => {
            let debit = args.debit_account.ok_or("`debit_account` is required to propose payment order")?;
            let benef_acc = args.beneficiary_account.ok_or("`beneficiary_account` is required")?;
            let benef_name = args.beneficiary_name.ok_or("`beneficiary_name` is required")?;
            let benef_bank = args.beneficiary_bank.ok_or("`beneficiary_bank` is required")?;
            let amount = args.amount_vnd.ok_or("`amount_vnd` is required")?;
            let purpose = args.purpose.unwrap_or_else(|| "Thanh toan tien hang".to_string());
            let maker = args.maker_id.ok_or("`maker_id` is required to propose payment order")?;

            let order = TreasuryEngine::propose(
                &debit,
                &benef_acc,
                &benef_name,
                &benef_bank,
                amount,
                &purpose,
                &maker,
            )?;

            // Persist to DB if connection available
            if let Some(c) = conn {
                let _ = init_payment_orders_table(c);
                let _ = save_payment_order(c, &order);
            }
            // Also store in memory fallback
            if let Ok(mut map) = MEMORY_PAYMENT_ORDERS.lock() {
                map.insert(order.id.clone(), order.clone());
            }

            Ok(TreasuryPaymentOrderResult {
                order_id: Some(order.id),
                status: order.status.as_str().to_string(),
                maker_id: Some(order.maker_id),
                checker_id: None,
                hitl_token: order.hitl_token,
                signature_hmac: None,
                created_at: Some(order.created_at),
                approved_at: None,
                error: None,
            })
        }
        "review" => {
            let order_id = args.order_id.ok_or("`order_id` is required for review")?;
            let mut order = load_order(&order_id, conn)?;
            TreasuryEngine::review(&mut order);

            if let Some(c) = conn {
                let _ = update_payment_order(c, &order);
            }
            if let Ok(mut map) = MEMORY_PAYMENT_ORDERS.lock() {
                map.insert(order.id.clone(), order.clone());
            }

            Ok(TreasuryPaymentOrderResult {
                order_id: Some(order.id),
                status: order.status.as_str().to_string(),
                maker_id: Some(order.maker_id),
                checker_id: order.checker_id,
                hitl_token: order.hitl_token,
                signature_hmac: order.signature_hmac,
                created_at: Some(order.created_at),
                approved_at: order.approved_at,
                error: None,
            })
        }
        "approve" => {
            let order_id = args.order_id.ok_or("`order_id` is required for approve")?;
            let checker = args.checker_id.ok_or("`checker_id` is required for approval")?;
            let token = args.hitl_token.ok_or("`hitl_token` is required for approval")?;

            let mut order = load_order(&order_id, conn)?;
            TreasuryEngine::approve(&mut order, &checker, &token, None, conn)
                .map_err(|e| format!("Payment order approval rejected: {e}"))?;

            if let Ok(mut map) = MEMORY_PAYMENT_ORDERS.lock() {
                map.insert(order.id.clone(), order.clone());
            }

            Ok(TreasuryPaymentOrderResult {
                order_id: Some(order.id),
                status: order.status.as_str().to_string(),
                maker_id: Some(order.maker_id),
                checker_id: order.checker_id,
                hitl_token: None,
                signature_hmac: order.signature_hmac,
                created_at: Some(order.created_at),
                approved_at: order.approved_at,
                error: None,
            })
        }
        "reject" => {
            let order_id = args.order_id.ok_or("`order_id` is required for reject")?;
            let checker = args.checker_id.ok_or("`checker_id` is required for rejection")?;
            let token = args.hitl_token.ok_or("`hitl_token` is required for rejection")?;
            let reason = args.rejection_reason.unwrap_or_else(|| "Rejected by authorizer".to_string());

            let mut order = load_order(&order_id, conn)?;
            TreasuryEngine::reject(&mut order, &checker, &token, &reason, None, conn)
                .map_err(|e| format!("Payment order rejection failed: {e}"))?;

            if let Ok(mut map) = MEMORY_PAYMENT_ORDERS.lock() {
                map.insert(order.id.clone(), order.clone());
            }

            Ok(TreasuryPaymentOrderResult {
                order_id: Some(order.id),
                status: order.status.as_str().to_string(),
                maker_id: Some(order.maker_id),
                checker_id: order.checker_id,
                hitl_token: None,
                signature_hmac: None,
                created_at: Some(order.created_at),
                approved_at: None,
                error: Some(format!("Order rejected: {reason}")),
            })
        }
        "get_status" => {
            let order_id = args.order_id.ok_or("`order_id` is required to query status")?;
            let order = load_order(&order_id, conn)?;

            Ok(TreasuryPaymentOrderResult {
                order_id: Some(order.id),
                status: order.status.as_str().to_string(),
                maker_id: Some(order.maker_id),
                checker_id: order.checker_id,
                hitl_token: order.hitl_token,
                signature_hmac: order.signature_hmac,
                created_at: Some(order.created_at),
                approved_at: order.approved_at,
                error: None,
            })
        }
        _ => Err(format!(
            "Unsupported action '{action}'. Valid actions: propose, review, approve, reject, get_status"
        )),
    }
}

fn load_order(order_id: &str, conn: Option<&Connection>) -> Result<PaymentOrder, String> {
    if let Some(c) = conn {
        if let Ok(Some(o)) = find_payment_order_by_id(c, order_id) {
            return Ok(o);
        }
    }
    if let Ok(map) = MEMORY_PAYMENT_ORDERS.lock() {
        if let Some(o) = map.get(order_id) {
            return Ok(o.clone());
        }
    }
    Err(format!("Payment order '{order_id}' not found"))
}

/// Execution handler for `compliance_aml_screen` tool.
pub fn handle_compliance_aml_screen(args: ComplianceAmlScreenArgs) -> Result<ComplianceAmlScreenResult, String> {
    let should_redact = args.redact_pii.unwrap_or(true);
    let total_screened = args.transactions.len();

    // 1. Convert to ScreenableTransaction for crates/liva-compliance Multi-Dimensional Anomaly Detection
    let compliance_txs: Vec<liva_compliance::ScreenableTransaction> = args
        .transactions
        .iter()
        .map(|tx| {
            let ts = tx.timestamp.unwrap_or_else(|| {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                i64::try_from(secs).unwrap_or(i64::MAX)
            });
            liva_compliance::ScreenableTransaction {
                tx_id: tx.tx_id.clone(),
                account_number: tx.account_number.clone().unwrap_or_default(),
                counterparty_name: tx.counterparty_name.clone().unwrap_or_default(),
                counterparty_account: tx.counterparty_account.clone().unwrap_or_default(),
                amount: Money::vnd(i64::try_from(tx.amount_vnd).unwrap_or(i64::MAX)),
                is_credit: tx.is_credit.unwrap_or(true),
                timestamp_seconds: ts,
                narration: tx.narration.clone(),
            }
        })
        .collect();

    let aml_engine = liva_compliance::AmlScreeningEngine::with_default_policy();
    let enhanced_alerts = aml_engine.screen_with_anomaly_detection(&compliance_txs);

    // 2. Also run statutory screening from liva-native-core
    let screening_items: Vec<TransactionScreeningItem> = args
        .transactions
        .iter()
        .map(|tx| TransactionScreeningItem {
            tx_id: tx.tx_id.clone(),
            account_number: tx.account_number.clone(),
            counterparty_name: tx.counterparty_name.clone(),
            counterparty_account: tx.counterparty_account.clone(),
            amount_vnd: tx.amount_vnd,
            timestamp: tx.timestamp.unwrap_or_else(|| {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                i64::try_from(secs).unwrap_or(i64::MAX)
            }),
            narration: tx.narration.clone(),
            is_credit: tx.is_credit,
        })
        .collect();

    let statutory_alerts = AmlScreeningEngine::screen_transactions(&screening_items, None);

    // 3. Assemble and map enhanced alerts
    let mut alerts_output: Vec<AmlAlertOutput> = Vec::new();
    let mut processed_alert_keys = std::collections::HashSet::new();

    for a in &enhanced_alerts {
        let rule_code_str = match a.rule_code {
            liva_compliance::AmlAlertCode::HighValueThreshold => "AML_HIGH_VALUE",
            liva_compliance::AmlAlertCode::StructuringSmurfing => "AML_STRUCTURING",
            liva_compliance::AmlAlertCode::VelocitySurge => "AML_VELOCITY_SURGE",
            liva_compliance::AmlAlertCode::PassThroughTransit => "AML_PASS_THROUGH",
            liva_compliance::AmlAlertCode::NightAnomaly => "AML_NIGHT_ANOMALY",
            liva_compliance::AmlAlertCode::WatchlistSanction => "AML_WATCHLIST",
            liva_compliance::AmlAlertCode::NearThresholdSmurfing => "AML_NEAR_THRESHOLD_SMURFING",
            liva_compliance::AmlAlertCode::AmountDistributionOutlier => "AML_AMOUNT_DISTRIBUTION_OUTLIER",
            liva_compliance::AmlAlertCode::SemanticNarrationAnomaly => "AML_SEMANTIC_NARRATION_ANOMALY",
            liva_compliance::AmlAlertCode::MicroBurstVelocity => "AML_MICRO_BURST_VELOCITY",
        };

        let severity_str = match a.severity {
            liva_compliance::AmlSeverity::Critical => "CRITICAL",
            liva_compliance::AmlSeverity::High => "HIGH",
            liva_compliance::AmlSeverity::Medium => "MEDIUM",
            liva_compliance::AmlSeverity::Low => "LOW",
        };

        // Strict basis points conversion at serialization boundary: 0..10_000 bps -> 0.0..1.0
        let score = a.anomaly_score_bps as f64 / 10_000.0;
        let reasons = a.anomaly_reasons.clone();
        let risk_prof = a.risk_profile.clone();
        let desc = if reasons.is_empty() {
            a.statutory_ref.clone()
        } else {
            reasons.join("; ")
        };

        processed_alert_keys.insert((a.tx_id.clone(), rule_code_str.to_string()));

        alerts_output.push(AmlAlertOutput {
            tx_id: a.tx_id.clone(),
            rule_code: rule_code_str.to_string(),
            severity: severity_str.to_string(),
            description: desc,
            ai_anomaly_score: Some(score),
            anomaly_reasons: Some(reasons),
            risk_profile: Some(risk_prof),
            vietnamese_explanation: a.vietnamese_explanation.clone(),
        });
    }

    // Include any statutory alerts not already in output
    for sa in statutory_alerts {
        let rc_str = sa.rule_code.as_str().to_string();
        if !processed_alert_keys.contains(&(sa.tx_id.clone(), rc_str.clone())) {
            let sev_str = match sa.severity {
                AmlSeverity::Critical => "CRITICAL".to_string(),
                AmlSeverity::High => "HIGH".to_string(),
            };
            let matching_enh = enhanced_alerts.iter().find(|e| e.tx_id == sa.tx_id);
            let ai_score = matching_enh.map(|e| e.anomaly_score_bps as f64 / 10_000.0);
            let reasons = matching_enh.map(|e| e.anomaly_reasons.clone());
            let risk_prof = matching_enh.map(|e| e.risk_profile.clone());
            let vn_exp = matching_enh.and_then(|e| e.vietnamese_explanation.clone());

            alerts_output.push(AmlAlertOutput {
                tx_id: sa.tx_id,
                rule_code: rc_str,
                severity: sev_str,
                description: sa.description,
                ai_anomaly_score: ai_score,
                anomaly_reasons: reasons,
                risk_profile: risk_prof,
                vietnamese_explanation: vn_exp,
            });
        }
    }

    // 4. Aggregate anomaly score & profile calculation
    let max_score_bps = enhanced_alerts.iter().map(|a| a.anomaly_score_bps).max();
    let (ai_anomaly_score, risk_profile) = if let Some(max_bps) = max_score_bps {
        let score = max_bps as f64 / 10_000.0;
        let profile = if max_bps >= 7500 {
            "CRITICAL_SUSPICIOUS".to_string()
        } else if max_bps >= 5000 {
            "HIGH_RISK".to_string()
        } else if max_bps >= 3500 {
            "ELEVATED_RISK".to_string()
        } else if max_bps >= 2000 {
            "MODERATE_RISK".to_string()
        } else {
            "LOW_RISK".to_string()
        };
        (Some(score), Some(profile))
    } else if total_screened > 0 {
        (Some(0.0), Some("LOW_RISK".to_string()))
    } else {
        (None, None)
    };

    let mut all_unique_reasons = Vec::new();
    for a in &enhanced_alerts {
        for r in &a.anomaly_reasons {
            if !all_unique_reasons.contains(r) {
                all_unique_reasons.push(r.clone());
            }
        }
    }
    let anomaly_reasons = if all_unique_reasons.is_empty() {
        None
    } else {
        Some(all_unique_reasons)
    };

    // 5. Decree 13 PII Sanitization
    let sanitized_transactions: Vec<SanitizedTxOutput> = args
        .transactions
        .iter()
        .map(|tx| {
            let masked_narr = if should_redact {
                sanitize_pii(&tx.narration)
            } else {
                tx.narration.clone()
            };
            let masked_counterparty = if should_redact {
                tx.counterparty_name
                    .as_deref()
                    .map(sanitize_counterparty_name)
                    .unwrap_or_default()
            } else {
                tx.counterparty_name.clone().unwrap_or_default()
            };
            SanitizedTxOutput {
                tx_id: tx.tx_id.clone(),
                masked_narration: masked_narr,
                masked_counterparty,
            }
        })
        .collect();

    let zero_egress_verified = verify_zero_egress().0;

    Ok(ComplianceAmlScreenResult {
        total_screened,
        alerts: alerts_output,
        sanitized_transactions,
        zero_egress_verified,
        ai_anomaly_score,
        anomaly_reasons,
        risk_profile,
    })
}

/// Synthesizes a qualitative financial risk narrative using local SLM metrics or deterministic fallback.
pub fn synthesize_qualitative_risk_summary(
    dscr: &DscrOutput,
    qr: &QuickRatioOutput,
    cf: &CashflowForecastOutput,
) -> String {
    let mut parts = Vec::new();

    // 1. Solvency evaluation (DSCR)
    let solvency_desc = match dscr.risk_category.as_str() {
        "HEALTHY" => format!(
            "Khả năng trả nợ vững chắc với hệ số DSCR đạt {:.2}x (ngưỡng an toàn >= 1.30x), dòng tiền hoạt động tạo vùng đệm an toàn {} VND sau khi thực hiện đầy đủ nghĩa vụ nợ gốc và lãi.",
            dscr.ratio, dscr.buffer_vnd
        ),
        "MODERATE" | "WATCHLIST" => format!(
            "Khả năng trả nợ ở mức trung bình với hệ số DSCR đạt {:.2}x, dòng tiền vừa đủ trang trải nghĩa vụ nợ nhưng có thể bị đe dọa nếu doanh thu biến động giảm.",
            dscr.ratio
        ),
        "DISTRESSED" | "CRITICAL" => format!(
            "CẢNH BÁO RỦI RO THANH TOÁN: Hệ số DSCR ở mức nguy hại ({:.2}x < 1.00x), doanh nghiệp thiếu hụt {} VND dòng tiền để thanh toán nợ đến hạn.",
            dscr.ratio, dscr.buffer_vnd.abs()
        ),
        _ => format!(
            "Hệ số chi trả nợ DSCR đạt {:.2}x ({}) với vùng đệm {} VND.",
            dscr.ratio, dscr.risk_category, dscr.buffer_vnd
        ),
    };
    parts.push(solvency_desc);

    // 2. Liquidity evaluation (Quick Ratio)
    let liquidity_desc = match qr.liquidity_status.as_str() {
        "STRONG" => format!(
            "Tỷ số thanh toán nhanh (Quick Ratio) ở mức dồi dào ({:.2}x >= 1.00x), tài sản ngắn hạn có tính thanh khoản cao đảm bảo chi trả ngay các khoản nợ ngắn hạn.",
            qr.ratio
        ),
        "ADEQUATE" => format!(
            "Tỷ số thanh toán nhanh (Quick Ratio) ở mức chấp nhận được ({:.2}x), cân đối được nghĩa vụ tài chính ngắn hạn.",
            qr.ratio
        ),
        "STRESSED" | "CRITICAL" | "CRITICAL_DEFICIT" => format!(
            "Thanh khoản tức thời bị căng thẳng nghiêm trọng với Quick Ratio chỉ đạt {:.2}x (< 1.00x), tiềm ẩn nguy cơ mất khả năng thanh toán ngắn hạn.",
            qr.ratio
        ),
        _ => format!(
            "Trạng thái thanh khoản tức thời: {} (Quick Ratio: {:.2}x).",
            qr.liquidity_status, qr.ratio
        ),
    };
    parts.push(liquidity_desc);

    // 3. Forward Cashflow Forecast
    if cf.shortfall_warning {
        if let Some(day) = cf.deficit_date_offset {
            parts.push(format!(
                "Dự báo dòng tiền xuất hiện thâm hụt tài chính tại ngày T+{} với mức số dư thấp nhất chạm đáy {} VND. Đề xuất kích hoạt hạn mức thấu chi hoặc điều phối nguồn vốn ngắn hạn trước thời điểm T+{}.",
                day, cf.minimum_balance_vnd, day
            ));
        } else {
            parts.push(format!(
                "Cảnh báo dòng tiền dự phóng bị thâm hụt với số dư thấp nhất {} VND. Cần theo dõi chặt chẽ tiến độ thu hồi công nợ.",
                cf.minimum_balance_vnd
            ));
        }
    } else {
        parts.push(format!(
            "Dự báo dòng tiền duy trì dương trong suốt chu kỳ dự báo, số dư cuối kỳ đạt {} VND, không phát sinh cảnh báo thiếu hụt ngân quỹ.",
            cf.projected_end_balance_vnd
        ));
    }

    parts.join(" ")
}

/// Execution handler for `credit_risk_scoring` tool.
pub fn handle_credit_risk_scoring(args: CreditRiskScoringArgs) -> Result<CreditRiskScoringResult, String> {
    let ebitda = args.ebitda_vnd.unwrap_or(0);
    let capex = args.capex_vnd.unwrap_or(0);
    let principal = args.debt_service_principal_vnd.unwrap_or(0);
    let interest = args.debt_service_interest_vnd.unwrap_or(0);

    let dscr_report = CreditRiskEngine::calculate_dscr(ebitda, capex, principal, interest);

    let cash = args.cash_and_equivalents_vnd.unwrap_or(0);
    let securities = args.marketable_securities_vnd.unwrap_or(0);
    let receivables = args.accounts_receivable_vnd.unwrap_or(0);
    let liabilities = args.current_liabilities_vnd.unwrap_or(0);

    let qr_report = CreditRiskEngine::calculate_quick_ratio(cash, securities, receivables, liabilities);

    let flows: Vec<DailyCashflowPoint> = args
        .historical_cashflow
        .unwrap_or_default()
        .into_iter()
        .map(|f| DailyCashflowPoint {
            day_offset: f.day_offset,
            net_inflow_vnd: f.net_inflow_vnd,
        })
        .collect();

    let days = args.forecast_days.unwrap_or(30);
    let starting_cash = cash.saturating_add(securities);
    let cf_report = CreditRiskEngine::simulate_cashflow(starting_cash, &flows, days, 0);

    let dscr_out = DscrOutput {
        ratio: dscr_report.ratio,
        risk_category: dscr_report.risk_category.as_str().to_string(),
        buffer_vnd: dscr_report.buffer_vnd,
    };
    let qr_out = QuickRatioOutput {
        ratio: qr_report.ratio,
        liquidity_status: qr_report.liquidity_status.as_str().to_string(),
    };
    let cf_out = CashflowForecastOutput {
        projected_end_balance_vnd: cf_report.projected_end_balance_vnd,
        minimum_balance_vnd: cf_report.minimum_balance_vnd,
        deficit_date_offset: cf_report.deficit_date_offset,
        shortfall_warning: cf_report.shortfall_warning,
    };

    let qualitative_risk_summary = synthesize_qualitative_risk_summary(&dscr_out, &qr_out, &cf_out);

    Ok(CreditRiskScoringResult {
        dscr: dscr_out,
        quick_ratio: qr_out,
        cashflow_forecast: cf_out,
        qualitative_risk_summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_aml_screen_handler() {
        let args = ComplianceAmlScreenArgs {
            transactions: vec![
                ScreeningTxInput {
                    tx_id: "TX-400M".to_string(),
                    account_number: Some("19034567890123".to_string()),
                    counterparty_name: Some("Nguyen Van A".to_string()),
                    counterparty_account: Some("0011001234567".to_string()),
                    amount_vnd: 500_000_000,
                    timestamp: Some(1725000000),
                    narration: "Khach hang Nguyen Van A, CCCD: 001095012345 chuyen tien 0912345678".to_string(),
                    is_credit: Some(true),
                },
            ],
            redact_pii: Some(true),
        };

        let res = handle_compliance_aml_screen(args).expect("Handler succeeds");
        assert_eq!(res.total_screened, 1);
        assert_eq!(res.alerts.len(), 1);
        assert_eq!(res.alerts[0].rule_code, "AML_HIGH_VALUE");
        assert!(res.alerts[0].severity == "HIGH" || res.alerts[0].severity == "CRITICAL");
        assert!(res.alerts[0].ai_anomaly_score.is_some());
        assert!(res.alerts[0].anomaly_reasons.is_some());
        assert!(res.alerts[0].risk_profile.is_some());
        assert!(res.alerts[0].vietnamese_explanation.is_some());
        assert!(res.ai_anomaly_score.is_some());

        assert_eq!(res.sanitized_transactions.len(), 1);
        let sanitized = &res.sanitized_transactions[0];
        assert_eq!(sanitized.masked_counterparty, "[REDACTED_NAME]");
        assert!(sanitized.masked_narration.contains("[REDACTED_CCCD]"));
        assert!(sanitized.masked_narration.contains("[REDACTED_PHONE]"));
        assert!(sanitized.masked_narration.contains("[REDACTED_NAME]"));
        assert!(res.zero_egress_verified);
    }

    #[test]
    fn test_mcp_credit_risk_scoring_handler() {
        let args = CreditRiskScoringArgs {
            ebitda_vnd: Some(1_500_000_000),
            capex_vnd: Some(200_000_000),
            debt_service_principal_vnd: Some(600_000_000),
            debt_service_interest_vnd: Some(200_000_000),
            cash_and_equivalents_vnd: Some(400_000_000),
            marketable_securities_vnd: Some(100_000_000),
            accounts_receivable_vnd: Some(500_000_000),
            current_liabilities_vnd: Some(800_000_000),
            historical_cashflow: Some(vec![
                CashflowDayInput { day_offset: 5, net_inflow_vnd: -600_000_000 },
            ]),
            forecast_days: Some(30),
        };

        let res = handle_credit_risk_scoring(args).expect("Scoring succeeds");
        assert_eq!(res.dscr.risk_category, "HEALTHY");
        assert_eq!(res.quick_ratio.liquidity_status, "STRONG");
        assert!(res.cashflow_forecast.shortfall_warning);
        assert_eq!(res.cashflow_forecast.deficit_date_offset, Some(5));
        assert!(!res.qualitative_risk_summary.is_empty());
        assert!(res.qualitative_risk_summary.contains("DSCR"));
        assert!(res.qualitative_risk_summary.contains("thanh khoản"));
    }

    #[test]
    fn test_mcp_reconcile_statement_batch_semantic_tier2() {
        let camt053_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>MSG-20260916-001</MsgId>
      <CreDtTm>2026-09-16T08:30:00Z</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-2026-09-16</Id>
      <Acct>
        <Id><Othr><Id>0011000123456</Id></Othr></Id>
      </Acct>
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
        <Dt><Dt>2026-09-16</Dt></Dt>
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

        let args = BankingReconcileArgs {
            statement_file_path: None,
            statement_content_base64: Some(b64),
            statement_format: Some("iso20022_xml".to_string()),
            erp_ledger_path: None,
            fee_tolerance_vnd: Some(11_000),
            enable_semantic_recon: Some(true),
        };

        let res = handle_reconcile_statement_batch(args).expect("Reconciliation succeeds");
        assert!(res.balance_invariant_valid);
        assert_eq!(res.statement_summary.opening_balance, 100_000_000);
        assert_eq!(res.statement_summary.closing_balance, 150_000_000);
        assert_eq!(res.statement_summary.total_credit, 50_000_000);
    }

    #[test]
    fn test_mcp_reconcile_fallback_statement_engine_swift_mt940() {
        let swift_mt940 = ":20:MT940-2026-001\r\n\
:25:9876543210\r\n\
:60F:C260901VND100000000,\r\n\
:61:260910C50000000NTRFNONREF//INV-999\r\n\
:86:Thanh toan hop dong cung cap vat tu\r\n\
:62F:C260910VND150000000,\r\n\
-";

        let b64 = BASE64_STANDARD.encode(swift_mt940.as_bytes());

        let args = BankingReconcileArgs {
            statement_file_path: None,
            statement_content_base64: Some(b64.clone()),
            statement_format: Some("swift_mt940".to_string()),
            erp_ledger_path: None,
            fee_tolerance_vnd: Some(11_000),
            enable_semantic_recon: Some(false),
        };

        let res = handle_reconcile_statement_batch(args).expect("Fallback StatementEngine parses SWIFT MT940");
        assert!(res.balance_invariant_valid);
        assert_eq!(res.statement_summary.account, "9876543210");
        assert_eq!(res.statement_summary.opening_balance, 100_000_000);
        assert_eq!(res.statement_summary.closing_balance, 150_000_000);
        assert_eq!(res.statement_summary.total_credit, 50_000_000);
        assert_eq!(res.statement_summary.tx_count, 1);

        // Also verify with auto format detection (statement_format: None)
        let auto_args = BankingReconcileArgs {
            statement_file_path: None,
            statement_content_base64: Some(b64),
            statement_format: None,
            erp_ledger_path: None,
            fee_tolerance_vnd: None,
            enable_semantic_recon: Some(false),
        };
        let auto_res = handle_reconcile_statement_batch(auto_args).expect("Auto-sniff fallback parses SWIFT MT940");
        assert!(auto_res.balance_invariant_valid);
        assert_eq!(auto_res.statement_summary.account, "9876543210");
        assert_eq!(auto_res.statement_summary.total_credit, 50_000_000);
    }

    #[test]
    fn test_mcp_treasury_payment_order_handler_full_lifecycle() {
        let propose_args = TreasuryPaymentOrderArgs {
            action: "propose".to_string(),
            order_id: None,
            debit_account: Some("19034567890123".to_string()),
            beneficiary_account: Some("0011001234567".to_string()),
            beneficiary_name: Some("CONG TY TNHH THEP VIET NHAT".to_string()),
            beneficiary_bank: Some("VIETCOMBANK".to_string()),
            amount_vnd: Some(350_000_000),
            purpose: Some("Thanh toan tien thep xay dung".to_string()),
            maker_id: Some("maker_accountant_01".to_string()),
            checker_id: None,
            hitl_token: None,
            rejection_reason: None,
        };

        let propose_res = handle_treasury_payment_order(propose_args, None).expect("Propose succeeds");
        assert_eq!(propose_res.status, "PENDING_APPROVAL");
        let order_id = propose_res.order_id.unwrap();
        let token = propose_res.hitl_token.unwrap();

        // Self-approval must fail
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
            checker_id: Some("maker_accountant_01".to_string()),
            hitl_token: Some(token.clone()),
            rejection_reason: None,
        };
        let self_err = handle_treasury_payment_order(self_approve_args, None).expect_err("Self-approval must error");
        assert!(self_err.contains("Self-approval") || self_err.contains("SelfApprovalProhibited"));

        // Valid checker approval succeeds
        let valid_approve_args = TreasuryPaymentOrderArgs {
            action: "approve".to_string(),
            order_id: Some(order_id.clone()),
            debit_account: None,
            beneficiary_account: None,
            beneficiary_name: None,
            beneficiary_bank: None,
            amount_vnd: None,
            purpose: None,
            maker_id: None,
            checker_id: Some("chief_accountant_02".to_string()),
            hitl_token: Some(token),
            rejection_reason: None,
        };
        let approve_res = handle_treasury_payment_order(valid_approve_args, None).expect("Approval succeeds");
        assert_eq!(approve_res.status, "APPROVED");
        assert!(approve_res.signature_hmac.is_some());
    }
}
