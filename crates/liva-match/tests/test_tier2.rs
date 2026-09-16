use liva_ledger::PostingType;
use liva_match::{
    compare_party_names, BankTransaction, FuzzyMatcher, LedgerEntry, MatchConfig, MatchType,
};
use liva_money::Money;

#[test]
fn test_party_name_comparison_vietnamese_diacritics() {
    let sim1 = compare_party_names("CÔNG TY TNHH AN PHÁT", "AN PHAT");
    assert!(
        sim1 >= 0.85,
        "Diacritics and legal prefix should match with score >= 0.85, got {sim1}"
    );

    let sim2 = compare_party_names(
        "NAPAS VIETQR TT CONG TY CO PHAN CONG NGHE SAO MAI",
        "SAO MAI",
    );
    assert!(
        sim2 >= 0.85,
        "Bank boilerplate + corporate prefix stripped should score >= 0.85, got {sim2}"
    );
}

#[test]
fn test_legal_noise_prevention_false_positive() {
    // Both share "CONG TY TNHH" but have completely different core names
    let sim = compare_party_names("CONG TY TNHH MINH ANH", "CONG TY TNHH PHUONG DONG");
    assert!(
        sim < 0.70,
        "Names sharing only corporate noise must score < 0.70, got {sim}"
    );
}

#[test]
fn test_tier2_fuzzy_match_success() {
    let config = MatchConfig::default();

    let bank_txs = vec![BankTransaction {
        id: "tx_f1".to_string(),
        tx_date: 1726358400,
        value_date: None,
        doc_ref: None, // No doc ref
        direction: PostingType::Credit,
        amount: Money::vnd(25_000_000),
        narration: "NAPAS247 CHUYEN KHOAN TU CONG TY TNHH DAI NAM".to_string(),
        counterparty_name: Some("DAI NAM".to_string()),
    }];

    let ledger_entries = vec![LedgerEntry {
        id: "led_f1".to_string(),
        doc_no: "INV999".to_string(),
        entry_date: 1726358400 + 7200, // 2 hours later
        direction: PostingType::Credit,
        amount: Money::vnd(25_000_000),
        partner_name: Some("CONG TY CO PHAN DAI NAM".to_string()),
        description: "Thu tien ban hang DAI NAM".to_string(),
    }];

    let unalloc_b = vec![0];
    let unalloc_l = vec![0];

    let (matches, final_b, final_l) =
        FuzzyMatcher::match_tier2(&bank_txs, &ledger_entries, &unalloc_b, &unalloc_l, &config);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].match_type, MatchType::FuzzyHeuristic);
    assert!(matches[0].confidence_score >= 0.85);
    assert!(final_b.is_empty());
    assert!(final_l.is_empty());
}

#[test]
fn test_tier2_time_window_exceeded_72h() {
    let config = MatchConfig::default();

    let bank_txs = vec![BankTransaction {
        id: "tx_f2".to_string(),
        tx_date: 1726358400,
        value_date: None,
        doc_ref: None,
        direction: PostingType::Credit,
        amount: Money::vnd(15_000_000),
        narration: "CONG TY TNHH DAI NAM".to_string(),
        counterparty_name: Some("DAI NAM".to_string()),
    }];

    // 73 hours later (exceeds 72h = 259,200s)
    let ledger_entries = vec![LedgerEntry {
        id: "led_f2".to_string(),
        doc_no: "INV999".to_string(),
        entry_date: 1726358400 + 265_000,
        direction: PostingType::Credit,
        amount: Money::vnd(15_000_000),
        partner_name: Some("DAI NAM".to_string()),
        description: "Thu tien DAI NAM".to_string(),
    }];

    let unalloc_b = vec![0];
    let unalloc_l = vec![0];

    let (matches, final_b, final_l) =
        FuzzyMatcher::match_tier2(&bank_txs, &ledger_entries, &unalloc_b, &unalloc_l, &config);

    assert!(matches.is_empty(), "Match outside 72h must be rejected by Tier 2");
    assert_eq!(final_b.len(), 1);
    assert_eq!(final_l.len(), 1);
}
