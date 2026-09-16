//! Adversarial Challenge Test Suite for Milestone M1 - Crates `liva-match`
//!
//! Rigorously verifies:
//! 1. Tier 1 Exact Matcher:
//!    - Exact ±86,400s (24h) boundary (pass) vs ±86,401s (fail)
//!    - Direction mismatch rejection (Credit bank vs Debit ledger, Debit bank vs Credit ledger)
//!    - Doc ref canonicalization permutations (HD00102, HD-102, INV/2026/00102, INV-2026-00102)
//! 2. Tier 2 Fuzzy Matcher & Fee Splitter:
//!    - Standard wire fee tariff boundaries: 1,080, 1,100, 21,600, 22,000 VND (accepted) vs 22,001 VND (rejected)
//!    - Double-entry balance invariant on fee splitting: verify_balance() succeeds and accounts are 1121, 6425, 131 per Circular 200
//!    - Party names with extreme noise or zero token overlap: verify score is capped < 0.70

use liva_ledger::PostingType;
use liva_match::{
    compare_party_names, create_fee_split_journal, create_pure_fee_journal, normalize_doc_ref,
    BankTransaction, FeeSplitter, FuzzyMatcher, HashMatcher, LedgerEntry, MatchConfig, MatchType,
    ReconciliationEngine, KNOWN_VIETNAMESE_WIRE_FEES,
};
use liva_money::{Currency, Money};

// =========================================================================
// SECTION 1: TIER 1 EXACT MATCHER ADVERSARIAL CHALLENGES
// =========================================================================

