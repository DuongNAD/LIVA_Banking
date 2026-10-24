//! LIVA Banking — Declarative Chart of Accounts (COA) Schema & Hierarchy Tree
//!
//! Implements Vietnamese Accounting Standards (VAS) Chart of Accounts representation:
//! - Circular 200/2014/TT-BTC (Standard Enterprises)
//! - Circular 133/2016/TT-BTC (Small & Medium Enterprises - SMEs)
//! - Custom enterprise charts of accounts
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.
//! Provides acyclicity validation (DFS cycle detection), parent-child type consistency,
//! leaf account posting enforcement, and recursive roll-up balance computation.

use crate::{Account, AccountType, Ledger, LedgerError};
use liva_money::{Currency, Money};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Vietnamese Accounting Standard classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStandard {
    #[serde(alias = "circular_200_2014", alias = "Circular200_2014")]
    Circular200_2014,
    #[serde(alias = "circular_133_2016", alias = "Circular133_2016")]
    Circular133_2016,
    #[serde(alias = "custom", alias = "Custom")]
    Custom,
}

/// Alias for Vietnamese Accounting Standard classification.
pub type VasStandard = AccountStandard;

/// Normal balance nature of an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountNature {
    /// Debit balance is normal (Assets, Expenses)
    #[serde(alias = "debit_normal", alias = "DebitNormal")]
    DebitNormal,
    /// Credit balance is normal (Liabilities, Equity, Revenue)
    #[serde(alias = "credit_normal", alias = "CreditNormal")]
    CreditNormal,
    /// Dual nature / Lưỡng tính (e.g. 131, 331, 421)
    #[serde(alias = "dual_nature", alias = "DualNature")]
    DualNature,
}

/// Category classification for accounts (aliased to AccountType).
pub type AccountCategory = AccountType;

/// Metadata header for a Chart of Accounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoaMetadata {
    pub standard: AccountStandard,
    pub name: String,
    pub version: String,
    pub description: String,
    pub base_currency: Currency,
}

/// Individual declarative account specification in a COA schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountDefinition {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub english_name: Option<String>,
    pub account_type: AccountType,
    pub nature: AccountNature,
    #[serde(default)]
    pub parent_code: Option<String>,
    #[serde(default = "default_level")]
    pub level: u8,
    #[serde(default = "default_true")]
    pub is_leaf: bool,
    #[serde(default = "default_true")]
    pub is_system: bool,
    #[serde(default)]
    pub allowed_currencies: Vec<Currency>,
    #[serde(default)]
    pub description: Option<String>,
}

const fn default_level() -> u8 {
    1
}

const fn default_true() -> bool {
    true
}

/// Container for Chart of Accounts loaded from YAML/JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartOfAccounts {
    pub metadata: CoaMetadata,
    pub accounts: Vec<AccountDefinition>,
}

impl ChartOfAccounts {
    /// Parses a Chart of Accounts from a YAML string.
    pub fn from_yaml(yaml_str: &str) -> Result<Self, CoaError> {
        serde_yaml::from_str(yaml_str).map_err(|e| CoaError::YamlError(e.to_string()))
    }

    /// Parses a Chart of Accounts from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, CoaError> {
        serde_json::from_str(json_str).map_err(|e| CoaError::JsonError(e.to_string()))
    }

    /// Serializes this Chart of Accounts into a YAML string.
    pub fn to_yaml(&self) -> Result<String, CoaError> {
        serde_yaml::to_string(self).map_err(|e| CoaError::YamlError(e.to_string()))
    }

    /// Serializes this Chart of Accounts into a JSON string.
    pub fn to_json(&self) -> Result<String, CoaError> {
        serde_json::to_string_pretty(self).map_err(|e| CoaError::JsonError(e.to_string()))
    }

    /// Compiles and strictly validates this Chart of Accounts into a hierarchical tree.
    pub fn compile(self) -> Result<CompiledCoaTree, CoaError> {
        CompiledCoaTree::compile(self)
    }
}

