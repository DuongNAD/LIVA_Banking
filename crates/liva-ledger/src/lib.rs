//! LIVA Banking — `liva-ledger` Crate
//!
//! Deterministic double-entry accounting state machine and append-only event store.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_money::{Currency, Money, MoneyError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod erp_posting;
pub use erp_posting::{ErpTarget, ReconciliationCertificate, ReconciliationVoucherProposal, VoucherStatus};

pub mod financial_risk;
pub use financial_risk::{
    classify_debt_group, compute_solvency_ratios, evaluate_credit_underwriting, run_stress_test,
    Circular11DebtGroup, DebtClassificationResult, FinancialStatementInput, LiquidityStatus,
    RiskCategory, SolvencyRatios, StressScenario, StressTestResult, UnderwritingDecision,
    UnderwritingResult, BPS_ONE,
};

pub mod vas_reporting;
pub use vas_reporting::{
    build_trial_balance, calculate_cit_finalization, compute_cashflow_statement,
    compute_income_statement, generate_vat_return, CashFlowStatementReport, CitFinalizationReport,
    IncomeStatementReport, TrialBalanceLine, TrialBalanceReport, VatReturnReport,
    CIT_STANDARD_RATE_BPS, VAT_REDUCED_RATE_BPS, VAT_STANDARD_RATE_BPS,
};

pub mod rbac_config;
pub use rbac_config::{
    can_perform_action, is_fee_within_tolerance, validate_sod_rule, BankApiConfig,
    ErpBridgeConfig, ReconciliationThresholdConfig, UserAccount, UserRole,
};

/// Errors arising from accounting invariant checks and ledger operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LedgerError {
    #[error("Unbalanced journal entry: Total Debit ({total_debit}) != Total Credit ({total_credit})")]
    UnbalancedEntry {
        total_debit: i64,
        total_credit: i64,
    },

    #[error("Account with code '{0}' not found in ledger")]
    AccountNotFound(String),

    #[error("Account with code '{0}' already exists")]
    AccountAlreadyExists(String),

    #[error("Currency mismatch: expected {expected:?}, found {found:?}")]
    CurrencyMismatch {
        expected: Currency,
        found: Currency,
    },

    #[error("Journal line amount must be strictly positive (> 0)")]
    NonPositiveAmount,

    #[error("Journal entry must have at least 2 lines")]
    InsufficientLines,

    #[error("Money calculation error: {0}")]
    Money(#[from] MoneyError),

    #[error("Balance invariant check failed: Expected closing {expected}, Calculated {calculated}")]
    InvariantViolation {
        expected: String,
        calculated: String,
    },
}

/// Standard accounting account classification according to Vietnam Circular 200/2014/TT-BTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountType {
    /// Loại 1 & 2: Tài sản (Asset) — Số dư bình thường bên NỢ
    Asset,
    /// Loại 3: Nợ phải trả (Liability) — Số dư bình thường bên CÓ
    Liability,
    /// Loại 4: Vốn chủ sở hữu (Equity) — Số dư bình thường bên CÓ
    Equity,
    /// Loại 5 & 7: Doanh thu & Thu nhập khác (Revenue) — Số dư bình thường bên CÓ
    Revenue,
    /// Loại 6 & 8: Chi phí & Chi phí khác (Expense) — Số dư bình thường bên NỢ
    Expense,
}

impl AccountType {
    /// Returns true if this account type normally maintains a Debit balance.
    pub const fn is_debit_normal(&self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Expense)
    }

    /// Returns true if this account type normally maintains a Credit balance.
    pub const fn is_credit_normal(&self) -> bool {
        matches!(
            self,
            AccountType::Liability | AccountType::Equity | AccountType::Revenue
        )
    }
}

/// Posting type for a journal line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostingType {
    /// Ghi Nợ (Debit)
    Debit,
    /// Ghi Có (Credit)
    Credit,
}

/// Individual ledger account holding code, name, type, and current normal balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    balance: Money,
}

