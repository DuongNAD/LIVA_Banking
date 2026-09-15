//! On-premise security, cryptographic auditing, and Decree 13 / Circular 09 compliance module.

pub mod aml;
pub mod audit_ledger;
pub mod maker_checker;
pub mod merkle_audit;
pub mod sanitizer;
pub mod security;

#[cfg(test)]
pub mod tests;

pub use aml::{
    AML_CRITICAL_VALUE_THRESHOLD_VND, AML_HIGH_VALUE_THRESHOLD_VND,
    AML_PASS_THROUGH_WINDOW_SECONDS, AML_STRUCTURING_WINDOW_SECONDS, AmlAlert,
    AmlRuleCode, AmlScreeningEngine, AmlSeverity, SuspiciousTransactionReport,
    TransactionScreeningItem,
};

pub use audit_ledger::{AuditLedger, AuditVerificationReport, genesis_hash};
pub use maker_checker::{
    ApprovalAuditRecord, AuthorizationRole, CIRCULAR_09_REF, CheckerDecision,
    HITL_TOKEN_TTL_SECONDS, HitlActionType, HitlProposal, MakerCheckerEngine, MakerCheckerError,
    ProposalDetails, ProposalStatus, compute_audit_signature,
};
pub use merkle_audit::{
    BinaryMerkleTree, MerkleAuditError, MerkleInclusionProof, ProofStep, RFC6962_LEAF_PREFIX,
    RFC6962_NODE_PREFIX, SiblingPosition, TransactionAuditLeaf, hash_leaf, hash_node,
    largest_power_of_two_less_than,
};
pub use sanitizer::sanitize_pii;
pub use security::{
    EgressTrafficReport, EgressTrafficTracker, EgressViolationError, GLOBAL_EGRESS_TRACKER,
    ZeroEgressNetfilter, get_audit_key, get_compliance_status, is_egress_permitted,
    verify_statement_processing_zero_egress, verify_zero_egress, verify_zero_egress_from,
};
