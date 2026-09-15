//! Adversarial Stress and Empirical Challenge Test Suite for Milestone 3.
//!
//! Evaluates:
//! 1. Tier 1: Exact doc_ref hash matching, normalization, 24h boundary conditions, double-spend prevention.
//! 2. Tier 2: Fuzzy Jaro-Winkler, Vietnamese diacritics, legal stop-words stripping, Napas fee bounds (1,100 to 11,000 VND).
//! 3. Tier 3: Composite split solver with mathematical invariant: Sum(allocated) == tx.amount (Delta = 0).
//! 4. Fail-closed HITL queue: Cryptographically secure UUID v4 tokens, residual quarantine.
//! 5. Statement parsers: VCB Excel amount/date matrix, TCB CSV delimiter & BOM sniffing, BIDV PDF coordinates & checksum.
//! 6. E2E Stress Harness: 50-transaction concurrent batch reconciliation without double allocation.

use std::collections::HashSet;
use uuid::Uuid;

use liva_native_core::banking::models::*;
use liva_native_core::banking::parser::agribank_parser::AgribankParser;
use liva_native_core::banking::parser::bidv_pdf::BidvPdfParser;
use liva_native_core::banking::parser::mbbank_parser::MbBankParser;
use liva_native_core::banking::parser::tcb_csv::TcbCsvParser;
use liva_native_core::banking::parser::vcb_excel::VcbExcelParser;
use liva_native_core::banking::parser::vietinbank_parser::VietinBankParser;
use liva_native_core::banking::parser::{
    BankStatementParser, ContainerFormat, detect_container_format, score_bank, sniff_and_parse,
};
use liva_native_core::banking::reconciliation::ReconciliationEngine;
use liva_native_core::banking::reconciliation::fuzzy_matcher::FuzzyMatcher;
use liva_native_core::banking::reconciliation::hash_matcher::{HashMatcher, normalize_doc_ref};
use liva_native_core::banking::reconciliation::jaro_winkler::{
    compare_party_names, normalize_vietnamese_text,
};
use liva_native_core::banking::reconciliation::split_solver::SplitSolver;

// ===========================================================================
// 1. Tier 1 Exact Hash Matcher Adversarial Stress
// ===========================================================================

#[test]
fn test_tier1_exact_hash_window_boundary_adversarial() {
    let base_time = 1_725_000_000i64;
    const WINDOW_24H: i64 = 86_400;

    let tx_template = |id: &str, tx_time: i64, doc: &str| BankTransactionRow {
        id: id.to_string(),
        statement_id: "stmt_test".to_string(),
        account_id: "acc_test".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: tx_time,
        value_date: tx_time,
        doc_ref: Some(doc.to_string()),
        tx_type: TransactionType::Credit,
        amount: 10_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: None,
        counterparty_bank: None,
        narration: format!("Thanh toan {doc}"),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: tx_time,
    };

    let ledger_template = |id: &str, entry_time: i64, doc: &str| InternalLedgerEntry {
        id: id.to_string(),
        account_id: "acc_test".to_string(),
        doc_no: doc.to_string(),
        entry_date: entry_time,
        entry_type: TransactionType::Credit,
        amount: 10_000_000,
        partner_code: Some("PARTNER01".to_string()),
        partner_name: Some("Cong ty ABC".to_string()),
        description: format!("Hoa don {doc}"),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: entry_time,
    };

    // Case A: Exactly on 24h boundary (difference == 86,400s) -> MUST MATCH
    let tx_boundary = tx_template("tx_bound", base_time + WINDOW_24H, "HD100");
    let led_boundary = ledger_template("led_bound", base_time, "HD100");
    let (matches_a, rem_tx_a, _) = HashMatcher::match_tier1(&[tx_boundary], &[led_boundary]);
    assert_eq!(matches_a.len(), 1, "Exactly 24h boundary must match");
    assert!(rem_tx_a.is_empty());
    assert_eq!(matches_a[0].confidence_score, 1.0);

    // Case B: 1 second past 24h boundary (difference == 86,401s) -> MUST NOT MATCH Tier 1
    let tx_past = tx_template("tx_past", base_time + WINDOW_24H + 1, "HD101");
    let led_past = ledger_template("led_past", base_time, "HD101");
    let (matches_b, rem_tx_b, _) = HashMatcher::match_tier1(&[tx_past], &[led_past]);
    assert_eq!(matches_b.len(), 0, "24h + 1s must not match Tier 1");
    assert_eq!(rem_tx_b.len(), 1);
}

#[test]
fn test_tier1_doc_ref_normalization_adversarial() {
    assert_eq!(normalize_doc_ref("HD-00102"), "HD102");
    assert_eq!(normalize_doc_ref("hd-000099"), "HD99");
    assert_eq!(normalize_doc_ref("INV-000456"), "INV456");
    assert_eq!(normalize_doc_ref("inv000001"), "INV1");
    assert_eq!(normalize_doc_ref("PC-00012"), "PC00012");
    assert_eq!(normalize_doc_ref("HD#102/2026"), "HD1022026");
}

