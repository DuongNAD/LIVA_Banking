//! HITL Quarantine Governance Engine.
//!
//! Enforces 4-tier risk classification, single-use anti-replay token validation,
//! configurable token TTL, and Maker-Checker release gates for quarantined transactions.

use super::schema::{QuarantineRiskTier, QuarantineStatus, QuarantinedItem};
use crate::maker_checker::{
    ApprovalAction, ApprovalRequest, MakerCheckerEngine, MakerCheckerError, ProposalStatus,
};
use std::collections::HashMap;
use subtle::ConstantTimeEq;

/// HITL Quarantine Governance Subsystem.
#[derive(Debug, Clone)]
pub struct HitlQuarantineGovernance {
    pub engine: MakerCheckerEngine,
    items: HashMap<String, QuarantinedItem>,
}

impl HitlQuarantineGovernance {
    /// Create a new governance manager wrapping a MakerCheckerEngine.
    pub fn new(engine: MakerCheckerEngine) -> Self {
        Self {
            engine,
            items: HashMap::new(),
        }
    }

    /// Create with default policy and standard HMAC key.
    pub fn with_default_policy() -> Self {
        Self::new(MakerCheckerEngine::with_default_policy())
    }

    /// 4-tier risk classification logic:
    /// - CRITICAL: amount >= 500M VND, statutory >= 400M VND, or active AML alert
    /// - HIGH: amount 100M - 500M VND
    /// - MEDIUM: amount 10M - 100M VND
    /// - LOW: amount < 10M VND
    pub fn classify_risk(amount_vnd: u64, has_aml_alert: bool) -> QuarantineRiskTier {
        if has_aml_alert || amount_vnd >= 500_000_000 || amount_vnd >= 400_000_000 {
            QuarantineRiskTier::Critical
        } else if amount_vnd >= 100_000_000 {
            QuarantineRiskTier::High
        } else if amount_vnd >= 10_000_000 {
            QuarantineRiskTier::Medium
        } else {
            QuarantineRiskTier::Low
        }
    }

    /// Quarantine a transaction or variance exception and initiate Maker-Checker governance.
    #[allow(clippy::too_many_arguments)]
    pub fn quarantine_item(
        &mut self,
        item_id: &str,
        source_tx_id: &str,
        amount_vnd: u64,
        reason: &str,
        has_aml_alert: bool,
        maker_id: &str,
        maker_role: &str,
        now_timestamp: i64,
    ) -> Result<QuarantinedItem, MakerCheckerError> {
        let risk_tier = Self::classify_risk(amount_vnd, has_aml_alert);

        let req = ApprovalRequest {
            request_id: format!("PROP-{}", item_id),
            amount_vnd,
            description: format!("{}: {}", reason, source_tx_id),
            maker_id: maker_id.to_string(),
            maker_role: maker_role.to_string(),
        };

        let proposal = self.engine.create_proposal(req, now_timestamp)?;

        let (status, current_token) = match proposal.status {
            ProposalStatus::Approved => (QuarantineStatus::Released, None),
            ProposalStatus::PendingChecker1 => {
                (QuarantineStatus::PendingReview, proposal.current_token.clone())
            }
            ProposalStatus::PendingChecker2 => (
                QuarantineStatus::UnderDualControl,
                proposal.current_token.clone(),
            ),
            ProposalStatus::Rejected => (QuarantineStatus::Rejected, None),
            ProposalStatus::Expired => (QuarantineStatus::Expired, None),
            ProposalStatus::Draft => (QuarantineStatus::PendingReview, None),
        };

        let item = QuarantinedItem {
            item_id: item_id.to_string(),
            source_tx_id: source_tx_id.to_string(),
            amount_vnd,
            risk_tier,
            reason: reason.to_string(),
            has_aml_alert,
            maker_id: maker_id.to_string(),
            created_at: now_timestamp,
            expires_at: proposal.expires_at,
            status,
            current_token,
            proposal_id: proposal.proposal_id,
        };

        self.items.insert(item_id.to_string(), item.clone());
        Ok(item)
    }

    /// Submit a Checker review to release, reject, or advance dual control on a quarantined item.
    pub fn review_quarantined_item(
        &mut self,
        item_id: &str,
        action: ApprovalAction,
        now_timestamp: i64,
    ) -> Result<QuarantinedItem, MakerCheckerError> {
        let item = self
            .items
            .get_mut(item_id)
            .ok_or_else(|| MakerCheckerError::ProposalNotFound {
                proposal_id: item_id.to_string(),
            })?;

        let proposal = self.engine.process_action(&item.proposal_id, action, now_timestamp)?;

        let (status, current_token) = match proposal.status {
            ProposalStatus::Approved => (QuarantineStatus::Released, None),
            ProposalStatus::PendingChecker1 => {
                (QuarantineStatus::PendingReview, proposal.current_token.clone())
            }
            ProposalStatus::PendingChecker2 => (
                QuarantineStatus::UnderDualControl,
                proposal.current_token.clone(),
            ),
            ProposalStatus::Rejected => (QuarantineStatus::Rejected, None),
            ProposalStatus::Expired => (QuarantineStatus::Expired, None),
            ProposalStatus::Draft => (QuarantineStatus::PendingReview, None),
        };

        item.status = status;
        item.current_token = current_token;
        item.expires_at = proposal.expires_at;

        Ok(item.clone())
    }

    /// Retrieve a quarantined item by ID.
    pub fn get_item(&self, item_id: &str) -> Option<&QuarantinedItem> {
        self.items.get(item_id)
    }

    /// Validate token with constant-time comparison.
    pub fn is_token_valid(&self, item_id: &str, candidate_token: &str, now_timestamp: i64) -> bool {
        if let Some(item) = self.items.get(item_id) {
            if now_timestamp > item.expires_at {
                return false;
            }
            if let Some(token) = &item.current_token {
                return bool::from(candidate_token.as_bytes().ct_eq(token.as_bytes()));
            }
        }
        false
    }
}
