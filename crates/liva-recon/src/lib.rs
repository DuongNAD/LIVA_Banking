//! LIVA Banking — `liva-recon` Crate
//!
//! Deterministic 3-tier reconciliation engine and HITL quarantine queue.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod dsl;
pub mod golden_dataset;
pub mod compliance_aml;
pub use compliance_aml::{
    build_str_report, compute_merkle_root, sanitize_decree13_pii, screen_transactions_aml,
    AmlAlert, AmlAlertCode, AmlSeverity, ScreenableTransaction, SuspiciousTransactionReport,
};
pub use dsl::{
    compute_semantic_similarity_bps, extract_initials, integer_jaro, integer_jaro_winkler,
    integer_sqrt_u128, integer_sqrt_u64, integer_token_cosine_similarity, is_acronym_pattern,
    solve_subset_sum, DeclarativeReconEngine, ReconBatchResult, ReconDslError, ReconRuleConfig,
    SemanticReconContext, SubsetCandidate, SubsetSolution, SubsetSolverConfig,
};

use liva_money::Money;
use liva_parse::CanonicalTx;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReconError {
    #[error("Reconciliation error: {0}")]
    General(String),

    #[error("HITL Token expired or invalid")]
    InvalidToken,

    #[error("Maker and Checker cannot be the same entity")]
    MakerCheckerViolation,
}

/// Internal ERP/Accounting Invoice or Ledger Entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalInvoice {
    pub invoice_id: String,
    pub doc_ref: String,
    pub counterparty_name: String,
    pub amount: Money,
    pub timestamp: i64,
}

/// Match classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchTier {
    Tier1Exact,
    Tier2Fuzzy,
    Tier2Semantic,
    Tier3Split,
    HitlQuarantine,
}

impl MatchTier {
    pub fn is_tier2(&self) -> bool {
        matches!(self, MatchTier::Tier2Fuzzy | MatchTier::Tier2Semantic)
    }
}

/// Successful reconciliation match record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationMatch {
    pub match_id: String,
    pub tier: MatchTier,
    pub bank_tx_id: usize,
    pub invoice_id: String,
    pub bank_amount: Money,
    pub invoice_amount: Money,
    pub fee_deducted: Option<Money>,
    pub confidence_score: u32, // 0..100
    #[serde(default)]
    pub constituent_invoice_ids: Option<Vec<String>>,
    #[serde(default)]
    pub constituent_bank_tx_ids: Option<Vec<usize>>,
    #[serde(default)]
    pub is_semantic: bool,
    #[serde(default)]
    pub semantic_similarity_bps: Option<u32>,
}

/// Quarantine item held for Human-In-The-Loop two-phase review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlQuarantineItem {
    pub token: String,
    pub bank_tx: CanonicalTx,
    pub candidate_invoices: Vec<InternalInvoice>,
    pub created_at: i64,
    pub ttl_seconds: i64,
    pub maker: Option<String>,
    pub checker: Option<String>,
    pub approved: bool,
    #[serde(default)]
    pub risk_tier: Option<crate::dsl::schema::QuarantineRiskTier>,
    #[serde(default)]
    pub statutory_reference: Option<String>,
}

