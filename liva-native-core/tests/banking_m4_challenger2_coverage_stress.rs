//! Milestone 4 (M4) Empirical Adversarial Challenge Test Suite (Challenger 2)
//!
//! White-box coverage and empirical adversarial stress testing for Milestone 4:
//! 1. Adversarial Test 1: High-combinatorial subset-sum stress with large candidate pools
//!    - Tests `solve_exact_subset_sum_bnb` and `SplitSolver::match_tier3_with_depth` up to depth 8.
//!    - Confirms branch-and-bound pruning never hangs, stack overflows, or panics on impossible
//!      sums, dense candidate sets, duplicate amounts, knapsack traps, or depth > 8 limits.
//! 2. Adversarial Test 2: Malicious / corrupted Vietnamese Unicode strings
//!    - Decomposed NFD with pathological repeating combining diacritics.
//!    - Invisible, zero-width, and formatting control characters (\u{200B}, \u{200C}, \u{200D}, \u{FEFF}).
//!    - ASCII control characters (\x00, \x07, \x08, \x1B, \t, \r, \n) and astral plane emojis.
//!    - Degenerate, oversized, and homoglyphic Cyrillic strings.
//!    - Confirms `normalize_vietnamese_text` and `compare_party_names` handle all strings safely
//!      without panicking, with scores bounded strictly in 0.0..=1.0.
//! 3. Adversarial Test 3: Extreme monetary boundary values & zero float drift
//!    - Large numbers up to u64::MAX without overflow.
//!    - Zero amounts, 1 VND micro-transactions, and Napas fee boundaries (-1,100 to -11,000 VND).
//!    - Strictly scaled integer arithmetic and zero float drift across high-volume reconciliation.
//! 4. Adversarial Test 4: Invariant tampering verification
//!    - Verifies that any deliberate tamper (+1, -1 VND) in opening balance, closing balance,
//!      transaction amounts, or running balance transitions is immediately detected by
//!      `verify_balance_invariants`, `check_balance_invariants`, and `ReconciliationEngine::reconcile`.

use std::collections::HashSet;
use std::time::Instant;

use liva_native_core::banking::models::*;
use liva_native_core::banking::reconciliation::ReconciliationEngine;
use liva_native_core::banking::reconciliation::jaro_winkler::{
    compare_party_names, normalize_vietnamese_text, strip_bank_narration_noise,
};
use liva_native_core::banking::reconciliation::split_solver::{
    MAX_SUPPORTED_SPLIT_DEPTH, SplitSolver, solve_exact_subset_sum_bnb,
};

// ===========================================================================
// Test Fixture Helpers
// ===========================================================================

fn make_bank_tx(
    id: &str,
    amount: u64,
    tx_type: TransactionType,
    balance_after: Option<u64>,
    doc_ref: Option<&str>,
    cp_name: Option<&str>,
    narration: &str,
    time: i64,
) -> BankTransactionRow {
    BankTransactionRow {
        id: id.to_string(),
        statement_id: "stmt_m4_ch2".to_string(),
        account_id: "acc_m4_ch2".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: time,
        value_date: time,
        doc_ref: doc_ref.map(|s| s.to_string()),
        tx_type,
        amount,
        balance_after,
        counterparty_account: None,
        counterparty_name: cp_name.map(|s| s.to_string()),
        counterparty_bank: None,
        narration: narration.to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: time,
    }
}

fn make_ledger_entry(
    id: &str,
    amount: u64,
    entry_type: TransactionType,
    doc_no: &str,
    partner_name: Option<&str>,
    time: i64,
) -> InternalLedgerEntry {
    InternalLedgerEntry {
        id: id.to_string(),
        account_id: "acc_m4_ch2".to_string(),
        doc_no: doc_no.to_string(),
        entry_date: time,
        entry_type,
        amount,
        partner_code: Some("PARTNER_CH2".to_string()),
        partner_name: partner_name.map(|s| s.to_string()),
        description: format!("Ledger invoice {doc_no}"),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: time,
    }
}

// ===========================================================================
// Adversarial Test 1: High-combinatorial subset-sum stress with large pools
// ===========================================================================

