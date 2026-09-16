use liva_ledger::PostingType;
use liva_match::{BankTransaction, HashMatcher, LedgerEntry, MatchConfig, MatchType};
use liva_money::Money;

#[test]
fn test_tier1_exact_match_success() {
    let config = MatchConfig::default();

    let bank_txs = vec![BankTransaction {
        id: "tx_001".to_string(),
        tx_date: 1726358400,
        value_date: None,
        doc_ref: Some("HD-00102".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(50_000_000),
        narration: "THANH TOAN TIEN HANG HD-00102".to_string(),
        counterparty_name: Some("CONG TY ABC".to_string()),
    }];

    let ledger_entries = vec![LedgerEntry {
        id: "led_001".to_string(),
        doc_no: "HD102".to_string(),
        entry_date: 1726358400 + 3600, // 1 hour later
        direction: PostingType::Credit,
        amount: Money::vnd(50_000_000),
        partner_name: Some("CONG TY ABC".to_string()),
        description: "Hoa don ban hang HD102".to_string(),
    }];

    let (matches, unalloc_b, unalloc_l) =
        HashMatcher::match_tier1(&bank_txs, &ledger_entries, &config);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].match_type, MatchType::Exact1To1);
    assert_eq!(matches[0].bank_tx_ids, vec!["tx_001"]);
    assert_eq!(matches[0].ledger_entry_ids, vec!["led_001"]);
    assert_eq!(matches[0].confidence_score, 1.0);
    assert!(unalloc_b.is_empty());
    assert!(unalloc_l.is_empty());
}

#[test]
fn test_tier1_time_window_rejection() {
    let config = MatchConfig::default();

    let bank_txs = vec![BankTransaction {
        id: "tx_002".to_string(),
        tx_date: 1726358400,
        value_date: None,
        doc_ref: Some("HD105".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(10_000_000),
        narration: "HD105".to_string(),
        counterparty_name: None,
    }];

    // 25 hours later (exceeds 24h = 86,400s)
    let ledger_entries = vec![LedgerEntry {
        id: "led_002".to_string(),
        doc_no: "HD105".to_string(),
        entry_date: 1726358400 + 90_000,
        direction: PostingType::Credit,
        amount: Money::vnd(10_000_000),
        partner_name: None,
        description: "HD105".to_string(),
    }];

    let (matches, unalloc_b, unalloc_l) =
        HashMatcher::match_tier1(&bank_txs, &ledger_entries, &config);

    assert!(matches.is_empty(), "Match outside 24h must be rejected by Tier 1");
    assert_eq!(unalloc_b.len(), 1);
    assert_eq!(unalloc_l.len(), 1);
}

#[test]
fn test_tier1_direction_invariance() {
    let config = MatchConfig::default();

    // Bank TX is Credit (deposit / receipt)
    let bank_txs = vec![BankTransaction {
        id: "tx_003".to_string(),
        tx_date: 1726358400,
        value_date: None,
        doc_ref: Some("HD200".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(20_000_000),
        narration: "HD200".to_string(),
        counterparty_name: None,
    }];

    // Ledger entry is Debit (payment / withdrawal)
    let ledger_entries = vec![LedgerEntry {
        id: "led_003".to_string(),
        doc_no: "HD200".to_string(),
        entry_date: 1726358400,
        direction: PostingType::Debit,
        amount: Money::vnd(20_000_000),
        partner_name: None,
        description: "HD200".to_string(),
    }];

    let (matches, unalloc_b, unalloc_l) =
        HashMatcher::match_tier1(&bank_txs, &ledger_entries, &config);

    assert!(
        matches.is_empty(),
        "Credit transaction must NEVER match Debit ledger entry"
    );
    assert_eq!(unalloc_b.len(), 1);
    assert_eq!(unalloc_l.len(), 1);
}
