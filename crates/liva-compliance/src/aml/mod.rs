//! AML Surveillance module.

pub mod engine;
pub mod schema;

pub use engine::{
    compute_amount_mad, compute_modified_zscore_bps, integer_jaro, integer_jaro_winkler,
    normalize_ascii, rule_code_prefix, AmlScreeningEngine,
};
pub use schema::{
    AmlAlert, AmlAlertCode, AmlPolicyConfig, AmlPolicyMetadata, AmlSeverity,
    AmountDistributionConfig, AnomalyDetectionConfig, EnhancedAmlAlert, HighValuePolicyConfig,
    NightVelocityPolicyConfig, ScreenableTransaction, SemanticNarrationConfig,
    SlmExplanationConfig, StrReportingConfig, StructuringPolicyConfig, SuspiciousCluster,
    SuspiciousTransactionReport, TemporalVelocityConfig, TransitMulePolicyConfig,
    WatchlistPolicyConfig,
};

