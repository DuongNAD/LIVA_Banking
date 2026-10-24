//! HITL Quarantine Governance Schema.
//!
//! Classifies financial exceptions and transactions into 4-tier risk categories
//! (Low, Medium, High, Critical) and governs the release lifecycle through Maker-Checker Dual Control.

use serde::{Deserialize, Serialize};

/// 4-tier risk classification for quarantined exceptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuarantineRiskTier {
    /// Low Risk: Minor variance < 10M VND -> Tier 1 Auto Eligible
    Low,
    /// Medium Risk: Standard variance 10M - 100M VND -> Tier 2 Single Checker
    Medium,
    /// High Risk: Significant variance 100M - 500M VND -> Tier 3 Senior Checker
    High,
    /// Critical Risk: Major variance >= 500M VND, statutory >= 400M, or AML Alert -> Tier 4 Dual Checker
    Critical,
}

/// Lifecycle status of a quarantined transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuarantineStatus {
    PendingReview,
    UnderDualControl,
    Released,
    Rejected,
    Expired,
}

/// A transaction or reconciliation exception placed into HITL quarantine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedItem {
    pub item_id: String,
    pub source_tx_id: String,
    pub amount_vnd: u64,
    pub risk_tier: QuarantineRiskTier,
    pub reason: String,
    pub has_aml_alert: bool,
    pub maker_id: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub status: QuarantineStatus,
    pub current_token: Option<String>,
    pub proposal_id: String,
}