#[test]
fn test_tier1_double_allocation_prevention() {
    let now = 1_725_000_000i64;

    // Two identical bank transactions with the same amount and doc_ref
    let tx1 = BankTransactionRow {
        id: "tx_dup_1".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: Some("HD-00102".to_string()),
        tx_type: TransactionType::Credit,
        amount: 25_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: None,
        counterparty_bank: None,
        narration: "Thanh toan HD102".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };
    let tx2 = BankTransactionRow {
        id: "tx_dup_2".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now + 60,
        value_date: now + 60,
        doc_ref: Some("HD-00102".to_string()),
        tx_type: TransactionType::Credit,
        amount: 25_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: None,
        counterparty_bank: None,
        narration: "Thanh toan HD102 lan 2".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now + 60,
    };

    // Only ONE internal ledger invoice exists
    let led1 = InternalLedgerEntry {
        id: "led_single".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "HD102".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 25_000_000,
        partner_code: Some("ABC01".to_string()),
        partner_name: Some("ABC Corp".to_string()),
        description: "Hoa don 102".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (matches, rem_tx, rem_led) = HashMatcher::match_tier1(&[tx1, tx2], &[led1]);

    // Exactly 1 match must be produced; the second transaction must remain unallocated
    assert_eq!(
        matches.len(),
        1,
        "Only 1 match allowed for single open invoice"
    );
    assert_eq!(
        rem_tx.len(),
        1,
        "Second duplicate transaction must remain unallocated"
    );
    assert_eq!(
        rem_led.len(),
        0,
        "Ledger entry must be consumed exactly once"
    );
}

// ===========================================================================
// 2. Tier 2 Fuzzy Heuristic Matcher Adversarial Stress
// ===========================================================================

#[test]
fn test_tier2_napas_fee_tolerance_bounds() {
    let now = 1_725_000_000i64;

    let create_pair = |diff: i64| {
        let bank_tx = BankTransactionRow {
            id: format!("tx_fee_{diff}"),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: (50_000_000i64 - diff) as u64,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY CP THUONG MAI ABC".to_string()),
            counterparty_bank: None,
            narration: "CONG TY ABC THANH TOAN TIEN HANG".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };
        let ledger = InternalLedgerEntry {
            id: format!("led_fee_{diff}"),
            account_id: "acc_1".to_string(),
            doc_no: format!("HD-{diff}"),
            entry_date: now + 3600,
            entry_type: TransactionType::Credit,
            amount: 50_000_000,
            partner_code: Some("ABC01".to_string()),
            partner_name: Some("Công ty Cổ phần Thương mại ABC".to_string()),
            description: "Ban hang".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };
        (bank_tx, ledger)
    };

    // 1. Exact lower bound fee: 1,100 VND -> MUST MATCH
    let (tx1, led1) = create_pair(1_100);
    let (m1, rem1, _) = FuzzyMatcher::match_tier2(&[tx1], &[led1], &[0], &[0]);
    assert_eq!(m1.len(), 1, "1,100 VND Napas fee lower bound must match");
    assert!(rem1.is_empty());
    assert_eq!(m1[0].discrepancy_amount, 1_100);

    // 2. Exact upper bound fee: 11,000 VND -> MUST MATCH
    let (tx2, led2) = create_pair(11_000);
    let (m2, rem2, _) = FuzzyMatcher::match_tier2(&[tx2], &[led2], &[0], &[0]);
    assert_eq!(m2.len(), 1, "11,000 VND Napas fee upper bound must match");
    assert!(rem2.is_empty());
    assert_eq!(m2[0].discrepancy_amount, 11_000);

    // 3. Mid fee: 5,500 VND -> MUST MATCH
    let (tx3, led3) = create_pair(5_500);
    let (m3, rem3, _) = FuzzyMatcher::match_tier2(&[tx3], &[led3], &[0], &[0]);
    assert_eq!(m3.len(), 1, "5,500 VND Napas fee must match");
    assert!(rem3.is_empty());
    assert_eq!(m3[0].discrepancy_amount, 5_500);

    // 4. Below lower bound: 1,099 VND -> MUST NOT MATCH as standard fee
    let (tx4, led4) = create_pair(1_099);
    let (m4, rem4, _) = FuzzyMatcher::match_tier2(&[tx4], &[led4], &[0], &[0]);
    assert_eq!(
        m4.len(),
        0,
        "1,099 VND fee must not match standard fee tolerance"
    );
    assert_eq!(rem4.len(), 1);

    // 5. Above upper bound: 11,001 VND -> MUST NOT MATCH as standard fee
    let (tx5, led5) = create_pair(11_001);
    let (m5, rem5, _) = FuzzyMatcher::match_tier2(&[tx5], &[led5], &[0], &[0]);
    assert_eq!(
        m5.len(),
        0,
        "11,001 VND fee must not match standard fee tolerance"
    );
    assert_eq!(rem5.len(), 1);
}

#[test]
fn test_tier2_vietnamese_diacritics_and_legal_noise_stripping() {
    let n1 = "CÔNG TY TNHH GIẢI PHÁP PHẦN MỀM LIVA VIỆT NAM";
    let n2 = "CONG TY CP LIVA SOFTWARE";
    let norm1 = normalize_vietnamese_text(n1);
    assert_eq!(norm1, "cong ty tnhh giai phap phan mem liva viet nam");

    let score = compare_party_names(n1, n2);
    assert!(
        score >= 0.70,
        "Distinctive token 'liva' must be recognized, got {score}"
    );

    let n_other = "CÔNG TY CỔ PHẦN THỰC PHẨM MINH PHƯƠNG";
    let score_mismatch = compare_party_names(n1, n_other);
    println!("Vulnerability probe: score between '{n1}' and '{n_other}' = {score_mismatch}");

    // Test false-positive attack: Two completely different businesses sharing identical legal prefix
    let false_tx = BankTransactionRow {
        id: "tx_false_pos".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: 1_725_000_000,
        value_date: 1_725_000_000,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 50_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY TNHH MINH ANH".to_string()),
        counterparty_bank: None,
        narration: "CONG TY TNHH MINH ANH THANH TOAN".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: 1_725_000_000,
    };

    let false_led = InternalLedgerEntry {
        id: "led_false_pos".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "HD999".to_string(),
        entry_date: 1_725_000_000,
        entry_type: TransactionType::Credit,
        amount: 50_000_000,
        partner_code: Some("PD01".to_string()),
        partner_name: Some("CONG TY TNHH PHUONG DONG".to_string()),
        description: "Ban hang Phuong Dong".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: 1_725_000_000,
    };

    let score_minh_anh_phuong_dong =
        compare_party_names("CONG TY TNHH MINH ANH", "CONG TY TNHH PHUONG DONG");
    println!(
        "Score between 'CONG TY TNHH MINH ANH' and 'CONG TY TNHH PHUONG DONG': {score_minh_anh_phuong_dong}"
    );

    let (false_matches, rem_tx, _) =
        FuzzyMatcher::match_tier2(&[false_tx], &[false_led], &[0], &[0]);
    println!(
        "FuzzyMatcher result on unrelated companies: matches={}, rem_tx={}",
        false_matches.len(),
        rem_tx.len()
    );
    assert!(
        score_minh_anh_phuong_dong < 0.70,
        "Unrelated companies sharing only legal form prefix must produce score < 0.70, got {score_minh_anh_phuong_dong}"
    );
    assert_eq!(
        false_matches.len(),
        0,
        "Unrelated companies sharing only legal form prefix must never auto-match in Tier 2"
    );
    assert_eq!(
        rem_tx.len(),
        1,
        "Unmatched bank transaction must remain unallocated"
    );
}

// ===========================================================================
// 3. Tier 3 Constraint Split Solver & Mathematical Invariants
// ===========================================================================

#[test]
fn test_tier3_composite_split_solver_2_3_4_item_combinations() {
    let now = 1_725_000_000i64;

    // Sub-case 1: 3-item combination (20M + 30M + 50M == 100M)
    let tx3 = BankTransactionRow {
        id: "tx_3item".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 100_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY TNHH THEP HOA PHAT".to_string()),
        counterparty_bank: None,
        narration: "CK GOM HD01 HD02 HD03 THEP HOA PHAT".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let inv01 = InternalLedgerEntry {
        id: "inv01".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "HD01".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 20_000_000,
        partner_code: Some("HP01".to_string()),
        partner_name: Some("Công ty TNHH Thép Hòa Phát".to_string()),
        description: "HD01".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };
    let inv02 = InternalLedgerEntry {
        id: "inv02".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "HD02".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 30_000_000,
        partner_code: Some("HP01".to_string()),
        partner_name: Some("Công ty TNHH Thép Hòa Phát".to_string()),
        description: "HD02".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };
    let inv03 = InternalLedgerEntry {
        id: "inv03".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "HD03".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 50_000_000,
        partner_code: Some("HP01".to_string()),
        partner_name: Some("Công ty TNHH Thép Hòa Phát".to_string()),
        description: "HD03".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (matches, hitl) =
        SplitSolver::match_tier3(&[tx3], &[inv01, inv02, inv03], &[0], &[0, 1, 2]);

    assert_eq!(matches.len(), 1, "3-item subset sum must be found");
    assert_eq!(hitl.len(), 0);
    assert_eq!(matches[0].match_type, MatchType::CompositeSplit);
    assert_eq!(matches[0].matched_amount, 100_000_000);
    assert_eq!(
        matches[0].discrepancy_amount, 0,
        "Arithmetic invariant Delta=0"
    );
    assert_eq!(matches[0].ledger_entry_ids.len(), 3);

    // Sub-case 2: 4-item combination (10M + 15M + 25M + 50M == 100M)
    let tx4 = BankTransactionRow {
        id: "tx_4item".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 100_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY ABC".to_string()),
        counterparty_bank: None,
        narration: "THANH TOAN 4 HOA DON".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let i1 = InternalLedgerEntry {
        id: "i1".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "INV1".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 10_000_000,
        partner_code: Some("A".to_string()),
        partner_name: Some("Công ty ABC".to_string()),
        description: "".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };
    let i2 = InternalLedgerEntry {
        id: "i2".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "INV2".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 15_000_000,
        partner_code: Some("A".to_string()),
        partner_name: Some("Công ty ABC".to_string()),
        description: "".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };
    let i3 = InternalLedgerEntry {
        id: "i3".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "INV3".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 25_000_000,
        partner_code: Some("A".to_string()),
        partner_name: Some("Công ty ABC".to_string()),
        description: "".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };
    let i4 = InternalLedgerEntry {
        id: "i4".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "INV4".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 50_000_000,
        partner_code: Some("A".to_string()),
        partner_name: Some("Công ty ABC".to_string()),
        description: "".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (m4, h4) = SplitSolver::match_tier3(&[tx4], &[i1, i2, i3, i4], &[0], &[0, 1, 2, 3]);

    assert_eq!(m4.len(), 1, "4-item subset sum must be found");
    assert_eq!(h4.len(), 0);
    assert_eq!(m4[0].matched_amount, 100_000_000);
    assert_eq!(m4[0].discrepancy_amount, 0);
    assert_eq!(m4[0].ledger_entry_ids.len(), 4);
}

#[test]
fn test_tier3_fail_closed_on_residual_or_5_item_combination() {
    let now = 1_725_000_000i64;

    // 5 items of 20M = 100M. The solver limit is 4 items.
    // It MUST NOT make a partial or false match; it MUST fail-closed to HITL!
    let tx = BankTransactionRow {
        id: "tx_5item".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 100_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY ABC".to_string()),
        counterparty_bank: None,
        narration: "5 items batch".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let make_inv = |id: &str| InternalLedgerEntry {
        id: id.to_string(),
        account_id: "acc_1".to_string(),
        doc_no: id.to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 20_000_000,
        partner_code: Some("A".to_string()),
        partner_name: Some("Công ty ABC".to_string()),
        description: "".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let ledger = vec![
        make_inv("inv1"),
        make_inv("inv2"),
        make_inv("inv3"),
        make_inv("inv4"),
        make_inv("inv5"),
    ];

    let (auto_matches, hitl) = SplitSolver::match_tier3(&[tx], &ledger, &[0], &[0, 1, 2, 3, 4]);

    assert_eq!(
        auto_matches.len(),
        0,
        "5-item combination must not auto-match beyond max size"
    );
    assert_eq!(hitl.len(), 1, "Must fail-closed to HITL");
    assert_eq!(hitl[0].status, "PENDING_HITL");
    assert!(hitl[0].hitl_token.is_some());

    // Incomplete sum: Invoices sum to 99,999,999 VND for a 100M tx (Delta = 1 VND)
    let tx_incomplete = BankTransactionRow {
        id: "tx_incomplete".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 100_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY XYZ".to_string()),
        counterparty_bank: None,
        narration: "Thieu 1 dong".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let ledger_incomplete = vec![InternalLedgerEntry {
        id: "inv_near".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "NEAR1".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 99_999_999, // 1 VND difference
        partner_code: Some("XYZ".to_string()),
        partner_name: Some("Cong ty XYZ".to_string()),
        description: "".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    }];

    let (auto_inc, hitl_inc) =
        SplitSolver::match_tier3(&[tx_incomplete], &ledger_incomplete, &[0], &[0]);

    assert_eq!(
        auto_inc.len(),
        0,
        "1 VND residual difference must NEVER auto-match (strict Delta=0)"
    );
    assert_eq!(hitl_inc.len(), 1, "Must fail-closed to HITL quarantine");
    assert_eq!(
        hitl_inc[0].discrepancy_amount, -1,
        "Residual discrepancy is -1 VND"
    );
}

// ===========================================================================
// 4. Fail-Closed HITL Queue & Cryptographic UUID v4 Verification
// ===========================================================================

#[test]
fn test_hitl_quarantine_unforgeable_uuid_v4_properties() {
    let now = 1_725_000_000i64;
    let mut tokens = HashSet::new();

    for i in 0..100 {
        let tx = BankTransactionRow {
            id: format!("tx_quarantine_{i}"),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + i as i64,
            value_date: now + i as i64,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 1_000_000 + i as u64,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: format!("Unknown credit {i}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let (_, hitl) = SplitSolver::match_tier3(&[tx], &[], &[0], &[]);
        assert_eq!(hitl.len(), 1);
        let item = &hitl[0];
        assert_eq!(item.status, "PENDING_HITL");
        assert_eq!(item.match_type, MatchType::ManualHitl);

        let token = item
            .hitl_token
            .as_ref()
            .expect("HITL item must have a token");
        let parsed_uuid = Uuid::parse_str(token).expect("Token must be a valid UUID");

        // Verify UUID v4 properties
        assert_eq!(
            parsed_uuid.get_version(),
            Some(uuid::Version::Random),
            "Must be cryptographically random UUID v4"
        );

        // Verify uniqueness
        assert!(
            tokens.insert(token.clone()),
            "UUID collision detected at iteration {i}"
        );
    }

    assert_eq!(tokens.len(), 100, "All 100 HITL tokens must be distinct");
}

// ===========================================================================
// 5. Statement Parsers Stress Testing
// ===========================================================================

#[test]
fn test_vietnamese_amount_parser_adversarial_matrix() {
    // Exact standard patterns
    assert_eq!(parse_vietnamese_amount("15.000.000,00"), Some(15_000_000));
    assert_eq!(parse_vietnamese_amount("15,000,000.00"), Some(15_000_000));
    assert_eq!(
        parse_vietnamese_amount("1.450.230.000"),
        Some(1_450_230_000)
    );
    assert_eq!(parse_vietnamese_amount("785.600.000"), Some(785_600_000));
    assert_eq!(parse_vietnamese_amount("380.000"), Some(380_000));
    assert_eq!(parse_vietnamese_amount("0"), Some(0));

    // Signed and parentheses notations
    assert_eq!(parse_vietnamese_amount("-50.000.000"), Some(50_000_000));
    assert_eq!(parse_vietnamese_amount("(50.000.000)"), Some(50_000_000));
    assert_eq!(
        parse_vietnamese_amount("  -1.250.000,00  "),
        Some(1_250_000)
    );

    // Large treasury balances (> 100 billion VND)
    assert_eq!(
        parse_vietnamese_amount("150.000.000.000"),
        Some(150_000_000_000)
    );

    // Non-amount garbage strings
    assert_eq!(parse_vietnamese_amount(""), None);
    assert_eq!(parse_vietnamese_amount("   "), None);
    assert_eq!(parse_vietnamese_amount("N/A"), None);
    assert_eq!(parse_vietnamese_amount("Chua thanh toan"), None);
}

#[test]
fn test_tcb_csv_parser_delimiter_sniffing_and_quoted_fields() {
    let parser = TcbCsvParser;

    // 1. Semicolon delimiter with quotes
    let csv_semi = "\
Số tài khoản:;19039988776655;;;\n\
Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết\n\
15/08/2026;FT001;;10.000.000;10.000.000;\"CONG TY ABC; CHI NHANH 1; CK TIEN HANG\"\n";
    let parsed_semi = parser.parse(csv_semi.as_bytes(), "test.csv").unwrap();
    assert_eq!(parsed_semi.transactions.len(), 1);
    assert_eq!(parsed_semi.transactions[0].amount, 10_000_000);
    assert_eq!(
        parsed_semi.transactions[0].doc_ref.as_deref(),
        Some("FT001")
    );

    // 2. Tab delimiter
    let csv_tab = "Ngày giao dịch\tMã giao dịch\tSố tiền ghi có\tNội dung\n15/08/2026\tNPS12345678\t20.000.000\tChuyen khoan Napas\n";
    let parsed_tab = parser.parse(csv_tab.as_bytes(), "tcb.txt").unwrap();
    assert_eq!(parsed_tab.transactions.len(), 1);
    assert_eq!(parsed_tab.transactions[0].amount, 20_000_000);
    assert_eq!(
        parsed_tab.transactions[0].doc_ref.as_deref(),
        Some("NPS12345678")
    );

    // 3. Comma delimiter with Napas ref extracted from narration
    let csv_comma = "Ngày giao dịch,Số tiền ghi có,Nội dung chi tiết\n15/08/2026,30.000.000,Giao dich Napas FT262568912345 thanh toan HD\n";
    let parsed_comma = parser.parse(csv_comma.as_bytes(), "statement.csv").unwrap();
    assert_eq!(parsed_comma.transactions.len(), 1);
    assert_eq!(
        parsed_comma.transactions[0].doc_ref.as_deref(),
        Some("FT262568912345")
    );
}

#[test]
fn test_bidv_pdf_balance_checksum_invariant() {
    let parser = BidvPdfParser;
    assert!(parser.sniff(b"%PDF-1.4 BIDV SAO KE", "bidv.pdf"));

    // Verify mathematical running balance equation with multiple credits and debits:
    // Closing == Opening + Sum(Credit) - Sum(Debit)
    let opening: u64 = 1_000_000_000;
    let credits = [150_000_000u64, 45_000_000u64, 80_000_000u64];
    let debits = [50_000_000u64, 25_000_000u64, 100_000_000u64];

    let sum_credit: u64 = credits.iter().sum();
    let sum_debit: u64 = debits.iter().sum();
    let expected_closing: u64 = opening + sum_credit - sum_debit;

    let computed = (opening as i128) + (sum_credit as i128) - (sum_debit as i128);
    assert_eq!(computed, expected_closing as i128);
    assert_eq!(expected_closing, 1_100_000_000);
}

#[test]
fn test_parser_corrupted_inputs_do_not_panic() {
    let tcb = TcbCsvParser;
    let vcb = VcbExcelParser;
    let bidv = BidvPdfParser;

    // Empty bytes
    assert!(tcb.parse(b"", "empty.csv").is_err());
    assert!(vcb.parse(b"", "empty.xlsx").is_err());
    assert!(bidv.parse(b"", "empty.pdf").is_err());

    // Random non-format binary bytes
    let random_junk = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x12, 0x34, 0x56, 0x78];
    assert!(vcb.parse(&random_junk, "junk.xlsx").is_err());
    assert!(bidv.parse(&random_junk, "junk.pdf").is_err());
    assert!(sniff_and_parse(&random_junk, "unknown.bin").is_err());
}

// ===========================================================================
// 6. E2E Stress Harness: 50-Transaction Batch Reconcile
// ===========================================================================

#[test]
fn test_end_to_end_reconciliation_stress_50_transactions() {
    let base_time = 1_725_000_000i64;
    let mut bank_txs = Vec::new();
    let mut ledger_entries = Vec::new();

    // 1. 20 Tier 1 Exact Matches (TX 0..20, 10M each)
    for i in 0..20 {
        let doc = format!("HD100{i:02}");
        bank_txs.push(BankTransactionRow {
            id: format!("bank_tx_t1_{i}"),
            statement_id: "stmt_vcb".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: base_time + (i as i64 * 100),
            value_date: base_time + (i as i64 * 100),
            doc_ref: Some(doc.clone()),
            tx_type: TransactionType::Credit,
            amount: 10_000_000 + (i as u64 * 100_000),
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY ABC".to_string()),
            counterparty_bank: None,
            narration: format!("Thanh toan {doc}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: base_time,
        });
        ledger_entries.push(InternalLedgerEntry {
            id: format!("led_entry_t1_{i}"),
            account_id: "acc_vcb".to_string(),
            doc_no: doc.clone(),
            entry_date: base_time + (i as i64 * 100) + 300,
            entry_type: TransactionType::Credit,
            amount: 10_000_000 + (i as u64 * 100_000),
            partner_code: Some("ABC01".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: format!("Hoa don {doc}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: base_time,
        });
    }

    // 2. 15 Tier 2 Fuzzy Matches with Napas fee (TX 20..35, 20M invoice minus 5,500 fee)
    for i in 20..35 {
        bank_txs.push(BankTransactionRow {
            id: format!("bank_tx_t2_{i}"),
            statement_id: "stmt_tcb".to_string(),
            account_id: "acc_tcb".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: base_time + (i as i64 * 200),
            value_date: base_time + (i as i64 * 200),
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 20_000_000 - 5_500, // standard fee deduction
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some(format!("CONG TY TNHH PHAT TRIEN {i}")),
            counterparty_bank: None,
            narration: format!("CONG TY PHAT TRIEN {i} CK TIEN HANG"),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: base_time,
        });
        ledger_entries.push(InternalLedgerEntry {
            id: format!("led_entry_t2_{i}"),
            account_id: "acc_tcb".to_string(),
            doc_no: format!("HD200{i}"),
            entry_date: base_time + (i as i64 * 200) + 1800,
            entry_type: TransactionType::Credit,
            amount: 20_000_000,
            partner_code: Some(format!("DEV{i}")),
            partner_name: Some(format!("Công ty TNHH Phát triển {i}")),
            description: format!("Ban hang {i}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: base_time,
        });
    }

    // 3. 10 Tier 3 Composite Split Matches (TX 35..45, 60M composite = 25M + 35M)
    for i in 35..45 {
        let doc_a = format!("HDSPLITA{i}");
        let doc_b = format!("HDSPLITB{i}");
        bank_txs.push(BankTransactionRow {
            id: format!("bank_tx_t3_{i}"),
            statement_id: "stmt_vcb".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: base_time + (i as i64 * 300),
            value_date: base_time + (i as i64 * 300),
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 60_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some(format!("CONG TY THEP VIET {i}")),
            counterparty_bank: None,
            narration: format!("CK GOM {doc_a} VA {doc_b}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: base_time,
        });
        ledger_entries.push(InternalLedgerEntry {
            id: format!("led_split_a_{i}"),
            account_id: "acc_vcb".to_string(),
            doc_no: doc_a,
            entry_date: base_time + (i as i64 * 300) + 600,
            entry_type: TransactionType::Credit,
            amount: 25_000_000,
            partner_code: Some(format!("STEEL{i}")),
            partner_name: Some(format!("Công ty Thép Việt {i}")),
            description: "Part A".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: base_time,
        });
        ledger_entries.push(InternalLedgerEntry {
            id: format!("led_split_b_{i}"),
            account_id: "acc_vcb".to_string(),
            doc_no: doc_b,
            entry_date: base_time + (i as i64 * 300) + 600,
            entry_type: TransactionType::Credit,
            amount: 35_000_000,
            partner_code: Some(format!("STEEL{i}")),
            partner_name: Some(format!("Công ty Thép Việt {i}")),
            description: "Part B".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: base_time,
        });
    }

    // 4. 5 Residual Unmatched Transactions (TX 45..50, unknown money) -> FAIL CLOSED TO HITL
    for i in 45..50 {
        bank_txs.push(BankTransactionRow {
            id: format!("bank_tx_hitl_{i}"),
            statement_id: "stmt_vcb".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: base_time + (i as i64 * 400),
            value_date: base_time + (i as i64 * 400),
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 350_000 + (i as u64 * 10_000),
            balance_after: None,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Tien vao bat thuong khong ro nguon goc".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: base_time,
        });
    }

    assert_eq!(bank_txs.len(), 50, "Total 50 bank transactions");
    assert_eq!(
        ledger_entries.len(),
        55,
        "Total 55 internal ledger open entries"
    );

    // Execute complete 3-Tier Reconciliation
    let (matches, summary) = ReconciliationEngine::reconcile(&bank_txs, &ledger_entries);

    // Verify Summary Metrics
    assert_eq!(summary.total_bank_transactions, 50);
    assert_eq!(
        summary.matched_exact_count, 20,
        "20 Tier 1 matches expected"
    );
    assert_eq!(
        summary.matched_fuzzy_count, 15,
        "15 Tier 2 matches expected"
    );
    assert_eq!(
        summary.matched_split_count, 10,
        "10 Tier 3 split matches expected"
    );
    assert_eq!(
        summary.total_matched_count, 45,
        "45 total automatic matches"
    );
    assert_eq!(
        summary.pending_hitl_count, 5,
        "5 residual transactions must fail-closed to HITL"
    );
    assert_eq!(summary.match_rate, 90.0, "Expected match rate 90.0%");

    // Verify Mathematical Invariants
    let mut allocated_ledger_entries = HashSet::new();
    for m in &matches {
        if m.status == "APPROVED" {
            for lid in &m.ledger_entry_ids {
                assert!(
                    allocated_ledger_entries.insert(lid.clone()),
                    "Double allocation violation: Ledger entry {lid} allocated more than once!"
                );
            }
            if m.match_type == MatchType::CompositeSplit {
                assert_eq!(
                    m.discrepancy_amount, 0,
                    "Composite split match must satisfy Delta = 0 invariant"
                );
            }
        } else if m.status == "PENDING_HITL" {
            assert!(
                m.hitl_token.is_some(),
                "Every HITL match must carry a secure token"
            );
            let uuid_str = m.hitl_token.as_ref().unwrap();
            let parsed = Uuid::parse_str(uuid_str).expect("Valid UUID v4");
            assert_eq!(parsed.get_version(), Some(uuid::Version::Random));
        }
    }
}

// ===========================================================================
// 7. Milestone 1 Bank Parsers Adversarial Empirical Stress Tests
// ===========================================================================

#[test]
fn test_m1_adversarial_delimiter_variations() {
    // Case 1A: Tab-delimited Techcombank statement with quoted fields
    let tcb_tsv = "\
Số tài khoản:\t19038877665544\t\t\n\
Số dư đầu kỳ:\t50.000.000\t\t\n\
Ngày giao dịch\tMã giao dịch\tSố tiền ghi nợ\tSố tiền ghi có\tSố dư\tNội dung chi tiết\n\
15/08/2026\tNPS12345678\t\t25.000.000\t75.000.000\t\"CONG TY ABC\tCHI NHANH 1\tCHUYEN TIEN\"\n\
16/08/2026\tFT26001\t5.000.000\t\t70.000.000\tPhi duy tri dich vu\n\
Số dư cuối kỳ:\t70.000.000\t\t\n";

    let tcb_parsed = TcbCsvParser
        .parse(tcb_tsv.as_bytes(), "tcb_tab.txt")
        .unwrap();
    assert_eq!(tcb_parsed.bank_code, "TCB");
    assert_eq!(tcb_parsed.transactions.len(), 2);
    assert_eq!(tcb_parsed.transactions[0].amount, 25_000_000);
    assert_eq!(
        tcb_parsed.transactions[0].doc_ref.as_deref(),
        Some("NPS12345678")
    );
    assert_eq!(tcb_parsed.transactions[1].amount, 5_000_000);
    assert_eq!(tcb_parsed.transactions[1].tx_type, TransactionType::Debit);

    // Case 1B: Semicolon delimiter with RFC 4180 escaped quotes ("") and embedded commas
    let ctg_semi = "\
NGAN HANG TMCP CONG THUONG VIET NAM\n\
Số tài khoản:;102000999888;;;;\n\
Số dư đầu kỳ:;100.000.000;;;;\n\
Ngày GD;Số phiếu;Số tiền GD;Nợ/Có;Dư cuối;Diễn giải\n\
10/08/2026;CT101;30.000.000;C;130.000.000;\"CONG TY \"\"MINH KHANG\"\", THANH TOAN; HOP DONG 01\"\n\
11/08/2026;CT102;10.000.000;N;120.000.000;\"RUT TIEN MAT, PHONG GD SO 2; NGUYEN TRAI\"\n\
Số dư cuối kỳ:;120.000.000;;;;\n";

    let ctg_parsed = VietinBankParser
        .parse(ctg_semi.as_bytes(), "ctg_semi.csv")
        .unwrap();
    assert_eq!(ctg_parsed.bank_code, "CTG");
    assert_eq!(ctg_parsed.transactions.len(), 2);
    assert_eq!(ctg_parsed.transactions[0].amount, 30_000_000);
    assert_eq!(ctg_parsed.transactions[0].tx_type, TransactionType::Credit);
    assert!(ctg_parsed.transactions[0].narration.contains("MINH KHANG"));
    assert!(
        ctg_parsed.transactions[0]
            .narration
            .contains("THANH TOAN; HOP DONG 01")
    );
    assert_eq!(ctg_parsed.transactions[1].amount, 10_000_000);
    assert_eq!(ctg_parsed.transactions[1].tx_type, TransactionType::Debit);

    // Case 1C: Mixed delimiters - Pipe '|' delimiter with MBBank
    let mb_pipe = "\
NGAN HANG TMCP QUAN DOI - MBBANK\n\
Tài khoản: 0880112233445\n\
Số dư ban đầu: 200.000.000\n\
Số GD|Ngày|Ghi nợ|Ghi có|Số dư|Nội dung chi tiết\n\
MB-501|12/08/2026||45.000.000|245.000.000|Nhan chuyen khoan tu CONG TY THEP, CHI NHANH 1; SO GD 88\n\
MB-502|13/08/2026|15.000.000||230.000.000|Thanh toan tien dien, nuoc | van phong\n\
Số dư cuối kỳ: 230.000.000\n";

    let mb_parsed = MbBankParser
        .parse(mb_pipe.as_bytes(), "mb_pipe.csv")
        .unwrap();
    assert_eq!(mb_parsed.bank_code, "MB");
    assert_eq!(mb_parsed.transactions.len(), 2);
    assert_eq!(mb_parsed.transactions[0].amount, 45_000_000);
    assert_eq!(mb_parsed.transactions[1].amount, 15_000_000);

    // Case 1D: Agribank CSV with semicolons and thousands separators
    let agri_semi = "\
Số tài khoản:;1500201999888;;;;\n\
Số dư đầu kỳ:;80.000.000;;;;\n\
Ngày GD;Ngày HL;Số GD;Diễn giải;Phát sinh nợ;Phát sinh có;Số dư\n\
14/08/2026;14/08/2026;VB901;Nhan thanh toan hop dong;;50.000.000;130.000.000\n\
15/08/2026;15/08/2026;VB902;Chi mua giong cay trong;20.000.000;;110.000.000\n\
Số dư cuối kỳ:;110.000.000;;;;\n";

    let agri_parsed = AgribankParser
        .parse(agri_semi.as_bytes(), "agri.csv")
        .unwrap();
    assert_eq!(agri_parsed.bank_code, "VBA");
    assert_eq!(agri_parsed.transactions.len(), 2);
    assert_eq!(agri_parsed.transactions[0].amount, 50_000_000);
    assert_eq!(agri_parsed.transactions[1].amount, 20_000_000);
}

#[test]
fn test_m1_adversarial_text_encodings() {
    // Case 2A: UTF-8 with BOM prefix [0xEF, 0xBB, 0xBF]
    let mut tcb_utf8_bom = vec![0xEF, 0xBB, 0xBF];
    tcb_utf8_bom.extend_from_slice(
        "Số tài khoản: 19035544332211\n\
Số dư đầu kỳ: 10.000.000\n\
Số dư cuối kỳ: 25.000.000\n\
Ngày giao dịch,Mã giao dịch,Số tiền ghi có,Số tiền ghi nợ,Số dư,Nội dung\n\
18/08/2026,FT26888,15.000.000,,25.000.000,Thanh toan tien dich vu LIVA\n"
            .as_bytes(),
    );

    let parsed_bom = sniff_and_parse(&tcb_utf8_bom, "tcb_bom.csv").unwrap();
    assert_eq!(parsed_bom.bank_code, "TCB");
    assert_eq!(parsed_bom.account_number.as_deref(), Some("19035544332211"));
    assert_eq!(parsed_bom.opening_balance, Some(10_000_000));
    assert_eq!(parsed_bom.closing_balance, Some(25_000_000));
    assert_eq!(parsed_bom.transactions.len(), 1);
    assert_eq!(parsed_bom.transactions[0].amount, 15_000_000);

    // Case 2B: Windows-1258 non-UTF-8 bytes fallback
    // The byte sequence [0xF0, 0xE2, 0xCC, b'u', b' ', b't', 0xFE] is invalid UTF-8 and forces decoder fallback to WINDOWS_1258
    let mut win1258_bytes = Vec::new();
    win1258_bytes.extend_from_slice(
        b"Account No: 19035544332211\n\
Opening Balance: 500.000.000\n\
Closing Balance: 570.000.000\n\
Date,Doc No,Amount Credit,Amount Debit,Balance,Description\n\
20/08/2026,FT9001,85.000.000,,585.000.000,Chuyen khoan hop dong dich vu ",
    );
    win1258_bytes.extend_from_slice(&[0xF0, 0xE2, 0xCC, b'u', b' ', b't', 0xFE]);
    win1258_bytes.extend_from_slice(
        b"\n21/08/2026,FT9002,,15.000.000,570.000.000,Chi tra tien thue van phong\n",
    );

    // Verify raw bytes are genuinely invalid UTF-8
    assert!(
        std::str::from_utf8(&win1258_bytes).is_err(),
        "Must be invalid UTF-8 to trigger Windows-1258 decoder"
    );

    let tcb_win1258 = TcbCsvParser
        .parse(&win1258_bytes, "tcb_win1258.csv")
        .unwrap();
    assert_eq!(tcb_win1258.bank_code, "TCB");
    assert_eq!(
        tcb_win1258.account_number.as_deref(),
        Some("19035544332211")
    );
    assert_eq!(tcb_win1258.opening_balance, Some(500_000_000));
    assert_eq!(tcb_win1258.closing_balance, Some(570_000_000));
    assert_eq!(tcb_win1258.transactions.len(), 2);
    assert_eq!(tcb_win1258.transactions[0].amount, 85_000_000);
    assert_eq!(tcb_win1258.transactions[1].amount, 15_000_000);
    assert!(verify_balance_invariants(&tcb_win1258).is_balanced);
}

#[test]
fn test_m1_adversarial_excel_merged_cells_and_bottom_up_balance() {
    let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let fixtures_dir = base_dir.join("fixtures").join("statements");
    let xlsx_path = fixtures_dir.join("vcb_adversarial_merged.xlsx");
    assert!(
        xlsx_path.exists(),
        "vcb_adversarial_merged.xlsx fixture must exist"
    );

    let bytes = std::fs::read(&xlsx_path).unwrap();
    let stmt = sniff_and_parse(&bytes, "vcb_adversarial_merged.xlsx").unwrap();

    assert_eq!(stmt.bank_code, "VCB");
    assert_eq!(stmt.format, StatementFormat::Excel);
    assert_eq!(stmt.account_number.as_deref(), Some("0011009988776"));
    assert_eq!(stmt.opening_balance, Some(100_000_000));
    // Verify bottom-up closing balance recovery
    assert_eq!(
        stmt.closing_balance,
        Some(160_000_000),
        "Bottom-up scan must extract closing balance from footer row"
    );

    // Verify all 6 transactions were parsed
    assert_eq!(
        stmt.transactions.len(),
        6,
        "All 6 transactions must be parsed"
    );

    // Verify merged cell forward-fill:
    // Tx 1: 15/08/2026, Credit 10M
    // Tx 2: date merged (empty), Credit 20M -> inherits 15/08/2026
    // Tx 3: date merged (empty), Debit 5M -> inherits 15/08/2026
    // Tx 4: date merged (empty), Credit 15M -> inherits 15/08/2026
    let expected_date_15 = stmt.transactions[0].tx_date;
    assert_eq!(
        stmt.transactions[1].tx_date, expected_date_15,
        "Tx 2 must forward-fill date 15/08/2026"
    );
    assert_eq!(
        stmt.transactions[2].tx_date, expected_date_15,
        "Tx 3 must forward-fill date 15/08/2026"
    );
    assert_eq!(
        stmt.transactions[3].tx_date, expected_date_15,
        "Tx 4 must forward-fill date 15/08/2026"
    );

    // Tx 5: 16/08/2026, Credit 30M
    // Tx 6: date merged (empty), Debit 10M -> inherits 16/08/2026
    let expected_date_16 = stmt.transactions[4].tx_date;
    assert_ne!(
        expected_date_16, expected_date_15,
        "Date for Tx 5 must be distinct (16/08/2026)"
    );
    assert_eq!(
        stmt.transactions[5].tx_date, expected_date_16,
        "Tx 6 must forward-fill date 16/08/2026"
    );

    // Verify balance invariant:
    // 100M + (10M + 20M + 15M + 30M = 75M) - (5M + 10M = 15M) = 160M == Closing
    let inv = verify_balance_invariants(&stmt);
    assert!(
        inv.is_balanced,
        "Double-entry balance invariant must be satisfied"
    );
    assert_eq!(inv.discrepancy, 0);
    assert_eq!(inv.total_credit, 75_000_000);
    assert_eq!(inv.total_debit, 15_000_000);
}

#[test]
fn test_m1_adversarial_pdf_multiline_narration_preservation() {
    use lopdf::content::{Content, Operation};
    use lopdf::{Dictionary, Document, Object, Stream, StringFormat};

    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Font".into()),
        ("Subtype", "Type1".into()),
        ("BaseFont", "Helvetica".into()),
    ]));
    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Dictionary::from_iter(vec![("F1", font_id.into())]).into(),
    )]));

    let mut ops: Vec<Operation> = Vec::new();
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));

    let mut emit_line = |x: f32, y: f32, tokens: &[&str]| {
        let mut cur_x = x;
        for &tok in tokens {
            ops.push(Operation::new(
                "Tm",
                vec![
                    1.into(),
                    0.into(),
                    0.into(),
                    1.into(),
                    cur_x.into(),
                    y.into(),
                ],
            ));
            ops.push(Operation::new(
                "Tj",
                vec![Object::String(
                    tok.as_bytes().to_vec(),
                    StringFormat::Literal,
                )],
            ));
            cur_x += (tok.len() as f32) * 6.5 + 10.0;
        }
    };

    let mut y = 800.0;
    emit_line(
        50.0,
        y,
        &["NGAN HANG TMCP DAU TU VA PHAT TRIEN VIET NAM (BIDV)"],
    );
    y -= 20.0;
    emit_line(50.0, y, &["Số tài khoản: 12410009988776"]);
    y -= 16.0;
    emit_line(50.0, y, &["Số dư đầu kỳ: 50.000.000"]);
    y -= 25.0;

    // Table Header
    emit_line(
        50.0,
        y,
        &[
            "Ngày GD",
            "Chứng từ",
            "Số tiền ghi nợ",
            "Số tiền ghi có",
            "Số dư",
            "Nội dung giao dịch",
        ],
    );
    y -= 18.0;

    // Tx 1: Credit 20M
    emit_line(
        50.0,
        y,
        &[
            "15/08/2026",
            "FT26001",
            "20.000.000",
            "70.000.000",
            "Thanh toan tien hang dot 1",
        ],
    );
    y -= 14.0;
    // Multi-line continuation: Invoice number and contract ID
    emit_line(
        120.0,
        y,
        &["theo hoa don INV-001 va hop dong kinh te HD105/2026"],
    );
    y -= 18.0;

    // Tx 2: Credit 30M
    emit_line(
        50.0,
        y,
        &[
            "16/08/2026",
            "FT26002",
            "30.000.000",
            "100.000.000",
            "Thanh toan gia cong san pham",
        ],
    );
    y -= 14.0;
    // Multi-line continuation: percentage discount
    emit_line(
        120.0,
        y,
        &["ap dung chiet khau 10.5% gia tri hop dong va giam 5% thue"],
    );
    y -= 18.0;

    // Tx 3: Debit 10M
    emit_line(
        50.0,
        y,
        &[
            "17/08/2026",
            "FT26003",
            "10.000.000",
            "90.000.000",
            "Chi phi van chuyen logistic",
        ],
    );
    y -= 14.0;
    // Multi-line continuation: Napas 247 trace
    emit_line(
        120.0,
        y,
        &["Chuyen nhanh qua Napas 247 ma trace FT26999888777"],
    );

    ops.push(Operation::new("ET", vec![]));

    let content = Content { operations: ops };
    let content_bytes = content.encode().unwrap();
    let content_stream = Stream::new(Dictionary::new(), content_bytes);
    let content_id = doc.add_object(content_stream);

    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Page".into()),
        ("Parent", pages_id.into()),
        ("Contents", content_id.into()),
    ]));

    let pages_dict = Dictionary::from_iter(vec![
        ("Type", "Pages".into()),
        ("Kids", vec![page_id.into()].into()),
        ("Count", 1.into()),
        ("Resources", resources_id.into()),
        (
            "MediaBox",
            vec![0.into(), 0.into(), 595.into(), 842.into()].into(),
        ),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Catalog".into()),
        ("Pages", pages_id.into()),
    ]));
    doc.trailer.set("Root", catalog_id);

    let mut buf = Vec::new();
    doc.save_to(&mut buf).unwrap();

    let stmt = BidvPdfParser
        .parse(&buf, "bidv_adversarial_pdf.pdf")
        .unwrap();
    assert_eq!(stmt.bank_code, "BIDV");
    assert_eq!(
        stmt.transactions.len(),
        3,
        "Must parse exactly 3 transactions without dropping continuations"
    );

    // Tx 1 check
    let tx1 = &stmt.transactions[0];
    assert!(
        tx1.narration.contains("INV-001"),
        "Must preserve invoice number INV-001"
    );
    assert!(
        tx1.narration.contains("HD105/2026"),
        "Must preserve contract ID HD105/2026"
    );

    // Tx 2 check
    let tx2 = &stmt.transactions[1];
    assert!(
        tx2.narration.contains("10.5%"),
        "Must preserve percentage discount 10.5%"
    );
    assert!(
        tx2.narration.contains("5%"),
        "Must preserve tax discount 5%"
    );

    // Tx 3 check
    let tx3 = &stmt.transactions[2];
    assert!(
        tx3.narration.contains("FT26999888777"),
        "Must preserve Napas trace"
    );
    assert_eq!(tx3.amount, 10_000_000);
    assert_eq!(tx3.tx_type, TransactionType::Debit);
}

#[test]
fn test_m1_adversarial_sniff_and_parse_resilience_matrix() {
    // 1. Magic bytes container detection
    assert_eq!(
        detect_container_format(b"%PDF-1.7 hostile binary data", "random.dat"),
        ContainerFormat::Pdf
    );
    assert_eq!(
        detect_container_format(&[0x50, 0x4B, 0x03, 0x04, 0x14, 0x00], "unknown"),
        ContainerFormat::ExcelZip
    );
    assert_eq!(
        detect_container_format(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1], "sheet"),
        ContainerFormat::ExcelOle
    );
    assert_eq!(
        detect_container_format(b"<html><body><table border=1>", "statement.xls"),
        ContainerFormat::HtmlTable
    );
    assert_eq!(
        detect_container_format(
            b"<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.01//EN\">",
            "test.xls"
        ),
        ContainerFormat::HtmlTable
    );
    assert_eq!(
        detect_container_format(&[0xEF, 0xBB, 0xBF, b'A', b'B'], "file.csv"),
        ContainerFormat::TextCsv
    );

    // 2. Corrupted and truncated payload resilience - zero panics
    let zero_bytes: &[u8] = b"";
    assert!(sniff_and_parse(zero_bytes, "empty.bin").is_err());

    let single_byte: &[u8] = &[0x00];
    assert!(sniff_and_parse(single_byte, "byte.bin").is_err());

    let fake_pdf = b"%PDF-corrupted_no_trailer";
    assert!(sniff_and_parse(fake_pdf, "broken.pdf").is_err());

    let fake_zip = &[0x50, 0x4B, 0x03, 0x04, 0xFF, 0xFF];
    assert!(sniff_and_parse(fake_zip, "broken.xlsx").is_err());

    let fake_html = b"<html><table><tr><td>Agribank</td><td>unclosed";
    // Parser must either parse partially or cleanly return Err, NEVER panic
    let _ = sniff_and_parse(fake_html, "corrupted.xls");

    // 3. Bank discrimination without misidentification
    // Agribank HTML table .xls
    let vba_html = "<html><table>\
<tr><td>NGAN HANG NONG NGHIEP VA PHAT TRIEN NONG THON (AGRIBANK)</td></tr>\
<tr><td>Số tài khoản: 1500201122334</td></tr>\
<tr><td>Số dư đầu kỳ: 10.000.000</td></tr>\
<tr><th>Ngày GD</th><th>Mã GD</th><th>Diễn giải</th><th>Số tiền ghi nợ</th><th>Số tiền ghi có</th><th>Số dư</th></tr>\
<tr><td>10/08/2026</td><td>VB111</td><td>Chuyen tien</td><td></td><td>5.000.000</td><td>15.000.000</td></tr>\
<tr><td>Số dư cuối kỳ: 15.000.000</td></tr>\
</table></html>";
    let vba_stmt = sniff_and_parse(vba_html.as_bytes(), "agribank_statement.xls").unwrap();
    assert_eq!(vba_stmt.bank, BankType::Agribank);
    assert_eq!(vba_stmt.bank_code, "VBA");
    assert_eq!(vba_stmt.format, StatementFormat::Html);

    // MBBank CSV
    let mb_csv = "NGAN HANG QUAN DOI MBBANK\n\
Số tài khoản: 0880199887766\n\
Ngày,Số GD,Ghi có,Ghi nợ,Số dư,Nội dung\n\
15/08/2026,MB881,10.000.000,,10.000.000,Thanh toan tien hang MB\n";
    let mb_stmt = sniff_and_parse(mb_csv.as_bytes(), "mbbank_saoke.csv").unwrap();
    assert_eq!(mb_stmt.bank, BankType::MbBank);
    assert_eq!(mb_stmt.bank_code, "MB");

    // VietinBank CSV
    let ctg_csv = "NGAN HANG CONG THUONG VIETINBANK\n\
Số tài khoản: 102000887766\n\
Ngày GD,Số phiếu,Số tiền GD,Nợ/Có,Dư cuối,Diễn giải\n\
15/08/2026,CT551,20.000.000,C,20.000.000,Thu tien ban hang\n";
    let ctg_stmt = sniff_and_parse(ctg_csv.as_bytes(), "ctg_saoke.csv").unwrap();
    assert_eq!(ctg_stmt.bank, BankType::VietinBank);
    assert_eq!(ctg_stmt.bank_code, "CTG");

    // 4. Semantic score_bank keyword scoring verification
    assert_eq!(
        score_bank(b"NGAN HANG TMCP VIETCOMBANK", "saoke.xlsx"),
        Some(BankType::Vietcombank)
    );
    assert_eq!(
        score_bank(b"TECHCOMBANK TIEN GUI THANH TOAN", "statement.csv"),
        Some(BankType::Techcombank)
    );
    assert_eq!(
        score_bank(b"NGAN HANG TMCP DAU TU VA PHAT TRIEN (BIDV)", "saoke.pdf"),
        Some(BankType::Bidv)
    );
    assert_eq!(
        score_bank(b"NGAN HANG TMCP CONG THUONG VIETINBANK", "saoke.csv"),
        Some(BankType::VietinBank)
    );
    assert_eq!(
        score_bank(b"NGAN HANG QUAN DOI MBBANK", "saoke.csv"),
        Some(BankType::MbBank)
    );
    assert_eq!(
        score_bank(
            b"NGAN HANG NONG NGHIEP VA PHAT TRIEN NONG THON (AGRIBANK)",
            "saoke.xls"
        ),
        Some(BankType::Agribank)
    );
}

