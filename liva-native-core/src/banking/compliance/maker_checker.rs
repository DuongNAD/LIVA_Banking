//! Maker-Checker 4-Eyes Authorization Engine adhering to Circular 09/2020/TT-NHNN.
//!
//! Articles 16 & 18 of Circular 09/2020/TT-NHNN mandate strict Dual Control (4-Eyes principle)
//! for financial transaction exception handling and HITL match overrides.
//! - Role separation: Maker (initiator/accountant) vs Checker (authorizer/chief accountant).
//! - Strict self-approval prohibition (maker_id != checker_id, fail-closed).
//! - Single-use UUIDv4 HITL tokens with 15-minute TTL (900 seconds).
//! - Cryptographic audit trail with HMAC-SHA256 digital signatures and tamper resistance.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Circular 09/2020/TT-NHNN regulatory reference.
pub const CIRCULAR_09_REF: &str =
    "Circular 09/2020/TT-NHNN Articles 16 & 18 (Dual Control / 4-Eyes Principle)";

/// HITL single-use token TTL: 15 minutes (900 seconds).
pub const HITL_TOKEN_TTL_SECONDS: i64 = 15 * 60;

/// Authorization roles in the Dual Control workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthorizationRole {
    /// Accountant / Operator who investigates and proposes resolution for an exception.
    Maker,
    /// Chief Accountant / Treasury Manager who reviews and authorizes or rejects.
    Checker,
    /// Compliance Officer / Internal Auditor (Read-only access).
    Auditor,
}

/// Permitted HITL actions requiring Dual Control authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HitlActionType {
    /// Authorize a fuzzy-heuristic or low-confidence match.
    ApproveMatch,
    /// Reject an automated candidate match and revert transaction to Unmatched.
    RejectMatch,
    /// Authorize a 1-to-N or N-to-1 composite split allocation.
    ManualSplit,
    /// Authorize an amount discrepancy override within tolerance policy.
    OverrideDiscrepancy,
}

/// Lifecycle status of a HITL proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Revoked,
}

/// Final decision executed by the Checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckerDecision {
    Approve,
    Reject,
}

/// Detailed payload of a reconciliation HITL proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalDetails {
    pub bank_tx_id: String,
    pub selected_ledger_entry_ids: Vec<String>,
    pub matched_amount: u64,
    pub discrepancy_amount: i64,
    pub notes: Option<String>,
}

/// Immutable proposal initiated by a Maker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlProposal {
    pub proposal_id: String,
    pub match_id: String,
    pub maker_id: String,
    pub maker_name: String,
    pub action_type: HitlActionType,
    pub details: ProposalDetails,
    pub created_at: i64,
    pub expires_at: i64,
    pub status: ProposalStatus,
    pub hitl_token: String,
    pub proposal_hash: String,
}

/// Immutable cryptographic audit log entry for every Checker action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAuditRecord {
    pub record_id: String,
    pub proposal_id: String,
    pub match_id: String,
    pub maker_id: String,
    pub checker_id: String,
    pub checker_name: String,
    pub decision: CheckerDecision,
    pub timestamp: i64,
    pub proposal_hash: String,
    pub digital_signature: String,
    pub circular_reference: String,
    pub notes: Option<String>,
}

/// Errors raised by the Maker-Checker authorization engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MakerCheckerError {
    SelfApprovalProhibited {
        maker_id: String,
    },
    TokenExpired {
        expired_at: i64,
        current_time: i64,
    },
    TokenAlreadyUsed {
        token: String,
    },
    InvalidToken,
    ProposalNotFound {
        id: String,
    },
    InvalidProposalState {
        current: ProposalStatus,
        expected: ProposalStatus,
    },
    CryptoVerificationFailed,
    EmptyCheckerId,
}