/// Errors occurring during Chart of Accounts validation, compilation, or querying.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoaError {
    #[error("Duplicate account code '{0}' in Chart of Accounts")]
    DuplicateCode(String),

    #[error("Parent account '{parent_code}' not found for account '{account_code}'")]
    ParentNotFound {
        account_code: String,
        parent_code: String,
    },

    #[error("Cycle detected in account hierarchy involving code '{0}'")]
    CycleDetected(String),

    #[error("Direct posting not allowed to non-leaf parent account '{0}'")]
    NonLeafPostingDisallowed(String),

    #[error(
        "Account type mismatch: Child '{child_code}' ({child_type:?}) differs from parent '{parent_code}' ({parent_type:?})"
    )]
    TypeMismatch {
        child_code: String,
        child_type: AccountType,
        parent_code: String,
        parent_type: AccountType,
    },

    #[error("Account '{0}' not found in compiled Chart of Accounts tree")]
    NotFound(String),

    #[error("Failed to parse COA YAML: {0}")]
    YamlError(String),

    #[error("Failed to parse COA JSON: {0}")]
    JsonError(String),

    #[error("Ledger error: {0}")]
    Ledger(#[from] LedgerError),
}

/// In-memory indexed Chart of Accounts Tree with parent-child relationships and leaf validation.
#[derive(Debug, Clone)]
pub struct CompiledCoaTree {
    metadata: CoaMetadata,
    accounts: HashMap<String, AccountDefinition>,
    children: HashMap<String, Vec<String>>,
    roots: Vec<String>,
}

impl CompiledCoaTree {
    /// Compiles and strictly validates a declarative ChartOfAccounts.
    ///
    /// Validations:
    /// 1. Uniqueness of account codes.
    /// 2. Existence of declared parent accounts.
    /// 3. Matching account types between children and parents.
    /// 4. Strict acyclicity via DFS cycle detection on parent pointers.
    /// 5. Correct leaf account designation (accounts with children cannot be leaves).
    pub fn compile(coa: ChartOfAccounts) -> Result<Self, CoaError> {
        let count = coa.accounts.len();
        let mut accounts = HashMap::with_capacity(count);
        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        let mut roots = Vec::new();

        // 1. Uniqueness check & index construction
        for acc in coa.accounts {
            if accounts.contains_key(&acc.code) {
                return Err(CoaError::DuplicateCode(acc.code));
            }
            if let Some(ref parent) = acc.parent_code {
                if parent == &acc.code {
                    return Err(CoaError::CycleDetected(acc.code));
                }
                children.entry(parent.clone()).or_default().push(acc.code.clone());
            } else {
                roots.push(acc.code.clone());
            }
            accounts.insert(acc.code.clone(), acc);
        }

        // 2. Parent existence check & type consistency
        for (code, acc) in &accounts {
            if let Some(ref parent_code) = acc.parent_code {
                let parent = accounts.get(parent_code).ok_or_else(|| CoaError::ParentNotFound {
                    account_code: code.clone(),
                    parent_code: parent_code.clone(),
                })?;

                if parent.account_type != acc.account_type {
                    return Err(CoaError::TypeMismatch {
                        child_code: code.clone(),
                        child_type: acc.account_type,
                        parent_code: parent_code.clone(),
                        parent_type: parent.account_type,
                    });
                }
            }
        }

        // 3. Acyclicity verification via DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        for code in accounts.keys() {
            Self::check_cycles(code, &accounts, &mut visited, &mut rec_stack)?;
        }

        // 4. Ensure leaf account consistency
        // Any account that has registered children cannot be a leaf account
        for parent_code in children.keys() {
            if let Some(parent_acc) = accounts.get_mut(parent_code) {
                parent_acc.is_leaf = false;
            }
        }