#[test]
fn test_m4_adversarial_subset_sum_bnb_combinatorial_stress() {
    // -----------------------------------------------------------------------
    // Sub-test 1.1: Dense Knapsack Trap with Impossible Parity Target
    // 100 candidate items with strictly even amounts.
    // Target is odd (150,001 VND). Brute-force would explore C(100, 8) ≈ 1.86e11.
    // BNB pruning must evaluate this impossible target and return None in < 50ms.
    // -----------------------------------------------------------------------
    let mut even_candidates: Vec<(usize, u64)> = Vec::with_capacity(100);
    for i in 1..=100 {
        even_candidates.push((i, i as u64 * 2_000)); // 2,000 to 200,000 VND
    }

    let start_parity = Instant::now();
    let res_parity = solve_exact_subset_sum_bnb(&even_candidates, 150_001, 8);
    let duration_parity = start_parity.elapsed();

    assert!(
        res_parity.is_none(),
        "Impossible odd target from all-even candidates must return None"
    );
    assert!(
        duration_parity.as_millis() < 50,
        "BNB pruning on 100-candidate parity trap must finish in < 50ms, took {:?}",
        duration_parity
    );

    // -----------------------------------------------------------------------
    // Sub-test 1.2: Boundary Depth 9 Unachievable at Depth Limit 8
    // 9 items of 10,000,000 VND each. Target is 90,000,000 VND.
    // At depth limit 8, max achievable sum is 80,000,000 VND.
    // BNB must return None without allocating partial amounts or stack overflow.
    // -----------------------------------------------------------------------
    let candidates_9: Vec<(usize, u64)> = (0..9).map(|i| (i, 10_000_000)).collect();
    let res_d8 = solve_exact_subset_sum_bnb(&candidates_9, 90_000_000, 8);
    assert!(
        res_d8.is_none(),
        "Target requiring depth 9 must return None under depth limit 8"
    );

    // If 8 items are given totaling 80M and target is 80M, it must succeed.
    let res_d8_ok = solve_exact_subset_sum_bnb(&candidates_9[..8], 80_000_000, 8);
    assert!(
        res_d8_ok.is_some(),
        "Target achievable with exactly 8 items must succeed"
    );
    let solved_d8 = res_d8_ok.unwrap();
    assert_eq!(solved_d8.len(), 8);
    let sum_d8: u64 = solved_d8.iter().map(|&idx| candidates_9[idx].1).sum();
    assert_eq!(sum_d8, 80_000_000);

    // -----------------------------------------------------------------------
    // Sub-test 1.3: Dense Distractor Pool with Hidden Exact 8-Split
    // 8 distinct target items embedded in 70 distractors.
    // -----------------------------------------------------------------------
    let target_items = [
        7_000_000u64,
        11_000_000,
        13_000_000,
        17_000_000,
        19_000_000,
        23_000_000,
        29_000_000,
        31_000_000,
    ];
    let exact_target: u64 = target_items.iter().sum(); // 150_000_000 VND

    let mut large_candidate_pool = Vec::new();
    let mut item_id = 0;
    // Add distractors that cannot form exact 150,000,000 with <= 8 items
    for i in 1..=70 {
        item_id += 1;
        let distractor = 50_000_000 + (i as u64 * 3_000_000);
        large_candidate_pool.push((item_id, distractor));
    }
    // Embed the 8 target items
    let mut embedded_indices = Vec::new();
    for &val in &target_items {
        item_id += 1;
        large_candidate_pool.push((item_id, val));
        embedded_indices.push(item_id);
    }

    let start_embed = Instant::now();
    let res_embed = solve_exact_subset_sum_bnb(&large_candidate_pool, exact_target, 8);
    let duration_embed = start_embed.elapsed();

    assert!(
        res_embed.is_some(),
        "BNB solver must find the exact 8-item subset embedded in 78 candidates"
    );
    let found_subset = res_embed.unwrap();
    assert_eq!(found_subset.len(), 8, "Must find exactly 8 items");
    assert!(
        duration_embed.as_millis() < 100,
        "BNB search on 78 items must finish in < 100ms, took {:?}",
        duration_embed
    );

    // Verify found items match embedded items exactly
    let mut found_set: HashSet<usize> = found_subset.into_iter().collect();
    for exp_id in embedded_indices {
        assert!(
            found_set.remove(&exp_id),
            "Expected embedded candidate ID {exp_id} in solution"
        );
    }
    assert!(found_set.is_empty());

    // -----------------------------------------------------------------------
    // Sub-test 1.4: Duplicate Identical Amounts & Disjoint Candidate Indices
    // 25 identical items of 5,000,000 VND each. Target is 35,000,000 VND (7 items).
    // BNB must return 7 strictly unique candidate indices.
    // -----------------------------------------------------------------------
    let duplicate_pool: Vec<(usize, u64)> = (100..125).map(|i| (i, 5_000_000)).collect();
    let res_dup = solve_exact_subset_sum_bnb(&duplicate_pool, 35_000_000, 8);
    assert!(res_dup.is_some());
    let dup_indices = res_dup.unwrap();
    assert_eq!(dup_indices.len(), 7, "Must select exactly 7 items of 5M");

    let unique_indices: HashSet<usize> = dup_indices.iter().copied().collect();
    assert_eq!(
        unique_indices.len(),
        7,
        "All candidate indices returned by BNB must be strictly disjoint"
    );

    // -----------------------------------------------------------------------
    // Sub-test 1.5: Extreme Edge Cases for BNB Solver
    // -----------------------------------------------------------------------
    assert!(
        solve_exact_subset_sum_bnb(&[], 100_000, 8).is_none(),
        "Empty candidates must return None"
    );
    assert!(
        solve_exact_subset_sum_bnb(&[(1, 100_000)], 0, 8).is_none(),
        "Zero target must return None"
    );
    assert!(
        solve_exact_subset_sum_bnb(&[(1, 0), (2, 0)], 50_000, 8).is_none(),
        "Zero candidate amounts must be filtered and return None"
    );
    assert_eq!(
        solve_exact_subset_sum_bnb(&[(42, 50_000)], 50_000, 8),
        Some(vec![42]),
        "Single candidate exact match must return Some(vec![42])"
    );
    assert!(
        solve_exact_subset_sum_bnb(&[(42, 49_999)], 50_000, 8).is_none(),
        "Single candidate < target must return None"
    );
    assert!(
        solve_exact_subset_sum_bnb(&[(1, 100_000), (2, 200_000)], 50_000, 8).is_none(),
        "All candidates > target must return None"
    );
    assert!(
        solve_exact_subset_sum_bnb(&[(1, 10_000), (2, 20_000)], 50_000, 8).is_none(),
        "Sum of all candidates < target must return None"
    );
    assert!(
        solve_exact_subset_sum_bnb(&[(1, 50_000)], 50_000, 0).is_none(),
        "max_depth = 0 must return None"
    );
    assert!(
        solve_exact_subset_sum_bnb(&[(1, 20_000), (2, 30_000)], 50_000, 1).is_none(),
        "max_depth = 1 when no single item matches must return None"
    );

    // -----------------------------------------------------------------------
    // Sub-test 1.6: End-to-End Tier 3 SplitSolver with High-Combinatorial Load
    // 30 Bank transactions vs 20 Ledger invoices, evaluated at depth 8.
    // -----------------------------------------------------------------------
    let mut bank_txs = Vec::new();
    let mut ledger_entries = Vec::new();
    let base_time = 1_773_400_000i64;

    // 1-to-5 split: Invoice L1 (100M VND) paid across 5 bank txs
    ledger_entries.push(make_ledger_entry(
        "L1",
        100_000_000,
        TransactionType::Credit,
        "INV-SPLIT-8D",
        Some("CONG TY AN KHANG"),
        base_time,
    ));
    bank_txs.push(make_bank_tx(
        "B1_1",
        20_000_000,
        TransactionType::Credit,
        None,
        None,
        Some("CONG TY AN KHANG"),
        "INV-SPLIT-8D dot 1",
        base_time,
    ));
    bank_txs.push(make_bank_tx(
        "B1_2",
        20_000_000,
        TransactionType::Credit,
        None,
        None,
        Some("CONG TY AN KHANG"),
        "INV-SPLIT-8D dot 2",
        base_time,
    ));
    bank_txs.push(make_bank_tx(
        "B1_3",
        20_000_000,
        TransactionType::Credit,
        None,
        None,
        Some("CONG TY AN KHANG"),
        "INV-SPLIT-8D dot 3",
        base_time,
    ));
    bank_txs.push(make_bank_tx(
        "B1_4",
        20_000_000,
        TransactionType::Credit,
        None,
        None,
        Some("CONG TY AN KHANG"),
        "INV-SPLIT-8D dot 4",
        base_time,
    ));
    bank_txs.push(make_bank_tx(
        "B1_5",
        20_000_000,
        TransactionType::Credit,
        None,
        None,
        Some("CONG TY AN KHANG"),
        "INV-SPLIT-8D dot 5",
        base_time,
    ));

    // Distractor transactions
    for i in 6..=25 {
        bank_txs.push(make_bank_tx(
            &format!("B_NOISE_{i}"),
            (i as u64) * 3_123_456,
            TransactionType::Credit,
            None,
            None,
            Some("CONG TY KHAC"),
            &format!("Noise transaction {i}"),
            base_time,
        ));
    }
    for i in 2..=15 {
        ledger_entries.push(make_ledger_entry(
            &format!("L_NOISE_{i}"),
            (i as u64) * 4_567_890,
            TransactionType::Credit,
            &format!("INV-NOISE-{i}"),
            Some("CONG TY KHAC"),
            base_time,
        ));
    }

    let unalloc_b: Vec<usize> = (0..bank_txs.len()).collect();
    let unalloc_l: Vec<usize> = (0..ledger_entries.len()).collect();

    let start_t3 = Instant::now();
    let (auto_matches, hitl_queue) = SplitSolver::match_tier3_with_depth(
        &bank_txs,
        &ledger_entries,
        &unalloc_b,
        &unalloc_l,
        MAX_SUPPORTED_SPLIT_DEPTH,
    );
    let duration_t3 = start_t3.elapsed();

    assert!(
        duration_t3.as_millis() < 500,
        "Tier 3 split solving under load must complete in < 500ms, took {:?}",
        duration_t3
    );
    assert!(
        !auto_matches.is_empty(),
        "Tier 3 must successfully match the 1-to-5 split invoice"
    );

    // Verify matching correctness & zero float drift
    let split_match = &auto_matches[0];
    assert!(
        split_match.confidence_score >= 0.95,
        "Composite split match confidence must be >= 0.95, got {}",
        split_match.confidence_score
    );
    assert_eq!(split_match.discrepancy_amount, 0);
    assert_eq!(split_match.bank_tx_ids.len(), 5);
    assert_eq!(split_match.ledger_entry_ids.len(), 1);
    assert_eq!(split_match.ledger_entry_ids[0], "L1");

    // All noise transactions routed to HITL or remain unallocated
    assert!(
        !hitl_queue.is_empty() || auto_matches.len() == 1,
        "Non-matching noise transactions must be routed to HITL or quarantine"
    );
}

