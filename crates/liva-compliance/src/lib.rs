//! LIVA Banking — `liva-compliance` Crate
//!
//! Declarative Policy Engine for AML Surveillance, Dynamic 4-Tier Maker-Checker Dual Control,
//! Cryptographic RFC 6962 SHA-256 Merkle Audit Trail Configuration Sealing,
//! HITL Quarantine Governance, and Decree 13/2023/NĐ-CP PII Sanitization.
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod aml;
pub mod audit;
pub mod maker_checker;
pub mod quarantine;
pub mod sanitizer;

// Crate-level re-exports for ergonomic access
pub use aml::{
    compute_amount_mad, compute_modified_zscore_bps, integer_jaro, integer_jaro_winkler,
    normalize_ascii, rule_code_prefix, AmlAlert, AmlAlertCode, AmlPolicyConfig, AmlPolicyMetadata,
    AmlScreeningEngine, AmlSeverity, AmountDistributionConfig, AnomalyDetectionConfig,
    EnhancedAmlAlert, HighValuePolicyConfig, NightVelocityPolicyConfig, ScreenableTransaction,
    SemanticNarrationConfig, SlmExplanationConfig, StrReportingConfig, StructuringPolicyConfig,
    SuspiciousCluster, SuspiciousTransactionReport, TemporalVelocityConfig,
    TransitMulePolicyConfig, WatchlistPolicyConfig,
};

pub use audit::{
    canonical_json_string, compute_config_merkle_tree, compute_config_seal, compute_merkle_root,
    create_config_seal, hash_leaf, hash_node, verify_config_integrity, BinaryMerkleTree,
    ConfigSealError, ConfigSealRecord, MerkleAuditError, MerkleInclusionProof, ProofStep,
    SiblingPosition, RFC6962_LEAF_PREFIX, RFC6962_NODE_PREFIX,
};

pub use maker_checker::{
    ApprovalAction, ApprovalAuditRecord, ApprovalDecision, ApprovalMode, ApprovalProposal,
    ApprovalRequest, ApprovalTierRule, MakerCheckerEngine, MakerCheckerError, MakerCheckerMetadata,
    MakerCheckerPolicyConfig, ProposalStatus, SegregationOfDutiesRules, TokenGovernanceConfig,
};

pub use quarantine::{HitlQuarantineGovernance, QuarantineRiskTier, QuarantineStatus, QuarantinedItem};

pub use sanitizer::{is_corporate_entity, sanitize_counterparty_name, sanitize_decree13_pii};
