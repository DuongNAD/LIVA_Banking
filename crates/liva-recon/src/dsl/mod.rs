//! Declarative Reconciliation DSL Module
//!
//! Exposes schema, engine, integer string metrics, and subset-sum solver.

pub mod engine;
pub mod jaro_winkler;
pub mod schema;
pub mod subset_sum;

pub use engine::{
    strip_ref_punctuation, DeclarativeReconEngine, ReconBatchResult, ReconDslError,
    SemanticReconContext,
};
pub use jaro_winkler::{
    compute_semantic_similarity_bps, extract_initials, integer_jaro, integer_jaro_winkler,
    integer_sqrt_u128, integer_sqrt_u64, integer_token_cosine_similarity, is_acronym_pattern,
};
pub use schema::{
    FeeScheduleConfig, HitlQuarantineRuleConfig, QuarantineRiskRule, QuarantineRiskTier,
    ReconRuleConfig, ReconRuleMetadata, ReferenceMatchingConfig, Tier1RuleConfig,
    Tier2ConfidenceConfig, Tier2RuleConfig, Tier3RuleConfig,
};
pub use subset_sum::{solve_subset_sum, SubsetCandidate, SubsetSolution, SubsetSolverConfig};