// ===========================================================================
// Adversarial Test 2: Malicious / Corrupted Vietnamese Unicode Strings
// ===========================================================================

#[test]
fn test_m4_adversarial_malicious_corrupted_vietnamese_unicode_strings() {
    // -----------------------------------------------------------------------
    // Sub-test 2.1: Decomposed NFD with Pathological Repeating Combining Diacritics
    // -----------------------------------------------------------------------
    let pathological_combining = "a\u{0300}\u{0301}\u{0303}\u{0309}\u{0323}b\u{0300}\u{0301}c";
    let norm_pathological = normalize_vietnamese_text(pathological_combining);
    assert_eq!(
        norm_pathological, "abc",
        "Pathological combining diacritics on adjacent letters must fold cleanly to 'abc'"
    );

    let spaced_combining = "a\u{0300}\u{0301}\u{0303}\u{0309}\u{0323} b\u{0300}\u{0301} c";
    let norm_spaced = normalize_vietnamese_text(spaced_combining);
    assert_eq!(
        norm_spaced, "a b c",
        "Spaced letters with pathological diacritics must fold cleanly to 'a b c'"
    );

    // Standalone combining diacritical marks with no base character
    let orphan_combining = "\u{0300}\u{0301}\u{0302}\u{0303}\u{0323}";
    let norm_orphan = normalize_vietnamese_text(orphan_combining);
    assert!(
        norm_orphan.is_empty(),
        "Orphan combining marks must be discarded without panic, got '{norm_orphan}'"
    );

    // Complex Vietnamese decomposed NFD words
    let nfd_party = "C\u{006F}\u{0302}ng ty C\u{00F4}\u{0309} ph\u{00E2}\u{0300}n Th\u{00E1}\u{0300}i Bi\u{006E}\u{0068}\u{0300}";
    let norm_nfd = normalize_vietnamese_text(nfd_party);
    assert_eq!(
        norm_nfd, "cong ty co phan thai binh",
        "Decomposed NFD Vietnamese string must normalize identically to precomposed NFC"
    );

    // -----------------------------------------------------------------------
    // Sub-test 2.2: Zero-Width, Invisible, and Formatting Control Characters
    // -----------------------------------------------------------------------
    let invisible_attack_name = "C\u{200B}O\u{200C}N\u{200D}G TY\u{2060} TNHH\u{FEFF} AN KHANG";
    let clean_name = "CONG TY TNHH AN KHANG";

    let norm_invisible = normalize_vietnamese_text(invisible_attack_name);
    assert_eq!(
        norm_invisible, "c o n g ty tnhh an khang",
        "Non-alphanumeric zero-width chars are mapped to whitespace and collapsed"
    );

    // compare_party_names must not panic and must produce a valid score in 0.0..=1.0
    let score_invisible = compare_party_names(invisible_attack_name, clean_name);
    assert!(
        !score_invisible.is_nan() && !score_invisible.is_infinite(),
        "Score must not be NaN or Inf"
    );
    assert!(
        (0.0..=1.0).contains(&score_invisible),
        "Score must be within [0.0, 1.0], got {score_invisible}"
    );

    // When invisible chars are embedded between words:
    let invisible_between_words = "CONG TY\u{200B} TNHH\u{FEFF} AN KHANG";
    let score_between = compare_party_names(invisible_between_words, clean_name);
    assert_eq!(
        score_between, 1.0,
        "Party names differing only by zero-width characters between words must match with 1.0"
    );

    // -----------------------------------------------------------------------
    // Sub-test 2.3: ASCII Control Characters and Whitespace Injection
    // -----------------------------------------------------------------------
    let control_chars_narration = "THANH TOAN\x00 HD\x07-2026\x08-001\x1B CHO CTY\tAN\r\nKHANG";
    let norm_ctrl = normalize_vietnamese_text(control_chars_narration);
    assert_eq!(
        norm_ctrl, "thanh toan hd 2026 001 cho cty an khang",
        "Control characters must be safely converted to whitespace and collapsed"
    );

    let stripped_ctrl = strip_bank_narration_noise(control_chars_narration);
    assert!(
        !stripped_ctrl.contains('\x00') && !stripped_ctrl.contains('\x07'),
        "Control chars must be completely absent from stripped narration"
    );

    // -----------------------------------------------------------------------
    // Sub-test 2.4: Directional Overrides and Astral Plane Emojis
    // -----------------------------------------------------------------------
    let emoji_party = "🏦 CÔNG TY CP ĐẦU TƯ MINH PHÚ 🚀💰";
    let target_party = "CÔNG TY CỔ PHẦN ĐẦU TƯ MINH PHÚ";

    let score_emoji = compare_party_names(emoji_party, target_party);
    assert!(
        score_emoji >= 0.85,
        "Emojis in party names must be stripped without penalizing core brand similarity, got {score_emoji}"
    );

    // Right-to-Left Override (\u{202E}) attack
    let rlo_name = "CÔNG TY \u{202E}gnôh gnôS\u{202C} TNHH";
    let score_rlo = compare_party_names(rlo_name, "CÔNG TY SÔNG HỒNG TNHH");
    assert!(
        !score_rlo.is_nan() && (0.0..=1.0).contains(&score_rlo),
        "RLO injection must be processed safely without panic"
    );

    // -----------------------------------------------------------------------
    // Sub-test 2.5: Degenerate, Oversized, and Homoglyphic Strings
    // -----------------------------------------------------------------------
    let huge_string = "CÔNG TY CỔ PHẦN CÔNG NGHỆ THÁI BÌNH ".repeat(300);
    let start_huge = Instant::now();
    let norm_huge = normalize_vietnamese_text(&huge_string);
    let score_huge = compare_party_names(&huge_string, "CÔNG TY THÁI BÌNH");
    let dur_huge = start_huge.elapsed();

    assert!(
        dur_huge.as_millis() < 50,
        "Oversized 10k-char normalization must finish in < 50ms, took {:?}",
        dur_huge
    );
    assert!(!norm_huge.is_empty());
    assert!(!score_huge.is_nan() && (0.0..=1.0).contains(&score_huge));

    // Pure degenerate inputs
    assert_eq!(normalize_vietnamese_text(""), "");
    assert_eq!(normalize_vietnamese_text("   \t\r\n   "), "");
    assert_eq!(
        normalize_vietnamese_text("!@#$%^&*()_+-=[]{}|;':,./<>?"),
        ""
    );
    assert_eq!(compare_party_names("", "ABC"), 0.0);
    assert_eq!(compare_party_names("   ", "ABC"), 0.0);
    let empty_score = compare_party_names("", "");
    assert!(!empty_score.is_nan() && (0.0..=1.0).contains(&empty_score));
    let punct_score = compare_party_names("!@#$%", "^&*()");
    assert!(!punct_score.is_nan() && (0.0..=1.0).contains(&punct_score));

    // Cyrillic lookalike attack: 'С' (U+0421), 'О' (U+041E), 'Т' (U+0422), 'Ү' (U+04AE)
    let cyrillic_imposter = "\u{0421}\u{041E}NG \u{0422}\u{04AE} AN KHANG";
    let score_cyrillic = compare_party_names(cyrillic_imposter, "CONG TY AN KHANG");
    assert!(
        !score_cyrillic.is_nan() && (0.0..=1.0).contains(&score_cyrillic),
        "Cyrillic homoglyphs must not crash string distance metrics"
    );
}