impl std::fmt::Display for MakerCheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MakerCheckerError::SelfApprovalProhibited { maker_id } => {
                write!(
                    f,
                    "Circular 09 Violation: Self-approval strictly prohibited for Maker '{maker_id}' (Fail-Closed Dual Control)"
                )
            }
            MakerCheckerError::TokenExpired {
                expired_at,
                current_time,
            } => {
                write!(
                    f,
                    "HITL Token expired at {expired_at} (current time: {current_time})"
                )
            }
            MakerCheckerError::TokenAlreadyUsed { token } => {
                write!(
                    f,
                    "HITL Token '{token}' has already been consumed (replay attempt rejected)"
                )
            }
            MakerCheckerError::InvalidToken => {
                write!(f, "Invalid HITL Token provided for proposal")
            }
            MakerCheckerError::ProposalNotFound { id } => {
                write!(f, "HITL Proposal '{id}' not found")
            }
            MakerCheckerError::InvalidProposalState { current, expected } => {
                write!(
                    f,
                    "Invalid Proposal State: current {current:?}, expected {expected:?}"
                )
            }
            MakerCheckerError::CryptoVerificationFailed => {
                write!(f, "Cryptographic signature verification failed")
            }
            MakerCheckerError::EmptyCheckerId => {
                write!(
                    f,
                    "Checker ID cannot be empty (Dual Control requires identified Checker)"
                )
            }
        }
    }
}

impl std::error::Error for MakerCheckerError {}

