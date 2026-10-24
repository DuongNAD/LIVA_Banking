//! Comprehensive unit tests for Dynamic Posting Rules Engine.
//!
//! Validates:
//! 1. Default posting rules load and compile cleanly from YAML.
//! 2. Pattern matching on narration regexes, direction, and counterparty.
//! 3. Double-entry balance invariant ($\sum \text{Debit} \equiv \sum \text{Credit}$, $\Delta = 0$).
//! 4. Unbundled fee voucher generation with 6427/1331/1121 lines.
//! 5. Financial fees to 635, interest to 515, taxes to 3331, salary to 334.
//! 6. Default receipt and payment fallbacks.
//! 7. Seamless conversion to `JournalEntry` and posting into `Ledger`.
//! 8. Chart of Accounts leaf account verification via `evaluate_with_coa`.

use liva_ledger::{
    load_default_coa_circular200, load_default_posting_engine, voucher_to_journal_entry,
    ErpTarget, Ledger, PostingContext, PostingType,
};
use liva_money::Money;

#[test]
fn test_default_engine_compiles_cleanly() {
    let engine = load_default_posting_engine().expect("Default posting engine must compile");
    assert!(!engine.config().rules.is_empty());
    assert_eq!(engine.config().defaults.bank_default_account, "1121");
    assert_eq!(engine.config().defaults.fee_expense_account, "6427");
}

#[test]
fn test_bank_fee_unbundled_posting() {
    let engine = load_default_posting_engine().unwrap();

    let ctx = PostingContext::new(
        "VCB",
        "FT2609160001",
        PostingType::Debit,
        Money::vnd(11_000),
        "PHI DUY TRI DICH VU DIGIBANK THANG 09",
        "1121",
        "NGAN HANG TMCP NGOAI THUONG VIET NAM",
        "uuid-fee-01",
        "batch-01",
        "match-01",
        "2026-09-16",
        "PC00001",
        ErpTarget::MisaAmis,
    );

    let proposal = engine.evaluate(&ctx).expect("Rule evaluation should succeed");

    // Must be balanced
    assert_eq!(proposal.total_debit, proposal.total_credit);
    assert_eq!(proposal.total_debit, Money::vnd(11_000));
    assert_eq!(proposal.lines.len(), 3);

    // Verify lines: Debit 6427 (10,000), Debit 1331 (1,000), Credit 1121 (11,000)
    let net_line = proposal.lines.iter().find(|l| l.account_code == "6427").unwrap();
    assert_eq!(net_line.posting_type, PostingType::Debit);
    assert_eq!(net_line.amount, Money::vnd(10_000));

    let vat_line = proposal.lines.iter().find(|l| l.account_code == "1331").unwrap();
    assert_eq!(vat_line.posting_type, PostingType::Debit);
    assert_eq!(vat_line.amount, Money::vnd(1_000));

    let bank_line = proposal.lines.iter().find(|l| l.account_code == "1121").unwrap();
    assert_eq!(bank_line.posting_type, PostingType::Credit);
    assert_eq!(bank_line.amount, Money::vnd(11_000));

    // Convert to JournalEntry and post into Ledger
    let entry = voucher_to_journal_entry(&proposal, 1, 1726400001).unwrap();
    assert!(entry.verify_balance().is_ok());

    let mut ledger = Ledger::new();
    let coa = load_default_coa_circular200().unwrap();
    coa.populate_ledger(&mut ledger).unwrap();
    ledger.post_entry(entry).unwrap();

    assert_eq!(ledger.get_account("6427").unwrap().balance(), Money::vnd(10_000));
    assert_eq!(ledger.get_account("1331").unwrap().balance(), Money::vnd(1_000));
}

#[test]
fn test_wire_transfer_fee_posting() {
    let engine = load_default_posting_engine().unwrap();

    let ctx = PostingContext::new(
        "TCB",
        "FT2609160002",
        PostingType::Debit,
        Money::vnd(3_300),
        "PHI CHUYEN TIEN NAPAS 247",
        "1121",
        "TECHCOMBANK",
        "uuid-fee-02",
        "batch-01",
        "match-02",
        "2026-09-16",
        "PC00002",
        ErpTarget::FastBusiness,
    );

    let proposal = engine.evaluate(&ctx).unwrap();
    assert_eq!(proposal.total_debit, proposal.total_credit);
    assert_eq!(proposal.total_debit, Money::vnd(3_300));
    assert_eq!(proposal.lines.len(), 3);
}

#[test]
fn test_financial_fees_routed_to_635() {
    let engine = load_default_posting_engine().unwrap();

    let ctx = PostingContext::new(
        "BIDV",
        "FT2609160003",
        PostingType::Debit,
        Money::vnd(22_000_000),
        "THU PHI PHAT HANH BAO LANH THANH TOAN",
        "1121",
        "BIDV",
        "uuid-fin-01",
        "batch-01",
        "match-03",
        "2026-09-16",
        "PC00003",
        ErpTarget::MisaAmis,
    );

    let proposal = engine.evaluate(&ctx).unwrap();
    assert_eq!(proposal.total_debit, proposal.total_credit);
    assert_eq!(proposal.total_debit, Money::vnd(22_000_000));

    let fee_line = proposal.lines.iter().find(|l| l.account_code == "635").unwrap();
    assert_eq!(fee_line.posting_type, PostingType::Debit);
    assert_eq!(fee_line.amount, Money::vnd(20_000_000));

    let vat_line = proposal.lines.iter().find(|l| l.account_code == "1331").unwrap();
    assert_eq!(vat_line.posting_type, PostingType::Debit);
    assert_eq!(vat_line.amount, Money::vnd(2_000_000));
}

