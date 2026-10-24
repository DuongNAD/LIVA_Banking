//! Dynamic 4-Tier Maker-Checker Policy Schema.
//!
//! Enforces Circular 09/2020/TT-NHNN Articles 16 & 18 (Dual Control / 4-Eyes Principle).
//! Defines the 4-tier approval threshold matrix, strict Segregation of Duties (SoD) invariants,
//! and cryptographic token governance.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Master declarative Maker-Checker policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerCheckerPolicyConfig {
    pub metadata: MakerCheckerMetadata,
    pub tiers: Vec<ApprovalTierRule>,
    pub sod_rules: SegregationOfDutiesRules,
    pub token_governance: TokenGovernanceConfig,
}

impl Default for MakerCheckerPolicyConfig {
    fn default() -> Self {
        let yaml = include_str!("../../config/maker_checker_policy_default.yaml");
        serde_yaml::from_str(yaml).expect("Default Maker-Checker policy YAML must be valid")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerCheckerMetadata {
    pub name: String,
    pub version: String,
    pub circular_reference: String,
}

/// Approval modes across tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalMode {
    /// Tier 1: Micro-variance auto-approval (< 10M VND)
    Auto,
    /// Tier 2: Standard single-checker review (10M - 100M VND)
    SingleChecker,
    /// Tier 3: Senior checker escalation (100M - 500M VND)
    SeniorChecker,
    /// Tier 4: Dual-checker 4-eyes dual control (>= 500M VND)
    DualChecker,
}

/// Definition of an approval tier rule in the threshold matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalTierRule {
    pub tier_level: u32,
    pub name: String,
    pub min_amount_vnd: u64,
    pub max_amount_vnd: Option<u64>,
    pub approval_mode: ApprovalMode,
    pub required_checkers: usize,
    pub allowed_checker_roles_stage1: Vec<String>,
    #[serde(default)]
    pub allowed_checker_roles_stage2: Vec<String>,
    pub statutory_reference: String,
}

/// Segregation of Duties (SoD) rules enforced by the policy engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegregationOfDutiesRules {
    pub enforce_maker_neq_checker: bool,
    pub enforce_checker1_neq_checker2: bool,
    pub lock_on_partial_approval: bool,
}

/// Token governance settings for HITL single-use authorization tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenGovernanceConfig {
    pub token_ttl_seconds: i64, // Default 900s (15 minutes)
    pub single_use: bool,
    pub hmac_key_label: String,
}

/// Lifecycle status of an approval proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalStatus {
    Draft,
    PendingChecker1,
    PendingChecker2,
    Approved,
    Rejected,
    Expired,
}

/// Action decision by checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalDecision {
    Approve,
    Reject,
}

/// Incoming proposal creation request by Maker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: String,
    pub amount_vnd: u64,
    pub description: String,
    pub maker_id: String,
    pub maker_role: String,
}

/// Action submitted by Checker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAction {
    pub decision: ApprovalDecision,
    pub checker_id: String,
    pub checker_role: String,
    pub token: String,
    pub notes: String,
}

/// Cryptographic audit record for every action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalAuditRecord {
    pub record_id: String,
    pub proposal_id: String,
    pub action: String,
    pub operator_id: String,
    pub operator_role: String,
    pub timestamp: i64,
    pub notes: String,
    pub signature_hmac: String,
}

/// Full proposal state with complete lifecycle audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalProposal {
    pub proposal_id: String,
    pub tier_level: u32,
    pub approval_mode: ApprovalMode,
    pub amount_vnd: u64,
    pub description: String,
    pub maker_id: String,
    pub maker_role: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub status: ProposalStatus,
    pub current_token: Option<String>,
    pub checker1_id: Option<String>,
    pub checker1_role: Option<String>,
    pub checker1_at: Option<i64>,
    pub checker2_id: Option<String>,
    pub checker2_role: Option<String>,
    pub checker2_at: Option<i64>,
    pub audit_records: Vec<ApprovalAuditRecord>,
}

/// Explicit error codes for Maker-Checker and SoD violations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MakerCheckerError {
    #[error("Segregation of Duties violation: Maker ({maker_id}) cannot approve their own proposal (Circular 09/2020/TT-NHNN)")]
    SelfApprovalProhibited { maker_id: String },

    #[error("Dual Control violation: Stage 2 Checker ({checker_id}) cannot be the same person as Stage 1 Checker")]
    DuplicateCheckerProhibited { checker_id: String },

    #[error("Unauthorized role: Operator role '{actual_role}' is not in permitted roles: {required_roles:?}")]
    UnauthorizedRole {
        required_roles: Vec<String>,
        actual_role: String,
    },

    #[error("HITL authorization token expired at timestamp {expired_at} (current timestamp: {now})")]
    TokenExpired { expired_at: i64, now: i64 },

    #[error("Invalid or malformed HITL authorization token")]
    InvalidToken,

    #[error("HITL authorization token has already been consumed (Anti-Replay violation)")]
    TokenAlreadyUsed { token: String },

    #[error("Proposal with ID '{proposal_id}' was not found")]
    ProposalNotFound { proposal_id: String },

    #[error("Invalid proposal status for this operation: expected '{expected}', actual '{actual}'")]
    InvalidStatus { expected: String, actual: String },

    #[error("No approval tier rule matches transaction amount {amount_vnd} VND")]
    NoMatchingTier { amount_vnd: u64 },

    #[error("Segregation of Duties violation: {0}")]
    SodViolation(String),
}