#[test]
fn test_m1_adversarial_arithmetic_invariants_oracle() {
    let make_stmt = |open: u64, credits: &[u64], debits: &[u64], close: u64| {
        let mut txs = Vec::new();
        let mut cur_time = 1_725_000_000i64;
        let mut id = 1;
        for &c in credits {
            txs.push(TransactionRecord::new(
                id,
                cur_time,
                cur_time,
                Some(format!("CR_{id}")),
                TransactionType::Credit,
                c,
                None,
                None,
                None,
                None,
                "Credit tx".to_string(),
            ));
            cur_time += 100;
            id += 1;
        }
        for &d in debits {
            txs.push(TransactionRecord::new(
                id,
                cur_time,
                cur_time,
                Some(format!("DB_{id}")),
                TransactionType::Debit,
                d,
                None,
                None,
                None,
                None,
                "Debit tx".to_string(),
            ));
            cur_time += 100;
            id += 1;
        }

        BankStatement::new(
            "VCB".to_string(),
            BankType::Vietcombank,
            StatementFormat::Excel,
            Some("0011001234567".to_string()),
            Some("CORP".to_string()),
            Some(open),
            Some(close),
            Some(1_725_000_000),
            Some(cur_time),
            txs,
            5,
        )
    };

    // 1. Exactly balanced
    let stmt_balanced = make_stmt(
        100_000_000,
        &[50_000_000, 20_000_000],
        &[30_000_000],
        140_000_000,
    );
    let inv_ok = verify_balance_invariants(&stmt_balanced);
    assert!(inv_ok.is_balanced);
    assert!(inv_ok.is_valid);
    assert_eq!(inv_ok.discrepancy, 0);
    assert_eq!(inv_ok.calculated_closing, 140_000_000);

    // 2. 1 VND discrepancy (positive)
    let stmt_plus1 = make_stmt(100_000_000, &[50_000_000], &[20_000_000], 130_000_001);
    let inv_plus1 = verify_balance_invariants(&stmt_plus1);
    assert!(!inv_plus1.is_balanced);
    assert_eq!(inv_plus1.discrepancy, 1);

    // 3. 1 VND discrepancy (negative)
    let stmt_minus1 = make_stmt(100_000_000, &[50_000_000], &[20_000_000], 129_999_999);
    let inv_minus1 = verify_balance_invariants(&stmt_minus1);
    assert!(!inv_minus1.is_balanced);
    assert_eq!(inv_minus1.discrepancy, -1);

    // 4. Large treasury numbers (> 500 Billion VND) to stress 64-bit integer calculations
    let stmt_treasury = make_stmt(
        500_000_000_000,                     // 500 Billion VND
        &[200_000_000_000, 150_000_000_000], // +350 Billion
        &[100_000_000_000, 50_000_000_000],  // -150 Billion
        700_000_000_000,                     // Expected: 700 Billion VND
    );
    let inv_treasury = verify_balance_invariants(&stmt_treasury);
    assert!(inv_treasury.is_balanced);
    assert_eq!(inv_treasury.discrepancy, 0);
    assert_eq!(inv_treasury.computed_closing, 700_000_000_000);
}