impl Account {
    /// Creates a new account with zero starting balance.
    pub fn new(code: impl Into<String>, name: impl Into<String>, account_type: AccountType, currency: Currency) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            account_type,
            currency,
            balance: Money::zero(currency),
        }
    }

    /// Creates an account with an explicit opening balance.
    pub fn with_opening_balance(
        code: impl Into<String>,
        name: impl Into<String>,
        account_type: AccountType,
        balance: Money,
    ) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            account_type,
            currency: balance.currency(),
            balance,
        }
    }

    /// Returns the current balance of the account in its normal balance direction.
    pub const fn balance(&self) -> Money {
        self.balance
    }

    /// Applies a posting to the account according to its accounting type.
    pub fn apply_posting(&mut self, posting: PostingType, amount: Money) -> Result<(), LedgerError> {
        if self.currency != amount.currency() {
            return Err(LedgerError::CurrencyMismatch {
                expected: self.currency,
                found: amount.currency(),
            });
        }

        let is_increase = if self.account_type.is_debit_normal() {
            posting == PostingType::Debit
        } else {
            posting == PostingType::Credit
        };

        if is_increase {
            self.balance = self.balance.checked_add(amount)?;
        } else {
            self.balance = self.balance.checked_sub(amount)?;
        }

        Ok(())
    }
}

/// Single line in a journal entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalLine {
    pub account_code: String,
    pub posting_type: PostingType,
    pub amount: Money,
}

impl JournalLine {
    /// Creates a new journal line.
    pub fn new(account_code: impl Into<String>, posting_type: PostingType, amount: Money) -> Result<Self, LedgerError> {
        if !amount.is_positive() {
            return Err(LedgerError::NonPositiveAmount);
        }
        Ok(Self {
            account_code: account_code.into(),
            posting_type,
            amount,
        })
    }

    /// Convenience helper for Debit line.
    pub fn debit(account_code: impl Into<String>, amount: Money) -> Result<Self, LedgerError> {
        Self::new(account_code, PostingType::Debit, amount)
    }

    /// Convenience helper for Credit line.
    pub fn credit(account_code: impl Into<String>, amount: Money) -> Result<Self, LedgerError> {
        Self::new(account_code, PostingType::Credit, amount)
    }
}

/// Atomic double-entry journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: u64,
    pub timestamp: i64,
    pub description: String,
    pub lines: Vec<JournalLine>,
}

impl JournalEntry {
    /// Validates the fundamental double-entry balance invariant: `Sum(Debit) == Sum(Credit)`.
    pub fn verify_balance(&self) -> Result<(), LedgerError> {
        if self.lines.len() < 2 {
            return Err(LedgerError::InsufficientLines);
        }

        let first_currency = self.lines[0].amount.currency();
        let mut total_debit = 0i64;
        let mut total_credit = 0i64;

        for line in &self.lines {
            if line.amount.currency() != first_currency {
                return Err(LedgerError::CurrencyMismatch {
                    expected: first_currency,
                    found: line.amount.currency(),
                });
            }
            if !line.amount.is_positive() {
                return Err(LedgerError::NonPositiveAmount);
            }

            match line.posting_type {
                PostingType::Debit => {
                    total_debit = total_debit
                        .checked_add(line.amount.amount())
                        .ok_or(MoneyError::Overflow)?;
                }
                PostingType::Credit => {
                    total_credit = total_credit
                        .checked_add(line.amount.amount())
                        .ok_or(MoneyError::Overflow)?;
                }
            }
        }

        if total_debit != total_credit {
            return Err(LedgerError::UnbalancedEntry {
                total_debit,
                total_credit,
            });
        }

        Ok(())
    }
}

