//! Empirical Adversarial Stress Test Suite for Milestone 2: Deterministic Reconciliation Engine.
//!
//! Written and executed by Challenger 1.
//! Rigorously verifies:
//! 1. Direction Invariance Empirical Verification (Credit vs Debit in Tier 1, Tier 2, Tier 3 + HITL UUID v4).
//! 2. Value Date 24h Boundary Stress (86,400s boundary vs 86,401s, zero value date fallback).
//! 3. Bank Noise Stripping & Tokenization (Vietnamese banking prefixes vs genuine party names).
//! 4. Amount-Bucket Pre-indexing (11,000 VND bucket grouping, adjacent bucket probes, fee tolerance window).
//! 5. Tier 3 Split Solver & Branch-and-Bound Bounds (1:N, N:1, default depth 4, depth up to 8, zero panics).
//! 6. Empirical Vulnerability Reproduction: Unstripped Industry Descriptor False Positive in Tier 2.

use uuid::Uuid;

use liva_native_core::banking::models::*;
use liva_native_core::banking::reconciliation::ReconciliationEngine;
use liva_native_core::banking::reconciliation::fuzzy_matcher::FuzzyMatcher;
use liva_native_core::banking::reconciliation::hash_matcher::HashMatcher;
use liva_native_core::banking::reconciliation::jaro_winkler::{
    compare_party_names, strip_bank_narration_noise,
};
use liva_native_core::banking::reconciliation::split_solver::{
    DEFAULT_MAX_SPLIT_DEPTH, MAX_SUPPORTED_SPLIT_DEPTH, SplitSolver, solve_exact_subset_sum_bnb,
};

// ===========================================================================
// 1. Direction Invariance Empirical Verification
// ===========================================================================

