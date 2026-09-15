//! LIVA Banking — `liva-ledger` Crate
//!
//! Deterministic double-entry accounting state machine and append-only event store.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_money::{Currency, Money, MoneyError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

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
}
