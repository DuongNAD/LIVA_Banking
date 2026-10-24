//! LIVA Banking — `compliance_aml.rs`
//!
//! Re-exports and delegates to `liva_compliance` for centralized statutory compliance:
//! 1. Vietnam Decree 13/2023/NĐ-CP Personal Data Protection (PII Sanitization & Corporate Preservation).
//! 2. Law No. 14/2022/QH15 & Decision 11/2023/QĐ-TTg (AML/CTF Statutory Screening & Form STR Generation).
//! 3. Tamper-evident SHA-256 Merkle Audit Tree certification.

pub use liva_compliance::{
    compute_merkle_root, sanitize_decree13_pii, AmlAlert, AmlAlertCode, AmlSeverity,
    ScreenableTransaction, SuspiciousTransactionReport,
};

/// Screen a slice of transactions against statutory AML indicators via `liva_compliance`.
pub fn screen_transactions_aml(txs: &[ScreenableTransaction]) -> Vec<AmlAlert> {
    let engine = liva_compliance::AmlScreeningEngine::with_default_policy();
    engine.screen_transactions(txs)
}

/// Helper to build Form STR according to Appendix II Circular 09/2023/TT-NHNN via `liva_compliance`.
pub fn build_str_report(
    alert: &AmlAlert,
    tx: &ScreenableTransaction,
    compliance_officer_notes: &str,
) -> SuspiciousTransactionReport {
    let engine = liva_compliance::AmlScreeningEngine::with_default_policy();
    engine.build_str_report(alert, tx, compliance_officer_notes)
}