/// Normalizes Vietnamese text by stripping diacritics and converting to uppercase.
pub fn normalize_vietnamese_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        let mapped = match c {
            'a' | 'A' | 'á' | 'Á' | 'à' | 'À' | 'ả' | 'Ả' | 'ã' | 'Ã' | 'ạ' | 'Ạ'
            | 'ă' | 'Ă' | 'ắ' | 'Ắ' | 'ằ' | 'Ằ' | 'ẳ' | 'Ẳ' | 'ẵ' | 'Ẵ' | 'ặ' | 'Ặ'
            | 'â' | 'Â' | 'ấ' | 'Ấ' | 'ầ' | 'Ầ' | 'ẩ' | 'Ẩ' | 'ẫ' | 'Ẫ' | 'ậ' | 'Ậ' => 'A',
            'd' | 'D' | 'đ' | 'Đ' => 'D',
            'e' | 'E' | 'é' | 'É' | 'è' | 'È' | 'ẻ' | 'Ẻ' | 'ẽ' | 'Ẽ' | 'ẹ' | 'Ẹ'
            | 'ê' | 'Ê' | 'ế' | 'Ế' | 'ề' | 'Ề' | 'ể' | 'Ể' | 'ễ' | 'Ễ' | 'ệ' | 'Ệ' => 'E',
            'i' | 'I' | 'í' | 'Í' | 'ì' | 'Ì' | 'ỉ' | 'Ỉ' | 'ĩ' | 'Ĩ' | 'ị' | 'Ị' => 'I',
            'o' | 'O' | 'ó' | 'Ó' | 'ò' | 'Ò' | 'ỏ' | 'Ỏ' | 'õ' | 'Õ' | 'ọ' | 'Ọ'
            | 'ô' | 'Ô' | 'ố' | 'Ố' | 'ồ' | 'Ồ' | 'ổ' | 'Ổ' | 'ỗ' | 'Ỗ' | 'ộ' | 'Ộ'
            | 'ơ' | 'Ơ' | 'ớ' | 'Ớ' | 'ờ' | 'Ờ' | 'ở' | 'Ở' | 'ỡ' | 'Ỡ' | 'ợ' | 'Ợ' => 'O',
            'u' | 'U' | 'ú' | 'Ú' | 'ù' | 'Ù' | 'ủ' | 'Ủ' | 'ũ' | 'Ũ' | 'ụ' | 'Ụ'
            | 'ư' | 'Ư' | 'ứ' | 'Ứ' | 'ừ' | 'Ừ' | 'ử' | 'Ử' | 'ữ' | 'Ữ' | 'ự' | 'Ự' => 'U',
            'y' | 'Y' | 'ý' | 'Ý' | 'ỳ' | 'Ỳ' | 'ỷ' | 'Ỷ' | 'ỹ' | 'Ỹ' | 'ỵ' | 'Ỵ' => 'Y',
            other => other.to_ascii_uppercase(),
        };
        out.push(mapped);
    }
    out
}

static DEFAULT_ENGINE: std::sync::OnceLock<DeclarativeReconEngine> = std::sync::OnceLock::new();

fn default_engine() -> &'static DeclarativeReconEngine {
    DEFAULT_ENGINE.get_or_init(DeclarativeReconEngine::default_rules)
}

/// Deterministic 3-Tier Reconciliation Engine.
pub struct ReconciliationEngine;

impl ReconciliationEngine {
    /// Executes Tier 1 exact match (Amount + DocRef/RefCode equality, window ±24h, regex extraction).
    pub fn match_tier1(
        txs: &[CanonicalTx],
        invoices: &[InternalInvoice],
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        default_engine().match_tier1(txs, invoices)
    }

    /// Executes Tier 2 fuzzy heuristic matching (party name token match + standard bank fee deduction).
    pub fn match_tier2(
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        unmatched_inv_ids: &[String],
        all_invoices: &[InternalInvoice],
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        default_engine().match_tier2(
            unmatched_tx_indices,
            all_txs,
            unmatched_inv_ids,
            all_invoices,
        )
    }

    /// Executes Tier 3 Subset-Sum Split Solver (1:N and N:1 composite match bounded by max_depth items, up to 8).
    pub fn match_tier3(
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        unmatched_inv_ids: &[String],
        all_invoices: &[InternalInvoice],
        max_depth: usize,
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        if max_depth == 8 {
            default_engine().match_tier3(
                unmatched_tx_indices,
                all_txs,
                unmatched_inv_ids,
                all_invoices,
            )
        } else {
            let mut engine = default_engine().clone();
            engine.config.tier3.max_depth = max_depth;
            engine.match_tier3(
                unmatched_tx_indices,
                all_txs,
                unmatched_inv_ids,
                all_invoices,
            )
        }
    }

    /// Enqueues remaining unresolved items into HITL Quarantine with configured TTL tokens and risk classification.
    pub fn enqueue_hitl(
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        remaining_invoices: &[InternalInvoice],
        now_timestamp: i64,
    ) -> Vec<HitlQuarantineItem> {
        default_engine().enqueue_hitl(
            unmatched_tx_indices,
            all_txs,
            remaining_invoices,
            now_timestamp,
        )
    }

    /// Maker-Checker two-phase approval for a quarantined item.
    pub fn approve_hitl(
        item: &mut HitlQuarantineItem,
        maker: impl Into<String>,
        checker: impl Into<String>,
        now: i64,
    ) -> Result<(), ReconError> {
        default_engine().approve_hitl(item, maker, checker, now)
    }

