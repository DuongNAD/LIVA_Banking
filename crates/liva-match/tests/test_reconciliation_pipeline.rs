use liva_ledger::PostingType;
use liva_match::{
    BankTransaction, LedgerEntry, MatchConfig, MatchType, ReconciliationEngine,
};
use liva_money::Money;

#[test]
fn test_reconciliation_pipeline_mixed_batch() {
    let engine = ReconciliationEngine::new(MatchConfig::default());

    let bank_txs = vec![
        // 1. Tier 1 exact candidate: matches HD-101
        BankTransaction {
            id: "tx_1".to_string(),
            tx_date: 1726358400,
            value_date: None,
            doc_ref: Some("HD-101".to_string()),
            direction: PostingType::Credit,
            amount: Money::vnd(10_000_000),
            narration: "TT TIEN HANG HD-101".to_string(),
            counterparty_name: Some("CONG TY ABC".to_string()),
        },
        // 2. Tier 2 fee-split candidate: invoice is 20,000,000, received 19,989,000 (-11,000 fee)
        BankTransaction {
            id: "tx_2".to_string(),
            tx_date: 1726358400 + 3600,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(19_989_000),
            narration: "CHUYEN KHOAN CONG TY TNHH PHAT DAT".to_string(),
            counterparty_name: Some("PHAT DAT".to_string()),
        },
        // 3. Tier 2 fuzzy candidate: exact amount 30,000,000, no doc ref, party name similarity
        BankTransaction {
            id: "tx_3".to_string(),
            tx_date: 1726358400 + 7200,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(30_000_000),
            narration: "CONG TY CP HOANG MAI THANH TOAN".to_string(),
            counterparty_name: Some("HOANG MAI".to_string()),
        },
        // 4. Unmatched transaction: random amount
        BankTransaction {
            id: "tx_4".to_string(),
            tx_date: 1726358400 + 10800,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(7_777_777),
            narration: "TIEN LAI TIET KIEM".to_string(),
            counterparty_name: None,
        },
    ];

    let ledger_entries = vec![
        LedgerEntry {
            id: "led_1".to_string(),
            doc_no: "HD101".to_string(),
            entry_date: 1726358400 + 600,
            direction: PostingType::Credit,
            amount: Money::vnd(10_000_000),
            partner_name: Some("CONG TY ABC".to_string()),
            description: "HD101".to_string(),
        },
        LedgerEntry {
            id: "led_2".to_string(),
            doc_no: "HD102".to_string(),
            entry_date: 1726358400 + 4000,
            direction: PostingType::Credit,
            amount: Money::vnd(20_000_000),
            partner_name: Some("CONG TY CO PHAN PHAT DAT".to_string()),
            description: "HD102".to_string(),
        },
        LedgerEntry {
            id: "led_3".to_string(),
            doc_no: "HD103".to_string(),
            entry_date: 1726358400 + 7500,
            direction: PostingType::Credit,
            amount: Money::vnd(30_000_000),
            partner_name: Some("HOANG MAI".to_string()),
            description: "HD103".to_string(),
        },
        LedgerEntry {
            id: "led_4".to_string(),
            doc_no: "HD104".to_string(),
            entry_date: 1726358400 + 15000,
            direction: PostingType::Credit,
            amount: Money::vnd(99_000_000),
            partner_name: Some("CONG TY XYZ".to_string()),
            description: "HD104".to_string(),
        },
    ];

    let summary = engine.reconcile(&bank_txs, &ledger_entries);

    assert_eq!(summary.total_bank_txs, 4);
    assert_eq!(summary.total_ledger_entries, 4);
    assert_eq!(summary.total_matched, 3);
    assert_eq!(summary.tier1_matches, 1);
    assert_eq!(summary.fee_split_matches, 1);
    assert_eq!(summary.tier2_matches, 1);
    assert_eq!(summary.unallocated_bank_count, 1);
    assert_eq!(summary.unallocated_ledger_count, 1);

    // Verify match types
    let match_types: Vec<MatchType> = summary.matches.iter().map(|m| m.match_type).collect();
    assert!(match_types.contains(&MatchType::Exact1To1));
    assert!(match_types.contains(&MatchType::FeeSplit));
    assert!(match_types.contains(&MatchType::FuzzyHeuristic));
}