/// Computes a standard RFC 2104 HMAC-SHA256 signature for approval audit records.
pub fn compute_audit_signature(
    key: &[u8; 32],
    proposal_id: &str,
    maker_id: &str,
    checker_id: &str,
    decision: CheckerDecision,
    timestamp: i64,
    proposal_hash: &str,
) -> String {
    let payload = format!(
        "{proposal_id}|{maker_id}|{checker_id}|{decision:?}|{timestamp}|{proposal_hash}|{CIRCULAR_09_REF}"
    );

    let mut k_pad = [0u8; 64];
    k_pad[..32].copy_from_slice(key);

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k_pad[i];
        opad[i] ^= k_pad[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(payload.as_bytes());
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let mac = outer.finalize();

    hex::encode(mac)
}

/// Computes SHA-256 digest of proposal contents to prevent modification after creation.
fn hash_proposal_payload(
    proposal_id: &str,
    match_id: &str,
    maker_id: &str,
    action_type: HitlActionType,
    details: &ProposalDetails,
    created_at: i64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(proposal_id.as_bytes());
    hasher.update(b"|");
    hasher.update(match_id.as_bytes());
    hasher.update(b"|");
    hasher.update(maker_id.as_bytes());
    hasher.update(b"|");
    hasher.update(format!("{action_type:?}").as_bytes());
    hasher.update(b"|");
    hasher.update(details.bank_tx_id.as_bytes());
    hasher.update(b"|");
    for id in &details.selected_ledger_entry_ids {
        hasher.update(id.as_bytes());
        hasher.update(b",");
    }
    hasher.update(details.matched_amount.to_le_bytes());
    hasher.update(details.discrepancy_amount.to_le_bytes());
    hasher.update(created_at.to_le_bytes());
    hex::encode(hasher.finalize())
}

/// In-memory Maker-Checker Authorization Engine.
#[derive(Debug, Clone, Default)]
pub struct MakerCheckerEngine {
    proposals: HashMap<String, HitlProposal>,
    token_to_proposal: HashMap<String, String>,
    used_tokens: HashSet<String>,
    audit_records: Vec<ApprovalAuditRecord>,
}

impl MakerCheckerEngine {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            token_to_proposal: HashMap::new(),
            used_tokens: HashSet::new(),
            audit_records: Vec::new(),
        }
    }

    /// Creates a new HITL proposal initiated by a Maker.
    /// Generates a single-use UUIDv4 token with 15-minute TTL.
    pub fn create_proposal(
        &mut self,
        maker_id: &str,
        maker_name: &str,
        match_id: &str,
        action_type: HitlActionType,
        details: ProposalDetails,
        now_ts: i64,
    ) -> Result<(HitlProposal, String), MakerCheckerError> {
        let proposal_id = Uuid::new_v4().to_string();
        let hitl_token = Uuid::new_v4().to_string();
        let expires_at = now_ts + HITL_TOKEN_TTL_SECONDS;

        let proposal_hash = hash_proposal_payload(
            &proposal_id,
            match_id,
            maker_id,
            action_type,
            &details,
            now_ts,
        );

        let proposal = HitlProposal {
            proposal_id: proposal_id.clone(),
            match_id: match_id.to_string(),
            maker_id: maker_id.to_string(),
            maker_name: maker_name.to_string(),
            action_type,
            details,
            created_at: now_ts,
            expires_at,
            status: ProposalStatus::Pending,
            hitl_token: hitl_token.clone(),
            proposal_hash,
        };

        self.proposals.insert(proposal_id.clone(), proposal.clone());
        self.token_to_proposal
            .insert(hitl_token.clone(), proposal_id);

        Ok((proposal, hitl_token))
    }

    /// Evaluates and submits a Checker authorization decision.
    ///
    /// Enforces:
    /// 1. `maker_id != checker_id` (Fail-Closed self-approval defense).
    /// 2. Single-use UUIDv4 token validity and 15-minute TTL.
    /// 3. Token replay prevention (marked used immediately).
    /// 4. Cryptographic audit record generation.
    pub fn submit_decision(
        &mut self,
        checker_id: &str,
        checker_name: &str,
        proposal_id: &str,
        hitl_token: &str,
        decision: CheckerDecision,
        notes: Option<String>,
        hmac_key: &[u8; 32],
        now_ts: i64,
    ) -> Result<ApprovalAuditRecord, MakerCheckerError> {
        // 1. Verify token has not already been used
        if self.used_tokens.contains(hitl_token) {
            return Err(MakerCheckerError::TokenAlreadyUsed {
                token: hitl_token.to_string(),
            });
        }

        // 2. Fetch proposal
        let proposal = self.proposals.get_mut(proposal_id).ok_or_else(|| {
            MakerCheckerError::ProposalNotFound {
                id: proposal_id.to_string(),
            }
        })?;

        // 3. Verify proposal is pending
        if proposal.status != ProposalStatus::Pending {
            return Err(MakerCheckerError::InvalidProposalState {
                current: proposal.status,
                expected: ProposalStatus::Pending,
            });
        }

        // 4. Verify token matches
        if proposal.hitl_token != hitl_token {
            return Err(MakerCheckerError::InvalidToken);
        }

        // 5. Verify TTL (15 minutes)
        if now_ts > proposal.expires_at {
            proposal.status = ProposalStatus::Expired;
            return Err(MakerCheckerError::TokenExpired {
                expired_at: proposal.expires_at,
                current_time: now_ts,
            });
        }

        // 6. ENFORCE DUAL CONTROL: Maker != Checker (Circular 09/2020/TT-NHNN)
        if checker_id.trim().is_empty() {
            return Err(MakerCheckerError::EmptyCheckerId);
        }
        if proposal.maker_id.trim() == checker_id.trim() {
            // Fail closed! Proposal remains pending, token remains unconsumed.
            return Err(MakerCheckerError::SelfApprovalProhibited {
                maker_id: proposal.maker_id.clone(),
            });
        }

        // 7. Consume token (single use)
        self.used_tokens.insert(hitl_token.to_string());

        // 8. Update proposal state
        proposal.status = match decision {
            CheckerDecision::Approve => ProposalStatus::Approved,
            CheckerDecision::Reject => ProposalStatus::Rejected,
        };

        // 9. Generate cryptographic audit record with digital signature
        let digital_signature = compute_audit_signature(
            hmac_key,
            &proposal.proposal_id,
            &proposal.maker_id,
            checker_id,
            decision,
            now_ts,
            &proposal.proposal_hash,
        );

        let record = ApprovalAuditRecord {
            record_id: Uuid::new_v4().to_string(),
            proposal_id: proposal.proposal_id.clone(),
            match_id: proposal.match_id.clone(),
            maker_id: proposal.maker_id.clone(),
            checker_id: checker_id.to_string(),
            checker_name: checker_name.to_string(),
            decision,
            timestamp: now_ts,
            proposal_hash: proposal.proposal_hash.clone(),
            digital_signature,
            circular_reference: CIRCULAR_09_REF.to_string(),
            notes,
        };

        self.audit_records.push(record.clone());
        Ok(record)
    }

    /// Verifies the cryptographic HMAC signature of an audit record.
    pub fn verify_audit_record(&self, record: &ApprovalAuditRecord, hmac_key: &[u8; 32]) -> bool {
        let expected = compute_audit_signature(
            hmac_key,
            &record.proposal_id,
            &record.maker_id,
            &record.checker_id,
            record.decision,
            record.timestamp,
            &record.proposal_hash,
        );
        expected == record.digital_signature
    }

    /// Fetches a proposal by ID.
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&HitlProposal> {
        self.proposals.get(proposal_id)
    }

    /// Lists all currently pending proposals.
    pub fn list_pending_proposals(&self) -> Vec<HitlProposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Pending)
            .cloned()
            .collect()
    }

    /// Returns audit records filtered optionally by match_id.
    pub fn get_audit_history(&self, match_id: Option<&str>) -> Vec<ApprovalAuditRecord> {
        match match_id {
            Some(mid) => self
                .audit_records
                .iter()
                .filter(|r| r.match_id == mid)
                .cloned()
                .collect(),
            None => self.audit_records.clone(),
        }
    }

    /// Revokes a pending proposal.
    pub fn revoke_proposal(&mut self, proposal_id: &str) -> Result<(), MakerCheckerError> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or_else(|| {
            MakerCheckerError::ProposalNotFound {
                id: proposal_id.to_string(),
            }
        })?;

        if proposal.status != ProposalStatus::Pending {
            return Err(MakerCheckerError::InvalidProposalState {
                current: proposal.status,
                expected: ProposalStatus::Pending,
            });
        }

        proposal.status = ProposalStatus::Revoked;
        self.used_tokens.insert(proposal.hitl_token.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HMAC_KEY: &[u8; 32] = b"maker-checker-test-audit-key-32b";

    fn dummy_details() -> ProposalDetails {
        ProposalDetails {
            bank_tx_id: "tx-bank-001".to_string(),
            selected_ledger_entry_ids: vec!["gl-entry-101".to_string()],
            matched_amount: 15_000_000,
            discrepancy_amount: 0,
            notes: Some("Discrepancy within normal wire fee".to_string()),
        }
    }

    #[test]
    fn test_maker_checker_happy_path() {
        let mut engine = MakerCheckerEngine::new();
        let t0 = 1726000000;

        // Maker creates proposal
        let (proposal, token) = engine
            .create_proposal(
                "maker_user_01",
                "Nguyen Van A (Accountant)",
                "match_99",
                HitlActionType::ApproveMatch,
                dummy_details(),
                t0,
            )
            .expect("Should create proposal successfully");

        assert_eq!(proposal.status, ProposalStatus::Pending);
        assert_eq!(proposal.expires_at, t0 + 15 * 60);
        assert!(!token.is_empty());

        // Checker approves proposal
        let audit = engine
            .submit_decision(
                "checker_user_02",
                "Tran Thi B (Chief Accountant)",
                &proposal.proposal_id,
                &token,
                CheckerDecision::Approve,
                Some("Verified against bank confirmation".to_string()),
                TEST_HMAC_KEY,
                t0 + 120, // 2 minutes later
            )
            .expect("Checker should approve successfully");

        assert_eq!(audit.decision, CheckerDecision::Approve);
        assert_eq!(audit.maker_id, "maker_user_01");
        assert_eq!(audit.checker_id, "checker_user_02");
        assert!(engine.verify_audit_record(&audit, TEST_HMAC_KEY));

        // Proposal updated
        let updated = engine.get_proposal(&proposal.proposal_id).unwrap();
        assert_eq!(updated.status, ProposalStatus::Approved);
    }

    #[test]
    fn test_maker_checker_fail_closed_on_self_approval() {
        let mut engine = MakerCheckerEngine::new();
        let t0 = 1726000000;

        let (proposal, token) = engine
            .create_proposal(
                "same_user_identity",
                "Nguyen Self Approver",
                "match_fail_closed",
                HitlActionType::ApproveMatch,
                dummy_details(),
                t0,
            )
            .unwrap();

        // Same user attempts to approve their own proposal
        let result = engine.submit_decision(
            "same_user_identity",
            "Nguyen Self Approver",
            &proposal.proposal_id,
            &token,
            CheckerDecision::Approve,
            None,
            TEST_HMAC_KEY,
            t0 + 30,
        );

        match result {
            Err(MakerCheckerError::SelfApprovalProhibited { maker_id }) => {
                assert_eq!(maker_id, "same_user_identity");
            }
            other => panic!("Expected SelfApprovalProhibited, got: {other:?}"),
        }

        // Proposal must still be pending (fail-closed)
        let state = engine.get_proposal(&proposal.proposal_id).unwrap();
        assert_eq!(state.status, ProposalStatus::Pending);
    }

    #[test]
    fn test_maker_checker_single_use_token_reuse_rejected() {
        let mut engine = MakerCheckerEngine::new();
        let t0 = 1726000000;

        let (proposal, token) = engine
            .create_proposal(
                "maker_user_01",
                "Maker",
                "match_single_use",
                HitlActionType::ApproveMatch,
                dummy_details(),
                t0,
            )
            .unwrap();

        // 1st consumption succeeds
        let _ = engine
            .submit_decision(
                "checker_user_02",
                "Checker",
                &proposal.proposal_id,
                &token,
                CheckerDecision::Approve,
                None,
                TEST_HMAC_KEY,
                t0 + 60,
            )
            .expect("First use should succeed");

        // 2nd consumption with same token MUST be rejected
        let second_attempt = engine.submit_decision(
            "checker_user_03",
            "Checker 2",
            &proposal.proposal_id,
            &token,
            CheckerDecision::Approve,
            None,
            TEST_HMAC_KEY,
            t0 + 120,
        );

        match second_attempt {
            Err(MakerCheckerError::TokenAlreadyUsed { .. }) => {}
            other => panic!("Expected TokenAlreadyUsed, got: {other:?}"),
        }
    }

    #[test]
    fn test_maker_checker_token_ttl_expiration() {
        let mut engine = MakerCheckerEngine::new();
        let t0 = 1726000000;

        let (proposal, token) = engine
            .create_proposal(
                "maker_user_01",
                "Maker",
                "match_ttl",
                HitlActionType::ApproveMatch,
                dummy_details(),
                t0,
            )
            .unwrap();

        // Attempt to submit at t0 + 901 seconds (> 15 minutes TTL)
        let expired_attempt = engine.submit_decision(
            "checker_user_02",
            "Checker",
            &proposal.proposal_id,
            &token,
            CheckerDecision::Approve,
            None,
            TEST_HMAC_KEY,
            t0 + 901,
        );

        match expired_attempt {
            Err(MakerCheckerError::TokenExpired {
                expired_at,
                current_time,
            }) => {
                assert_eq!(expired_at, t0 + 900);
                assert_eq!(current_time, t0 + 901);
            }
            other => panic!("Expected TokenExpired, got: {other:?}"),
        }

        let updated = engine.get_proposal(&proposal.proposal_id).unwrap();
        assert_eq!(updated.status, ProposalStatus::Expired);
    }

    #[test]
    fn test_maker_checker_signature_tamper_detection() {
        let mut engine = MakerCheckerEngine::new();
        let t0 = 1726000000;

        let (proposal, token) = engine
            .create_proposal(
                "maker_user_01",
                "Maker",
                "match_tamper",
                HitlActionType::ApproveMatch,
                dummy_details(),
                t0,
            )
            .unwrap();

        let mut audit = engine
            .submit_decision(
                "checker_user_02",
                "Checker",
                &proposal.proposal_id,
                &token,
                CheckerDecision::Approve,
                None,
                TEST_HMAC_KEY,
                t0 + 30,
            )
            .unwrap();

        assert!(engine.verify_audit_record(&audit, TEST_HMAC_KEY));

        // Tamper with checker_id
        audit.checker_id = "malicious_actor".to_string();
        assert!(
            !engine.verify_audit_record(&audit, TEST_HMAC_KEY),
            "Tampered checker_id must fail signature verification"
        );
    }
}