// ===========================================================================
// Adversarial Test 3: Extreme Monetary Boundary Values & Zero Float Drift
// ===========================================================================

#[test]
fn test_m4_adversarial_extreme_monetary_boundaries_and_zero_drift() {
    // -----------------------------------------------------------------------
    // Sub-test 3.1: Large Numbers up to u64::MAX without Arithmetic Overflow
    // -----------------------------------------------------------------------
    let high_opening = u64::MAX - 50_000_000_000;
    let high_credit = 50_000_000_000u64;
    let high_closing = u64::MAX;

    let tx_high = TransactionRecord::new(
        1,
        1_773_400_000,
        1_773_400_000,
        Some("DOC-MAX-001".to_string()),
        TransactionType::Credit,
        high_credit,
        Some(high_closing),
        None,
        Some("TREASURY_MAX".to_string()),
        None,
        "Extreme high u64 transaction".to_string(),
    );

    let stmt_high = BankStatement::new(
        "VCB".to_string(),
        BankType::Vietcombank,
        StatementFormat::Excel,
        Some("ACC_MAX_U64".to_string()),
        Some("HIGH VALUE CORP".to_string()),
        Some(high_opening),
        Some(high_closing),
        Some(1_773_400_000),
        Some(1_773_400_000),
        vec![tx_high],
        10,
    );

    let inv_high = stmt_high.verify_balance_invariants();
    assert!(
        inv_high.is_balanced,
        "Statement at u64::MAX boundaries must balance cleanly: calculated_closing = {}, discrepancy = {}",
        inv_high.calculated_closing, inv_high.discrepancy
    );
    assert_eq!(inv_high.discrepancy, 0);

    // If closing balance is tampered by 1 VND at u64::MAX:
    let mut stmt_high_tampered = stmt_high.clone();
    stmt_high_tampered.closing_balance = Some(high_closing - 1);
    let inv_tampered = stmt_high_tampered.verify_balance_invariants();
    assert!(
        !inv_tampered.is_balanced,
        "1 VND tamper at u64::MAX boundary must be detected"
    );
    assert_eq!(inv_tampered.discrepancy, -1);

    // -----------------------------------------------------------------------
    // Sub-test 3.2: Zero-Amount Transactions and Zero Target
    // -----------------------------------------------------------------------
    let tx_zero = make_bank_tx(
        "TX_ZERO",
        0,
        TransactionType::Credit,
        Some(50_000_000),
        None,
        None,
        "Zero amount ping",
        1_773_400_000,
    );
    let tx_subsequent = make_bank_tx(
        "TX_SUB",
        10_000_000,
        TransactionType::Credit,
        Some(60_000_000),
        None,
        None,
        "Normal credit after zero",
        1_773_400_100,
    );

    let (checked_zero, passed_zero, disc_zero) =
        ReconciliationEngine::check_balance_invariants(&[tx_zero, tx_subsequent]);
    assert!(checked_zero);
    assert!(
        passed_zero,
        "Consecutive transactions with 0 VND amount must pass running balance invariants"
    );
    assert_eq!(disc_zero, 0);

    // -----------------------------------------------------------------------
    // Sub-test 3.3: 1 VND Micro-Transactions & Running Balances (Zero Drift)
    // -----------------------------------------------------------------------
    let mut micro_txs = Vec::with_capacity(100);
    let mut curr_bal = 10_000_000u64;
    let mut base_time = 1_773_400_000i64;

    for i in 0..100 {
        base_time += 10;
        let is_credit = i % 2 == 0;
        let tx_type = if is_credit {
            curr_bal += 1;
            TransactionType::Credit
        } else {
            curr_bal -= 1;
            TransactionType::Debit
        };

        micro_txs.push(make_bank_tx(
            &format!("MICRO_{i}"),
            1,
            tx_type,
            Some(curr_bal),
            Some(&format!("DOC-MICRO-{i}")),
            Some("MICRO_PARTNER"),
            &format!("Micro transfer 1 VND {i}"),
            base_time,
        ));
    }

    let (micro_chk, micro_passed, micro_disc) =
        ReconciliationEngine::check_balance_invariants(&micro_txs);
    assert!(micro_chk);
    assert!(
        micro_passed,
        "100 micro-transactions of 1 VND must pass running balance invariants exactly"
    );
    assert_eq!(
        micro_disc, 0,
        "Discrepancy on 1 VND micro-transactions must be strictly 0"
    );
    assert_eq!(
        curr_bal, 10_000_000,
        "Net balance after 50 +1 and 50 -1 must remain exactly 10,000,000"
    );

    // -----------------------------------------------------------------------
    // Sub-test 3.4: Napas Fee Boundary Reconciliation (-1,100 to -11,000 VND)
    // -----------------------------------------------------------------------
    let fee_tiers = [1_100u64, 2_200, 3_300, 5_500, 7_700, 8_800, 11_000];
    let mut fee_tx_chain = Vec::new();
    let mut fee_records = Vec::new();
    let mut fee_bal = 500_000_000u64;
    let mut row_idx = 0;

    // Start with opening balance transaction
    fee_tx_chain.push(make_bank_tx(
        "TX_INIT",
        0,
        TransactionType::Credit,
        Some(fee_bal),
        None,
        None,
        "Initial Balance",
        1_773_500_000,
    ));

    for (idx, &fee) in fee_tiers.iter().enumerate() {
        let tx_time = 1_773_500_000 + ((idx as i64 + 1) * 100);
        // Principal Debit
        let principal = (idx as u64 + 1) * 10_000_000;
        fee_bal -= principal;
        row_idx += 1;
        fee_tx_chain.push(make_bank_tx(
            &format!("TX_PRIN_{idx}"),
            principal,
            TransactionType::Debit,
            Some(fee_bal),
            Some(&format!("DOC-PRIN-{idx}")),
            Some("SUPPLIER CORP"),
            &format!("Chuyen khoan thanh toan {idx}"),
            tx_time,
        ));
        fee_records.push(TransactionRecord::new(
            row_idx,
            tx_time,
            tx_time,
            Some(format!("DOC-PRIN-{idx}")),
            TransactionType::Debit,
            principal,
            Some(fee_bal),
            None,
            Some("SUPPLIER CORP".to_string()),
            None,
            format!("Chuyen khoan thanh toan {idx}"),
        ));

        // Napas 24/7 Service Fee Debit
        fee_bal -= fee;
        row_idx += 1;
        fee_tx_chain.push(make_bank_tx(
            &format!("TX_FEE_{idx}"),
            fee,
            TransactionType::Debit,
            Some(fee_bal),
            None,
            None,
            &format!("Phi chuyen tien nhanh Napas 247 {fee} VND"),
            tx_time + 1,
        ));
        fee_records.push(TransactionRecord::new(
            row_idx,
            tx_time + 1,
            tx_time + 1,
            None,
            TransactionType::Debit,
            fee,
            Some(fee_bal),
            None,
            None,
            None,
            format!("Phi chuyen tien nhanh Napas 247 {fee} VND"),
        ));
    }

    // Verify running balances across all 7 Napas fee events
    let (fee_chk, fee_pass, fee_d) = ReconciliationEngine::check_balance_invariants(&fee_tx_chain);
    assert!(fee_chk);
    assert!(
        fee_pass,
        "Napas fee running balance verification must pass with 0 discrepancy"
    );
    assert_eq!(fee_d, 0);

    // Verify Statement balance invariant
    let fee_stmt = BankStatement::new(
        "VCB".to_string(),
        BankType::Vietcombank,
        StatementFormat::Excel,
        Some("ACC_FEE_TEST".to_string()),
        Some("FEE CORP".to_string()),
        Some(500_000_000),
        Some(fee_bal),
        Some(1_773_500_000),
        Some(1_773_501_000),
        fee_records,
        15,
    );
    let fee_report = fee_stmt.verify_balance_invariants();
    assert!(
        fee_report.is_balanced,
        "Statement containing 7 Napas fee tiers must strictly balance"
    );
    assert_eq!(fee_report.discrepancy, 0);

    // -----------------------------------------------------------------------
    // Sub-test 3.5: Full Pipeline Zero Float Drift Audit
    // -----------------------------------------------------------------------
    let mut ledger_invoices = Vec::new();
    for idx in 0..fee_tiers.len() {
        let principal = (idx as u64 + 1) * 10_000_000;
        ledger_invoices.push(make_ledger_entry(
            &format!("INV_PRIN_{idx}"),
            principal,
            TransactionType::Debit,
            &format!("DOC-PRIN-{idx}"),
            Some("SUPPLIER CORP"),
            1_773_500_000 + ((idx as i64 + 1) * 100),
        ));
    }

    let (matches, summary) = ReconciliationEngine::reconcile(&fee_tx_chain, &ledger_invoices);
    assert!(summary.balance_invariant_checked);
    assert!(summary.balance_invariant_passed);
    assert_eq!(
        summary.balance_discrepancy_amount, 0,
        "Zero float drift: discrepancy must be exactly 0"
    );

    // Principals must be matched in Tier 1; fee transactions and initial zero balance are routed to HITL
    let approved_matches: Vec<&ReconciliationMatch> = matches
        .iter()
        .filter(|m| m.match_type != MatchType::ManualHitl)
        .collect();
    assert_eq!(
        approved_matches.len(),
        fee_tiers.len(),
        "All 7 principal invoices must be matched"
    );
    assert_eq!(summary.matched_exact_count, fee_tiers.len());
    assert_eq!(summary.pending_hitl_count, 8); // 1 initial + 7 fees

    for m in &approved_matches {
        assert_eq!(
            m.discrepancy_amount, 0,
            "Each approved match must have exactly 0 discrepancy"
        );
        assert_eq!(m.confidence_score, 1.0);
    }
}

