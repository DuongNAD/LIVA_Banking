//! Comprehensive unit tests for Declarative Chart of Accounts (COA) Schema & Hierarchy Tree.
//!
//! Validates:
//! 1. Default Circular 200/2014 & Circular 133/2016 YAML configurations load and compile cleanly.
//! 2. Cycle detection in parent pointers (DFS traversal).
//! 3. Parent existence validation and account type consistency.
//! 4. Leaf account enforcement: non-leaf accounts cannot receive postings.
//! 5. Recursive roll-up balance computation from leaf accounts.
//! 6. Populating in-memory Ledger from COA tree.

use liva_ledger::{
    load_default_coa_circular133, load_default_coa_circular200, AccountStandard, ChartOfAccounts,
    CoaError, CompiledCoaTree, JournalLine, Ledger,
};
use liva_money::{Currency, Money};

#[test]
fn test_circular200_default_compiles_cleanly() {
    let tree = load_default_coa_circular200().expect("Circular 200 COA must compile cleanly");
    assert_eq!(tree.metadata().standard, AccountStandard::Circular200_2014);
    assert_eq!(tree.metadata().base_currency, Currency::VND);

    // Verify key accounts
    assert!(tree.contains("111"));
    assert!(tree.contains("1121"));
    assert!(tree.contains("131"));
    assert!(tree.contains("1331"));
    assert!(tree.contains("331"));
    assert!(tree.contains("3331"));
    assert!(tree.contains("511"));
    assert!(tree.contains("6427"));
    assert!(tree.contains("6425"));

    // Verify leaf status
    assert!(!tree.is_leaf("112").unwrap(), "112 has sub-accounts and must not be a leaf");
    assert!(tree.is_leaf("1121").unwrap(), "1121 is a leaf sub-account");
    assert!(!tree.is_leaf("642").unwrap(), "642 has sub-accounts and must not be a leaf");
    assert!(tree.is_leaf("6427").unwrap(), "6427 is a leaf account");

    // Verify posting permission
    assert_eq!(tree.can_post("1121").unwrap(), true);
    assert_eq!(tree.can_post("112").unwrap(), false);
    assert!(matches!(
        tree.validate_postable("112"),
        Err(CoaError::NonLeafPostingDisallowed(_))
    ));
}

#[test]
fn test_circular133_default_compiles_cleanly() {
    let tree = load_default_coa_circular133().expect("Circular 133 COA must compile cleanly");
    assert_eq!(tree.metadata().standard, AccountStandard::Circular133_2016);
    assert_eq!(tree.metadata().base_currency, Currency::VND);

    // Verify key accounts under TT 133
    assert!(tree.contains("1121"));
    assert!(tree.contains("1331"));
    assert!(tree.contains("6422")); // Management expense including bank fees
    assert!(!tree.contains("6427")); // 6427 does not exist in TT 133 (merged into 6422)

    assert!(!tree.is_leaf("642").unwrap());
    assert!(tree.is_leaf("6422").unwrap());
    assert!(tree.can_post("6422").unwrap());
}

#[test]
fn test_cycle_detection_rejected() {
    let yaml = r#"
metadata:
  standard: "custom"
  name: "Cyclic Test COA"
  version: "1.0.0"
  description: "Detect cyclic dependencies"
  base_currency: "VND"

accounts:
  - code: "100"
    name: "Account 100"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: "102"
    level: 1
    is_leaf: false
  - code: "101"
    name: "Account 101"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: "100"
    level: 2
    is_leaf: false
  - code: "102"
    name: "Account 102"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: "101"
    level: 3
    is_leaf: true
"#;

    let coa = ChartOfAccounts::from_yaml(yaml).unwrap();
    let result = CompiledCoaTree::compile(coa);
    assert!(matches!(result, Err(CoaError::CycleDetected(_))));
}