        Ok(Self {
            metadata: coa.metadata,
            accounts,
            children,
            roots,
        })
    }

    fn check_cycles(
        current: &str,
        accounts: &HashMap<String, AccountDefinition>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<(), CoaError> {
        if rec_stack.contains(current) {
            return Err(CoaError::CycleDetected(current.to_string()));
        }
        if visited.contains(current) {
            return Ok(());
        }

        visited.insert(current.to_string());
        rec_stack.insert(current.to_string());

        if let Some(acc) = accounts.get(current) {
            if let Some(ref parent) = acc.parent_code {
                Self::check_cycles(parent, accounts, visited, rec_stack)?;
            }
        }

        rec_stack.remove(current);
        Ok(())
    }

    /// Returns the metadata header of the compiled COA tree.
    pub const fn metadata(&self) -> &CoaMetadata {
        &self.metadata
    }

    /// Retrieves an account definition by code.
    pub fn get(&self, code: &str) -> Option<&AccountDefinition> {
        self.accounts.get(code)
    }

    /// Returns true if an account with the given code exists in the tree.
    pub fn contains(&self, code: &str) -> bool {
        self.accounts.contains_key(code)
    }

    /// Returns all account definitions in the tree.
    pub const fn accounts(&self) -> &HashMap<String, AccountDefinition> {
        &self.accounts
    }

    /// Returns the root account codes (accounts without parent codes).
    pub fn roots(&self) -> &[String] {
        &self.roots
    }

    /// Returns the child account codes for a given parent account code.
    pub fn children_of(&self, code: &str) -> Option<&[String]> {
        self.children.get(code).map(|v| v.as_slice())
    }

    /// Returns whether the account is a leaf account (eligible for direct journal postings).
    pub fn is_leaf(&self, code: &str) -> Result<bool, CoaError> {
        let acc = self.accounts.get(code).ok_or_else(|| CoaError::NotFound(code.to_string()))?;
        Ok(acc.is_leaf)
    }

    /// Checks if direct journal posting is permitted for the account.
    pub fn can_post(&self, code: &str) -> Result<bool, CoaError> {
        self.is_leaf(code)
    }

    /// Validates that an account exists and is postable (is a leaf account).
    /// Returns `Err(CoaError::NonLeafPostingDisallowed)` if it has children or is not a leaf.
    pub fn validate_postable(&self, code: &str) -> Result<(), CoaError> {
        let acc = self.accounts.get(code).ok_or_else(|| CoaError::NotFound(code.to_string()))?;
        if !acc.is_leaf {
            return Err(CoaError::NonLeafPostingDisallowed(code.to_string()));
        }
        Ok(())
    }

    /// Validates a list of account codes to ensure they all exist and are postable.
    pub fn validate_posting_accounts<'a, I>(&self, codes: I) -> Result<(), CoaError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        for code in codes {
            self.validate_postable(code)?;
        }
        Ok(())
    }

    /// Recursively computes aggregate balance for a parent or leaf account from ledger balances.
    ///
    /// For a leaf account, returns its balance in the ledger (or zero if no postings exist yet).
    /// For a parent account, recursively sums the roll-up balances of all child accounts.
    pub fn compute_roll_up_balance(&self, code: &str, ledger: &Ledger) -> Result<Money, LedgerError> {
        let acc = self
            .accounts
            .get(code)
            .ok_or_else(|| LedgerError::AccountNotFound(code.to_string()))?;
        let base_cur = self.metadata.base_currency;

        if acc.is_leaf {
            return Ok(ledger
                .get_account(code)
                .map(|a| a.balance())
                .unwrap_or_else(|| Money::zero(base_cur)));
        }

        let mut total = Money::zero(base_cur);
        if let Some(child_codes) = self.children.get(code) {
            for child_code in child_codes {
                let child_bal = self.compute_roll_up_balance(child_code, ledger)?;
                total = total.checked_add(child_bal)?;
            }
        }

        Ok(total)
    }

    /// Registers all leaf accounts from this tree into a `Ledger` instance.
    pub fn populate_ledger(&self, ledger: &mut Ledger) -> Result<(), LedgerError> {
        for acc in self.accounts.values() {
            if acc.is_leaf && ledger.get_account(&acc.code).is_none() {
                let initial_currency = acc
                    .allowed_currencies
                    .first()
                    .copied()
                    .unwrap_or(self.metadata.base_currency);
                ledger.add_account(Account::new(
                    &acc.code,
                    &acc.name,
                    acc.account_type,
                    initial_currency,
                ))?;
            }
        }
        Ok(())
    }
}