#[test]
fn test_empirical_direction_invariance_across_all_tiers_and_hitl_quarantine() {
    let now = 1_725_000_000i64;

    let tx_tier1 = BankTransactionRow {
        id: "tx_c_t1".to_string(),
        statement_id: "stmt_dir".to_string(),
        account_id: "acc_dir".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: Some("HD-101".to_string()),
        tx_type: TransactionType::Credit,
        amount: 25_000_000,
        balance_after: Some(125_000_000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY ABC".to_string()),
        counterparty_bank: None,
        narration: "Thanh toan HD-101".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let tx_tier2 = BankTransactionRow {
        id: "tx_c_t2".to_string(),
        statement_id: "stmt_dir".to_string(),
        account_id: "acc_dir".to_string(),
        bank_code: "TCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 49_989_000,
        balance_after: Some(174_989_000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY CO PHAN CONG NGHE XYZ".to_string()),
        counterparty_bank: None,
        narration: "CONG TY XYZ CK TIEN DICH VU".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let tx_tier3 = BankTransactionRow {
        id: "tx_c_t3".to_string(),
        statement_id: "stmt_dir".to_string(),
        account_id: "acc_dir".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 30_000_000,
        balance_after: Some(204_989_000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY TNHH PHUONG DONG".to_string()),
        counterparty_bank: None,
        narration: "THANH TOAN HD-A VA HD-B".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let led_tier1_mismatch = InternalLedgerEntry {
        id: "led_d_t1".to_string(),
        account_id: "acc_dir".to_string(),
        doc_no: "HD101".to_string(),
        entry_date: now + 60,
        entry_type: TransactionType::Debit,
        amount: 25_000_000,
        partner_code: Some("ABC".to_string()),
        partner_name: Some("Công ty ABC".to_string()),
        description: "Hoa don 101".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let led_tier2_mismatch = InternalLedgerEntry {
        id: "led_d_t2".to_string(),
        account_id: "acc_dir".to_string(),
        doc_no: "HD-XYZ-2026".to_string(),
        entry_date: now + 300,
        entry_type: TransactionType::Debit,
        amount: 50_000_000,
        partner_code: Some("XYZ".to_string()),
        partner_name: Some("Công ty Cổ phần Công nghệ XYZ".to_string()),
        description: "Dich vu phan mem".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let led_tier3_mismatch_a = InternalLedgerEntry {
        id: "led_d_t3_a".to_string(),
        account_id: "acc_dir".to_string(),
        doc_no: "HDA".to_string(),
        entry_date: now + 100,
        entry_type: TransactionType::Debit,
        amount: 10_000_000,
        partner_code: Some("PD01".to_string()),
        partner_name: Some("Cong ty TNHH Phuong Dong".to_string()),
        description: "Dot 1".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let led_tier3_mismatch_b = InternalLedgerEntry {
        id: "led_d_t3_b".to_string(),
        account_id: "acc_dir".to_string(),
        doc_no: "HDB".to_string(),
        entry_date: now + 100,
        entry_type: TransactionType::Debit,
        amount: 20_000_000,
        partner_code: Some("PD01".to_string()),
        partner_name: Some("Cong ty TNHH Phuong Dong".to_string()),
        description: "Dot 2".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (t1_matches, rem_tx1, rem_led1) =
        HashMatcher::match_tier1(&[tx_tier1.clone()], &[led_tier1_mismatch.clone()]);
    assert!(
        t1_matches.is_empty(),
        "Tier 1 must NOT match opposite directions"
    );
    assert_eq!(rem_tx1.len(), 1);
    assert_eq!(rem_led1.len(), 1);

    let (t2_matches, rem_tx2, rem_led2) = FuzzyMatcher::match_tier2(
        &[tx_tier2.clone()],
        &[led_tier2_mismatch.clone()],
        &[0],
        &[0],
    );
    assert!(
        t2_matches.is_empty(),
        "Tier 2 must NOT match opposite directions"
    );
    assert_eq!(rem_tx2.len(), 1);
    assert_eq!(rem_led2.len(), 1);

    let (t3_matches, hitl3) = SplitSolver::match_tier3(
        &[tx_tier3.clone()],
        &[led_tier3_mismatch_a.clone(), led_tier3_mismatch_b.clone()],
        &[0],
        &[0, 1],
    );
    assert!(
        t3_matches.is_empty(),
        "Tier 3 must NOT match opposite directions"
    );
    assert_eq!(hitl3.len(), 1, "Tier 3 residual must route to HITL");
    assert_eq!(hitl3[0].status, "PENDING_HITL");

    let all_txs = vec![tx_tier1, tx_tier2, tx_tier3];
    let all_led = vec![
        led_tier1_mismatch,
        led_tier2_mismatch,
        led_tier3_mismatch_a,
        led_tier3_mismatch_b,
    ];

    let (matches, summary) = ReconciliationEngine::reconcile(&all_txs, &all_led);
    assert_eq!(summary.matched_exact_count, 0);
    assert_eq!(summary.matched_fuzzy_count, 0);
    assert_eq!(summary.matched_split_count, 0);
    assert_eq!(summary.total_matched_count, 0);
    assert_eq!(summary.pending_hitl_count, 3);
    assert_eq!(summary.match_rate, 0.0);

    let hitl_items: Vec<_> = matches
        .iter()
        .filter(|m| m.match_type == MatchType::ManualHitl)
        .collect();
    assert_eq!(hitl_items.len(), 3);
    for item in hitl_items {
        assert_eq!(item.status, "PENDING_HITL");
        let token_str = item.hitl_token.as_ref().expect("HITL token must exist");
        let parsed = Uuid::parse_str(token_str).expect("HITL token must be valid UUID string");
        assert_eq!(
            parsed.get_version(),
            Some(uuid::Version::Random),
            "Must be UUID v4"
        );
    }
}

// ===========================================================================
// 2. Value Date 24h Boundary Stress
// ===========================================================================

#[test]
fn test_empirical_value_date_24h_boundary_and_zero_fallback() {
    let base_time = 1_725_000_000i64;
    const WINDOW_24H: i64 = 86_400;

    let make_tx = |id: &str, tx_time: i64, val_time: i64, doc: &str| BankTransactionRow {
        id: id.to_string(),
        statement_id: "stmt_vd".to_string(),
        account_id: "acc_vd".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: tx_time,
        value_date: val_time,
        doc_ref: Some(doc.to_string()),
        tx_type: TransactionType::Credit,
        amount: 15_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: None,
        counterparty_bank: None,
        narration: format!("Thanh toan {doc}"),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: tx_time,
    };

    let make_led = |id: &str, entry_time: i64, doc: &str| InternalLedgerEntry {
        id: id.to_string(),
        account_id: "acc_vd".to_string(),
        doc_no: doc.to_string(),
        entry_date: entry_time,
        entry_type: TransactionType::Credit,
        amount: 15_000_000,
        partner_code: Some("PARTNER_VD".to_string()),
        partner_name: Some("Cong ty Test".to_string()),
        description: format!("Hoa don {doc}"),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: entry_time,
    };

    // Subtest 2.1: Exactly on 24h boundary (+86,400s) -> MUST MATCH
    let tx_pos_exact = make_tx(
        "tx_pos_86400",
        base_time + WINDOW_24H,
        base_time + WINDOW_24H,
        "HD-EXACT-POS",
    );
    let led_pos_exact = make_led("led_pos_86400", base_time, "HDEXACTPOS");
    let (m1, r1, _) = HashMatcher::match_tier1(&[tx_pos_exact], &[led_pos_exact]);
    assert_eq!(m1.len(), 1, "Diff exactly +86,400s must match in Tier 1");
    assert!(r1.is_empty());

    // Subtest 2.2: Exactly on 24h boundary (-86,400s) -> MUST MATCH
    let tx_neg_exact = make_tx("tx_neg_86400", base_time, base_time, "HD-EXACT-NEG");
    let led_neg_exact = make_led("led_neg_86400", base_time + WINDOW_24H, "HDEXACTNEG");
    let (m2, r2, _) = HashMatcher::match_tier1(&[tx_neg_exact], &[led_neg_exact]);
    assert_eq!(m2.len(), 1, "Diff exactly -86,400s must match in Tier 1");
    assert!(r2.is_empty());

    // Subtest 2.3: 1 second past boundary (+86,401s) -> MUST NOT MATCH
    let tx_pos_exceed = make_tx(
        "tx_pos_86401",
        base_time + WINDOW_24H + 1,
        base_time + WINDOW_24H + 1,
        "HD-EXCEED-POS",
    );
    let led_pos_exceed = make_led("led_pos_86401", base_time, "HDEXCEEDPOS");
    let (m3, r3, _) = HashMatcher::match_tier1(&[tx_pos_exceed], &[led_pos_exceed]);
    assert_eq!(m3.len(), 0, "Diff +86,401s must NOT match in Tier 1");
    assert_eq!(r3.len(), 1);

    // Subtest 2.4: 1 second past boundary (-86,401s) -> MUST NOT MATCH
    let tx_neg_exceed = make_tx("tx_neg_86401", base_time, base_time, "HD-EXCEED-NEG");
    let led_neg_exceed = make_led("led_neg_86401", base_time + WINDOW_24H + 1, "HDEXCEEDNEG");
    let (m4, r4, _) = HashMatcher::match_tier1(&[tx_neg_exceed], &[led_neg_exceed]);
    assert_eq!(m4.len(), 0, "Diff -86,401s must NOT match in Tier 1");
    assert_eq!(r4.len(), 1);

    // Subtest 2.5: Zero value date (`value_date == 0`) fallback to `tx_date`
    let tx_zero_val = make_tx("tx_zero_val", base_time, 0, "HD-ZERO-VAL");
    assert_eq!(tx_zero_val.effective_date(), base_time);

    let led_zero_match = make_led("led_zero_match", base_time + WINDOW_24H, "HDZEROVAL");
    let (m5, r5, _) = HashMatcher::match_tier1(&[tx_zero_val.clone()], &[led_zero_match]);
    assert_eq!(
        m5.len(),
        1,
        "Zero value_date fallback must match within 86,400s of tx_date"
    );
    assert!(r5.is_empty());

    let led_zero_fail = make_led("led_zero_fail", base_time + WINDOW_24H + 1, "HDZEROVAL");
    let (m6, r6, _) = HashMatcher::match_tier1(&[tx_zero_val], &[led_zero_fail]);
    assert_eq!(
        m6.len(),
        0,
        "Zero value_date fallback must reject diff of 86,401s"
    );
    assert_eq!(r6.len(), 1);

    // Subtest 2.6: Non-zero value date override (lagged settlement, e.g. interbank weekend)
    let friday_tx = base_time - (3 * 86_400);
    let monday_val = base_time;
    let tx_weekend = make_tx("tx_weekend", friday_tx, monday_val, "HD-WEEKEND");
    assert_eq!(tx_weekend.effective_date(), monday_val);

    let led_monday = make_led("led_monday", monday_val + 3600, "HDWEEKEND");
    let (m7, r7, _) = HashMatcher::match_tier1(&[tx_weekend], &[led_monday]);
    assert_eq!(
        m7.len(),
        1,
        "Must match using Monday value_date despite Friday tx_date"
    );
    assert!(r7.is_empty());
}

// ===========================================================================
// 3. Bank Noise Stripping & Tokenization
// ===========================================================================

#[test]
fn test_empirical_bank_noise_stripping_and_tokenization() {
    let prefixes = [
        "MBVCB.123456789.CONG TY MINH ANH",
        "Napas VietQR TT FT242568912345 CONG TY MINH ANH",
        "QRIBFT chuyen khoan tu tk 987654321 CTY MINH ANH",
        "IBVCB chuyen tien den tk 123456 MINH ANH CO LTD",
        "CT TU TK 001100223344 TAP DOAN HOA PHAT",
        "CHUYEN KHOAN TU CONG TY TNHH PHAN MEM LIVA",
        "thanh toan tien hang hoa don HD102 CONG TY ABC",
    ];

    for raw in &prefixes {
        let stripped = strip_bank_narration_noise(raw);
        assert!(
            !stripped.contains("mbvcb"),
            "MBVCB must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("napas"),
            "Napas must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("vietqr"),
            "VietQR must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("qribft"),
            "QRIBFT must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("ibvcb"),
            "IBVCB must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("ct tu"),
            "CT TU must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("chuyen khoan tu"),
            "CHUYEN KHOAN TU must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("123456789"),
            "Account digits must be stripped: {}",
            stripped
        );
        assert!(
            !stripped.contains("ft242568912345"),
            "Trace code must be stripped: {}",
            stripped
        );
    }

    let score_minh_anh = compare_party_names(
        "Công ty TNHH Minh Anh",
        "MBVCB.123456789.012345.CT TU CONG TY TNHH MINH ANH THANH TOAN HD102",
    );
    assert!(
        score_minh_anh >= 0.85,
        "Minh Anh similarity must be >= 0.85, got {score_minh_anh}"
    );

    let score_hoa_phat = compare_party_names(
        "TẬP ĐOÀN HOÀ PHÁT",
        "Napas VietQR TT FT262568912345 Tu: CONG TY CP TAP DOAN HOA PHAT",
    );
    assert!(
        score_hoa_phat >= 0.85,
        "Hoa Phat similarity must be >= 0.85, got {score_hoa_phat}"
    );

    let score_liva = compare_party_names(
        "Công ty TNHH Giải pháp Phần mềm LIVA",
        "QRIBFT chuyen khoan tu tk 190311223344 CTY LIVA SOFTWARE",
    );
    assert!(
        score_liva >= 0.70,
        "LIVA software similarity must be >= 0.70, got {score_liva}"
    );

    // Legal stop forms are stripped properly
    let score_unrelated_1 =
        compare_party_names("CONG TY TNHH MINH ANH", "CONG TY TNHH PHUONG DONG");
    assert!(
        score_unrelated_1 < 0.70,
        "Unrelated companies sharing TNHH must score < 0.70, got {score_unrelated_1}"
    );

    let score_unrelated_tm = compare_party_names(
        "CÔNG TY CỔ PHẦN THƯƠNG MẠI MINH ANH",
        "CÔNG TY CỔ PHẦN THƯƠNG MẠI PHƯƠNG ĐÔNG",
    );
    assert!(
        score_unrelated_tm < 0.70,
        "Unrelated companies sharing TM CP must score < 0.70, got {score_unrelated_tm}"
    );
}

// ===========================================================================
// 4. Amount-Bucket Pre-indexing
// ===========================================================================

#[test]
fn test_empirical_amount_bucket_pre_indexing() {
    let now = 1_725_000_000i64;
    const BUCKET_SIZE: u64 = 11_000;

    let target_amount = 50_000_000u64;
    let target_bucket = target_amount / BUCKET_SIZE;

    let amt_diff_0 = target_amount;
    assert_eq!(amt_diff_0 / BUCKET_SIZE, target_bucket);

    let amt_diff_1100 = target_amount - 1_100;
    assert_eq!(amt_diff_1100 / BUCKET_SIZE, target_bucket);

    let amt_diff_11000 = target_amount - 11_000;
    assert_eq!(amt_diff_11000 / BUCKET_SIZE, target_bucket - 1);

    let probe_buckets = [target_bucket - 1, target_bucket, target_bucket + 1];
    assert!(probe_buckets.contains(&(amt_diff_0 / BUCKET_SIZE)));
    assert!(probe_buckets.contains(&(amt_diff_1100 / BUCKET_SIZE)));
    assert!(probe_buckets.contains(&(amt_diff_11000 / BUCKET_SIZE)));

    let make_fuzz_pair = |tx_amt: u64, led_amt: u64| {
        let tx = BankTransactionRow {
            id: "tx_fuzz".to_string(),
            statement_id: "stmt_fuzz".to_string(),
            account_id: "acc_fuzz".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: tx_amt,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY CP ABC".to_string()),
            counterparty_bank: None,
            narration: "CONG TY ABC THANH TOAN".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };
        let led = InternalLedgerEntry {
            id: "led_fuzz".to_string(),
            account_id: "acc_fuzz".to_string(),
            doc_no: "HD-FUZZ".to_string(),
            entry_date: now + 3600,
            entry_type: TransactionType::Credit,
            amount: led_amt,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty Cổ phần ABC".to_string()),
            description: "Ban hang ABC".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };
        (tx, led)
    };

    // Subtest 4.1: Diff = 0 -> Must match
    let (t1, l1) = make_fuzz_pair(50_000_000, 50_000_000);
    let (m1, _, _) = FuzzyMatcher::match_tier2(&[t1], &[l1], &[0], &[0]);
    assert_eq!(m1.len(), 1);
    assert_eq!(m1[0].discrepancy_amount, 0);

    // Subtest 4.2: Diff = 1,100 -> Must match
    let (t2, l2) = make_fuzz_pair(49_998_900, 50_000_000);
    let (m2, _, _) = FuzzyMatcher::match_tier2(&[t2], &[l2], &[0], &[0]);
    assert_eq!(m2.len(), 1);
    assert_eq!(m2[0].discrepancy_amount, 1_100);

    // Subtest 4.3: Diff = 11,000 -> Must match
    let (t3, l3) = make_fuzz_pair(49_989_000, 50_000_000);
    let (m3, _, _) = FuzzyMatcher::match_tier2(&[t3], &[l3], &[0], &[0]);
    assert_eq!(m3.len(), 1);
    assert_eq!(m3[0].discrepancy_amount, 11_000);

    // Subtest 4.4: Diff = 1,099 -> Out of standard fee range -> Must NOT match
    let (t4, l4) = make_fuzz_pair(49_998_901, 50_000_000);
    let (m4, _, _) = FuzzyMatcher::match_tier2(&[t4], &[l4], &[0], &[0]);
    assert_eq!(m4.len(), 0);

    // Subtest 4.5: Diff = 11,001 -> Out of standard fee range -> Must NOT match
    let (t5, l5) = make_fuzz_pair(49_988_999, 50_000_000);
    let (m5, _, _) = FuzzyMatcher::match_tier2(&[t5], &[l5], &[0], &[0]);
    assert_eq!(m5.len(), 0);

    // Subtest 4.6: Scalability test with 500 entries across distant buckets
    let mut distant_ledger = Vec::new();
    for i in 1..=500 {
        distant_ledger.push(InternalLedgerEntry {
            id: format!("led_dist_{i}"),
            account_id: "acc_fuzz".to_string(),
            doc_no: format!("HD{i}"),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: (i as u64) * 1_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty Cổ phần ABC".to_string()),
            description: "".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        });
    }

    let query_tx = BankTransactionRow {
        id: "tx_query".to_string(),
        statement_id: "stmt_fuzz".to_string(),
        account_id: "acc_fuzz".to_string(),
        bank_code: "TCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 49_989_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY CP ABC".to_string()),
        counterparty_bank: None,
        narration: "CONG TY ABC THANH TOAN".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let unalloc_led: Vec<usize> = (0..distant_ledger.len()).collect();
    let (matches, rem_tx, rem_led) =
        FuzzyMatcher::match_tier2(&[query_tx], &distant_ledger, &[0], &unalloc_led);

    assert_eq!(
        matches.len(),
        1,
        "Must find exactly the target entry via bucket indexing"
    );
    assert_eq!(matches[0].ledger_entry_ids, vec!["led_dist_50"]);
    assert_eq!(matches[0].discrepancy_amount, 11_000);
    assert_eq!(rem_tx.len(), 0);
    assert_eq!(rem_led.len(), 499);
}

// ===========================================================================
// 5. Tier 3 Split Solver & Branch-and-Bound Bounds
// ===========================================================================

#[test]
fn test_empirical_tier3_split_solver_and_bnb_bounds() {
    let now = 1_725_000_000i64;

    // Subtest 5.1: 1-to-N composite invoices (1 Bank Tx -> 3 Ledger Invoices)
    let tx_composite = BankTransactionRow {
        id: "tx_comp_1".to_string(),
        statement_id: "stmt_s".to_string(),
        account_id: "acc_s".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 90_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY ABC".to_string()),
        counterparty_bank: None,
        narration: "CK THANH TOAN HD 101 HD 102 HD 103".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let invs = vec![
        InternalLedgerEntry {
            id: "inv_101".to_string(),
            account_id: "acc_s".to_string(),
            doc_no: "HD101".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 20_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: "".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "inv_102".to_string(),
            account_id: "acc_s".to_string(),
            doc_no: "HD102".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 30_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: "".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "inv_103".to_string(),
            account_id: "acc_s".to_string(),
            doc_no: "HD103".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 40_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: "".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
    ];

    let (m_comp, hitl_comp) = SplitSolver::match_tier3(&[tx_composite], &invs, &[0], &[0, 1, 2]);

    assert_eq!(m_comp.len(), 1);
    assert_eq!(hitl_comp.len(), 0);
    assert_eq!(m_comp[0].matched_amount, 90_000_000);
    assert_eq!(m_comp[0].discrepancy_amount, 0);
    assert_eq!(m_comp[0].ledger_entry_ids.len(), 3);
    assert!(m_comp[0].is_split_one_to_n());

    // Subtest 5.2: N-to-1 multi-installment payments (4 Bank Txs -> 1 Ledger Invoice)
    let inv_install = InternalLedgerEntry {
        id: "inv_inst_main".to_string(),
        account_id: "acc_s".to_string(),
        doc_no: "INV-SPLIT-4".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 150_000_000,
        partner_code: Some("XYZ".to_string()),
        partner_name: Some("Cong ty XYZ".to_string()),
        description: "Hop dong lon".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let tx_insts = vec![
        BankTransactionRow {
            id: "tx_ins_1".to_string(),
            statement_id: "stmt_s".to_string(),
            account_id: "acc_s".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 10,
            value_date: now + 10,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 30_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY XYZ".to_string()),
            counterparty_bank: None,
            narration: "Thanh toan dot 1 INV-SPLIT-4".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx_ins_2".to_string(),
            statement_id: "stmt_s".to_string(),
            account_id: "acc_s".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 20,
            value_date: now + 20,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 40_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY XYZ".to_string()),
            counterparty_bank: None,
            narration: "Thanh toan dot 2 INV-SPLIT-4".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx_ins_3".to_string(),
            statement_id: "stmt_s".to_string(),
            account_id: "acc_s".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 30,
            value_date: now + 30,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 50_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY XYZ".to_string()),
            counterparty_bank: None,
            narration: "Thanh toan dot 3 INV-SPLIT-4".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx_ins_4".to_string(),
            statement_id: "stmt_s".to_string(),
            account_id: "acc_s".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 40,
            value_date: now + 40,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 30_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY XYZ".to_string()),
            counterparty_bank: None,
            narration: "Thanh toan dot 4 INV-SPLIT-4".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
    ];

    let (m_ins, hitl_ins) =
        SplitSolver::match_tier3(&tx_insts, &[inv_install], &[0, 1, 2, 3], &[0]);

    assert_eq!(m_ins.len(), 1, "N-to-1 4-installment match must succeed");
    assert_eq!(hitl_ins.len(), 0);
    assert_eq!(m_ins[0].matched_amount, 150_000_000);
    assert_eq!(m_ins[0].bank_tx_ids.len(), 4);
    assert!(m_ins[0].is_split_n_to_one());

    // Subtest 5.3: Depth 4 Contract vs Depth up to 8
    let pool_5: Vec<(usize, u64)> = (0..5).map(|i| (i, 10_000_000)).collect();
    let target_50 = 50_000_000u64;

    let res_d4 = solve_exact_subset_sum_bnb(&pool_5, target_50, DEFAULT_MAX_SPLIT_DEPTH);
    assert!(
        res_d4.is_none(),
        "Depth 4 must fail on 5-item combination (contract preservation)"
    );

    let res_d5 = solve_exact_subset_sum_bnb(&pool_5, target_50, 5);
    assert!(res_d5.is_some(), "Depth 5 must find 5-item combination");
    assert_eq!(res_d5.unwrap().len(), 5);

    let pool_8: Vec<(usize, u64)> = vec![
        (0, 5_000_000),
        (1, 5_000_000),
        (2, 10_000_000),
        (3, 15_000_000),
        (4, 15_000_000),
        (5, 15_000_000),
        (6, 15_000_000),
        (7, 20_000_000),
    ];
    let target_100 = 100_000_000u64;
    let res_d8 = solve_exact_subset_sum_bnb(&pool_8, target_100, MAX_SUPPORTED_SPLIT_DEPTH);
    assert!(res_d8.is_some(), "Depth 8 must find 8-item combination");
    let sol8 = res_d8.unwrap();
    assert_eq!(sol8.len(), 8);
    let sum8: u64 = sol8.iter().map(|&i| pool_8[i].1).sum();
    assert_eq!(sum8, target_100);

    // Subtest 5.4: Robustness & Zero Panics
    let impossible = solve_exact_subset_sum_bnb(&pool_5, 999_999_999, 8);
    assert!(
        impossible.is_none(),
        "Impossible target must return None without panicking"
    );

    let zero_tgt = solve_exact_subset_sum_bnb(&pool_5, 0, 8);
    assert!(
        zero_tgt.is_none(),
        "Target 0 must return None without panicking"
    );

    let empty_cand = solve_exact_subset_sum_bnb(&[], 50_000_000, 8);
    assert!(
        empty_cand.is_none(),
        "Empty candidates must return None without panicking"
    );

    let zero_amt_pool = vec![(0, 0u64), (1, 0u64)];
    let res_zero_amt = solve_exact_subset_sum_bnb(&zero_amt_pool, 10_000, 8);
    assert!(res_zero_amt.is_none());

    let large_pool: Vec<(usize, u64)> = (0..60)
        .map(|i| (i, 20_000_000 + (i as u64) * 1_000_000))
        .collect();
    let start_instant = std::time::Instant::now();
    let unachievable = solve_exact_subset_sum_bnb(&large_pool, 15_000_000, 8);
    assert!(unachievable.is_none());
    assert!(
        start_instant.elapsed().as_millis() < 50,
        "Pruning bounds must return immediately"
    );
}

// ===========================================================================
// 6. Empirical Vulnerability Reproduction: Unstripped Industry Descriptor False Positive
// ===========================================================================

#[test]
fn test_empirical_vulnerability_unstripped_industry_descriptors_cause_false_positive() {
    // Two completely unrelated entities sharing "CONG NGHE" (Technology) which is not
    // included in `strip_corporate_legal_noise`'s hardcoded phrase list.
    let party_a = "CÔNG TY CỔ PHẦN CÔNG NGHỆ THÁI BÌNH";
    let party_b = "CÔNG TY CỔ PHẦN CÔNG NGHỆ SÔNG HỒNG";

    let score = compare_party_names(party_a, party_b);

    // M4 HARDENING VERIFICATION:
    // With generic industry descriptors stripped, core1 is "thai binh" and core2 is "song hong".
    // 1. Zero token overlap between "thai binh" and "song hong".
    // 2. Jaro-Winkler score is strictly < 0.70 (measured: ~0.444).
    assert!(
        score < 0.70,
        "Hardened in M4: Industry descriptor stripped, score {score} < 0.70"
    );

    // CONSEQUENCE IN FUZZY MATCHER:
    // Tier 2 Fuzzy Matcher must correctly REJECT matching unrelated entities!
    let now = 1_725_000_000i64;
    let tx_thai_binh = BankTransactionRow {
        id: "tx_thai_binh".to_string(),
        statement_id: "stmt_vuln".to_string(),
        account_id: "acc_vuln".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 100_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some(party_a.to_string()),
        counterparty_bank: None,
        narration: format!("{party_a} THANH TOAN"),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let ledger_song_hong = InternalLedgerEntry {
        id: "led_song_hong".to_string(),
        account_id: "acc_vuln".to_string(),
        doc_no: "HD-SONG-HONG".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 100_000_000,
        partner_code: Some("SH01".to_string()),
        partner_name: Some(party_b.to_string()),
        description: "Hop dong Song Hong".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (matches, rem_tx, _) =
        FuzzyMatcher::match_tier2(&[tx_thai_binh], &[ledger_song_hong], &[0], &[0]);

    // Confirms that FuzzyMatcher correctly rejects false positive between
    // Thai Binh and Song Hong in M4!
    assert_eq!(
        matches.len(),
        0,
        "Hardened in M4: FuzzyMatcher correctly rejected false positive between unrelated companies Thai Binh and Song Hong"
    );
    assert_eq!(rem_tx.len(), 1);
}
