//! Declarative AML Surveillance Policy Schema.
//!
//! Enforces Law No. 14/2022/QH15, Decision 11/2023/QĐ-TTg, and Circular 09/2023/TT-NHNN.
//! All arithmetic is integer-only minor units (VND) and basis points (bps).

use liva_money::Money;
use serde::{Deserialize, Serialize};

/// Master declarative AML surveillance policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlPolicyConfig {
    pub metadata: AmlPolicyMetadata,
    pub high_value: HighValuePolicyConfig,
    pub structuring: StructuringPolicyConfig,
    pub transit_mule: TransitMulePolicyConfig,
    pub night_velocity: NightVelocityPolicyConfig,
    pub watchlist: WatchlistPolicyConfig,
    pub reporting: StrReportingConfig,
    #[serde(default)]
    pub anomaly_detection: AnomalyDetectionConfig,
}

impl Default for AmlPolicyConfig {
    fn default() -> Self {
        let yaml = include_str!("../../config/aml_policy_default.yaml");
        serde_yaml::from_str(yaml).expect("Default AML policy YAML must be valid")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlPolicyMetadata {
    pub name: String,
    pub version: String,
    pub jurisdiction: String,
    pub statutory_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighValuePolicyConfig {
    pub enabled: bool,
    pub statutory_threshold_vnd: u64, // Default 400,000,000 VND
    pub critical_threshold_vnd: u64,  // Default 1,000,000,000 VND
    pub statutory_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuringPolicyConfig {
    pub enabled: bool,
    pub window_seconds: i64,           // Default 86,400s (24h)
    pub min_tx_count: usize,           // Default >= 3
    pub min_candidate_amount_vnd: u64, // Default >= 50,000,000 VND
    pub max_candidate_amount_vnd: u64, // Default < 400,000,000 VND
    pub aggregate_threshold_vnd: u64,  // Default >= 400,000,000 VND
    #[serde(default)]
    pub group_by: Vec<String>,
    pub statutory_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitMulePolicyConfig {
    pub enabled: bool,
    pub min_inflow_vnd: u64,             // Default >= 100,000,000 VND
    pub max_transit_window_seconds: i64, // Default <= 900s (15 min)
    pub min_outflow_ratio_bps: u32,      // Default >= 9,500 bps (95.00%)
    pub statutory_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightVelocityPolicyConfig {
    pub enabled: bool,
    pub night_start_hour: u32,      // Default 23 (23:00)
    pub night_end_hour: u32,        // Default 5 (05:00)
    pub timezone_offset_hours: i32, // Default +7 for Vietnam (GMT+7)
    pub min_amount_vnd: u64,        // Default >= 50,000,000 VND
    pub count_threshold: usize,     // Default >= 3 transactions in night window
    pub aggregate_threshold_vnd: u64, // Default >= 200,000,000 VND
    pub statutory_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistPolicyConfig {
    pub enabled: bool,
    pub narration_keywords: Vec<String>,
    pub blacklisted_accounts: Vec<String>,
    pub sanctioned_names: Vec<String>,
    pub name_similarity_threshold_bps: u32, // Default 9,000 bps (90.00%)
    pub statutory_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrReportingConfig {
    pub reporting_entity_name: String,
    pub risk_score_threshold_file_sbv: u32,
    pub risk_score_threshold_compliance_hold: u32,
    pub critical_alert_weight: u32,
    pub high_alert_weight: u32,
    pub medium_alert_weight: u32,
}

/// Statutory AML Alert Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmlAlertCode {
    /// Decision 11/2023/QĐ-TTg: Transaction amount >= 400,000,000 VND
    HighValueThreshold,
    /// Structuring / Smurfing: Multiple transactions near statutory threshold within rolling window
    StructuringSmurfing,
    /// Velocity Surge: Turnover spikes over historical baseline
    VelocitySurge,
    /// Pass-Through / Transit Mule Account: Rapid cash in and immediate cash out (>= 95% in <= 900s)
    PassThroughTransit,
    /// Night Velocity Anomaly: Transactions during 23:00 - 05:00 GMT+7
    NightAnomaly,
    /// Watchlist / Sanctions / Blacklist match
    WatchlistSanction,
    /// Near-Threshold Smurfing: Transactions in danger band [380M, 399.999M VND]
    NearThresholdSmurfing,
    /// Amount Distribution Outlier: Robust Modified Z-score with MAD > 3.5
    AmountDistributionOutlier,
    /// Semantic Narration Anomaly: Narration matching suspicious centroids
    SemanticNarrationAnomaly,
    /// Micro-Burst Velocity: Rapid successive transactions (< 60s) on same account
    MicroBurstVelocity,
}

/// Severity classification of an AML alert
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AmlSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Statutory Alert Output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlAlert {
    pub alert_id: String,
    pub rule_code: AmlAlertCode,
    pub severity: AmlSeverity,
    pub tx_id: String,
    pub amount: Money,
    pub description: String,
    pub statutory_ref: String,
    pub detected_at: String,
}

/// Transaction input item for AML screening
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenableTransaction {
    pub tx_id: String,
    pub account_number: String,
    pub counterparty_name: String,
    pub counterparty_account: String,
    pub amount: Money,
    pub is_credit: bool,
    pub timestamp_seconds: i64,
    pub narration: String,
}

/// Form STR (Suspicious Transaction Report) according to Appendix II Circular 09/2023/TT-NHNN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousTransactionReport {
    pub str_id: String,
    pub report_date: String,
    pub reporting_entity: String,
    pub suspect_name: String,
    pub suspect_account: String,
    pub alert_type: String,
    pub severity: String,
    pub total_amount: Money,
    pub transaction_count: usize,
    pub statutory_rule_ref: String,
    pub narrative_summary: String,
    pub compliance_officer_notes: String,
    pub merkle_proof_hash: String,
    pub risk_score: u32,
    pub recommended_action: String,
}

/// Enhanced AML Alert with Multi-Dimensional Anomaly details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAmlAlert {
    pub alert_id: String,
    pub rule_code: AmlAlertCode,
    pub severity: AmlSeverity,
    pub tx_id: String,
    pub amount: Money,
    pub anomaly_score_bps: u32,              // 0..10,000 bps (0.00..1.00)
    pub anomaly_reasons: Vec<String>,
    pub risk_profile: String,
    pub vietnamese_explanation: Option<String>,
    pub statutory_ref: String,
    pub detected_at: String,
}

impl EnhancedAmlAlert {
    pub fn to_standard_alert(&self) -> AmlAlert {
        AmlAlert {
            alert_id: self.alert_id.clone(),
            rule_code: self.rule_code,
            severity: self.severity,
            tx_id: self.tx_id.clone(),
            amount: self.amount,
            description: if self.anomaly_reasons.is_empty() {
                self.risk_profile.clone()
            } else {
                self.anomaly_reasons.join("; ")
            },
            statutory_ref: self.statutory_ref.clone(),
            detected_at: self.detected_at.clone(),
        }
    }
}

/// Multi-Dimensional Anomaly Detection Triggers Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub temporal_velocity: TemporalVelocityConfig,
    #[serde(default)]
    pub amount_distribution: AmountDistributionConfig,
    #[serde(default)]
    pub semantic_narration: SemanticNarrationConfig,
    #[serde(default)]
    pub slm_explanation: SlmExplanationConfig,
}

fn default_true() -> bool {
    true
}

impl Default for AnomalyDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            temporal_velocity: TemporalVelocityConfig::default(),
            amount_distribution: AmountDistributionConfig::default(),
            semantic_narration: SemanticNarrationConfig::default(),
            slm_explanation: SlmExplanationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalVelocityConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_window_seconds")]
    pub window_seconds: i64,                 // e.g. 3600 (1 hour)
    #[serde(default = "default_max_tx_count")]
    pub max_tx_count_per_window: usize,      // e.g. 5
    #[serde(default = "default_velocity_surge_bps")]
    pub velocity_surge_threshold_bps: u32,   // e.g. 30,000 bps (300.00% of baseline)
    #[serde(default = "default_off_hours_volume")]
    pub off_hours_volume_threshold_vnd: u64, // e.g. 200,000,000 VND
    #[serde(default = "default_burst_interval")]
    pub burst_interval_min_seconds: i64,     // e.g. 60 seconds
}

fn default_window_seconds() -> i64 {
    3600
}
fn default_max_tx_count() -> usize {
    5
}
fn default_velocity_surge_bps() -> u32 {
    30_000
}
fn default_off_hours_volume() -> u64 {
    200_000_000
}
fn default_burst_interval() -> i64 {
    60
}

impl Default for TemporalVelocityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_seconds: 3600,
            max_tx_count_per_window: 5,
            velocity_surge_threshold_bps: 30_000,
            off_hours_volume_threshold_vnd: 200_000_000,
            burst_interval_min_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountDistributionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_smurfing_lower")]
    pub smurfing_band_lower_vnd: u64,        // e.g. 380,000,000 VND (near 400M threshold)
    #[serde(default = "default_smurfing_upper")]
    pub smurfing_band_upper_vnd: u64,        // e.g. 399,999,999 VND
    #[serde(default = "default_zscore_thresh")]
    pub modified_zscore_threshold_bps: u32,  // e.g. 35,000 bps (3.5x MAD)
    #[serde(default = "default_min_sample")]
    pub min_sample_size: usize,              // e.g. 5 transactions for baseline
}

fn default_smurfing_lower() -> u64 {
    380_000_000
}
fn default_smurfing_upper() -> u64 {
    399_999_999
}
fn default_zscore_thresh() -> u32 {
    35_000
}
fn default_min_sample() -> usize {
    5
}

impl Default for AmountDistributionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            smurfing_band_lower_vnd: 380_000_000,
            smurfing_band_upper_vnd: 399_999_999,
            modified_zscore_threshold_bps: 35_000,
            min_sample_size: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticNarrationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_sim_thresh")]
    pub similarity_threshold_bps: u32,       // e.g. 8,200 bps (0.82)
    #[serde(default = "default_clusters")]
    pub suspicious_clusters: Vec<SuspiciousCluster>,
}

fn default_sim_thresh() -> u32 {
    8_200
}
fn default_cluster_weight() -> u32 {
    2500
}

fn default_clusters() -> Vec<SuspiciousCluster> {
    vec![
        SuspiciousCluster {
            category: "crypto_otc".to_string(),
            keywords: vec![
                "USDT".to_string(),
                "BINANCE".to_string(),
                "P2P".to_string(),
                "CRYPTO".to_string(),
                "TIEN AO".to_string(),
                "COIN".to_string(),
                "OKX".to_string(),
                "BYBIT".to_string(),
                "MUA USDT".to_string(),
                "BAN USDT".to_string(),
            ],
            weight_bps: 2500,
        },
        SuspiciousCluster {
            category: "underground_remittance".to_string(),
            keywords: vec![
                "CHUYEN HO".to_string(),
                "DOI NGOAI TE".to_string(),
                "DOI NGOAI TE CHUI".to_string(),
                "DAY TIEN HO".to_string(),
                "CHUYEN TIEN TAY BA".to_string(),
                "TIEN CAMPUCHIA".to_string(),
                "TIEN TRUNG QUOC".to_string(),
                "RUT TIEN HO".to_string(),
            ],
            weight_bps: 2500,
        },
        SuspiciousCluster {
            category: "kickbacks".to_string(),
            keywords: vec![
                "HOA HONG DU AN".to_string(),
                "HOA HONG KIN".to_string(),
                "TIEN CAM ON".to_string(),
                "CAM ON SEP".to_string(),
                "CHI NGOAI SO".to_string(),
                "LOT TAY".to_string(),
                "BOI TRON".to_string(),
                "PHAN TRAM DU AN".to_string(),
            ],
            weight_bps: 2000,
        },
        SuspiciousCluster {
            category: "money_mule".to_string(),
            keywords: vec![
                "DON DEP TAI KHOAN".to_string(),
                "CHUYEN GAP".to_string(),
                "KHONG GHI NOI DUNG".to_string(),
                "TAI KHOAN MUON".to_string(),
                "KEO TIEN VE".to_string(),
                "CHUYEN TIEP NGAY".to_string(),
                "RUT NGAY".to_string(),
                "BAN NGAY".to_string(),
            ],
            weight_bps: 2500,
        },
    ]
}

impl Default for SemanticNarrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            similarity_threshold_bps: 8_200,
            suspicious_clusters: default_clusters(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousCluster {
    pub category: String,
    pub keywords: Vec<String>,
    #[serde(default = "default_cluster_weight")]
    pub weight_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlmExplanationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_model_id")]
    pub model_id: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    #[serde(default = "default_score_thresh")]
    pub score_threshold_bps: u32,            // 4,000 bps (0.40) to trigger explanation
}

fn default_model_id() -> String {
    "Qwen_Qwen3-14B-Q5_K_M".to_string()
}
fn default_max_tokens() -> usize {
    128
}
fn default_score_thresh() -> u32 {
    4_000
}

impl Default for SlmExplanationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model_id: "Qwen_Qwen3-14B-Q5_K_M".to_string(),
            max_tokens: 128,
            score_threshold_bps: 4_000,
        }
    }
}