/// Double-entry Ledger state machine with append-only event store.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Ledger {
    accounts: HashMap<String, Account>,
    entries: Vec<JournalEntry>,
    next_entry_id: u64,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            entries: Vec::new(),
            next_entry_id: 1,
        }
    }

    /// Registers an account into the ledger.
    pub fn add_account(&mut self, account: Account) -> Result<(), LedgerError> {
        if self.accounts.contains_key(&account.code) {
            return Err(LedgerError::AccountAlreadyExists(account.code));
        }
        self.accounts.insert(account.code.clone(), account);
        Ok(())
    }

    /// Retrieves an account by its code.
    pub fn get_account(&self, code: &str) -> Option<&Account> {
        self.accounts.get(code)
    }

    /// Returns the count of recorded journal entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Atomically posts a validated journal entry into the ledger and updates balances.
    pub fn post(
        &mut self,
        description: impl Into<String>,
        lines: Vec<JournalLine>,
        timestamp: i64,
    ) -> Result<u64, LedgerError> {
        let entry = JournalEntry {
            id: self.next_entry_id,
            timestamp,
            description: description.into(),
            lines,
        };

        // 1. Verify internal double-entry equality (Debit == Credit)
        entry.verify_balance()?;

        // 2. Pre-verify that all target accounts exist and currencies match
        for line in &entry.lines {
            let acc = self
                .accounts
                .get(&line.account_code)
                .ok_or_else(|| LedgerError::AccountNotFound(line.account_code.clone()))?;

            if acc.currency != line.amount.currency() {
                return Err(LedgerError::CurrencyMismatch {
                    expected: acc.currency,
                    found: line.amount.currency(),
                });
            }
        }

        // 3. Atomically update account balances
        for line in &entry.lines {
            let acc = self.accounts.get_mut(&line.account_code).unwrap();
            acc.apply_posting(line.posting_type, line.amount)?;
        }

        // 4. Append to append-only event store
        let id = entry.id;
        self.entries.push(entry);
        self.next_entry_id += 1;

        Ok(id)
    }

    /// Verifies the Fundamental Accounting Equation for a given currency:
    /// `Total Assets + Total Expenses == Total Liabilities + Total Equity + Total Revenue`.
    pub fn verify_accounting_equation(&self, currency: Currency) -> Result<bool, LedgerError> {
        let mut debit_side = 0i64;
        let mut credit_side = 0i64;

        for acc in self.accounts.values() {
            if acc.currency != currency {
                continue;
            }
            let amt = acc.balance.amount();
            if acc.account_type.is_debit_normal() {
                debit_side = debit_side.checked_add(amt).ok_or(MoneyError::Overflow)?;
            } else {
                credit_side = credit_side.checked_add(amt).ok_or(MoneyError::Overflow)?;
            }
        }

        Ok(debit_side == credit_side)
    }

    /// Verifies the bank statement invariant:
    /// `Closing = Opening + Sum(Credit) - Sum(Debit)`.
    pub fn verify_statement_balance(
        opening: Money,
        closing: Money,
        transactions: &[(PostingType, Money)],
    ) -> Result<bool, LedgerError> {
        if opening.currency() != closing.currency() {
            return Err(LedgerError::CurrencyMismatch {
                expected: opening.currency(),
                found: closing.currency(),
            });
        }

        let mut running = opening;

        for (post_type, amount) in transactions {
            if amount.currency() != opening.currency() {
                return Err(LedgerError::CurrencyMismatch {
                    expected: opening.currency(),
                    found: amount.currency(),
                });
            }
            match post_type {
                PostingType::Credit => {
                    // Credit to bank account = Cash Inflow
                    running = running.checked_add(*amount)?;
                }
                PostingType::Debit => {
                    // Debit to bank account = Cash Outflow
                    running = running.checked_sub(*amount)?;
                }
            }
        }

        Ok(running == closing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_entry_balance_and_posting() {
        let mut ledger = Ledger::new();

        // TK 1121: Tiền gửi ngân hàng (Asset)
        ledger
            .add_account(Account::new("1121", "VCB Banking", AccountType::Asset, Currency::VND))
            .unwrap();
        // TK 5111: Doanh thu bán hàng (Revenue)
        ledger
            .add_account(Account::new("5111", "Doanh thu", AccountType::Revenue, Currency::VND))
            .unwrap();

        let lines = vec![
            JournalLine::debit("1121", Money::vnd(50_000_000)).unwrap(),
            JournalLine::credit("5111", Money::vnd(50_000_000)).unwrap(),
        ];

        let tx_id = ledger.post("Khách hàng thanh toán hợp đồng HD01", lines, 1726400000).unwrap();
        assert_eq!(tx_id, 1);

        assert_eq!(ledger.get_account("1121").unwrap().balance(), Money::vnd(50_000_000));
        assert_eq!(ledger.get_account("5111").unwrap().balance(), Money::vnd(50_000_000));
        assert!(ledger.verify_accounting_equation(Currency::VND).unwrap());
    }

    #[test]
    fn test_unbalanced_entry_rejected() {
        let mut ledger = Ledger::new();
        ledger
            .add_account(Account::new("1121", "VCB Banking", AccountType::Asset, Currency::VND))
            .unwrap();
        ledger
            .add_account(Account::new("5111", "Doanh thu", AccountType::Revenue, Currency::VND))
            .unwrap();

        let unbalanced_lines = vec![
            JournalLine::debit("1121", Money::vnd(50_000_000)).unwrap(),
            JournalLine::credit("5111", Money::vnd(49_000_000)).unwrap(),
        ];

        let res = ledger.post("Lệch tiền 1 triệu", unbalanced_lines, 1726400000);
        assert!(matches!(res, Err(LedgerError::UnbalancedEntry { .. })));
    }

    #[test]
    fn test_statement_balance_invariant() {
        let opening = Money::vnd(100_000_000);
        let closing = Money::vnd(145_000_000);

        let txs = vec![
            (PostingType::Credit, Money::vnd(50_000_000)), // +50M
            (PostingType::Debit, Money::vnd(5_000_000)),   // -5M (phí/rút)
        ];

        let is_valid = Ledger::verify_statement_balance(opening, closing, &txs).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_erp_voucher_exact_customer_receipt_misa_json() {
        let voucher = ReconciliationVoucherProposal::new_customer_receipt(
            "guid-test-01",
            "BATCH-01",
            "MATCH-01",
            "PKT-VCB-001",
            "2026-09-15",
            "Công ty Cổ phần An Phát",
            Some("0102030405".to_string()),
            "Thu tiền bán hàng qua Napas",
            Money::vnd(50_000_000),
            "1121",
            "131",
            ErpTarget::MisaAmis,
        )
        .unwrap();

        assert!(voucher.is_balanced());
        assert_eq!(voucher.total_debit, Money::vnd(50_000_000));
        assert_eq!(voucher.total_credit, Money::vnd(50_000_000));
        assert!(voucher.idempotency_hash.starts_with("0x"));

        let misa_json = voucher.to_misa_json();
        assert!(misa_json.contains("LIVA_ENTERPRISE_VN"));
        assert!(misa_json.contains("guid-test-01"));
        assert!(misa_json.contains("50000000"));
    }

    #[test]
    fn test_erp_voucher_fee_deduction() {
        // Invoice: 10,000,000 VND
        // Net received: 9,988,000 VND
        // Fee: 12,000 VND
        let voucher = ReconciliationVoucherProposal::new_customer_receipt_with_fee(
            "guid-fee-01",
            "BATCH-01",
            "MATCH-02",
            "PKT-VCB-002",
            "2026-09-15",
            "Công ty CP Cơ Khí Minh Tâm",
            Some("0301020304".to_string()),
            "Thu tiền gia công (trừ phí NH 12k)",
            Money::vnd(9_988_000),
            Money::vnd(12_000),
            Money::vnd(10_000_000),
            "1121",
            "6425",
            "131",
            ErpTarget::MisaAmis,
        )
        .unwrap();

        assert!(voucher.is_balanced());
        assert_eq!(voucher.lines.len(), 3);
        assert_eq!(voucher.lines[1].account_code, "6425");
        assert_eq!(voucher.lines[1].amount, Money::vnd(12_000));

        // Mismatched fee amount should fail
        let unbalanced_res = ReconciliationVoucherProposal::new_customer_receipt_with_fee(
            "guid-fee-bad",
            "BATCH-01",
            "MATCH-02",
            "PKT-BAD",
            "2026-09-15",
            "Partner",
            None,
            "Lỗi số tiền",
            Money::vnd(9_988_000),
            Money::vnd(10_000), // Sai 2,000 VND
            Money::vnd(10_000_000),
            "1121",
            "6425",
            "131",
            ErpTarget::MisaAmis,
        );
        assert!(unbalanced_res.is_err());
    }

    #[test]
    fn test_erp_voucher_split_receipt_fast_xml() {
        let splits = vec![
            ("131".to_string(), Money::vnd(12_000_000)),
            ("131".to_string(), Money::vnd(18_000_000)),
        ];

        let voucher = ReconciliationVoucherProposal::new_split_receipt(
            "guid-split-01",
            "BATCH-01",
            "MATCH-SPLIT-01",
            "PKT-FAST-003",
            "2026-09-15",
            "Tập đoàn TM Dương Đông",
            Some("0405060708".to_string()),
            "Thanh toán 2 hóa đơn dịch vụ kho bãi và vận chuyển",
            Money::vnd(30_000_000),
            &splits,
            "1121",
            ErpTarget::FastBusiness,
        )
        .unwrap();

        assert!(voucher.is_balanced());
        assert_eq!(voucher.lines.len(), 3);
        let xml = voucher.to_fast_xml();
        assert!(xml.contains("<VoucherGUID>guid-split-01</VoucherGUID>"));
        assert!(xml.contains("<TotalAmount>30000000</TotalAmount>"));
        assert!(xml.contains("<AccountCode>1121</AccountCode>"));
    }

    #[test]
    fn test_reconciliation_certificate_zero_variance() {
        // Bank Closing: 350,000,000
        // In-Transit Deposits: +20,000,000
        // In-Transit Withdrawals: -10,000,000
        // Adjusted Bank = 350M + 20M - 10M = 360,000,000
        // GL Closing: 360,000,000
        // Variance = 0 VND
        let cert = ReconciliationCertificate::create_and_verify(
            "CERT-20260915-01",
            "VCB-00710009821",
            "2026-09-01",
            "2026-09-15",
            Money::vnd(300_000_000),
            Money::vnd(70_000_000),
            Money::vnd(20_000_000),
            Money::vnd(350_000_000),
            Money::vnd(300_000_000),
            Money::vnd(80_000_000),
            Money::vnd(20_000_000),
            Money::vnd(360_000_000),
            Money::vnd(20_000_000),
            Money::vnd(10_000_000),
            "0x98fbc4e021a8...",
        )
        .unwrap();

        assert!(cert.is_certified);
        assert!(cert.variance.is_zero());
        assert_eq!(cert.adjusted_bank_balance, Money::vnd(360_000_000));
    }

    #[test]
    fn test_solvency_ratios_healthy_and_edge_cases() {
        let input = FinancialStatementInput {
            currency: Currency::VND,
            ebitda: Money::vnd(1_500_000_000),
            capex: Money::vnd(200_000_000),
            debt_service_principal: Money::vnd(600_000_000),
            debt_service_interest: Money::vnd(200_000_000),
            cash_and_equivalents: Money::vnd(400_000_000),
            marketable_securities: Money::vnd(100_000_000),
            accounts_receivable: Money::vnd(500_000_000),
            inventory: Money::vnd(400_000_000),
            current_assets: Money::vnd(1_400_000_000),
            current_liabilities: Money::vnd(800_000_000),
            ebit: Money::vnd(1_300_000_000),
            interest_expense: Money::vnd(200_000_000),
        };

        let ratios = compute_solvency_ratios(&input).unwrap();
        // NOI = 1.3B, Debt Service = 800M => DSCR = (1.3B * 10,000) / 800M = 16,250 bps (1.625x)
        assert_eq!(ratios.dscr_bps, 16_250);
        assert_eq!(ratios.dscr_category, RiskCategory::Healthy);
        assert_eq!(ratios.dscr_buffer, Money::vnd(500_000_000));

        // Quick Ratio = 1.0B / 800M = 12,500 bps (1.25x)
        assert_eq!(ratios.quick_ratio_bps, 12_500);
        assert_eq!(ratios.quick_ratio_status, LiquidityStatus::Adequate);

        // Edge case: Debt Free
        let mut debt_free_input = input.clone();
        debt_free_input.debt_service_principal = Money::zero(Currency::VND);
        debt_free_input.debt_service_interest = Money::zero(Currency::VND);
        let debt_free_ratios = compute_solvency_ratios(&debt_free_input).unwrap();
        assert_eq!(debt_free_ratios.dscr_category, RiskCategory::DebtFree);
        assert_eq!(debt_free_ratios.dscr_buffer, Money::vnd(1_300_000_000));
    }

    #[test]
    fn test_circular11_debt_classification_provisions() {
        let principal = Money::vnd(1_000_000_000); // 1 Tỷ VND

        // Nhóm 1: Overdue 5 days
        let g1 = classify_debt_group(principal, 5, false).unwrap();
        assert_eq!(g1.group, Circular11DebtGroup::Group1Standard);
        assert_eq!(g1.specific_provision_rate_bps, 0);
        assert_eq!(g1.general_provision_rate_bps, 75); // 0.75% = 7.5M VND
        assert_eq!(g1.required_general_provision, Money::vnd(7_500_000));
        assert_eq!(g1.required_specific_provision, Money::zero(Currency::VND));

        // Nhóm 2: Overdue 45 days => 5% specific
        let g2 = classify_debt_group(principal, 45, false).unwrap();
        assert_eq!(g2.group, Circular11DebtGroup::Group2SpecialMention);
        assert_eq!(g2.specific_provision_rate_bps, 500); // 500 bps = 50M VND
        assert_eq!(g2.required_specific_provision, Money::vnd(50_000_000));

        // Nhóm 3: Overdue 120 days => 20% specific
        let g3 = classify_debt_group(principal, 120, false).unwrap();
        assert_eq!(g3.group, Circular11DebtGroup::Group3Substandard);
        assert_eq!(g3.specific_provision_rate_bps, 2_000); // 200M VND
        assert_eq!(g3.required_specific_provision, Money::vnd(200_000_000));

        // Nhóm 5: Overdue 400 days => 100% specific, 0% general
        let g5 = classify_debt_group(principal, 400, false).unwrap();
        assert_eq!(g5.group, Circular11DebtGroup::Group5Loss);
        assert_eq!(g5.specific_provision_rate_bps, 10_000);
        assert_eq!(g5.required_specific_provision, Money::vnd(1_000_000_000));
        assert_eq!(g5.required_general_provision, Money::zero(Currency::VND));
    }

    #[test]
    fn test_stress_testing_and_underwriting_decision() {
        let input = FinancialStatementInput {
            currency: Currency::VND,
            ebitda: Money::vnd(2_000_000_000),
            capex: Money::vnd(200_000_000),
            debt_service_principal: Money::vnd(700_000_000),
            debt_service_interest: Money::vnd(300_000_000),
            cash_and_equivalents: Money::vnd(600_000_000),
            marketable_securities: Money::vnd(200_000_000),
            accounts_receivable: Money::vnd(800_000_000),
            inventory: Money::vnd(500_000_000),
            current_assets: Money::vnd(2_100_000_000),
            current_liabilities: Money::vnd(1_000_000_000),
            ebit: Money::vnd(1_800_000_000),
            interest_expense: Money::vnd(300_000_000),
        };

        // Moderate stress
        let stress = run_stress_test(&input, StressScenario::ModerateStress).unwrap();
        assert!(stress.passes_stress_test);
        assert!(stress.stressed_dscr_bps >= 10_000);

        // Underwriting decision
        let debt_class = classify_debt_group(Money::vnd(1_000_000_000), 0, false).unwrap();
        let underwriting = evaluate_credit_underwriting(&input, &debt_class).unwrap();
        assert_eq!(underwriting.decision, UnderwritingDecision::Approved);
        assert!(underwriting.max_recommended_credit_limit.amount() > 0);
    }

    #[test]
    fn test_trial_balance_invariant_validation() {
        let lines = vec![
            TrialBalanceLine {
                account_code: "1121".to_string(),
                account_name: "Tiền gửi ngân hàng VCB".to_string(),
                opening_debit: Money::vnd(500_000_000),
                opening_credit: Money::zero(Currency::VND),
                period_debit: Money::vnd(1_200_000_000),
                period_credit: Money::vnd(800_000_000),
                closing_debit: Money::vnd(900_000_000),
                closing_credit: Money::zero(Currency::VND),
            },
            TrialBalanceLine {
                account_code: "331".to_string(),
                account_name: "Phải trả cho người bán".to_string(),
                opening_debit: Money::zero(Currency::VND),
                opening_credit: Money::vnd(500_000_000),
                period_debit: Money::vnd(800_000_000),
                period_credit: Money::vnd(1_200_000_000),
                closing_debit: Money::zero(Currency::VND),
                closing_credit: Money::vnd(900_000_000),
            },
        ];

        let report = build_trial_balance("2026-Q3", lines, Currency::VND).unwrap();
        assert!(report.is_balanced);
        assert_eq!(report.total_opening_debit, Money::vnd(500_000_000));
        assert_eq!(report.total_opening_credit, Money::vnd(500_000_000));
        assert_eq!(report.total_period_debit, Money::vnd(2_000_000_000));
        assert_eq!(report.total_period_credit, Money::vnd(2_000_000_000));
        assert_eq!(report.total_closing_debit, Money::vnd(900_000_000));
        assert_eq!(report.total_closing_credit, Money::vnd(900_000_000));

        // Unbalanced case
        let bad_lines = vec![TrialBalanceLine {
            account_code: "1121".to_string(),
            account_name: "Tiền gửi ngân hàng VCB".to_string(),
            opening_debit: Money::vnd(500_000_000),
            opening_credit: Money::zero(Currency::VND),
            period_debit: Money::vnd(1_000_000_000),
            period_credit: Money::vnd(800_000_000),
            closing_debit: Money::vnd(700_000_000),
            closing_credit: Money::zero(Currency::VND),
        }];
        assert!(build_trial_balance("2026-Q3", bad_lines, Currency::VND).is_err());
    }

    #[test]
    fn test_income_statement_computation() {
        let report = compute_income_statement(
            "2026-Q3",
            Money::vnd(5_000_000_000), // Gross revenue
            Money::vnd(100_000_000),   // Deductions
            Money::vnd(3_000_000_000), // COGS
            Money::vnd(50_000_000),    // Financial income
            Money::vnd(80_000_000),    // Financial expense
            Money::vnd(60_000_000),    // Interest
            Money::vnd(200_000_000),   // Selling
            Money::vnd(300_000_000),   // Admin
            Money::vnd(20_000_000),    // Other income
            Money::vnd(10_000_000),    // Other expense
            CIT_STANDARD_RATE_BPS,     // 20%
        )
        .unwrap();

        assert_eq!(report.net_revenue, Money::vnd(4_900_000_000));
        assert_eq!(report.gross_profit, Money::vnd(1_900_000_000));
        // Operating profit = 1,900M + 50M - (80M + 200M + 300M) = 1,370M
        assert_eq!(report.operating_profit, Money::vnd(1_370_000_000));
        // PBT = 1,370M + (20M - 10M) = 1,380M
        assert_eq!(report.profit_before_tax, Money::vnd(1_380_000_000));
        // Tax = 1,380M * 20% = 276M
        assert_eq!(report.current_cit_expense, Money::vnd(276_000_000));
        // PAT = 1,380M - 276M = 1,104M
        assert_eq!(report.net_profit_after_tax, Money::vnd(1_104_000_000));
    }

    #[test]
    fn test_cashflow_statement_computation() {
        let report = compute_cashflow_statement(
            "2026-Q3",
            Money::vnd(4_000_000_000), // Op In
            Money::vnd(2_500_000_000), // Op Out => Net Op = +1,500M
            Money::vnd(100_000_000),   // Inv In
            Money::vnd(600_000_000),   // Inv Out => Net Inv = -500M
            Money::vnd(500_000_000),   // Fin In
            Money::vnd(800_000_000),   // Fin Out => Net Fin = -300M
            Money::vnd(1_000_000_000), // Opening Cash
        )
        .unwrap();

        assert_eq!(report.net_operating_flow, Money::vnd(1_500_000_000));
        assert_eq!(report.net_investing_flow, Money::vnd(-500_000_000));
        assert_eq!(report.net_financing_flow, Money::vnd(-300_000_000));
        // Net Cash Flow = 1,500M - 500M - 300M = +700M
        assert_eq!(report.net_cash_flow, Money::vnd(700_000_000));
        // Closing Cash = 1,000M + 700M = 1,700M
        assert_eq!(report.closing_cash, Money::vnd(1_700_000_000));
    }

    #[test]
    fn test_vat_return_calculation() {
        // Sales: 1B @ 8% (80M output) + 2B @ 10% (200M output) = 280M total output
        // Input: 150M deductible input tax
        // Net VAT payable = 280M - 150M = 130M
        let vat = generate_vat_return(
            "2026-Q3",
            Money::vnd(150_000_000),
            Money::vnd(1_000_000_000),
            Money::vnd(2_000_000_000),
        )
        .unwrap();

        assert_eq!(vat.output_tax_8pct, Money::vnd(80_000_000));
        assert_eq!(vat.output_tax_10pct, Money::vnd(200_000_000));
        assert_eq!(vat.total_output_tax, Money::vnd(280_000_000));
        assert_eq!(vat.net_vat_payable, Money::vnd(130_000_000));
        assert_eq!(vat.carried_forward_tax, Money::zero(Currency::VND));

        // Excess input tax case
        let excess_vat = generate_vat_return(
            "2026-Q3",
            Money::vnd(350_000_000),
            Money::vnd(1_000_000_000),
            Money::vnd(2_000_000_000),
        )
        .unwrap();
        assert_eq!(excess_vat.net_vat_payable, Money::zero(Currency::VND));
        assert_eq!(excess_vat.carried_forward_tax, Money::vnd(70_000_000));
    }

    #[test]
    fn test_cit_finalization_computation() {
        // Accounting PBT = 1,000M
        // B4 non-deductible expenses = 100M => Taxable income = 1,100M
        // 20% CIT = 220M
        // Provisional paid 4 quarters = 180M
        // Remaining payable = 40M
        let cit = calculate_cit_finalization(
            "2026",
            Money::vnd(1_000_000_000),
            Money::vnd(100_000_000),
            Money::vnd(180_000_000),
        )
        .unwrap();

        assert_eq!(cit.taxable_income, Money::vnd(1_100_000_000));
        assert_eq!(cit.total_cit_liability, Money::vnd(220_000_000));
        assert_eq!(cit.remaining_tax_payable, Money::vnd(40_000_000));
        assert_eq!(cit.overpaid_tax, Money::zero(Currency::VND));
    }

    #[test]
    fn test_sod_rule_enforcement() {
        // Valid Maker != Checker
        assert!(validate_sod_rule("MAKER_NAM", "CHECKER_TRI").is_ok());

        // Identical Maker and Checker must fail SoD
        let violation = validate_sod_rule("USER_NAM", "USER_NAM");
        assert!(violation.is_err());

        // Case-insensitive match must also fail SoD
        assert!(validate_sod_rule("User_Nam", "USER_NAM").is_err());

        // Empty IDs must fail
        assert!(validate_sod_rule("", "CHECKER_TRI").is_err());
        assert!(validate_sod_rule("MAKER_NAM", "   ").is_err());
    }

    #[test]
    fn test_rbac_permission_matrix() {
        // Maker permissions
        assert!(can_perform_action(&UserRole::Maker, "RECONCILE_EXECUTE"));
        assert!(can_perform_action(&UserRole::Maker, "HITL_PROPOSE"));
        assert!(can_perform_action(&UserRole::Maker, "PAYMENT_DRAFT"));
        assert!(!can_perform_action(&UserRole::Maker, "HITL_APPROVE"));
        assert!(!can_perform_action(&UserRole::Maker, "PAYMENT_AUTHORIZE"));

        // Checker permissions
        assert!(can_perform_action(&UserRole::Checker, "HITL_APPROVE"));
        assert!(can_perform_action(&UserRole::Checker, "PAYMENT_AUTHORIZE"));
        assert!(!can_perform_action(&UserRole::Checker, "RECONCILE_EXECUTE"));

        // Compliance officer permissions
        assert!(can_perform_action(&UserRole::ComplianceOfficer, "AML_SIGN_STR"));
        assert!(!can_perform_action(&UserRole::ComplianceOfficer, "PAYMENT_AUTHORIZE"));

        // Admin permissions
        assert!(can_perform_action(&UserRole::Admin, "SYSTEM_CONFIGURE"));
        assert!(!can_perform_action(&UserRole::Admin, "PAYMENT_AUTHORIZE"));

        // All roles can view reports
        assert!(can_perform_action(&UserRole::Auditor, "VIEW_REPORTS"));
        assert!(can_perform_action(&UserRole::Admin, "VIEW_REPORTS"));
    }

    #[test]
    fn test_reconciliation_threshold_tolerance() {
        let config = ReconciliationThresholdConfig::default();

        // Exact match
        assert!(is_fee_within_tolerance(Money::vnd(0), &config));

        // Fee variance <= 50,000 VND is accepted under Tier 2
        assert!(is_fee_within_tolerance(Money::vnd(11_000), &config));
        assert!(is_fee_within_tolerance(Money::vnd(-50_000), &config));

        // Excess variance > 50,000 VND is rejected
        assert!(!is_fee_within_tolerance(Money::vnd(50_001), &config));
        assert!(!is_fee_within_tolerance(Money::vnd(-100_000), &config));
    }
}
