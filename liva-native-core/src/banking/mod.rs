//! LIVA Banking Native Core module.
//!
//! Provides high-speed statement parsers (VCB, TCB, BIDV, CTG, MBB, VBA),
//! zero-hallucination deterministic reconciliation matching engine,
//! and Decree 13/2023/NĐ-CP & Circular 09/2020/TT-NHNN on-premise security & compliance.

pub mod compliance;
pub mod generator;
pub mod mcp;
pub mod models;
pub mod parser;
pub mod reconciliation;
pub mod risk;
pub mod treasury;

pub use mcp::{
    BankingReconcileArgs, BankingReconcileResult, ComplianceAmlScreenArgs,
    ComplianceAmlScreenResult, CreditRiskScoringArgs, CreditRiskScoringResult,
    TreasuryPaymentOrderArgs, TreasuryPaymentOrderResult,
    handle_banking_reconcile, handle_compliance_aml_screen, handle_credit_risk_scoring,
    handle_treasury_payment_order,
};

pub use risk::{
    CashflowForecastReport, CreditRiskEngine, DailyBalanceProjection, DailyCashflowPoint,
    DscrReport, DscrRiskCategory, LiquidityStatus, QuickRatioReport,
};
pub use treasury::{
    PaymentOrder, PaymentOrderStatus, TreasuryEngine, find_payment_order_by_id,
    init_payment_orders_table, list_payment_orders, save_payment_order, update_payment_order,
};

pub use compliance::{
    ApprovalAuditRecord, AuditLedger, BinaryMerkleTree, EgressTrafficReport, EgressTrafficTracker,
    HitlProposal, MakerCheckerEngine, MerkleInclusionProof, ZeroEgressNetfilter,
    get_compliance_status, is_egress_permitted, sanitize_pii,
    verify_statement_processing_zero_egress, verify_zero_egress, verify_zero_egress_from,
};
pub use generator::*;
pub use models::*;
pub use parser::{BankStatementParser, ParserError, sniff_and_parse};
pub use reconciliation::ReconciliationEngine;

#[cfg(test)]
mod tests;
