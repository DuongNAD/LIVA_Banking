//! LIVA Banking — Dynamic Journal Posting Rules Schema
//!
//! Declarative schema for matching statement events and synthesizing double-entry vouchers.
//! Evaluates predicates: bank code, transaction direction, narration regex, counterparty regex,
//! and amount thresholds.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

use crate::PostingType;
use serde::{Deserialize, Serialize};

/// Top-level configuration container for dynamic journal posting rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostingRuleConfig {
    pub metadata: PostingRuleMetadata,
    pub defaults: PostingGlobalDefaults,
    pub rules: Vec<PostingRule>,
}

impl PostingRuleConfig {
    /// Deserializes posting rules from a YAML string.
    pub fn from_yaml(yaml_str: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml_str)
    }

    /// Deserializes posting rules from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Serializes posting rules to a YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Serializes posting rules to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Metadata header for a posting ruleset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostingRuleMetadata {
    pub name: String,
    pub version: String,
    pub standard: String,
    pub description: String,
}

/// Global fallback accounts and standard parameters for rule execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostingGlobalDefaults {
    /// Default bank GL account (e.g. "1121")
    pub bank_default_account: String,
    /// Default fee expense GL account (e.g. "6427" or "6422")
    pub fee_expense_account: String,
    /// Default fee tax GL account (e.g. "6425")
    pub fee_tax_account: String,
    /// Default input VAT GL account (e.g. "1331")
    pub input_vat_account: String,
    /// Default customer receivable GL account (e.g. "131")
    pub customer_default_account: String,
    /// Default vendor payable GL account (e.g. "331")
    pub vendor_default_account: String,
    /// Standard VAT rate in basis points (e.g. 1000 for 10%)
    #[serde(default = "default_vat_rate_bps")]
    pub standard_vat_rate_bps: i64,
}

const fn default_vat_rate_bps() -> i64 {
    1_000
}

/// Individual dynamic posting rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostingRule {
    /// Unique identifier for this rule (e.g. "RULE_BANK_FEE_SERVICE_UNBUNDLE")
    pub id: String,
    /// Human-readable rule title
    pub name: String,
    /// Evaluation priority (higher priority evaluated first)
    #[serde(default)]
    pub priority: i32,
    /// Whether this rule is currently enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Matching conditions (predicates)
    pub when: PostingCondition,
    /// Resulting journal line actions
    pub then: PostingAction,
}

const fn default_true() -> bool {
    true
}

/// Condition predicates for matching statement transactions.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PostingCondition {
    /// Transaction posting direction (Debit = money out, Credit = money in)
    #[serde(default)]
    pub direction: Option<PostingType>,
    /// Target bank codes (e.g. ["VCB", "TCB", "BIDV"]). Empty means any bank.
    #[serde(default)]
    pub bank_codes: Vec<String>,
    /// Regular expressions matched against transaction narration. Any match triggers.
    #[serde(default)]
    pub narration_patterns: Vec<String>,
    /// Regular expressions matched against counterparty name or tax ID.
    #[serde(default)]
    pub counterparty_patterns: Vec<String>,
    /// Specific high-level event type if pre-classified
    #[serde(default)]
    pub event_type: Option<PostingEventType>,
    /// Minimum gross amount in minor units (VND)
    #[serde(default)]
    pub min_amount: Option<i64>,
    /// Maximum gross amount in minor units (VND)
    #[serde(default)]
    pub max_amount: Option<i64>,
}

/// Recognized high-level event types for explicit condition tagging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostingEventType {
    CustomerReceipt,
    CustomerReceiptWithFee,
    VendorPayment,
    BankFeeStandalone,
    BankFeeUnbundled,
    InterestEarned,
    LoanDisbursement,
    LoanRepayment,
    LoanInterest,
    TaxPayment,
    SalaryDisbursement,
    InternalTransfer,
}

/// Journal voucher generation action template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostingAction {
    /// Template for voucher description (supports `{{bank}}`, `{{ref}}`, `{{narration}}`, `{{partner}}`)
    pub description_template: String,
    /// Template specifications for each balanced journal line
    pub lines: Vec<PostingLineTemplate>,
}

/// Template specification for an individual journal line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostingLineTemplate {
    /// Target account expression (literal e.g. "6427", or template e.g. "{{bank_account}}", "{{partner_account}}")
    pub account_expr: String,
    /// Debit or Credit line
    pub posting_type: PostingType,
    /// Formula for computing line amount
    pub amount_expr: AmountFormula,
    /// Optional specific line description
    #[serde(default)]
    pub line_description: Option<String>,
}

/// Dynamic amount calculation formulas for journal lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AmountFormula {
    /// The full gross transaction amount
    FullAmount,
    /// Net exclusive fee amount calculated via integer unbundling
    NetExclusive { vat_rate_bps: i64 },
    /// Deductible VAT portion calculated via integer unbundling
    VatPortion { vat_rate_bps: i64 },
    /// Deducted fee amount (from payment gateway or intermediary)
    DeductedFee,
    /// Net amount received (Gross minus Deducted Fee)
    NetReceived,
    /// Invoice amount being settled
    InvoiceAmount,
    /// Fixed explicit amount in minor currency units
    Fixed { amount_minor: i64 },
}
