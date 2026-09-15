//! Property-based testing for `liva-ledger` state machine and balance invariants.
//!
//! Validates:
//! 1. Double-entry fundamental accounting equation holds across random transaction batches.
//! 2. Bank statement invariant Closing = Opening + Sum(Credit) - Sum(Debit) holds 100%.
//! 3. Single-unit discrepancies (1 VND) are immediately detected.

use liva_ledger::{Account, AccountType, JournalLine, Ledger, PostingType};
use liva_money::{Currency, Money};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    #[test]
    fn prop_double_entry_equation_invariant(
        tx1_amount in 1_000i64..1_000_000_000i64,
        pct2 in 10i64..40i64,
        pct3 in 10i64..40i64,
    ) {
        let mut ledger = Ledger::new();

        // Register accounts: Assets, Liabilities, Revenue, Expenses
        ledger.add_account(Account::new("1121", "VCB", AccountType::Asset, Currency::VND)).unwrap();
        ledger.add_account(Account::new("131", "Phải thu KH", AccountType::Asset, Currency::VND)).unwrap();
        ledger.add_account(Account::new("331", "Phải trả NCC", AccountType::Liability, Currency::VND)).unwrap();
        ledger.add_account(Account::new("511", "Doanh thu", AccountType::Revenue, Currency::VND)).unwrap();
        ledger.add_account(Account::new("642", "Chi phí", AccountType::Expense, Currency::VND)).unwrap();

        // Tx1: Bán hàng thu tiền ngay vào NH (Nợ 1121 / Có 511)
        let lines1 = vec![
            JournalLine::debit("1121", Money::vnd(tx1_amount)).unwrap(),
            JournalLine::credit("511", Money::vnd(tx1_amount)).unwrap(),
        ];
        ledger.post("Thu tiền bán hàng", lines1, 1726400001).unwrap();

        // Tx2: Chi tiền NH trả nhà cung cấp (Nợ 331 / Có 1121)
        let slice2 = (tx1_amount * pct2 / 100).max(1);
        let lines2 = vec![
            JournalLine::debit("331", Money::vnd(slice2)).unwrap(),
            JournalLine::credit("1121", Money::vnd(slice2)).unwrap(),
        ];
        ledger.post("Thanh toán NCC", lines2, 1726400002).unwrap();

        // Tx3: Ghi nhận chi phí quản lý (Nợ 642 / Có 1121)
        let slice3 = (tx1_amount * pct3 / 100).max(1);
        let lines3 = vec![
            JournalLine::debit("642", Money::vnd(slice3)).unwrap(),
            JournalLine::credit("1121", Money::vnd(slice3)).unwrap(),
        ];
        ledger.post("Chi phí dịch vụ", lines3, 1726400003).unwrap();

        // Bất biến kế toán: Tài sản + Chi phí == Nợ phải trả + Vốn chủ + Doanh thu
        prop_assert!(ledger.verify_accounting_equation(Currency::VND).unwrap());
    }

    #[test]
    fn prop_bank_statement_balance_invariant(
        opening_amt in 100_000_000i64..1_000_000_000i64,
        cr1 in 1_000i64..100_000_000i64,
        cr2 in 1_000i64..100_000_000i64,
        db1 in 1_000i64..50_000_000i64,
        db2 in 1_000i64..50_000_000i64,
    ) {
        let opening = Money::vnd(opening_amt);
        let txs = vec![
            (PostingType::Credit, Money::vnd(cr1)),
            (PostingType::Credit, Money::vnd(cr2)),
            (PostingType::Debit, Money::vnd(db1)),
            (PostingType::Debit, Money::vnd(db2)),
        ];

        let expected_closing_amt = opening_amt + cr1 + cr2 - db1 - db2;
        let closing = Money::vnd(expected_closing_amt);

        // Khẳng định 1: Đẳng thức toán học bảo toàn 100%
        let valid = Ledger::verify_statement_balance(opening, closing, &txs).unwrap();
        prop_assert!(valid);

        // Khẳng định 2: Lệch 1 VND phát hiện ngay lập tức
        let false_closing = Money::vnd(expected_closing_amt + 1);
        let invalid = Ledger::verify_statement_balance(opening, false_closing, &txs).unwrap();
        prop_assert!(!invalid);
    }
}