    /// Executes Tier 2 fuzzy & hybrid semantic matching with optional semantic context.
    pub fn match_tier2_with_semantic(
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        unmatched_inv_ids: &[String],
        all_invoices: &[InternalInvoice],
        semantic_context: Option<&SemanticReconContext>,
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        default_engine().match_tier2_with_semantic(
            unmatched_tx_indices,
            all_txs,
            unmatched_inv_ids,
            all_invoices,
            semantic_context,
        )
    }

    /// Executes full 3-tier batch reconciliation with optional semantic context.
    pub fn reconcile_batch_with_semantic(
        txs: &[CanonicalTx],
        invoices: &[InternalInvoice],
        semantic_context: Option<&SemanticReconContext>,
    ) -> ReconBatchResult {
        default_engine().reconcile_batch_with_semantic(txs, invoices, semantic_context)
    }

    /// Executes full 3-tier batch reconciliation with default rules.
    pub fn reconcile_batch(
        txs: &[CanonicalTx],
        invoices: &[InternalInvoice],
    ) -> ReconBatchResult {
        default_engine().reconcile_batch(txs, invoices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liva_ledger::PostingType;

    #[test]
    fn test_tier1_exact_reconciliation() {
        let txs = vec![CanonicalTx {
            row_id: 1,
            bank_code: "VCB".to_string(),
            ref_code: "INV-101".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(50_000_000),
            balance_after: None,
            counterparty_name: None,
            counterparty_account: None,
            narration: "Thanh toan INV-101".to_string(),
            tx_timestamp: 1726400000,
        }];

        let invoices = vec![InternalInvoice {
            invoice_id: "INV-001".to_string(),
            doc_ref: "INV-101".to_string(),
            counterparty_name: "CONG TY TNHH MINH LONG".to_string(),
            amount: Money::vnd(50_000_000),
            timestamp: 1726400000,
        }];

        let (matched, rem_tx, rem_inv) = ReconciliationEngine::match_tier1(&txs, &invoices);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].tier, MatchTier::Tier1Exact);
        assert_eq!(matched[0].confidence_score, 100);
        assert!(rem_tx.is_empty());
        assert!(rem_inv.is_empty());
    }