#[test]
fn test_interest_earned_routed_to_515() {
    let engine = load_default_posting_engine().unwrap();

    let ctx = PostingContext::new(
        "VCB",
        "FT2609160004",
        PostingType::Credit,
        Money::vnd(1_250_000),
        "VCB TRA LAI TIEN GUI THANG 08/2026",
        "1121",
        "VIETCOMBANK",
        "uuid-int-01",
        "batch-01",
        "match-04",
        "2026-09-16",
        "PT00001",
        ErpTarget::MisaAmis,
    );

    let proposal = engine.evaluate(&ctx).unwrap();
    assert_eq!(proposal.total_debit, proposal.total_credit);
    assert_eq!(proposal.total_debit, Money::vnd(1_250_000));
    assert_eq!(proposal.lines.len(), 2);

    let bank_line = proposal.lines.iter().find(|l| l.account_code == "1121").unwrap();
    assert_eq!(bank_line.posting_type, PostingType::Debit);

    let rev_line = proposal.lines.iter().find(|l| l.account_code == "515").unwrap();
    assert_eq!(rev_line.posting_type, PostingType::Credit);
}

#[test]
fn test_tax_payment_routed_to_3331() {
    let engine = load_default_posting_engine().unwrap();

    let ctx = PostingContext::new(
        "VCB",
        "FT2609160005",
        PostingType::Debit,
        Money::vnd(15_000_000),
        "NOP THUE GTGT VAO KHO BAC NHA NUOC",
        "1121",
        "KBNN QUAN 1",
        "uuid-tax-01",
        "batch-01",
        "match-05",
        "2026-09-16",
        "PC00004",
        ErpTarget::MisaAmis,
    );

    let proposal = engine.evaluate(&ctx).unwrap();
    assert_eq!(proposal.total_debit, proposal.total_credit);
    assert_eq!(proposal.total_debit, Money::vnd(15_000_000));
    assert_eq!(proposal.lines.len(), 2);

    let tax_line = proposal.lines.iter().find(|l| l.account_code == "3331").unwrap();
    assert_eq!(tax_line.posting_type, PostingType::Debit);
}

#[test]
fn test_payroll_disbursement_routed_to_334() {
    let engine = load_default_posting_engine().unwrap();

    let ctx = PostingContext::new(
        "TCB",
        "FT2609160006",
        PostingType::Debit,
        Money::vnd(85_000_000),
        "THANH TOAN LUONG NHAN VIEN KY 1 THANG 09/2026",
        "1121",
        "CAN BO NHAN VIEN",
        "uuid-sal-01",
        "batch-01",
        "match-06",
        "2026-09-16",
        "PC00005",
        ErpTarget::MisaAmis,
    );

    let proposal = engine.evaluate(&ctx).unwrap();
    assert_eq!(proposal.total_debit, proposal.total_credit);
    assert_eq!(proposal.total_debit, Money::vnd(85_000_000));
    assert_eq!(proposal.lines.len(), 2);

    let sal_line = proposal.lines.iter().find(|l| l.account_code == "334").unwrap();
    assert_eq!(sal_line.posting_type, PostingType::Debit);
}

#[test]
fn test_default_customer_receipt_fallback() {
    let engine = load_default_posting_engine().unwrap();

    let mut ctx = PostingContext::new(
        "VCB",
        "FT2609160007",
        PostingType::Credit,
        Money::vnd(50_000_000),
        "CONG TY ABC THANH TOAN DON HANG 889",
        "1121",
        "CONG TY TNHH ABC",
        "uuid-rec-01",
        "batch-01",
        "match-07",
        "2026-09-16",
        "PT00002",
        ErpTarget::MisaAmis,
    );
    ctx.partner_account_code = Some("131");

    let proposal = engine.evaluate(&ctx).unwrap();
    assert_eq!(proposal.total_debit, proposal.total_credit);
    assert_eq!(proposal.total_debit, Money::vnd(50_000_000));
    assert_eq!(proposal.lines.len(), 2);

    let debit_line = proposal.lines.iter().find(|l| l.posting_type == PostingType::Debit).unwrap();
    assert_eq!(debit_line.account_code, "1121");

    let credit_line = proposal.lines.iter().find(|l| l.posting_type == PostingType::Credit).unwrap();
    assert_eq!(credit_line.account_code, "131");
}

#[test]
fn test_evaluate_with_coa_verification() {
    let engine = load_default_posting_engine().unwrap();
    let coa = load_default_coa_circular200().unwrap();

    let ctx = PostingContext::new(
        "VCB",
        "FT2609160008",
        PostingType::Debit,
        Money::vnd(22_000),
        "PHI DIGIBANK THANG 09",
        "1121",
        "VIETCOMBANK",
        "uuid-fee-03",
        "batch-01",
        "match-08",
        "2026-09-16",
        "PC00006",
        ErpTarget::MisaAmis,
    );

    // Verifies all generated accounts exist and are leaf accounts in Circular 200 COA
    let proposal = engine.evaluate_with_coa(&ctx, &coa).expect("COA validation must pass");
    assert_eq!(proposal.total_debit, proposal.total_credit);
}
