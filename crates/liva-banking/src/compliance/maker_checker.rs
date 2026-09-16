//! Maker-Checker Dual Control Engine conforming to Circular 09/2020/TT-NHNN.
//!
//! Enforces:
//! - Separation of Duties (SoD): Maker != Checker (fail-closed)
//! - Tri-Identity Binding: Verification of user_id, employee_id, and citizen_id_hash (CCCD)
//! - 15-Minute TTL (900 seconds) single-use cryptographic challenge tokens
//! - STALE Invalidation: Payload mutation changes hash, revoking pending approvals

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

pub const CHALLENGE_TOKEN_TTL_SECONDS: i64 = 15 * 60; // 900 seconds

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Maker,
    Checker,
    Auditor,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserIdentity {
    pub user_id: String,
    pub employee_id: String,
    pub citizen_id_hash: String,
    pub role: UserRole,
    pub name: String,
}

impl UserIdentity {
    /// Checks if two identities refer to the same physical individual.
    pub fn is_same_person(&self, other: &UserIdentity) -> bool {
        self.user_id == other.user_id
            || self.employee_id == other.employee_id
            || self.citizen_id_hash == other.citizen_id_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalPayload {
    pub proposal_id: String,
    pub bank_tx_id: usize,
    pub erp_doc_ids: Vec<String>,
    pub matched_amount: u64,
    pub fee_amount: u64,
    pub version: u64,
}

impl ApprovalPayload {
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.proposal_id.as_bytes());
        hasher.update(&self.bank_tx_id.to_le_bytes());
        for id in &self.erp_doc_ids {
            hasher.update(id.as_bytes());
        }
        hasher.update(&self.matched_amount.to_le_bytes());
        hasher.update(&self.fee_amount.to_le_bytes());
        hasher.update(&self.version.to_le_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalLifecycle {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Stale,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeToken {
    pub token: String,
    pub proposal_id: String,
    pub payload_hash: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub used: bool,
}

pub struct MakerCheckerEngine {
    secret_salt: Vec<u8>,
    tokens: HashMap<String, ChallengeToken>,
}

impl MakerCheckerEngine {
    pub fn new(secret_salt: &[u8]) -> Self {
        Self {
            secret_salt: secret_salt.to_vec(),
            tokens: HashMap::new(),
        }
    }

    /// Generates a single-use 15-minute TTL challenge token for Checker authorization.
    pub fn issue_challenge(
        &mut self,
        maker: &UserIdentity,
        checker: &UserIdentity,
        payload: &ApprovalPayload,
        now_ts: i64,
    ) -> Result<ChallengeToken, &'static str> {
        // Enforce Separation of Duties
        if maker.is_same_person(checker) {
            return Err("SoDViolation: Maker cannot be Checker on the same transaction (Circular 09/2020)");
        }

        if checker.role != UserRole::Checker && checker.role != UserRole::Admin {
            return Err("UnauthorizedRole: Only designated Checkers can authorize exceptions");
        }

        let payload_hash = payload.compute_hash();
        let token_uuid = Uuid::new_v4().to_string();

        let mut hasher = Sha256::new();
        hasher.update(&self.secret_salt);
        hasher.update(token_uuid.as_bytes());
        hasher.update(checker.user_id.as_bytes());
        hasher.update(payload_hash.as_bytes());
        hasher.update(&now_ts.to_le_bytes());
        let token_str = hex::encode(hasher.finalize());

        let challenge = ChallengeToken {
            token: token_str.clone(),
            proposal_id: payload.proposal_id.clone(),
            payload_hash,
            issued_at: now_ts,
            expires_at: now_ts + CHALLENGE_TOKEN_TTL_SECONDS,
            used: false,
        };

        self.tokens.insert(token_str, challenge.clone());
        Ok(challenge)
    }

    /// Verifies and executes the Checker approval decision.
    pub fn verify_and_approve(
        &mut self,
        maker: &UserIdentity,
        checker: &UserIdentity,
        payload: &ApprovalPayload,
        token_str: &str,
        now_ts: i64,
    ) -> Result<ProposalLifecycle, &'static str> {
        // Enforce Separation of Duties
        if maker.is_same_person(checker) {
            return Err("SoDViolation: Maker and Checker cannot be the same physical individual");
        }

        let token = self
            .tokens
            .get_mut(token_str)
            .ok_or("InvalidToken: Challenge token does not exist")?;

        if token.used {
            return Err("TokenReused: Challenge token has already been consumed");
        }

        if now_ts > token.expires_at {
            token.used = true;
            return Err("TokenExpired: Challenge token exceeded 15-minute TTL");
        }

        let current_payload_hash = payload.compute_hash();
        if token.payload_hash != current_payload_hash {
            // Data modified after challenge issuance -> STALE invalidation
            token.used = true;
            return Ok(ProposalLifecycle::Stale);
        }

        token.used = true;
        Ok(ProposalLifecycle::Approved)
    }
}