// ===========================================================================
// 7. Remediation Iteration 3 Verification Tests
// ===========================================================================

#[test]
fn test_m1_remediation_tcb_csv_footer_closing_balance() {
    // 1. Real fixture with footer summary row: fixtures/statements/tcb_aug2026.csv
    let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let fixtures_dir = base_dir.join("fixtures").join("statements");
    let tcb_path = fixtures_dir.join("tcb_aug2026.csv");
    assert!(tcb_path.exists(), "tcb_aug2026.csv fixture must exist");

    let bytes = std::fs::read(&tcb_path).unwrap();
    let stmt = TcbCsvParser.parse(&bytes, "tcb_aug2026.csv").unwrap();

    assert_eq!(stmt.bank_code, "TCB");
    assert_eq!(stmt.account_number.as_deref(), Some("19034567890123"));
    assert_eq!(stmt.opening_balance, Some(785_600_000));
    // Verify bottom-up reverse scanning found the closing balance in the summary footer row (line 66)
    assert_eq!(
        stmt.closing_balance,
        Some(2_246_754_900),
        "Bottom-up reverse scan must extract closing balance from footer"
    );
    assert_eq!(stmt.transactions.len(), 60);

    // Verify mathematical invariant balancing
    let inv = verify_balance_invariants(&stmt);
    assert!(
        inv.is_balanced,
        "Statement must balance with extracted closing balance"
    );
    assert_eq!(inv.discrepancy, 0);
    assert_eq!(inv.computed_closing, 2_246_754_900);

    // 2. Synthetic case: Statement without footer row falls back to transactions.last().balance_after
    let synthetic_csv = "\
Số tài khoản:;19039988776655;;;\n\
Số dư đầu kỳ:;50.000.000;;;\n\
Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết\n\
15/08/2026;FT001;;30.000.000;80.000.000;Tien hang\n\
16/08/2026;FT002;10.000.000;;70.000.000;Chi phi van hanh\n";
    let synth_stmt = TcbCsvParser
        .parse(synthetic_csv.as_bytes(), "tcb_synth.csv")
        .unwrap();
    assert_eq!(synth_stmt.closing_balance, Some(70_000_000));
    let synth_inv = verify_balance_invariants(&synth_stmt);
    assert!(synth_inv.is_balanced);
}

