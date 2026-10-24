//! Declarative Reconciliation Rule DSL Schema
//!
//! Strongly-typed configuration schema for 3-tier reconciliation and HITL quarantine.
//! Can be deserialized from YAML or JSON.
//!
//! Invariants:
//! - Strictly `#![deny(clippy::float_arithmetic)]`: all thresholds and scores are integers in basis points ($0 \dots 10,000$ bps).
//! - All monetary amounts are integers in atomic VND (`i64`).

use serde::{Deserialize, Serialize};

/// Top-level reconciliation rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconRuleConfig {
    pub metadata: ReconRuleMetadata,
    pub tier1: Tier1RuleConfig,
    pub tier2: Tier2RuleConfig,
    pub tier3: Tier3RuleConfig,
    pub hitl_quarantine: HitlQuarantineRuleConfig,
}

/// Metadata identifying the rule set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconRuleMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Tier 1: Exact 1-to-1 matching configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier1RuleConfig {
    pub enabled: bool,
    /// Maximum allowed time drift between statement transaction and ledger invoice in seconds.
    /// Default: 86,400s (±24 hours).
    pub timestamp_tolerance_seconds: i64,
    /// Confidence score in basis points (default 10,000 = 100.00%).
    pub confidence_score_bps: u32,
    pub reference_matching: ReferenceMatchingConfig,
}

/// Reference normalization and regex extraction settings for Tier 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceMatchingConfig {
    pub extract_from_narration: bool,
    /// Regex patterns used to extract invoice/document references from transaction narrations.
    pub extraction_patterns: Vec<String>,
    pub strip_punctuation: bool,
    pub case_sensitive: bool,
}

/// Tier 2: Fuzzy heuristic matching configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier2RuleConfig {
    pub enabled: bool,
    /// Maximum allowed time drift in seconds (default: 259,200s = ±72 hours).
    pub timestamp_tolerance_seconds: i64,
    /// Minimum Integer Jaro-Winkler similarity threshold in basis points (default 8,500 = 85.00%).
    pub similarity_threshold_bps: u32,
    /// Minimum token containment percentage in basis points (default 8,000 = 80.00%).
    pub token_containment_threshold_bps: u32,
    /// Flag to enable hybrid semantic reconciliation in Tier 2 (default: true).
    #[serde(default = "default_true")]
    pub enable_semantic_recon: bool,
    /// Lower bound of the Jaro-Winkler uncertainty band in basis points (default: 5,000 = 50.00%).
    #[serde(default = "default_uncertainty_lower_bound_bps")]
    pub uncertainty_lower_bound_bps: u32,
    /// Minimum semantic similarity threshold required to elevate a candidate in Gate 2 (default: 8,200 = 82.00%).
    #[serde(default = "default_semantic_elevation_threshold_bps")]
    pub semantic_elevation_threshold_bps: u32,
    /// Weight of Jaro-Winkler in the combined score calculation (default: 40%).
    #[serde(default = "default_jw_weight")]
    pub jw_weight: u32,
    /// Weight of semantic similarity in the combined score calculation (default: 60%).
    #[serde(default = "default_semantic_weight")]
    pub semantic_weight: u32,
    /// Dynamic fee deduction schedules.
    pub fee_schedules: Vec<FeeScheduleConfig>,
    /// Multi-word legal forms to strip first (sorted descending by length).
    pub legal_phrases: Vec<String>,
    /// Single legal form tokens to filter out.
    pub legal_tokens: Vec<String>,
    /// Common banking boilerplate phrases to remove from narration.
    pub bank_narration_phrases: Vec<String>,
    pub confidence: Tier2ConfidenceConfig,
}

fn default_true() -> bool {
    true
}

fn default_uncertainty_lower_bound_bps() -> u32 {
    5_000
}

fn default_semantic_elevation_threshold_bps() -> u32 {
    8_200
}

fn default_jw_weight() -> u32 {
    40
}

fn default_semantic_weight() -> u32 {
    60
}

/// Fee schedule definition: discrete fee list or continuous range band.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FeeScheduleConfig {
    Discrete {
        name: String,
        fees_vnd: Vec<i64>,
    },
    Range {
        name: String,
        min_vnd: i64,
        max_vnd: i64,
    },
}

impl FeeScheduleConfig {
    /// Returns true if the fee delta matches this schedule.
    pub fn matches_fee(&self, delta: i64) -> bool {
        match self {
            Self::Discrete { fees_vnd, .. } => fees_vnd.contains(&delta),
            Self::Range { min_vnd, max_vnd, .. } => delta >= *min_vnd && delta <= *max_vnd,
        }
    }
}

/// Confidence scores for Tier 2 matches in basis points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier2ConfidenceConfig {
    pub exact_match_bps: u32,  // Default: 9200 (92.00%)
    pub fee_deducted_bps: u32, // Default: 8800 (88.00%)
}

/// Tier 3: Subset-Sum composite split matching configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier3RuleConfig {
    pub enabled: bool,
    /// Maximum number of items in a subset ($2 \le k \le 8$). Default is 8.
    pub max_depth: usize,
    /// Maximum search iterations (default 100,000).
    pub max_iterations: usize,
    /// Timeout budget in milliseconds (default 100ms).
    pub timeout_ms: u64,
    /// Requires exact zero arithmetic difference ($\Delta = 0$).
    pub require_zero_delta: bool,
    pub base_confidence_bps: u32, // Default: 9500 (95.00%)
    pub item_penalty_bps: u32,    // Default: 200 (2.00%)
}

