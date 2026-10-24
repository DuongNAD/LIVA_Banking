//! LIVA Banking — DSL Subsystem
//!
//! Declarative, data-driven domain-specific language engine for Chart of Accounts (COA),
//! journal posting rules, and pure-integer bank fee unbundling.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

pub mod coa_schema;
pub mod engine;
pub mod fee_unbundling;
pub mod posting_rules;

pub use coa_schema::{
    AccountCategory, AccountDefinition, AccountNature, AccountStandard, ChartOfAccounts, CoaError,
    CoaMetadata, CompiledCoaTree, VasStandard,
};
pub use engine::{DeclarativePostingEngine, PostingContext, RuleEngineError, voucher_to_journal_entry};
pub use fee_unbundling::{
    route_and_unbundle_fee, unbundle_gross_fee, UnbundledFeeResult, BPS_DIVISOR,
    VAT_EXEMPT_RATE_BPS, VAT_REDUCED_RATE_BPS, VAT_STANDARD_RATE_BPS,
};
pub use posting_rules::{
    AmountFormula, PostingAction, PostingCondition, PostingEventType, PostingGlobalDefaults,
    PostingLineTemplate, PostingRule, PostingRuleConfig, PostingRuleMetadata,
};

/// Embedded statutory Chart of Accounts configuration per Circular 200/2014/TT-BTC.
pub const DEFAULT_COA_CIRCULAR200_YAML: &str = include_str!("../../config/coa_circular200.yaml");

/// Embedded statutory Chart of Accounts configuration per Circular 133/2016/TT-BTC.
pub const DEFAULT_COA_CIRCULAR133_YAML: &str = include_str!("../../config/coa_circular133.yaml");

/// Embedded default Vietnamese banking journal posting rules configuration.
pub const DEFAULT_POSTING_RULES_YAML: &str = include_str!("../../config/posting_rules_default.yaml");

/// Compiles and returns the standard Circular 200/2014 Chart of Accounts tree.
pub fn load_default_coa_circular200() -> Result<CompiledCoaTree, CoaError> {
    let coa = ChartOfAccounts::from_yaml(DEFAULT_COA_CIRCULAR200_YAML)?;
    coa.compile()
}

/// Compiles and returns the standard Circular 133/2016 Chart of Accounts tree.
pub fn load_default_coa_circular133() -> Result<CompiledCoaTree, CoaError> {
    let coa = ChartOfAccounts::from_yaml(DEFAULT_COA_CIRCULAR133_YAML)?;
    coa.compile()
}

/// Parses and returns the default posting rules configuration.
pub fn load_default_posting_rules() -> Result<PostingRuleConfig, serde_yaml::Error> {
    PostingRuleConfig::from_yaml(DEFAULT_POSTING_RULES_YAML)
}

/// Compiles and returns the precompiled DeclarativePostingEngine with default Vietnamese banking rules.
pub fn load_default_posting_engine() -> Result<DeclarativePostingEngine, RuleEngineError> {
    DeclarativePostingEngine::from_yaml(DEFAULT_POSTING_RULES_YAML)
}
