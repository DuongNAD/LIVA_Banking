//! 3-Tuple RBAC Authorization Gate conforming to Enterprise Multi-Entity Banking.
//!
//! Enforces authorization across: (Action, Legal Entity, Bank Account).
//! Prevents privilege escalation and cross-entity data contamination.

use crate::compliance::UserRole;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BankingAction {
    ImportStatement,
    ViewTransactions,
    CreateProposal,
    ApproveProposal,
    RejectProposal,
    ClosePeriod,
    ReopenPeriod,
    ExportAuditReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionGrant {
    pub user_id: String,
    pub role: UserRole,
    pub allowed_entities: HashSet<String>,
    pub allowed_accounts: HashSet<String>,
    pub allowed_actions: HashSet<BankingAction>,
}

pub struct RbacGate;

impl RbacGate {
    /// Evaluates if a user has permission to perform an action on a specific legal entity and account.
    pub fn authorize(
        grant: &UserPermissionGrant,
        action: &BankingAction,
        legal_entity: &str,
        account_number: &str,
    ) -> Result<(), &'static str> {
        // 1. Action permission check
        if !grant.allowed_actions.contains(action) {
            return Err("AccessDenied: User does not have permission to perform this action");
        }

        // 2. Legal Entity scope check
        if !grant.allowed_entities.contains("*") && !grant.allowed_entities.contains(legal_entity) {
            return Err("AccessDenied: User is not authorized for this legal entity");
        }

        // 3. Bank Account scope check
        if !grant.allowed_accounts.contains("*") && !grant.allowed_accounts.contains(account_number) {
            return Err("AccessDenied: User is not authorized for this bank account");
        }

        Ok(())
    }
}