/// HITL Quarantine and Maker-Checker governance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlQuarantineRuleConfig {
    /// Expiration lifetime of single-use review tokens in seconds (default 900s = 15 minutes).
    pub token_ttl_seconds: i64,
    /// Enforces segregation of duties: maker cannot equal checker.
    pub enforce_maker_checker_sod: bool,
    /// Automated risk classification rules.
    pub risk_rules: Vec<QuarantineRiskRule>,
}

/// Risk level classification for quarantined items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuarantineRiskTier {
    Low,
    Medium,
    High,
    Critical,
}

/// Rule mapping transaction properties to risk tiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineRiskRule {
    pub risk_tier: QuarantineRiskTier,
    #[serde(default)]
    pub min_amount_vnd: Option<i64>,
    #[serde(default)]
    pub fee_exceeded: bool,
    pub statutory_reference: String,
}

impl Default for ReconRuleConfig {
    fn default() -> Self {
        Self {
            metadata: ReconRuleMetadata {
                name: "Standard Vietnamese Banking Reconciliation".to_string(),
                version: "2.0.0".to_string(),
                description: "Standard declarative reconciliation rules for VCB, TCB, BIDV, CTG under Decree 13 and Circular 09".to_string(),
            },
            tier1: Tier1RuleConfig {
                enabled: true,
                timestamp_tolerance_seconds: 86_400, // ±24 hours
                confidence_score_bps: 10_000,        // 100.00%
                reference_matching: ReferenceMatchingConfig {
                    extract_from_narration: true,
                    extraction_patterns: vec![
                        r"(?i)(?:INV|HD|REF|DH|BILL)[-_]?[0-9A-Z]+(?:[-_][0-9A-Z]+)*".to_string(),
                        r"(?i)REF2026[0-9]{8}".to_string(),
                        r"(?i)FT[0-9A-Z]{8,}".to_string(),
                    ],
                    strip_punctuation: true,
                    case_sensitive: false,
                },
            },
            tier2: Tier2RuleConfig {
                enabled: true,
                timestamp_tolerance_seconds: 259_200, // ±72 hours
                similarity_threshold_bps: 8_500,      // 85.00%
                token_containment_threshold_bps: 8_000, // 80.00%
                enable_semantic_recon: true,
                uncertainty_lower_bound_bps: 5_000,   // 50.00%
                semantic_elevation_threshold_bps: 8_200, // 82.00%
                jw_weight: 40,
                semantic_weight: 60,
                fee_schedules: vec![
                    FeeScheduleConfig::Discrete {
                        name: "Standard Interbank Wire Fees".to_string(),
                        fees_vnd: vec![1_100, 1_650, 2_200, 3_300, 5_500, 7_700, 8_800, 11_000, 22_000],
                    },
                    FeeScheduleConfig::Range {
                        name: "Napas / Fast Transfer Fee Tolerance Band".to_string(),
                        min_vnd: 1_000,
                        max_vnd: 25_000,
                    },
                ],
                legal_phrases: vec![
                    "TRACH NHIEM HUU HAN".to_string(),
                    "THUONG MAI DICH VU".to_string(),
                    "MOT THANH VIEN".to_string(),
                    "BAT DONG SAN".to_string(),
                    "DOANH NGHIEP TU NHAN".to_string(),
                    "XUAT NHAP KHAU".to_string(),
                ],
                legal_tokens: vec![
                    "CONG".to_string(),
                    "TY".to_string(),
                    "CP".to_string(),
                    "CO".to_string(),
                    "PHAN".to_string(),
                    "TNHH".to_string(),
                    "MTV".to_string(),
                    "TAP".to_string(),
                    "DOAN".to_string(),
                    "TONG".to_string(),
                    "CHI".to_string(),
                    "NHANH".to_string(),
                    "CTY".to_string(),
                ],
                bank_narration_phrases: vec![
                    "NAPAS VIETQR TT".to_string(),
                    "NAPAS247".to_string(),
                    "VIETQR".to_string(),
                    "CHUYEN KHOAN".to_string(),
                    "THANH TOAN".to_string(),
                    "IBFT".to_string(),
                    "CK".to_string(),
                    "TT".to_string(),
                ],
                confidence: Tier2ConfidenceConfig {
                    exact_match_bps: 9_200,  // 92.00%
                    fee_deducted_bps: 8_800, // 88.00%
                },
            },
            tier3: Tier3RuleConfig {
                enabled: true,
                max_depth: 8,
                max_iterations: 100_000,
                timeout_ms: 100,
                require_zero_delta: true,
                base_confidence_bps: 9_500,
                item_penalty_bps: 200,
            },
            hitl_quarantine: HitlQuarantineRuleConfig {
                token_ttl_seconds: 900, // 15 minutes
                enforce_maker_checker_sod: true,
                risk_rules: vec![
                    QuarantineRiskRule {
                        risk_tier: QuarantineRiskTier::Critical,
                        min_amount_vnd: Some(400_000_000), // >= 400M VND Decision 11/2023/QĐ-TTg
                        fee_exceeded: false,
                        statutory_reference: "Quyết định 11/2023/QĐ-TTg & Thông tư 09/2023/TT-NHNN".to_string(),
                    },
                    QuarantineRiskRule {
                        risk_tier: QuarantineRiskTier::Medium,
                        min_amount_vnd: None,
                        fee_exceeded: true,
                        statutory_reference: "Quy trình kiểm soát chênh lệch phí ngân hàng".to_string(),
                    },
                ],
            },
        }
    }
}
