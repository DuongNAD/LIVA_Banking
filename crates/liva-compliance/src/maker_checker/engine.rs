//! Dynamic 4-Tier Maker-Checker Policy Engine.
//!
//! Implements strict Segregation of Duties (SoD), 4-tier approval threshold matrix,
//! dual-checker escalation for high-value transactions, single-use token governance with TTL,
//! anti-replay registry, and constant-time token comparison.

use super::schema::{
    ApprovalAction, ApprovalAuditRecord, ApprovalDecision, ApprovalMode, ApprovalProposal,
    ApprovalRequest, ApprovalTierRule, MakerCheckerError, MakerCheckerPolicyConfig, ProposalStatus,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use subtle::ConstantTimeEq;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Engine executing the 4-Tier Maker-Checker workflow.
#[derive(Debug, Clone)]
pub struct MakerCheckerEngine {
    pub config: MakerCheckerPolicyConfig,
    hmac_key: Vec<u8>,
    proposals: HashMap<String, ApprovalProposal>,
    consumed_tokens: HashSet<String>,
}

impl MakerCheckerEngine {
    /// Create a new MakerCheckerEngine with policy configuration and audit HMAC secret key.
    pub fn new(config: MakerCheckerPolicyConfig, hmac_key: &[u8]) -> Self {
        Self {
            config,
            hmac_key: hmac_key.to_vec(),
            proposals: HashMap::new(),
            consumed_tokens: HashSet::new(),
        }
    }

    /// Create an engine with the default policy and a standard key.
    pub fn with_default_policy() -> Self {
        Self::new(
            MakerCheckerPolicyConfig::default(),
            b"LIVA_DEFAULT_MAKER_CHECKER_HMAC_SECRET_KEY",
        )
    }

    /// Classify which approval tier applies to a transaction amount.
    pub fn classify_tier(&self, amount_vnd: u64) -> Result<&ApprovalTierRule, MakerCheckerError> {
        for tier in &self.config.tiers {
            let min_ok = amount_vnd >= tier.min_amount_vnd;
            let max_ok = tier.max_amount_vnd.is_none_or(|max| amount_vnd < max);
            if min_ok && max_ok {
                return Ok(tier);
            }
        }
        Err(MakerCheckerError::NoMatchingTier { amount_vnd })
    }

    /// Submit a new proposal as Maker.
    pub fn create_proposal(
        &mut self,
        request: ApprovalRequest,
        now_timestamp: i64,
    ) -> Result<ApprovalProposal, MakerCheckerError> {
        let tier = self.classify_tier(request.amount_vnd)?;
        let tier_level = tier.tier_level;
        let approval_mode = tier.approval_mode;
        let ttl = self.config.token_governance.token_ttl_seconds;

        let mut proposal = ApprovalProposal {
            proposal_id: request.request_id.clone(),
            tier_level,
            approval_mode,
            amount_vnd: request.amount_vnd,
            description: request.description.clone(),
            maker_id: request.maker_id.clone(),
            maker_role: request.maker_role.clone(),
            created_at: now_timestamp,
            expires_at: now_timestamp.saturating_add(ttl),
            status: ProposalStatus::Draft,
            current_token: None,
            checker1_id: None,
            checker1_role: None,
            checker1_at: None,
            checker2_id: None,
            checker2_role: None,
            checker2_at: None,
            audit_records: Vec::new(),
        };

        // Record creation audit trail
        let init_record = Self::build_audit_record(
            &self.hmac_key,
            &proposal.proposal_id,
            "MAKER_SUBMITTED",
            &request.maker_id,
            &request.maker_role,
            now_timestamp,
            &format!("Submitted for Tier {} approval", tier_level),
        );
        proposal.audit_records.push(init_record);

        // Tier 1: Micro-variance auto-approval
        if approval_mode == ApprovalMode::Auto {
            proposal.status = ProposalStatus::Approved;
            let auto_record = Self::build_audit_record(
                &self.hmac_key,
                &proposal.proposal_id,
                "SYSTEM_AUTO_APPROVED",
                "SYSTEM_ENGINE",
                "SYSTEM_AUTOMATION",
                now_timestamp,
                "Tier 1 auto-approval: variance within micro limit (< 10M VND)",
            );
            proposal.audit_records.push(auto_record);
        } else {
            // Generate initial single-use token for Stage 1
            let token = Uuid::new_v4().to_string();
            proposal.current_token = Some(token);
            proposal.status = ProposalStatus::PendingChecker1;
        }

        self.proposals
            .insert(proposal.proposal_id.clone(), proposal.clone());
        Ok(proposal)
    }

    /// Process a Checker action (Approve or Reject).
    pub fn process_action(
        &mut self,
        proposal_id: &str,
        action: ApprovalAction,
        now_timestamp: i64,
    ) -> Result<ApprovalProposal, MakerCheckerError> {
        // Read state snapshot first to avoid borrow conflicts
        let (amount_vnd, status, expires_at, current_token, maker_id, checker1_id) = {
            let p = self.proposals.get(proposal_id).ok_or_else(|| {
                MakerCheckerError::ProposalNotFound {
                    proposal_id: proposal_id.to_string(),
                }
            })?;
            (
                p.amount_vnd,
                p.status,
                p.expires_at,
                p.current_token.clone(),
                p.maker_id.clone(),
                p.checker1_id.clone(),
            )
        };

        // 1. Verify proposal status allows review
        if status != ProposalStatus::PendingChecker1 && status != ProposalStatus::PendingChecker2 {
            return Err(MakerCheckerError::InvalidStatus {
                expected: "PendingChecker1 or PendingChecker2".to_string(),
                actual: format!("{:?}", status),
            });
        }

        // 2. Verify token expiration
        if now_timestamp > expires_at {
            if let Some(p) = self.proposals.get_mut(proposal_id) {
                p.status = ProposalStatus::Expired;
                p.current_token = None;
            }
            return Err(MakerCheckerError::TokenExpired {
                expired_at: expires_at,
                now: now_timestamp,
            });
        }

        // 3. Anti-Replay: Verify token was not already consumed
        if self.consumed_tokens.contains(&action.token) {
            return Err(MakerCheckerError::TokenAlreadyUsed {
                token: action.token.clone(),
            });
        }

        // 4. Constant-Time Token Comparison
        let expected_token = match &current_token {
            Some(t) => t,
            None => return Err(MakerCheckerError::InvalidToken),
        };
        if !bool::from(action.token.as_bytes().ct_eq(expected_token.as_bytes())) {
            return Err(MakerCheckerError::InvalidToken);
        }

        // 5. Segregation of Duties: Maker != Checker
        if self.config.sod_rules.enforce_maker_neq_checker && action.checker_id == maker_id {
            return Err(MakerCheckerError::SelfApprovalProhibited {
                maker_id: maker_id.clone(),
            });
        }

        let tier = self.classify_tier(amount_vnd)?.clone();
        let ttl = self.config.token_governance.token_ttl_seconds;

        // 6. Handle Stage 1 Review
        if status == ProposalStatus::PendingChecker1 {
            // Verify authorized role for Stage 1
            if !tier
                .allowed_checker_roles_stage1
                .contains(&action.checker_role)
            {
                return Err(MakerCheckerError::UnauthorizedRole {
                    required_roles: tier.allowed_checker_roles_stage1.clone(),
                    actual_role: action.checker_role.clone(),
                });
            }

            // Consume token
            self.consumed_tokens.insert(action.token.clone());

            let proposal = self.proposals.get_mut(proposal_id).unwrap();

            match action.decision {
                ApprovalDecision::Reject => {
                    proposal.status = ProposalStatus::Rejected;
                    proposal.current_token = None;
                    let audit = Self::build_audit_record(
                        &self.hmac_key,
                        &proposal.proposal_id,
                        "CHECKER1_REJECTED",
                        &action.checker_id,
                        &action.checker_role,
                        now_timestamp,
                        &action.notes,
                    );
                    proposal.audit_records.push(audit);
                }
                ApprovalDecision::Approve => {
                    proposal.checker1_id = Some(action.checker_id.clone());
                    proposal.checker1_role = Some(action.checker_role.clone());
                    proposal.checker1_at = Some(now_timestamp);

                    let audit = Self::build_audit_record(
                        &self.hmac_key,
                        &proposal.proposal_id,
                        "CHECKER1_APPROVED",
                        &action.checker_id,
                        &action.checker_role,
                        now_timestamp,
                        &action.notes,
                    );
                    proposal.audit_records.push(audit);

                    // If Tier 4: Dual-Checker required
                    if tier.approval_mode == ApprovalMode::DualChecker {
                        proposal.status = ProposalStatus::PendingChecker2;
                        // Issue fresh token for Stage 2 with refreshed TTL
                        let new_token = Uuid::new_v4().to_string();
                        proposal.current_token = Some(new_token);
                        proposal.expires_at = now_timestamp.saturating_add(ttl);
                    } else {
                        // Tier 2 or 3: Fully approved by single checker
                        proposal.status = ProposalStatus::Approved;
                        proposal.current_token = None;
                    }
                }
            }
        } else if status == ProposalStatus::PendingChecker2 {
            // 7. Handle Stage 2 Review (Tier 4 Dual Checker)
            // Segregation of Duties: Checker 1 != Checker 2
            if self.config.sod_rules.enforce_checker1_neq_checker2 {
                if let Some(c1) = &checker1_id {
                    if action.checker_id == *c1 {
                        return Err(MakerCheckerError::DuplicateCheckerProhibited {
                            checker_id: action.checker_id.clone(),
                        });
                    }
                }
            }

            // Verify authorized role for Stage 2
            if !tier
                .allowed_checker_roles_stage2
                .contains(&action.checker_role)
            {
                return Err(MakerCheckerError::UnauthorizedRole {
                    required_roles: tier.allowed_checker_roles_stage2.clone(),
                    actual_role: action.checker_role.clone(),
                });
            }

            // Consume token
            self.consumed_tokens.insert(action.token.clone());

            let proposal = self.proposals.get_mut(proposal_id).unwrap();

            match action.decision {
                ApprovalDecision::Reject => {
                    proposal.status = ProposalStatus::Rejected;
                    proposal.current_token = None;
                    let audit = Self::build_audit_record(
                        &self.hmac_key,
                        &proposal.proposal_id,
                        "CHECKER2_REJECTED",
                        &action.checker_id,
                        &action.checker_role,
                        now_timestamp,
                        &action.notes,
                    );
                    proposal.audit_records.push(audit);
                }
                ApprovalDecision::Approve => {
                    proposal.checker2_id = Some(action.checker_id.clone());
                    proposal.checker2_role = Some(action.checker_role.clone());
                    proposal.checker2_at = Some(now_timestamp);
                    proposal.status = ProposalStatus::Approved;
                    proposal.current_token = None;

                    let audit = Self::build_audit_record(
                        &self.hmac_key,
                        &proposal.proposal_id,
                        "CHECKER2_APPROVED_FINAL",
                        &action.checker_id,
                        &action.checker_role,
                        now_timestamp,
                        &action.notes,
                    );
                    proposal.audit_records.push(audit);
                }
            }
        }

        let result = self.proposals.get(proposal_id).unwrap().clone();
        Ok(result)
    }

    /// Retrieve an existing proposal by ID.
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&ApprovalProposal> {
        self.proposals.get(proposal_id)
    }

    pub fn build_audit_record(
        hmac_key: &[u8],
        proposal_id: &str,
        action: &str,
        operator_id: &str,
        operator_role: &str,
        timestamp: i64,
        notes: &str,
    ) -> ApprovalAuditRecord {
        let record_id = Uuid::new_v4().to_string();
        let mut mac =
            HmacSha256::new_from_slice(hmac_key).expect("HMAC accepts any key size");
        mac.update(record_id.as_bytes());
        mac.update(b"|");
        mac.update(proposal_id.as_bytes());
        mac.update(b"|");
        mac.update(action.as_bytes());
        mac.update(b"|");
        mac.update(operator_id.as_bytes());
        mac.update(b"|");
        mac.update(operator_role.as_bytes());
        mac.update(b"|");
        mac.update(&timestamp.to_be_bytes());

        let sig: [u8; 32] = mac.finalize().into_bytes().into();

        ApprovalAuditRecord {
            record_id,
            proposal_id: proposal_id.to_string(),
            action: action.to_string(),
            operator_id: operator_id.to_string(),
            operator_role: operator_role.to_string(),
            timestamp,
            notes: notes.to_string(),
            signature_hmac: hex::encode(sig),
        }
    }
}
