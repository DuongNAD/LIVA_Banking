use liva_ledger::PostingType;
use liva_match::{
    create_fee_split_journal, create_pure_fee_journal, BankTransaction, FeeSplitter, LedgerEntry,
    MatchConfig, MatchType, ReconciliationEngine,
};
use liva_money::Money;

#[test]
fn test_known_fees_recognition() {
    for &fee in &[1_100, 2_200, 3_300, 5_500, 7_700, 8_800, 9_900, 11_000, 22_000] {
        assert!(
            FeeSplitter::is_standard_fee(fee),
            "Fee {fee} VND must be recognized as standard wire fee"
        );
    }
}

#[test]
fn test_fee_split_journal_balance_invariant() {
    let config = MatchConfig::default();
    let net_bank = Money::vnd(49_989_000);
    let fee = Money::vnd(11_000);

    let journal = create_fee_split_journal(101, 1726358400, net_bank, fee, "HD102", &config)
        .expect("Journal creation must succeed");

    // Circular 200/2014/TT-BTC Verification:
    // Line 1: Debit 1121 (Net bank cash)
    assert_eq!(journal.lines[0].account_code, "1121");
    assert_eq!(journal.lines[0].posting_type, PostingType::Debit);
    assert_eq!(journal.lines[0].amount, net_bank);

    // Line 2: Debit 6425 (Bank wire fee expense)
    assert_eq!(journal.lines[1].account_code, "6425");
    assert_eq!(journal.lines[1].posting_type, PostingType::Debit);
    assert_eq!(journal.lines[1].amount, fee);

    // Line 3: Credit 131 (Customer receivable invoice cleared in full)
    assert_eq!(journal.lines[2].account_code, "131");
    assert_eq!(journal.lines[2].posting_type, PostingType::Credit);
    assert_eq!(journal.lines[2].amount, Money::vnd(50_000_000));

    // Must strictly satisfy Debit == Credit with 0 penny drift
    assert!(journal.verify_balance().is_ok());
}

#[test]
fn test_pure_fee_journal_balance_invariant() {
    let config = MatchConfig::default();
    let fee = Money::vnd(33_000);

    let journal = create_pure_fee_journal(
        102,
        1726358400,
        fee,
        "Phi duy tri tai khoan thang 08/2026",
        &config,
    )
    .expect("Pure fee journal creation must succeed");

    assert_eq!(journal.lines[0].account_code, "6425");
    assert_eq!(journal.lines[0].posting_type, PostingType::Debit);
    assert_eq!(journal.lines[0].amount, fee);

    assert_eq!(journal.lines[1].account_code, "1121");
    assert_eq!(journal.lines[1].posting_type, PostingType::Credit);
    assert_eq!(journal.lines[1].amount, fee);

    assert!(journal.verify_balance().is_ok());
}

#[test]
fn test_fee_splitter_in_reconciliation_engine() {
    let engine = ReconciliationEngine::with_default_config();

    let bank_txs = vec![BankTransaction {
        id: "tx_wire_01".to_string(),
        tx_date: 1726358400,
        value_date: None,
        doc_ref: None,
        direction: PostingType::Credit,
        amount: Money::vnd(49_989_000), // 50,000,000 - 11,000
        narration: "THANH TOAN TIEN HANG HD888 PHI CHUYEN TIEN 11000".to_string(),
        counterparty_name: Some("CONG TY TNHH MINH ANH".to_string()),
    }];

    let ledger_entries = vec![LedgerEntry {
        id: "led_inv_01".to_string(),
        doc_no: "HD888".to_string(),
        entry_date: 1726358400 + 1800,
        direction: PostingType::Credit,
        amount: Money::vnd(50_000_000), // Full invoice amount
        partner_name: Some("MINH ANH".to_string()),
        description: "Hoa don HD888".to_string(),
    }];

    let summary = engine.reconcile(&bank_txs, &ledger_entries);

    assert_eq!(summary.total_matched, 1);
    assert_eq!(summary.fee_split_matches, 1);
    assert_eq!(summary.matches[0].match_type, MatchType::FeeSplit);
    assert_eq!(summary.matches[0].matched_amount, Money::vnd(49_989_000));
    assert_eq!(summary.matches[0].fee_amount, Money::vnd(11_000));
    assert_eq!(
        summary.matches[0].discrepancy_amount,
        Money::vnd(11_000)
    );
}