#[test]
fn test_self_parent_cycle_detected() {
    let yaml = r#"
metadata:
  standard: "custom"
  name: "Self Parent COA"
  version: "1.0.0"
  description: "Self cycle"
  base_currency: "VND"

accounts:
  - code: "112"
    name: "Self Parent Bank Account"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: "112"
    level: 1
    is_leaf: false
"#;

    let coa = ChartOfAccounts::from_yaml(yaml).unwrap();
    let result = CompiledCoaTree::compile(coa);
    assert!(matches!(result, Err(CoaError::CycleDetected(_))));
}

#[test]
fn test_parent_not_found_rejected() {
    let yaml = r#"
metadata:
  standard: "custom"
  name: "Missing Parent COA"
  version: "1.0.0"
  description: "Missing parent"
  base_currency: "VND"

accounts:
  - code: "1121"
    name: "Sub Account"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: "999"
    level: 2
    is_leaf: true
"#;

    let coa = ChartOfAccounts::from_yaml(yaml).unwrap();
    let result = CompiledCoaTree::compile(coa);
    assert!(matches!(result, Err(CoaError::ParentNotFound { .. })));
}

#[test]
fn test_account_type_mismatch_rejected() {
    let yaml = r#"
metadata:
  standard: "custom"
  name: "Type Mismatch COA"
  version: "1.0.0"
  description: "Parent Asset, Child Liability"
  base_currency: "VND"

accounts:
  - code: "112"
    name: "Parent Asset"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: null
    level: 1
    is_leaf: false
  - code: "1121"
    name: "Child Liability"
    account_type: "liability"
    nature: "credit_normal"
    parent_code: "112"
    level: 2
    is_leaf: true
"#;

    let coa = ChartOfAccounts::from_yaml(yaml).unwrap();
    let result = CompiledCoaTree::compile(coa);
    assert!(matches!(result, Err(CoaError::TypeMismatch { .. })));
}

#[test]
fn test_duplicate_code_rejected() {
    let yaml = r#"
metadata:
  standard: "custom"
  name: "Duplicate Code COA"
  version: "1.0.0"
  description: "Duplicate codes"
  base_currency: "VND"

accounts:
  - code: "1121"
    name: "First 1121"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: null
    level: 1
    is_leaf: true
  - code: "1121"
    name: "Second 1121"
    account_type: "asset"
    nature: "debit_normal"
    parent_code: null
    level: 1
    is_leaf: true
"#;

    let coa = ChartOfAccounts::from_yaml(yaml).unwrap();
    let result = CompiledCoaTree::compile(coa);
    assert!(matches!(result, Err(CoaError::DuplicateCode(_))));
}

#[test]
fn test_recursive_roll_up_balance_computation() {
    let tree = load_default_coa_circular200().unwrap();
    let mut ledger = Ledger::new();
    tree.populate_ledger(&mut ledger).unwrap();

    // Post transactions to sub-accounts 1121 and 1122
    let lines = vec![
        JournalLine::debit("1121", Money::vnd(50_000_000)).unwrap(),
        JournalLine::credit("131", Money::vnd(50_000_000)).unwrap(),
    ];
    ledger.post("Receipt to 1121", lines, 1726400001).unwrap();

    let lines2 = vec![
        JournalLine::debit("1122", Money::vnd(30_000_000)).unwrap(),
        JournalLine::credit("131", Money::vnd(30_000_000)).unwrap(),
    ];
    ledger.post("Receipt to 1122", lines2, 1726400002).unwrap();

    // Verify leaf balances
    assert_eq!(ledger.get_account("1121").unwrap().balance(), Money::vnd(50_000_000));
    assert_eq!(ledger.get_account("1122").unwrap().balance(), Money::vnd(30_000_000));

    // Verify parent roll-up balance for "112"
    let rollup_112 = tree.compute_roll_up_balance("112", &ledger).unwrap();
    assert_eq!(rollup_112, Money::vnd(80_000_000));
}