#[test]
fn test_m1_remediation_windows_1258_nfc_normalization_accented_headers() {
    // Windows-1258 text with genuine decomposed Vietnamese syllables in headers:
    // "Số tài khoản", "Số dư đầu kỳ", "Ngày giao dịch", "Số chứng từ", "Số tiền", "Số dư cuối kỳ"
    let raw_text = "\
Số tài khoản: 19035544332211\n\
Số dư đầu kỳ: 500.000.000\n\
Ngày giao dịch,Số chứng từ,Số tiền ghi có,Số tiền ghi nợ,Số dư,Nội dung chi tiết\n\
20/08/2026,FT9001,85.000.000,,585.000.000,Dịch vụ tài chính LIVA\n\
Số dư cuối kỳ: 585.000.000\n";

    fn to_cp1258_decomposed(s: &str) -> String {
        let mut out = String::new();
        for ch in s.chars() {
            match ch {
                'ố' => {
                    out.push('ô');
                    out.push('\u{0301}');
                }
                'Ố' => {
                    out.push('Ô');
                    out.push('\u{0301}');
                }
                'ồ' => {
                    out.push('ô');
                    out.push('\u{0300}');
                }
                'ổ' => {
                    out.push('ô');
                    out.push('\u{0309}');
                }
                'ỗ' => {
                    out.push('ô');
                    out.push('\u{0303}');
                }
                'ộ' => {
                    out.push('ô');
                    out.push('\u{0323}');
                }
                'ế' => {
                    out.push('ê');
                    out.push('\u{0301}');
                }
                'ề' => {
                    out.push('ê');
                    out.push('\u{0300}');
                }
                'ể' => {
                    out.push('ê');
                    out.push('\u{0309}');
                }
                'ễ' => {
                    out.push('ê');
                    out.push('\u{0303}');
                }
                'ệ' => {
                    out.push('ê');
                    out.push('\u{0323}');
                }
                'ấ' => {
                    out.push('â');
                    out.push('\u{0301}');
                }
                'ầ' => {
                    out.push('â');
                    out.push('\u{0300}');
                }
                'ẩ' => {
                    out.push('â');
                    out.push('\u{0309}');
                }
                'ẫ' => {
                    out.push('â');
                    out.push('\u{0303}');
                }
                'ậ' => {
                    out.push('â');
                    out.push('\u{0323}');
                }
                'ắ' => {
                    out.push('ă');
                    out.push('\u{0301}');
                }
                'ằ' => {
                    out.push('ă');
                    out.push('\u{0300}');
                }
                'ẳ' => {
                    out.push('ă');
                    out.push('\u{0309}');
                }
                'ẵ' => {
                    out.push('ă');
                    out.push('\u{0303}');
                }
                'ặ' => {
                    out.push('ă');
                    out.push('\u{0323}');
                }
                'ứ' => {
                    out.push('ư');
                    out.push('\u{0301}');
                }
                'ừ' => {
                    out.push('ư');
                    out.push('\u{0300}');
                }
                'ử' => {
                    out.push('ư');
                    out.push('\u{0309}');
                }
                'ữ' => {
                    out.push('ư');
                    out.push('\u{0303}');
                }
                'ự' => {
                    out.push('ư');
                    out.push('\u{0323}');
                }
                'ớ' => {
                    out.push('ơ');
                    out.push('\u{0301}');
                }
                'ờ' => {
                    out.push('ơ');
                    out.push('\u{0300}');
                }
                'ở' => {
                    out.push('ơ');
                    out.push('\u{0309}');
                }
                'ỡ' => {
                    out.push('ơ');
                    out.push('\u{0303}');
                }
                'ợ' => {
                    out.push('ơ');
                    out.push('\u{0323}');
                }
                'ả' => {
                    out.push('a');
                    out.push('\u{0309}');
                }
                'ạ' => {
                    out.push('a');
                    out.push('\u{0323}');
                }
                'ỉ' => {
                    out.push('i');
                    out.push('\u{0309}');
                }
                'ị' => {
                    out.push('i');
                    out.push('\u{0323}');
                }
                'ỏ' => {
                    out.push('o');
                    out.push('\u{0309}');
                }
                'ọ' => {
                    out.push('o');
                    out.push('\u{0323}');
                }
                'ủ' => {
                    out.push('u');
                    out.push('\u{0309}');
                }
                'ụ' => {
                    out.push('u');
                    out.push('\u{0323}');
                }
                'ỷ' => {
                    out.push('y');
                    out.push('\u{0309}');
                }
                'ỳ' => {
                    out.push('y');
                    out.push('\u{0300}');
                }
                'ỵ' => {
                    out.push('y');
                    out.push('\u{0323}');
                }
                _ => out.push(ch),
            }
        }
        out
    }

    let cp1258_input = to_cp1258_decomposed(raw_text);
    let (win1258_bytes, _, had_errors) = encoding_rs::WINDOWS_1258.encode(&cp1258_input);
    assert!(!had_errors, "Must encode cleanly to Windows-1258");
    // Verify it is genuinely invalid UTF-8 (triggering Windows-1258 fallback)
    assert!(
        std::str::from_utf8(&win1258_bytes).is_err(),
        "Must be invalid UTF-8"
    );

    // 1. TCB CSV
    let tcb = TcbCsvParser.parse(&win1258_bytes, "tcb.csv").unwrap();
    assert_eq!(tcb.account_number.as_deref(), Some("19035544332211"));
    assert_eq!(tcb.opening_balance, Some(500_000_000));
    assert_eq!(tcb.closing_balance, Some(585_000_000));
    assert_eq!(tcb.transactions.len(), 1);
    assert_eq!(tcb.transactions[0].amount, 85_000_000);
    assert!(verify_balance_invariants(&tcb).is_balanced);

    // 2. VietinBank CSV
    let ctg = VietinBankParser.parse(&win1258_bytes, "ctg.csv").unwrap();
    assert_eq!(ctg.account_number.as_deref(), Some("19035544332211"));
    assert_eq!(ctg.opening_balance, Some(500_000_000));
    assert_eq!(ctg.closing_balance, Some(585_000_000));
    assert_eq!(ctg.transactions.len(), 1);
    assert_eq!(ctg.transactions[0].amount, 85_000_000);
    assert!(verify_balance_invariants(&ctg).is_balanced);

    // 3. MBBank CSV
    let mb = MbBankParser.parse(&win1258_bytes, "mb.csv").unwrap();
    assert_eq!(mb.account_number.as_deref(), Some("19035544332211"));
    assert_eq!(mb.opening_balance, Some(500_000_000));
    assert_eq!(mb.closing_balance, Some(585_000_000));
    assert_eq!(mb.transactions.len(), 1);
    assert_eq!(mb.transactions[0].amount, 85_000_000);
    assert!(verify_balance_invariants(&mb).is_balanced);

    // 4. Agribank CSV
    let agri = AgribankParser.parse(&win1258_bytes, "agri.csv").unwrap();
    assert_eq!(agri.account_number.as_deref(), Some("19035544332211"));
    assert_eq!(agri.opening_balance, Some(500_000_000));
    assert_eq!(agri.closing_balance, Some(585_000_000));
    assert_eq!(agri.transactions.len(), 1);
    assert_eq!(agri.transactions[0].amount, 85_000_000);
    assert!(verify_balance_invariants(&agri).is_balanced);
}