// ===========================================================================
// Adversarial Test 4: Invariant Tampering Verification
// ===========================================================================

#[test]
fn test_m4_adversarial_invariant_tampering_verification() {
    // -----------------------------------------------------------------------
    // Sub-test 4.1: Comprehensive Tamper Matrix on verify_balance_invariants
    // -----------------------------------------------------------------------
    let open_bal = 100_000_000u64;
    let close_bal = 100_000_000u64;
    let baseline_txs = vec![
        TransactionRecord::new(
            1,
            1_773_400_000,
            1_773_400_000,
            Some("INV-01".to_string()),
            TransactionType::Credit,
            25_000_000,
            Some(125_000_000),
            None,
            None,
            None,
            "Deposit 1".to_string(),
        ),
        TransactionRecord::new(
            2,
            1_773_400_100,
            1_773_400_100,
            Some("INV-02".to_string()),
            TransactionType::Credit,
            15_000_000,
            Some(140_000_000),
            None,
            None,
            None,
            "Deposit 2".to_string(),
        ),
        TransactionRecord::new(
            3,
            1_773_400_200,
            1_773_400_200,
            Some("INV-03".to_string()),
            TransactionType::Debit,
            10_000_000,
            Some(130_000_000),
            None,
            None,
            None,
            "Withdrawal 1".to_string(),
        ),
        TransactionRecord::new(
            4,
            1_773_400_300,
            1_773_400_300,
            Some("INV-04".to_string()),
            TransactionType::Debit,
            30_000_000,
            Some(100_000_000),
            None,
            None,
            None,
            "Withdrawal 2".to_string(),
        ),
    ];

    let base_stmt = BankStatement::new(
        "VCB".to_string(),
        BankType::Vietcombank,
        StatementFormat::Excel,
        Some("ACC_TAMPER".to_string()),
        Some("TAMPER CORP".to_string()),
        Some(open_bal),
        Some(close_bal),
        Some(1_773_400_000),
        Some(1_773_400_300),
        baseline_txs.clone(),
        10,
    );

    let base_rep = base_stmt.verify_balance_invariants();
    assert!(base_rep.is_balanced);
    assert_eq!(base_rep.discrepancy, 0);

    // Tamper 1: Opening balance +1 VND
    let mut stmt_t1 = base_stmt.clone();
    stmt_t1.opening_balance = Some(open_bal + 1);
    let rep_t1 = stmt_t1.verify_balance_invariants();
    assert!(!rep_t1.is_balanced, "Opening +1 VND must fail invariant");
    assert_eq!(rep_t1.discrepancy, -1);

    // Tamper 2: Opening balance -1 VND
    let mut stmt_t2 = base_stmt.clone();
    stmt_t2.opening_balance = Some(open_bal - 1);
    let rep_t2 = stmt_t2.verify_balance_invariants();
    assert!(!rep_t2.is_balanced, "Opening -1 VND must fail invariant");
    assert_eq!(rep_t2.discrepancy, 1);

    // Tamper 3: Closing balance +1 VND
    let mut stmt_t3 = base_stmt.clone();
    stmt_t3.closing_balance = Some(close_bal + 1);
    let rep_t3 = stmt_t3.verify_balance_invariants();
    assert!(!rep_t3.is_balanced, "Closing +1 VND must fail invariant");
    assert_eq!(rep_t3.discrepancy, 1);

    // Tamper 4: Closing balance -1 VND
    let mut stmt_t4 = base_stmt.clone();
    stmt_t4.closing_balance = Some(close_bal - 1);
    let rep_t4 = stmt_t4.verify_balance_invariants();
    assert!(!rep_t4.is_balanced, "Closing -1 VND must fail invariant");
    assert_eq!(rep_t4.discrepancy, -1);

    // Tamper 5: Single transaction amount perturbed by 1 VND via BalanceInvariantReport::verify
    let mut tampered_tx_list = baseline_txs.clone();
    tampered_tx_list[0].amount += 1;
    let rep_tx_pert =
        BalanceInvariantReport::verify(Some(open_bal), Some(close_bal), &tampered_tx_list);
    assert!(
        !rep_tx_pert.is_balanced,
        "Transaction list with 1 VND perturbation must fail verify"
    );
    assert_eq!(rep_tx_pert.discrepancy, -1);

    // Tamper 6: Invert transaction direction (Debit -> Credit)
    let mut dir_tampered_txs = baseline_txs.clone();
    dir_tampered_txs[2].tx_type = TransactionType::Credit; // Was Debit 10M
    let rep_dir =
        BalanceInvariantReport::verify(Some(open_bal), Some(close_bal), &dir_tampered_txs);
    assert!(!rep_dir.is_balanced);
    assert_eq!(rep_dir.discrepancy, -20_000_000);

    // Tamper 7: Drop transaction from list
    let dropped_txs = vec![baseline_txs[0].clone(), baseline_txs[1].clone()];
    let rep_dropped = BalanceInvariantReport::verify(Some(open_bal), Some(close_bal), &dropped_txs);
    assert!(!rep_dropped.is_balanced);
    assert_eq!(rep_dropped.discrepancy, -40_000_000);

    // Tamper 8: Inject duplicate transaction
    let mut dup_txs = baseline_txs.clone();
    dup_txs.push(baseline_txs[1].clone()); // Extra 15M Credit
    let rep_dup = BalanceInvariantReport::verify(Some(open_bal), Some(close_bal), &dup_txs);
    assert!(!rep_dup.is_balanced);
    assert_eq!(rep_dup.discrepancy, -15_000_000);

    // -----------------------------------------------------------------------
    // Sub-test 4.2: Comprehensive Tamper Matrix on check_balance_invariants
    // -----------------------------------------------------------------------
    let baseline_bank_chain = vec![
        make_bank_tx(
            "B0",
            0,
            TransactionType::Credit,
            Some(100_000_000),
            None,
            None,
            "Init",
            100,
        ),
        make_bank_tx(
            "B1",
            25_000_000,
            TransactionType::Credit,
            Some(125_000_000),
            None,
            None,
            "Cr 1",
            200,
        ),
        make_bank_tx(
            "B2",
            15_000_000,
            TransactionType::Credit,
            Some(140_000_000),
            None,
            None,
            "Cr 2",
            300,
        ),
        make_bank_tx(
            "B3",
            10_000_000,
            TransactionType::Debit,
            Some(130_000_000),
            None,
            None,
            "Db 1",
            400,
        ),
        make_bank_tx(
            "B4",
            30_000_000,
            TransactionType::Debit,
            Some(100_000_000),
            None,
            None,
            "Db 2",
            500,
        ),
    ];

    let (chk_base, pass_base, disc_base) =
        ReconciliationEngine::check_balance_invariants(&baseline_bank_chain);
    assert!(chk_base && pass_base && disc_base == 0);

    // Attack A: Perturb mid-sequence balance_after by +1 VND
    let mut chain_a = baseline_bank_chain.clone();
    chain_a[2].balance_after = Some(140_000_001);
    let (_, pass_a, disc_a) = ReconciliationEngine::check_balance_invariants(&chain_a);
    assert!(
        !pass_a,
        "Perturbed running balance by +1 VND must be detected"
    );
    assert_ne!(disc_a, 0);

    // Attack B: Perturb mid-sequence balance_after by -1 VND
    let mut chain_b = baseline_bank_chain.clone();
    chain_b[3].balance_after = Some(129_999_999);
    let (_, pass_b, disc_b) = ReconciliationEngine::check_balance_invariants(&chain_b);
    assert!(
        !pass_b,
        "Perturbed running balance by -1 VND must be detected"
    );
    assert_ne!(disc_b, 0);

    // Attack C: Perturb transaction amount by 1 VND while leaving balance_after unchanged
    let mut chain_c = baseline_bank_chain.clone();
    chain_c[2].amount = 15_000_001; // Was 15M
    let (_, pass_c, disc_c) = ReconciliationEngine::check_balance_invariants(&chain_c);
    assert!(
        !pass_c,
        "Amount tampering without balance_after update must fail check"
    );
    assert_ne!(disc_c, 0);

    // Attack D: Invert transaction direction without changing balance_after
    let mut chain_d = baseline_bank_chain.clone();
    chain_d[3].tx_type = TransactionType::Credit; // Was Debit 10M
    let (_, pass_d, disc_d) = ReconciliationEngine::check_balance_invariants(&chain_d);
    assert!(
        !pass_d,
        "Swapped transaction direction must fail running balance invariant"
    );
    assert_ne!(disc_d, 0);

    // Attack E: Reversing consecutive transactions with different amounts at same timestamp
    let mut chain_e = baseline_bank_chain.clone();
    chain_e[1].tx_date = 250;
    chain_e[2].tx_date = 250;
    chain_e.swap(1, 2); // Swapped B1 (25M, bal 125M) and B2 (15M, bal 140M)
    let (_, pass_e, _) = ReconciliationEngine::check_balance_invariants(&chain_e);
    assert!(
        !pass_e,
        "Swapping transactions must violate chronological running balances"
    );

    // Attack F: Delete a middle transaction from the chain
    let chain_f = vec![
        baseline_bank_chain[0].clone(),
        baseline_bank_chain[1].clone(),
        baseline_bank_chain[3].clone(), // B2 deleted!
        baseline_bank_chain[4].clone(),
    ];
    let (_, pass_f, disc_f) = ReconciliationEngine::check_balance_invariants(&chain_f);
    assert!(
        !pass_f,
        "Deleted transaction must create jump in running balance"
    );
    assert_eq!(disc_f.abs(), 15_000_000);

    // -----------------------------------------------------------------------
    // Sub-test 4.3: Multi-Account Isolation Under Selective Tampering
    // Account A is tampered (+50,000 VND), Account B is completely valid.
    // -----------------------------------------------------------------------
    let mut multi_account_batch = Vec::new();
    // Account A
    let mut a_bal = 50_000_000u64;
    for i in 0..5 {
        a_bal += 10_000_000;
        let mut tx = make_bank_tx(
            &format!("A_{i}"),
            10_000_000,
            TransactionType::Credit,
            Some(a_bal),
            None,
            None,
            "Acc A credit",
            100 + (i as i64 * 10),
        );
        tx.account_id = "ACC_ALPHA".to_string();
        multi_account_batch.push(tx);
    }
    // Tamper Account A in mid-chain
    multi_account_batch[2].balance_after =
        Some(multi_account_batch[2].balance_after.unwrap() + 50_000);

    // Account B (pristine)
    let mut b_bal = 200_000_000u64;
    for i in 0..5 {
        b_bal -= 20_000_000;
        let mut tx = make_bank_tx(
            &format!("B_{i}"),
            20_000_000,
            TransactionType::Debit,
            Some(b_bal),
            None,
            None,
            "Acc B debit",
            105 + (i as i64 * 10),
        );
        tx.account_id = "ACC_BETA".to_string();
        multi_account_batch.push(tx);
    }

    let (multi_chk, multi_passed, multi_disc) =
        ReconciliationEngine::check_balance_invariants(&multi_account_batch);
    assert!(multi_chk);
    assert!(
        !multi_passed,
        "Tampering in Account A must fail overall check_balance_invariants"
    );
    assert_eq!(
        multi_disc.abs(),
        50_000,
        "Maximum discrepancy across accounts must match the 50,000 VND tamper"
    );

    // -----------------------------------------------------------------------
    // Sub-test 4.4: End-to-End Pipeline Invariant Alert
    // When tampered bank transactions are supplied to ReconciliationEngine::reconcile,
    // the resulting summary must flag balance_invariant_passed = false.
    // -----------------------------------------------------------------------
    let dummy_ledger = vec![make_ledger_entry(
        "L_DUMMY",
        25_000_000,
        TransactionType::Credit,
        "DOC-NONE",
        None,
        100,
    )];
    let (_, summary_tampered) = ReconciliationEngine::reconcile(&chain_a, &dummy_ledger);
    assert!(
        summary_tampered.balance_invariant_checked,
        "Reconciliation engine must check balance invariants"
    );
    assert!(
        !summary_tampered.balance_invariant_passed,
        "Reconciliation engine must flag balance_invariant_passed = false on tampered data"
    );
    assert_ne!(
        summary_tampered.balance_discrepancy_amount, 0,
        "Reconciliation engine must record non-zero balance discrepancy amount"
    );
}