    #[test]
    fn test_tier2_fuzzy_with_fee_deduction() {
        let txs = vec![CanonicalTx {
            row_id: 2,
            bank_code: "TCB".to_string(),
            ref_code: "FT269999".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(9_997_800), // 10M minus 2.200 fee
            balance_after: None,
            counterparty_name: None,
            counterparty_account: None,
            narration: "Napas VietQR TT Tu: AN PHAT LOGISTICS".to_string(),
            tx_timestamp: 1726400000,
        }];

        let invoices = vec![InternalInvoice {
            invoice_id: "INV-002".to_string(),
            doc_ref: "HD-999".to_string(),
            counterparty_name: "CÔNG TY CỔ PHẦN AN PHÁT".to_string(),
            amount: Money::vnd(10_000_000),
            timestamp: 1726400000,
        }];

        let (matched, rem_tx, rem_inv) = ReconciliationEngine::match_tier2(&[0], &txs, &["INV-002".to_string()], &invoices);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].tier, MatchTier::Tier2Fuzzy);
        assert_eq!(matched[0].fee_deducted, Some(Money::vnd(2_200)));
        assert!(rem_tx.is_empty());
        assert!(rem_inv.is_empty());
    }

    #[test]
    fn test_hitl_quarantine_maker_checker() {
        let txs = vec![CanonicalTx {
            row_id: 3,
            bank_code: "BIDV".to_string(),
            ref_code: "UNKNOWN_REF".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(1_234_567),
            balance_after: None,
            counterparty_name: None,
            counterparty_account: None,
            narration: "Chuyen tien khong ro noi dung".to_string(),
            tx_timestamp: 1726400000,
        }];

        let mut queue = ReconciliationEngine::enqueue_hitl(&[0], &txs, &[], 1726400000);
        assert_eq!(queue.len(), 1);

        // Same maker & checker must fail
        let same_res = ReconciliationEngine::approve_hitl(&mut queue[0], "accountant_a", "accountant_a", 1726400010);
        assert!(matches!(same_res, Err(ReconError::MakerCheckerViolation)));

        // Distinct maker & checker must succeed
        let valid_res = ReconciliationEngine::approve_hitl(&mut queue[0], "accountant_a", "chief_accountant_b", 1726400010);
        assert!(valid_res.is_ok());
        assert!(queue[0].approved);
    }

    #[test]
    fn test_tier3_split_solver() {
        let txs = vec![CanonicalTx {
            row_id: 4,
            bank_code: "VCB".to_string(),
            ref_code: "BATCH-01".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(30_000_000),
            balance_after: None,
            counterparty_name: Some("CONG TY DUONG DONG".to_string()),
            counterparty_account: None,
            narration: "Thanh toan gom cac hoa don CTY DUONG DONG".to_string(),
            tx_timestamp: 1726400000,
        }];

        let invoices = vec![
            InternalInvoice {
                invoice_id: "INV-SPLIT-1".to_string(),
                doc_ref: "HD-1".to_string(),
                counterparty_name: "Tập đoàn Thương Mại Dương Đông".to_string(),
                amount: Money::vnd(12_000_000),
                timestamp: 1726400000,
            },
            InternalInvoice {
                invoice_id: "INV-SPLIT-2".to_string(),
                doc_ref: "HD-2".to_string(),
                counterparty_name: "Tập đoàn Thương Mại Dương Đông".to_string(),
                amount: Money::vnd(18_000_000),
                timestamp: 1726400000,
            },
        ];

        let (matched, rem_tx, rem_inv) = ReconciliationEngine::match_tier3(
            &[0],
            &txs,
            &["INV-SPLIT-1".to_string(), "INV-SPLIT-2".to_string()],
            &invoices,
            8,
        );

        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].tier, MatchTier::Tier3Split);
        assert_eq!(matched[0].bank_amount, Money::vnd(30_000_000));
        assert_eq!(matched[0].invoice_amount, Money::vnd(30_000_000));
        assert_eq!(
            matched[0].constituent_invoice_ids.as_ref().unwrap().len(),
            2
        );
        assert!(rem_tx.is_empty());
        assert!(rem_inv.is_empty());
    }

    #[test]
    fn test_500_golden_dataset_benchmark() {
        use crate::golden_dataset::{generate_500_golden_dataset, run_golden_benchmark};

        let (txs, invoices) = generate_500_golden_dataset();
        assert!(txs.len() >= 500);
        assert!(invoices.len() >= 500);

        let res = run_golden_benchmark(&txs, &invoices);

        // Verification of Master Plan KPIs
        assert!(res.auto_match_rate_pct >= 85, "Auto-match rate must be >= 85% on diverse golden set, got {}%", res.auto_match_rate_pct);
        assert_eq!(res.false_match_count, 0, "False auto-match count must be 0");
        assert!(res.balance_invariant_holds, "Mathematical balance invariant must hold");
        assert!(res.tier3_matches > 0, "Must have tier 3 split matches");
    }

    #[test]
    fn test_500_golden_dataset_reconcilable_high_accuracy() {
        use crate::golden_dataset::{generate_500_valid_reconcilable_dataset, run_golden_benchmark};

        let (txs, invoices) = generate_500_valid_reconcilable_dataset();
        assert_eq!(txs.len(), 500);
        assert!(invoices.len() >= 500);

        let res = run_golden_benchmark(&txs, &invoices);

        // Verification of Master Plan KPIs: >= 99.8% match accuracy (10,000 bps = 100.0%) and 0% mismatch
        assert!(
            res.reconcilable_match_rate_bps >= 9_980,
            "Reconcilable match rate must be >= 99.8% (9980 bps), got {} bps",
            res.reconcilable_match_rate_bps
        );
        assert_eq!(res.auto_match_count, 500, "All 500 valid bank transactions must be matched");
        assert_eq!(res.false_match_count, 0, "False auto-match count must be exactly 0 (0% mismatch)");
        assert_eq!(res.hitl_quarantine_count, 0, "No valid items should be quarantined");
        assert!(res.balance_invariant_holds, "Mathematical balance invariant must hold");
        assert!(res.tier1_matches > 0);
        assert!(res.tier2_matches > 0);
        assert!(res.tier3_matches > 0);
    }

    #[test]
    fn test_decree13_pii_sanitization_and_corporate_preservation() {
        // Personal data masking
        let cccd_input = "Chuyen tien thanh toan cho so CCCD 031092008451 tai VCB";
        let cccd_sanitized = sanitize_decree13_pii(cccd_input);
        assert!(cccd_sanitized.contains("[REDACTED_CCCD]"));
        assert!(!cccd_sanitized.contains("031092008451"));

        let phone_input = "Lien he anh Nam 0908123456 de xac nhan don hang";
        let phone_sanitized = sanitize_decree13_pii(phone_input);
        assert!(phone_sanitized.contains("[REDACTED_PHONE]"));
        assert!(!phone_sanitized.contains("0908123456"));

        let account_input = "Thanh toan tien hang vao stk 0071000982123 ngan hang VCB";
        let acc_sanitized = sanitize_decree13_pii(account_input);
        assert!(acc_sanitized.contains("[REDACTED_ACCOUNT]"));
        assert!(!acc_sanitized.contains("0071000982123"));

        let personal_name = "NGUYEN VAN BINH";
        assert_eq!(sanitize_decree13_pii(personal_name), "[REDACTED_NAME]");

        // Corporate entity preservation (MUST NOT be redacted!)
        let corporate_name = "CONG TY TNHH PHAT TRIEN NGUYEN HOANG";
        let corp_sanitized = sanitize_decree13_pii(corporate_name);
        assert_eq!(corp_sanitized, corporate_name);
        assert!(!corp_sanitized.contains("[REDACTED_NAME]"));
    }

    #[test]
    fn test_aml_statutory_screening_and_str_generation() {
        let txs = vec![
            ScreenableTransaction {
                tx_id: "TX-HV-01".to_string(),
                account_number: "0071000123".to_string(),
                counterparty_name: "Doanh nghiep A".to_string(),
                counterparty_account: "190345678".to_string(),
                amount: Money::vnd(500_000_000), // >= 400M
                is_credit: true,
                timestamp_seconds: 1726300000,
                narration: "Thanh toan hop dong may moc".to_string(),
            },
            ScreenableTransaction {
                tx_id: "TX-PT-IN".to_string(),
                account_number: "0071000888".to_string(),
                counterparty_name: "Ong B".to_string(),
                counterparty_account: "120100099".to_string(),
                amount: Money::vnd(120_000_000),
                is_credit: true,
                timestamp_seconds: 1726300100,
                narration: "Nhan tien".to_string(),
            },
            ScreenableTransaction {
                tx_id: "TX-PT-OUT".to_string(),
                account_number: "0071000888".to_string(),
                counterparty_name: "Ba C".to_string(),
                counterparty_account: "120100077".to_string(),
                amount: Money::vnd(118_000_000), // 98.3% out within 300s
                is_credit: false,
                timestamp_seconds: 1726300400,
                narration: "Rut tien gap".to_string(),
            },
        ];

        let alerts = screen_transactions_aml(&txs);
        assert!(!alerts.is_empty());

        // Verify High Value alert
        let hv = alerts.iter().find(|a| a.rule_code == AmlAlertCode::HighValueThreshold);
        assert!(hv.is_some());
        assert_eq!(hv.unwrap().severity, AmlSeverity::High);

        // Verify Pass Through alert
        let pt = alerts.iter().find(|a| a.rule_code == AmlAlertCode::PassThroughTransit);
        assert!(pt.is_some());
        assert_eq!(pt.unwrap().severity, AmlSeverity::Critical);

        // Verify STR generation
        let str_rep = build_str_report(pt.unwrap(), &txs[1], "Nghi ngo tai khoan trung chuyen rua tien.");
        assert_eq!(str_rep.severity, "CRITICAL");
        assert!(str_rep.merkle_proof_hash.starts_with("0x"));
    }

    #[test]
    fn test_merkle_audit_root_tamper_evidence() {
        let hashes_1 = vec![
            "0x1111111111111111".to_string(),
            "0x2222222222222222".to_string(),
            "0x3333333333333333".to_string(),
        ];
        let root_1 = compute_merkle_root(&hashes_1);
        assert!(root_1.starts_with("0x"));

        // Tamper single bit in second hash
        let hashes_2 = vec![
            "0x1111111111111111".to_string(),
            "0x2222222222222223".to_string(),
            "0x3333333333333333".to_string(),
        ];
        let root_2 = compute_merkle_root(&hashes_2);

        // Roots must diverge completely (avalanche effect)
        assert_ne!(root_1, root_2);
    }
}