#[test]
fn challenge_tier1_time_window_exact_boundary_pass() {
    let config = MatchConfig::default(); // tier1_window_secs = 86_400 (24 hours)
    let t_base = 1726358400i64;

    // Positive boundary: t_bank - t_ledger = +86,400s (exactly 24h) -> MUST PASS
    let bank_pos = vec![BankTransaction {
        id: "tx_pos_86400".to_string(),
        tx_date: t_base + 86_400,
        value_date: None,
        doc_ref: Some("HD-1001".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(15_000_000),
        narration: "HD-1001".to_string(),
        counterparty_name: None,
    }];
    let ledger_pos = vec![LedgerEntry {
        id: "led_pos_86400".to_string(),
        doc_no: "HD1001".to_string(),
        entry_date: t_base,
        direction: PostingType::Credit,
        amount: Money::vnd(15_000_000),
        partner_name: None,
        description: "HD1001".to_string(),
    }];

    let (matches, unalloc_b, unalloc_l) =
        HashMatcher::match_tier1(&bank_pos, &ledger_pos, &config);
    assert_eq!(
        matches.len(),
        1,
        "Exact boundary +86,400s must PASS Tier 1 exact match"
    );
    assert!(unalloc_b.is_empty());
    assert!(unalloc_l.is_empty());

    // Negative boundary: t_bank - t_ledger = -86,400s (bank transaction occurred 24h before ledger) -> MUST PASS
    let bank_neg = vec![BankTransaction {
        id: "tx_neg_86400".to_string(),
        tx_date: t_base,
        value_date: None,
        doc_ref: Some("HD-1002".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(15_000_000),
        narration: "HD-1002".to_string(),
        counterparty_name: None,
    }];
    let ledger_neg = vec![LedgerEntry {
        id: "led_neg_86400".to_string(),
        doc_no: "HD1002".to_string(),
        entry_date: t_base + 86_400,
        direction: PostingType::Credit,
        amount: Money::vnd(15_000_000),
        partner_name: None,
        description: "HD1002".to_string(),
    }];

    let (matches_neg, unalloc_b_neg, unalloc_l_neg) =
        HashMatcher::match_tier1(&bank_neg, &ledger_neg, &config);
    assert_eq!(
        matches_neg.len(),
        1,
        "Exact boundary -86,400s must PASS Tier 1 exact match"
    );
    assert!(unalloc_b_neg.is_empty());
    assert!(unalloc_l_neg.is_empty());
}

#[test]
fn challenge_tier1_time_window_boundary_fail() {
    let config = MatchConfig::default(); // tier1_window_secs = 86_400
    let t_base = 1726358400i64;

    // Positive boundary + 1s: t_bank - t_ledger = 86,401s -> MUST FAIL
    let bank_pos = vec![BankTransaction {
        id: "tx_pos_86401".to_string(),
        tx_date: t_base + 86_401,
        value_date: None,
        doc_ref: Some("HD-2001".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(20_000_000),
        narration: "HD-2001".to_string(),
        counterparty_name: None,
    }];
    let ledger_pos = vec![LedgerEntry {
        id: "led_pos_86401".to_string(),
        doc_no: "HD2001".to_string(),
        entry_date: t_base,
        direction: PostingType::Credit,
        amount: Money::vnd(20_000_000),
        partner_name: None,
        description: "HD2001".to_string(),
    }];

    let (matches, unalloc_b, unalloc_l) =
        HashMatcher::match_tier1(&bank_pos, &ledger_pos, &config);
    assert_eq!(
        matches.len(),
        0,
        "Boundary +86,401s (24h + 1s) MUST BE REJECTED by Tier 1"
    );
    assert_eq!(unalloc_b.len(), 1);
    assert_eq!(unalloc_l.len(), 1);

    // Negative boundary - 1s: t_bank - t_ledger = -86,401s -> MUST FAIL
    let bank_neg = vec![BankTransaction {
        id: "tx_neg_86401".to_string(),
        tx_date: t_base,
        value_date: None,
        doc_ref: Some("HD-2002".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(20_000_000),
        narration: "HD-2002".to_string(),
        counterparty_name: None,
    }];
    let ledger_neg = vec![LedgerEntry {
        id: "led_neg_86401".to_string(),
        doc_no: "HD2002".to_string(),
        entry_date: t_base + 86_401,
        direction: PostingType::Credit,
        amount: Money::vnd(20_000_000),
        partner_name: None,
        description: "HD2002".to_string(),
    }];

    let (matches_neg, unalloc_b_neg, unalloc_l_neg) =
        HashMatcher::match_tier1(&bank_neg, &ledger_neg, &config);
    assert_eq!(
        matches_neg.len(),
        0,
        "Boundary -86,401s (-24h - 1s) MUST BE REJECTED by Tier 1"
    );
    assert_eq!(unalloc_b_neg.len(), 1);
    assert_eq!(unalloc_l_neg.len(), 1);
}

#[test]
fn challenge_tier1_direction_mismatch_strict_rejection() {
    let config = MatchConfig::default();
    let t_base = 1726358400i64;

    // Case A: Bank Credit (receipt) vs Ledger Debit (disbursement)
    let bank_a = vec![BankTransaction {
        id: "tx_credit".to_string(),
        tx_date: t_base,
        value_date: None,
        doc_ref: Some("HD-3001".to_string()),
        direction: PostingType::Credit,
        amount: Money::vnd(10_000_000),
        narration: "HD-3001".to_string(),
        counterparty_name: None,
    }];
    let ledger_a = vec![LedgerEntry {
        id: "led_debit".to_string(),
        doc_no: "HD3001".to_string(),
        entry_date: t_base,
        direction: PostingType::Debit,
        amount: Money::vnd(10_000_000),
        partner_name: None,
        description: "HD3001".to_string(),
    }];

    let (matches_a, unalloc_b_a, unalloc_l_a) =
        HashMatcher::match_tier1(&bank_a, &ledger_a, &config);
    assert!(
        matches_a.is_empty(),
        "Bank Credit vs Ledger Debit must NEVER match"
    );
    assert_eq!(unalloc_b_a.len(), 1);
    assert_eq!(unalloc_l_a.len(), 1);

    // Case B: Bank Debit (disbursement) vs Ledger Credit (receipt)
    let bank_b = vec![BankTransaction {
        id: "tx_debit".to_string(),
        tx_date: t_base,
        value_date: None,
        doc_ref: Some("HD-3002".to_string()),
        direction: PostingType::Debit,
        amount: Money::vnd(10_000_000),
        narration: "HD-3002".to_string(),
        counterparty_name: None,
    }];
    let ledger_b = vec![LedgerEntry {
        id: "led_credit".to_string(),
        doc_no: "HD3002".to_string(),
        entry_date: t_base,
        direction: PostingType::Credit,
        amount: Money::vnd(10_000_000),
        partner_name: None,
        description: "HD3002".to_string(),
    }];

    let (matches_b, unalloc_b_b, unalloc_l_b) =
        HashMatcher::match_tier1(&bank_b, &ledger_b, &config);
    assert!(
        matches_b.is_empty(),
        "Bank Debit vs Ledger Credit must NEVER match"
    );
    assert_eq!(unalloc_b_b.len(), 1);
    assert_eq!(unalloc_l_b.len(), 1);
}

#[test]
fn challenge_tier1_doc_ref_permutations() {
    let config = MatchConfig::default();
    let t_base = 1726358400i64;

    // Test permutations:
    // 1. HD00102 vs HD-102
    assert_eq!(normalize_doc_ref("HD00102"), "HD102");
    assert_eq!(normalize_doc_ref("HD-102"), "HD102");
    assert_eq!(normalize_doc_ref("HD00102"), normalize_doc_ref("HD-102"));

    // 2. INV/2026/00102 vs INV-2026-00102 vs INV202600102
    assert_eq!(normalize_doc_ref("INV/2026/00102"), "INV202600102");
    assert_eq!(normalize_doc_ref("INV-2026-00102"), "INV202600102");
    assert_eq!(normalize_doc_ref("INV202600102"), "INV202600102");

    // Integration test with BankTransaction and LedgerEntry for HD00102 and HD-102
    let bank_txs = vec![
        BankTransaction {
            id: "tx_hd102".to_string(),
            tx_date: t_base,
            value_date: None,
            doc_ref: Some("HD-102".to_string()),
            direction: PostingType::Credit,
            amount: Money::vnd(30_000_000),
            narration: "THANH TOAN THEO HD-102".to_string(),
            counterparty_name: None,
        },
        BankTransaction {
            id: "tx_inv102".to_string(),
            tx_date: t_base + 100,
            value_date: None,
            doc_ref: None, // Discovered from narration
            direction: PostingType::Credit,
            amount: Money::vnd(45_000_000),
            narration: "CTY CONG NGHE TT INV/2026/00102 NGAY 15/09".to_string(),
            counterparty_name: None,
        },
    ];

    let ledger_entries = vec![
        LedgerEntry {
            id: "led_hd00102".to_string(),
            doc_no: "HD00102".to_string(), // Formatted with leading zeros
            entry_date: t_base + 300,
            direction: PostingType::Credit,
            amount: Money::vnd(30_000_000),
            partner_name: None,
            description: "HD00102".to_string(),
        },
        LedgerEntry {
            id: "led_inv00102".to_string(),
            doc_no: "INV/2026/00102".to_string(), // Formatted with slashes
            entry_date: t_base + 400,
            direction: PostingType::Credit,
            amount: Money::vnd(45_000_000),
            partner_name: None,
            description: "INV/2026/00102".to_string(),
        },
    ];

    let (matches, unalloc_b, unalloc_l) =
        HashMatcher::match_tier1(&bank_txs, &ledger_entries, &config);
    assert_eq!(matches.len(), 2, "Both doc ref permutations must match 1:1");
    assert!(unalloc_b.is_empty());
    assert!(unalloc_l.is_empty());

    let match_hd = matches
        .iter()
        .find(|m| m.bank_tx_ids.contains(&"tx_hd102".to_string()))
        .expect("HD match must exist");
    assert_eq!(match_hd.ledger_entry_ids, vec!["led_hd00102"]);

    let match_inv = matches
        .iter()
        .find(|m| m.bank_tx_ids.contains(&"tx_inv102".to_string()))
        .expect("INV match must exist");
    assert_eq!(match_inv.ledger_entry_ids, vec!["led_inv00102"]);
}

// =========================================================================
// SECTION 2: TIER 2 FUZZY MATCHER & FEE SPLITTER CHALLENGES
// =========================================================================

#[test]
fn challenge_wire_fee_tariff_boundaries() {
    // Standard Vietnamese wire transfer fee schedule verification:
    // 1,080 VND (Napas 1,000 + 8% VAT) -> ACCEPT
    // 1,100 VND (Napas 1,000 + 10% VAT) -> ACCEPT
    // 21,600 VND (Over counter 20,000 + 8% VAT) -> ACCEPT
    // 22,000 VND (Over counter 20,000 + 10% VAT) -> ACCEPT
    // 22,001 VND -> REJECT (> 22,000 max tolerance and not standard fee)
    // 1,079 VND -> REJECT (< 1,100 min tolerance and not standard fee)

    assert!(FeeSplitter::is_standard_fee(1_080));
    assert!(FeeSplitter::is_standard_fee(1_100));
    assert!(FeeSplitter::is_standard_fee(21_600));
    assert!(FeeSplitter::is_standard_fee(22_000));
    assert!(!FeeSplitter::is_standard_fee(22_001));
    assert!(!FeeSplitter::is_standard_fee(1_079));

    let config = MatchConfig::default();
    let t_base = 1726358400i64;
    let inv_amount = 50_000_000i64;

    // Helper to test fuzzy match with fee difference
    let test_fee_reconciliation = |fee_diff: i64, expected_match: bool| {
        let bank = vec![BankTransaction {
            id: format!("tx_fee_{fee_diff}"),
            tx_date: t_base,
            value_date: None,
            doc_ref: None, // Forces Tier 2 fuzzy search
            direction: PostingType::Credit,
            amount: Money::vnd(inv_amount - fee_diff),
            narration: "CONG TY TNHH PHU MY THANH TOAN TIEN HANG".to_string(),
            counterparty_name: Some("PHU MY".to_string()),
        }];
        let ledger = vec![LedgerEntry {
            id: format!("led_fee_{fee_diff}"),
            doc_no: "INV_UNREF".to_string(),
            entry_date: t_base + 3600,
            direction: PostingType::Credit,
            amount: Money::vnd(inv_amount),
            partner_name: Some("CONG TY CP PHU MY".to_string()),
            description: "INV_UNREF".to_string(),
        }];

        let unalloc_b = vec![0];
        let unalloc_l = vec![0];
        let (matches, _, _) =
            FuzzyMatcher::match_tier2(&bank, &ledger, &unalloc_b, &unalloc_l, &config);

        if expected_match {
            assert_eq!(
                matches.len(),
                1,
                "Fee diff of {fee_diff} VND must be accepted"
            );
            assert_eq!(matches[0].match_type, MatchType::FeeSplit);
            assert_eq!(matches[0].fee_amount, Money::vnd(fee_diff));
        } else {
            assert_eq!(
                matches.len(),
                0,
                "Fee diff of {fee_diff} VND must be REJECTED"
            );
        }
    };

    // Accepted boundaries:
    test_fee_reconciliation(1_080, true);
    test_fee_reconciliation(1_100, true);
    test_fee_reconciliation(21_600, true);
    test_fee_reconciliation(22_000, true);

    // Rejected boundaries:
    test_fee_reconciliation(22_001, false);
    test_fee_reconciliation(1_079, false);
    test_fee_reconciliation(25_000, false);
}

#[test]
fn challenge_double_entry_balance_invariant_circular_200() {
    let config = MatchConfig::default();

    // Adversarial sweep across all recognized standard fees and diverse invoice principal scales
    let test_principals = [
        100_000i64,
        1_000_000,
        10_000_000,
        50_000_000,
        500_000_000,
        10_000_000_000,
        1_000_000_000_000,
    ];

    for &principal in &test_principals {
        for &fee in KNOWN_VIETNAMESE_WIRE_FEES {
            let net_bank = Money::vnd(principal.saturating_sub(fee));
            let fee_money = Money::vnd(fee);

            let journal = create_fee_split_journal(
                12345,
                1726358400,
                net_bank,
                fee_money,
                "INV-ADVERSARIAL",
                &config,
            )
            .expect("Fee split journal creation must succeed for valid positive amounts");

            // 1. Invariant: verify_balance() must succeed with zero drift
            assert!(
                journal.verify_balance().is_ok(),
                "Balance verification failed for principal={principal}, fee={fee}"
            );

            // 2. Exactly 3 lines per Circular 200/2014/TT-BTC
            assert_eq!(journal.lines.len(), 3);

            // Line 0: Debit TK 1121 (Bank Cash)
            assert_eq!(journal.lines[0].account_code, "1121");
            assert_eq!(journal.lines[0].posting_type, PostingType::Debit);
            assert_eq!(journal.lines[0].amount, net_bank);

            // Line 1: Debit TK 6425 (Wire Transfer Fee Expense)
            assert_eq!(journal.lines[1].account_code, "6425");
            assert_eq!(journal.lines[1].posting_type, PostingType::Debit);
            assert_eq!(journal.lines[1].amount, fee_money);

            // Line 2: Credit TK 131 (Receivable from customer cleared in full)
            assert_eq!(journal.lines[2].account_code, "131");
            assert_eq!(journal.lines[2].posting_type, PostingType::Credit);
            assert_eq!(journal.lines[2].amount, Money::vnd(principal));

            // Explicit debit sum == credit sum check
            let debit_sum = journal.lines[0]
                .amount
                .checked_add(journal.lines[1].amount)
                .unwrap();
            let credit_sum = journal.lines[2].amount;
            assert_eq!(debit_sum, credit_sum);
        }
    }

    // Pure fee journal invariant test
    for &fee in &[33_000i64, 55_000, 110_000, 220_000] {
        let fee_money = Money::vnd(fee);
        let pure_journal = create_pure_fee_journal(
            999,
            1726358400,
            fee_money,
            "Phi quan ly tai khoan",
            &config,
        )
        .expect("Pure fee journal creation must succeed");

        assert!(pure_journal.verify_balance().is_ok());
        assert_eq!(pure_journal.lines.len(), 2);
        assert_eq!(pure_journal.lines[0].account_code, "6425");
        assert_eq!(pure_journal.lines[0].posting_type, PostingType::Debit);
        assert_eq!(pure_journal.lines[1].account_code, "1121");
        assert_eq!(pure_journal.lines[1].posting_type, PostingType::Credit);
        assert_eq!(pure_journal.lines[0].amount, pure_journal.lines[1].amount);
    }
}

#[test]
fn challenge_party_names_noise_and_zero_overlap_cap() {
    // 1. Zero token overlap: must be capped < 0.70 (specifically <= 0.65)
    let pairs_zero_overlap = [
        ("CONG TY TNHH MINH ANH", "CONG TY TNHH PHUONG DONG"),
        ("TAP DOAN HOA PHAT", "CONG TY CP COTECCONS"),
        ("CONG TY DUOC PHAM HAU GIANG", "CONG TY CO PHAN FPT"),
        ("CONG TY TNHH ABC", "CONG TY TNHH XYZ"),
        ("VINAMILK", "PETROVIETNAM"),
    ];

    for (name1, name2) in pairs_zero_overlap {
        let sim = compare_party_names(name1, name2);
        assert!(
            sim < 0.70,
            "Zero token overlap for '{name1}' vs '{name2}' must be strictly < 0.70, got {sim}"
        );
    }

    // 2. Extreme noise in bank narration:
    let noisy_narrations = [
        "NAPAS247 CHUYEN TIEN DEN TK 01234567890123 FT260915123456 TT TIEN HANG",
        "IBVCB.09152026.123456.CK TIEN HANG ND GD 987654321",
        "QRIBFT VIETQR TT DEN TK 1029384756 TAI NGAN HANG MBBANK",
        "ASDFGHJKL QWERTYUIOP ZXCVBNM 1234567890",
    ];
    let legit_partner = "CONG TY CO PHAN BAO MINH";

    for noise in &noisy_narrations {
        let sim = compare_party_names(legit_partner, noise);
        assert!(
            sim < 0.70,
            "Legitimate partner vs extreme noise '{noise}' must score < 0.70, got {sim}"
        );
    }

    // 3. Complete mismatch with garbage strings
    let sim_garbage = compare_party_names("ZZZZZZZZ", "AAAAAAAA");
    assert!(
        sim_garbage < 0.70,
        "Completely mismatched letters must score < 0.70, got {sim_garbage}"
    );

    // 4. Empty strings
    assert_eq!(compare_party_names("", "CONG TY ABC"), 0.0);
    assert_eq!(compare_party_names("CONG TY ABC", ""), 0.0);
    assert_eq!(compare_party_names("", ""), 1.0);
}

#[test]
fn challenge_fuzzy_matcher_rejects_zero_token_overlap_even_with_close_amount() {
    let config = MatchConfig::default();
    let t_base = 1726358400i64;

    // Both have exact same amount, but zero party overlap (MINH ANH vs PHUONG DONG)
    // Must NOT match under Tier 2
    let bank = vec![BankTransaction {
        id: "tx_zero_overlap".to_string(),
        tx_date: t_base,
        value_date: None,
        doc_ref: None,
        direction: PostingType::Credit,
        amount: Money::vnd(35_000_000),
        narration: "CONG TY TNHH MINH ANH THANH TOAN".to_string(),
        counterparty_name: Some("CONG TY TNHH MINH ANH".to_string()),
    }];
    let ledger = vec![LedgerEntry {
        id: "led_zero_overlap".to_string(),
        doc_no: "DOC_RANDOM".to_string(),
        entry_date: t_base + 3600,
        direction: PostingType::Credit,
        amount: Money::vnd(35_000_000),
        partner_name: Some("CONG TY TNHH PHUONG DONG".to_string()),
        description: "Thu tien ban hang".to_string(),
    }];

    let unalloc_b = vec![0];
    let unalloc_l = vec![0];
    let (matches, final_b, final_l) =
        FuzzyMatcher::match_tier2(&bank, &ledger, &unalloc_b, &unalloc_l, &config);

    assert!(
        matches.is_empty(),
        "Tier 2 MUST NOT match entries with zero party overlap (< 0.70) despite exact amount"
    );
    assert_eq!(final_b.len(), 1);
    assert_eq!(final_l.len(), 1);
}

#[test]
fn challenge_reconciliation_pipeline_stress_batch() {
    let engine = ReconciliationEngine::with_default_config();
    let t_base = 1726358400i64;

    // 10 transactions with mixed scenarios:
    // 0: Tier 1 Exact match (HD-501 vs HD00501)
    // 1: Tier 1 Exact match with boundary 86,400s
    // 2: Tier 1 Rejected due to boundary 86,401s (falls through to Tier 2 if party matches, or stays unallocated)
    // 3: Tier 1 Rejected due to Direction Mismatch (Credit bank vs Debit ledger)
    // 4: Tier 2 Fuzzy match with standard wire fee (11,000 VND)
    // 5: Tier 2 Fuzzy match with standard wire fee (1,080 VND)
    // 6: Tier 2 Fuzzy match with over-the-counter wire fee (22,000 VND)
    // 7: Tier 2 Rejected fee (22,001 VND)
    // 8: Tier 2 Rejected due to zero party overlap
    // 9: Completely unmatched random transaction

    let bank_txs = vec![
        // 0: Tier 1
        BankTransaction {
            id: "tx_0".to_string(),
            tx_date: t_base,
            value_date: None,
            doc_ref: Some("HD-501".to_string()),
            direction: PostingType::Credit,
            amount: Money::vnd(12_000_000),
            narration: "HD-501".to_string(),
            counterparty_name: None,
        },
        // 1: Tier 1 boundary pass (86,400s)
        BankTransaction {
            id: "tx_1".to_string(),
            tx_date: t_base + 86_400,
            value_date: None,
            doc_ref: Some("INV/2026/00701".to_string()),
            direction: PostingType::Credit,
            amount: Money::vnd(24_000_000),
            narration: "INV/2026/00701".to_string(),
            counterparty_name: None,
        },
        // 2: Tier 1 boundary fail (86,401s), but within Tier 2 (72h) window with party match
        BankTransaction {
            id: "tx_2".to_string(),
            tx_date: t_base + 86_401,
            value_date: None,
            doc_ref: Some("HD-801".to_string()),
            direction: PostingType::Credit,
            amount: Money::vnd(18_000_000),
            narration: "CONG TY TNHH PHU THAI".to_string(),
            counterparty_name: Some("PHU THAI".to_string()),
        },
        // 3: Direction mismatch
        BankTransaction {
            id: "tx_3".to_string(),
            tx_date: t_base,
            value_date: None,
            doc_ref: Some("HD-901".to_string()),
            direction: PostingType::Credit,
            amount: Money::vnd(10_000_000),
            narration: "HD-901".to_string(),
            counterparty_name: None,
        },
        // 4: Fee Split (11,000 VND)
        BankTransaction {
            id: "tx_4".to_string(),
            tx_date: t_base + 3600,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(49_989_000),
            narration: "CONG TY CO PHAN AN PHAT".to_string(),
            counterparty_name: Some("AN PHAT".to_string()),
        },
        // 5: Fee Split (1,080 VND)
        BankTransaction {
            id: "tx_5".to_string(),
            tx_date: t_base + 7200,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(9_998_920), // 10,000,000 - 1,080
            narration: "CONG TY TNHH TIEN DAT".to_string(),
            counterparty_name: Some("TIEN DAT".to_string()),
        },
        // 6: Fee Split (22,000 VND)
        BankTransaction {
            id: "tx_6".to_string(),
            tx_date: t_base + 10800,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(99_978_000), // 100,000,000 - 22,000
            narration: "CONG TY CP THIEN LONG".to_string(),
            counterparty_name: Some("THIEN LONG".to_string()),
        },
        // 7: Rejected Fee (22,001 VND diff)
        BankTransaction {
            id: "tx_7".to_string(),
            tx_date: t_base + 14400,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(49_977_999), // 50,000,000 - 22,001
            narration: "CONG TY TNHH VAN XUAN".to_string(),
            counterparty_name: Some("VAN XUAN".to_string()),
        },
        // 8: Zero party overlap
        BankTransaction {
            id: "tx_8".to_string(),
            tx_date: t_base + 18000,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(30_000_000),
            narration: "CONG TY TNHH MINH ANH".to_string(),
            counterparty_name: Some("MINH ANH".to_string()),
        },
        // 9: Random unmatched
        BankTransaction {
            id: "tx_9".to_string(),
            tx_date: t_base + 21600,
            value_date: None,
            doc_ref: None,
            direction: PostingType::Credit,
            amount: Money::vnd(1_234_567),
            narration: "TIEN LAI TIET KIEM".to_string(),
            counterparty_name: None,
        },
    ];

    let ledger_entries = vec![
        // 0: Matches tx_0 via Tier 1
        LedgerEntry {
            id: "led_0".to_string(),
            doc_no: "HD00501".to_string(),
            entry_date: t_base + 600,
            direction: PostingType::Credit,
            amount: Money::vnd(12_000_000),
            partner_name: None,
            description: "HD00501".to_string(),
        },
        // 1: Matches tx_1 via Tier 1 (boundary pass)
        LedgerEntry {
            id: "led_1".to_string(),
            doc_no: "INV/2026/00701".to_string(),
            entry_date: t_base,
            direction: PostingType::Credit,
            amount: Money::vnd(24_000_000),
            partner_name: None,
            description: "INV/2026/00701".to_string(),
        },
        // 2: Matches tx_2 via Tier 2 (since Tier 1 rejected 86,401s, but Tier 2 72h passes)
        LedgerEntry {
            id: "led_2".to_string(),
            doc_no: "HD801".to_string(),
            entry_date: t_base,
            direction: PostingType::Credit,
            amount: Money::vnd(18_000_000),
            partner_name: Some("CONG TY TNHH PHU THAI".to_string()),
            description: "HD801".to_string(),
        },
        // 3: Debit ledger, must NOT match tx_3
        LedgerEntry {
            id: "led_3".to_string(),
            doc_no: "HD901".to_string(),
            entry_date: t_base,
            direction: PostingType::Debit,
            amount: Money::vnd(10_000_000),
            partner_name: None,
            description: "HD901".to_string(),
        },
        // 4: Matches tx_4 via FeeSplit (11,000 VND)
        LedgerEntry {
            id: "led_4".to_string(),
            doc_no: "INV_4".to_string(),
            entry_date: t_base + 3600,
            direction: PostingType::Credit,
            amount: Money::vnd(50_000_000),
            partner_name: Some("AN PHAT".to_string()),
            description: "INV_4".to_string(),
        },
        // 5: Matches tx_5 via FeeSplit (1,080 VND)
        LedgerEntry {
            id: "led_5".to_string(),
            doc_no: "INV_5".to_string(),
            entry_date: t_base + 7200,
            direction: PostingType::Credit,
            amount: Money::vnd(10_000_000),
            partner_name: Some("TIEN DAT".to_string()),
            description: "INV_5".to_string(),
        },
        // 6: Matches tx_6 via FeeSplit (22,000 VND)
        LedgerEntry {
            id: "led_6".to_string(),
            doc_no: "INV_6".to_string(),
            entry_date: t_base + 10800,
            direction: PostingType::Credit,
            amount: Money::vnd(100_000_000),
            partner_name: Some("THIEN LONG".to_string()),
            description: "INV_6".to_string(),
        },
        // 7: 50M invoice - should NOT match tx_7 (22,001 diff)
        LedgerEntry {
            id: "led_7".to_string(),
            doc_no: "INV_7".to_string(),
            entry_date: t_base + 14400,
            direction: PostingType::Credit,
            amount: Money::vnd(50_000_000),
            partner_name: Some("VAN XUAN".to_string()),
            description: "INV_7".to_string(),
        },
        // 8: 30M invoice - should NOT match tx_8 (PHUONG DONG vs MINH ANH)
        LedgerEntry {
            id: "led_8".to_string(),
            doc_no: "INV_8".to_string(),
            entry_date: t_base + 18000,
            direction: PostingType::Credit,
            amount: Money::vnd(30_000_000),
            partner_name: Some("CONG TY TNHH PHUONG DONG".to_string()),
            description: "INV_8".to_string(),
        },
    ];

    let summary = engine.reconcile(&bank_txs, &ledger_entries);

    assert_eq!(summary.total_bank_txs, 10);
    assert_eq!(summary.total_ledger_entries, 9);

    // Expected matches:
    // tx_0 (Tier 1 exact)
    // tx_1 (Tier 1 boundary exact)
    // tx_2 (Tier 2 fuzzy, since Tier 1 rejected boundary)
    // tx_4 (Fee split 11k)
    // tx_5 (Fee split 1,080)
    // tx_6 (Fee split 22k)
    // Total matches: 6
    assert_eq!(summary.total_matched, 6);
    assert_eq!(summary.tier1_matches, 2);
    assert_eq!(summary.tier2_matches, 1);
    assert_eq!(summary.fee_split_matches, 3);
    assert_eq!(summary.unallocated_bank_count, 4); // tx_3, tx_7, tx_8, tx_9
    assert_eq!(summary.unallocated_ledger_count, 3); // led_3, led_7, led_8
}

#[test]
fn challenge_tier1_currency_mismatch_cross_contamination() {
    let config = MatchConfig::default();
    let t_base = 1726358400i64;

    // Bank transaction is 50,000,000 USD
    let bank = vec![BankTransaction {
        id: "tx_usd".to_string(),
        tx_date: t_base,
        value_date: None,
        doc_ref: Some("HD-USD-VND".to_string()),
        direction: PostingType::Credit,
        amount: Money::from_minor(50_000_000, Currency::USD),
        narration: "HD-USD-VND".to_string(),
        counterparty_name: None,
    }];

    // Ledger entry is 50,000,000 VND (identical numeric amount, but DIFFERENT currency!)
    let ledger = vec![LedgerEntry {
        id: "led_vnd".to_string(),
        doc_no: "HD-USD-VND".to_string(),
        entry_date: t_base,
        direction: PostingType::Credit,
        amount: Money::from_minor(50_000_000, Currency::VND),
        partner_name: None,
        description: "HD-USD-VND".to_string(),
    }];

    let (matches, unalloc_b, unalloc_l) =
        HashMatcher::match_tier1(&bank, &ledger, &config);

    assert!(
        matches.is_empty(),
        "CRITICAL VULNERABILITY: Tier 1 matched USD transaction with VND ledger entry due to missing currency check in HashMatcher!"
    );
    assert_eq!(unalloc_b.len(), 1);
    assert_eq!(unalloc_l.len(), 1);

    // Also verify USD bank vs EUR ledger
    let ledger_eur = vec![LedgerEntry {
        id: "led_eur".to_string(),
        doc_no: "HD-USD-VND".to_string(),
        entry_date: t_base,
        direction: PostingType::Credit,
        amount: Money::from_minor(50_000_000, Currency::EUR),
        partner_name: None,
        description: "HD-USD-VND".to_string(),
    }];
    let (matches_eur, unalloc_b_eur, unalloc_l_eur) =
        HashMatcher::match_tier1(&bank, &ledger_eur, &config);
    assert!(matches_eur.is_empty(), "Tier 1 must reject USD vs EUR mismatch");
    assert_eq!(unalloc_b_eur.len(), 1);
    assert_eq!(unalloc_l_eur.len(), 1);
}

#[test]
fn challenge_tier2_currency_mismatch_cross_contamination() {
    let config = MatchConfig::default();
    let t_base = 1726358400i64;

    // Bank transaction is 25,000,000 EUR
    let bank = vec![BankTransaction {
        id: "tx_eur".to_string(),
        tx_date: t_base,
        value_date: None,
        doc_ref: None,
        direction: PostingType::Credit,
        amount: Money::from_minor(25_000_000, Currency::EUR),
        narration: "CONG TY TNHH DAI NAM".to_string(),
        counterparty_name: Some("DAI NAM".to_string()),
    }];

    // Ledger entry is 25,000,000 VND
    let ledger = vec![LedgerEntry {
        id: "led_vnd".to_string(),
        doc_no: "DOC_EUR_VND".to_string(),
        entry_date: t_base + 3600,
        direction: PostingType::Credit,
        amount: Money::from_minor(25_000_000, Currency::VND),
        partner_name: Some("CONG TY TNHH DAI NAM".to_string()),
        description: "DAI NAM".to_string(),
    }];

    let unalloc_b = vec![0];
    let unalloc_l = vec![0];
    let (matches, unalloc_b_final, unalloc_l_final) =
        FuzzyMatcher::match_tier2(&bank, &ledger, &unalloc_b, &unalloc_l, &config);

    assert!(
        matches.is_empty(),
        "CRITICAL VULNERABILITY: Tier 2 matched EUR transaction with VND ledger entry due to missing currency check in FuzzyMatcher!"
    );
    assert_eq!(unalloc_b_final.len(), 1);
    assert_eq!(unalloc_l_final.len(), 1);

    // Also verify EUR bank vs USD ledger
    let ledger_usd = vec![LedgerEntry {
        id: "led_usd".to_string(),
        doc_no: "DOC_EUR_USD".to_string(),
        entry_date: t_base + 3600,
        direction: PostingType::Credit,
        amount: Money::from_minor(25_000_000, Currency::USD),
        partner_name: Some("CONG TY TNHH DAI NAM".to_string()),
        description: "DAI NAM".to_string(),
    }];
    let (matches_usd, unalloc_b_usd, unalloc_l_usd) =
        FuzzyMatcher::match_tier2(&bank, &ledger_usd, &unalloc_b, &unalloc_l, &config);
    assert!(matches_usd.is_empty(), "Tier 2 must reject EUR vs USD mismatch");
    assert_eq!(unalloc_b_usd.len(), 1);
    assert_eq!(unalloc_l_usd.len(), 1);
}

#[test]
fn challenge_fee_disentanglement_substring_collision() {
    // 1. Prefix collision cases: e.g. "phi 10000" vs "phi 1000"
    let pairs = [
        ("thanh toan hoa don phi 10000 vnd", 10_000),
        ("thanh toan hoa don phi 11000 vnd", 11_000),
        ("thanh toan hoa don phi 20000 vnd", 20_000),
        ("thanh toan hoa don phi 22000 vnd", 22_000),
        ("thanh toan hoa don fee 10000 vnd", 10_000),
        ("thanh toan hoa don fee 22000 vnd", 22_000),
        ("thanh toan hoa don phi: 10000 vnd", 10_000),
        ("thanh toan hoa don phi chuyen tien 22000 vnd", 22_000),
    ];

    for (memo, expected_fee) in pairs {
        let disentangled = FeeSplitter::disentangle_fee(Money::vnd(50_000_000), memo);
        assert_eq!(
            disentangled.fee_amount,
            Money::vnd(expected_fee),
            "Substring collision bug for '{}': expected fee {}, got {:?}",
            memo,
            expected_fee,
            disentangled.fee_amount
        );
    }

    // 2. Preceding alphanumeric guard: "coffee 1000" should not trigger "fee 1000"
    let non_fee = FeeSplitter::disentangle_fee(Money::vnd(50_000_000), "mua coffee 1000 ly");
    assert_eq!(
        non_fee.fee_amount,
        Money::vnd(0),
        "Word boundary violation: 'coffee 1000' incorrectly detected as fee!"
    );
}


