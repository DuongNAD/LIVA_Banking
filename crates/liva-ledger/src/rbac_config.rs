//! RBAC, System Configuration & Bank/ERP Integration Engine
//! =============================================================
//! Enforces Circular 09/2020/TT-NHNN information security & Segregation of Duties (SoD).
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

use crate::LedgerError;
use liva_money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Maker,
    Checker,
    ComplianceOfficer,
    Auditor,
}

impl UserRole {
    pub const fn code(&self) -> &'static str {
        match self {
            UserRole::Admin => "ADMIN",
            UserRole::Maker => "MAKER",
            UserRole::Checker => "CHECKER",
            UserRole::ComplianceOfficer => "COMPLIANCE_OFFICER",
            UserRole::Auditor => "AUDITOR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub cert_serial: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BankApiConfig {
    pub bank_code: String,
    pub api_endpoint: String,
    pub is_connected: bool,
    pub latency_ms: u32,
    pub is_sandbox: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErpBridgeConfig {
    pub erp_name: String,
    pub endpoint: String,
    pub is_active: bool,
    pub sync_interval_mins: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationThresholdConfig {
    pub exact_tolerance_minor: i64,
    pub max_fee_variance_minor: i64,
    pub split_solver_max_k: usize,
    pub auto_quarantine_ttl_secs: u64,
    pub aml_high_value_threshold_minor: i64,
}

impl Default for ReconciliationThresholdConfig {
    fn default() -> Self {
        Self {
            exact_tolerance_minor: 0,
            max_fee_variance_minor: 50_000,
            split_solver_max_k: 8,
            auto_quarantine_ttl_secs: 900,
            aml_high_value_threshold_minor: 400_000_000,
        }
    }
}

/// Enforces Segregation of Duties (SoD) under Circular 09/2020/TT-NHNN:
/// Maker and Checker MUST NOT be the same person.
pub fn validate_sod_rule(maker_id: &str, checker_id: &str) -> Result<(), LedgerError> {
    if maker_id.trim().is_empty() || checker_id.trim().is_empty() {
        return Err(LedgerError::InvariantViolation {
            expected: "Valid non-empty maker and checker IDs".to_string(),
            calculated: "Empty maker or checker ID".to_string(),
        });
    }

    if maker_id.trim().eq_ignore_ascii_case(checker_id.trim()) {
        return Err(LedgerError::InvariantViolation {
            expected: "Segregation of Duties (maker_id != checker_id)".to_string(),
            calculated: format!("SoD violation: maker '{maker_id}' is identical to checker '{checker_id}'"),
        });
    }

    Ok(())
}

/// Deterministic RBAC permission lookup table.
pub fn can_perform_action(role: &UserRole, action: &str) -> bool {
    match action {
        "RECONCILE_EXECUTE" => matches!(role, UserRole::Maker),
        "HITL_PROPOSE" => matches!(role, UserRole::Maker),
        "HITL_APPROVE" => matches!(role, UserRole::Checker),
        "PAYMENT_DRAFT" => matches!(role, UserRole::Maker),
        "PAYMENT_AUTHORIZE" => matches!(role, UserRole::Checker),
        "AML_SIGN_STR" => matches!(role, UserRole::ComplianceOfficer),
        "VIEW_REPORTS" => matches!(
            role,
            UserRole::Admin
                | UserRole::Maker
                | UserRole::Checker
                | UserRole::ComplianceOfficer
                | UserRole::Auditor
        ),
        "SYSTEM_CONFIGURE" => matches!(role, UserRole::Admin),
        _ => false,
    }
}

/// Validates whether a variance falls within the configured fee tolerance limit.
pub fn is_fee_within_tolerance(variance: Money, config: &ReconciliationThresholdConfig) -> bool {
    let abs_variance = variance.amount().abs();
    abs_variance <= config.max_fee_variance_minor
}