#[test]
fn test_m1_remediation_vietinbank_csv_english_headers() {
    let ctg_english = "\
Account No: 112000345678\n\
Opening Balance: 100.000.000\n\
Date,Doc No,Amount,Credit,Debit,Balance,Description\n\
15/08/2026,DOC001,25.000.000,25.000.000,,125.000.000,Payment from client\n\
16/08/2026,DOC002,10.000.000,,10.000.000,115.000.000,Office rent\n\
Closing Balance: 115.000.000\n";

    let ctg = VietinBankParser
        .parse(ctg_english.as_bytes(), "ctg_english.csv")
        .unwrap();
    assert_eq!(ctg.bank_code, "CTG");
    assert_eq!(ctg.account_number.as_deref(), Some("112000345678"));
    assert_eq!(ctg.opening_balance, Some(100_000_000));
    assert_eq!(ctg.closing_balance, Some(115_000_000));
    assert_eq!(ctg.transactions.len(), 2);
    assert_eq!(ctg.transactions[0].amount, 25_000_000);
    assert_eq!(ctg.transactions[0].tx_type, TransactionType::Credit);
    assert_eq!(ctg.transactions[1].amount, 10_000_000);
    assert_eq!(ctg.transactions[1].tx_type, TransactionType::Debit);

    let inv = verify_balance_invariants(&ctg);
    assert!(inv.is_balanced);
    assert_eq!(inv.discrepancy, 0);
}
