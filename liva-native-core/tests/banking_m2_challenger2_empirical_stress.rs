//! Empirical Adversarial Stress Test Suite for Milestone 2: Deterministic Reconciliation Engine.
//!
//! Authored and executed independently by Challenger 2.
//! Adversarially challenges:
//! 1. Direction Invariance Stress & Mixed Direction Attacks (Credit vs Debit in 1-to-N and N-to-1 splits, HITL UUID v4).
//! 2. Value Date 24h Boundary Stress & Trace Extraction (86,400s vs 86,401s, value_date override, FT/NPS/VN patterns).
//! 3. Vietnamese Bank Noise Stripping & Unicode NFC/NFD Equivalence (VCB, TCB, BIDV prefixes, diacritic normalization).
//! 4. Bidirectional Split Solving & Branch-and-Bound Bounds (1-to-N, N-to-1, depth 4 default, depth up to 8, 9-item fail-closed, adversarial subset sum).
//! 5. Balance Invariants Runtime Check & Zero Float Drift Audit (multi-account, chronological reordering, scaled u64 exactness).

use uuid::Uuid;

use liva_native_core::banking::models::*;
use liva_native_core::banking::reconciliation::ReconciliationEngine;
use liva_native_core::banking::reconciliation::fuzzy_matcher::FuzzyMatcher;
use liva_native_core::banking::reconciliation::hash_matcher::HashMatcher;
use liva_native_core::banking::reconciliation::jaro_winkler::{
    compare_party_names, normalize_vietnamese_text, strip_bank_narration_noise,
};
use liva_native_core::banking::reconciliation::split_solver::{
    SplitSolver, solve_exact_subset_sum_bnb,
};

// ===========================================================================
// Helper constructors
// ===========================================================================

fn make_bank_tx(
    id: &str,
    tx_type: TransactionType,
    amount: u64,
    tx_date: i64,
    value_date: i64,
    doc_ref: Option<&str>,
    narration: &str,
    partner: Option<&str>,
) -> BankTransactionRow {
    BankTransactionRow {
        id: id.to_string(),
        statement_id: "stmt_ch2".to_string(),
        account_id: "acc_ch2".to_string(),
        bank_code: "VCB".to_string(),
        tx_date,
        value_date,
        doc_ref: doc_ref.map(|s| s.to_string()),
        tx_type,
        amount,
        balance_after: Some(100_000_000),
        counterparty_account: None,
        counterparty_name: partner.map(|s| s.to_string()),
        counterparty_bank: None,
        narration: narration.to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: tx_date,
    }
}

fn make_ledger_entry(
    id: &str,
    entry_type: TransactionType,
    amount: u64,
    entry_date: i64,
    doc_no: &str,
    partner_name: Option<&str>,
    description: &str,
) -> InternalLedgerEntry {
    InternalLedgerEntry {
        id: id.to_string(),
        account_id: "acc_ch2".to_string(),
        doc_no: doc_no.to_string(),
        entry_date,
        entry_type,
        amount,
        partner_code: Some("PARTNER_CH2".to_string()),
        partner_name: partner_name.map(|s| s.to_string()),
        description: description.to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: entry_date,
    }
}

// ===========================================================================
// 1. Direction Invariance Stress & Mixed Direction Attacks
// ===========================================================================

#[test]
fn test_challenger2_direction_invariance_and_mixed_split_attack() {
    let base_time = 1_726_000_000i64;

    // 1.1 Direct 1-to-1 opposite direction probe in Tier 1:
    // Credit Bank Tx vs Debit Ledger Entry (identical amounts, identical doc_no)
    let tx_credit = make_bank_tx(
        "tx_c1",
        TransactionType::Credit,
        50_000_000,
        base_time,
        base_time,
        Some("HD-2026-01"),
        "Thanh toan HD-2026-01",
        Some("CONG TY ABC"),
    );
    let led_debit = make_ledger_entry(
        "led_d1",
        TransactionType::Debit,
        50_000_000,
        base_time + 60,
        "HD-2026-01",
        Some("CONG TY ABC"),
        "Hoa don xuat 01",
    );

    let (t1_matches, unalloc_b1, unalloc_l1) =
        HashMatcher::match_tier1(&[tx_credit.clone()], &[led_debit.clone()]);
    assert!(
        t1_matches.is_empty(),
        "Tier 1 MUST reject opposite directions even with identical doc_ref and amount"
    );
    assert_eq!(unalloc_b1, vec![0]);
    assert_eq!(unalloc_l1, vec![0]);

    // 1.2 Direct 1-to-1 opposite direction probe in Tier 2:
    let (t2_matches, unalloc_b2, unalloc_l2) = FuzzyMatcher::match_tier2(
        &[tx_credit.clone()],
        &[led_debit.clone()],
        &unalloc_b1,
        &unalloc_l1,
    );
    assert!(
        t2_matches.is_empty(),
        "Tier 2 MUST reject opposite directions even with identical party name"
    );
    assert_eq!(unalloc_b2, vec![0]);
    assert_eq!(unalloc_l2, vec![0]);

    // 1.3 Mixed Direction Attack on 1-to-N Split:
    // Bank tx is CREDIT of 100,000,000 VND.
    // Attacker supplies two ledger entries:
    // - Invoice A: CREDIT 60,000,000 VND
    // - Entry B: DEBIT 40,000,000 VND
    // Sum is mathematically 100,000,000 VND if signs are ignored!
    // But in banking, a Credit receipt cannot settle a Debit expenditure.
    let tx_composite = make_bank_tx(
        "tx_c_100m",
        TransactionType::Credit,
        100_000_000,
        base_time,
        base_time,
        None,
        "CONG TY ABC THANH TOAN HD-A VA HD-B",
        Some("CONG TY ABC"),
    );
    let led_credit_60m = make_ledger_entry(
        "led_c_60m",
        TransactionType::Credit,
        60_000_000,
        base_time + 100,
        "HD-A",
        Some("CONG TY ABC"),
        "Dot 1",
    );
    let led_debit_40m = make_ledger_entry(
        "led_d_40m",
        TransactionType::Debit, // MALICIOUS: Opposite direction!
        40_000_000,
        base_time + 200,
        "HD-B",
        Some("CONG TY ABC"),
        "Dot 2",
    );

    let (split_matches, hitl_matches) = SplitSolver::match_tier3(
        &[tx_composite.clone()],
        &[led_credit_60m.clone(), led_debit_40m.clone()],
        &[0],
        &[0, 1],
    );

    assert!(
        split_matches.is_empty(),
        "Tier 3 MUST NOT form a split match by combining Credit and Debit entries"
    );
    assert_eq!(
        hitl_matches.len(),
        1,
        "Unallocated transaction must route to HITL queue"
    );
    let hitl = &hitl_matches[0];
    assert_eq!(hitl.status, "PENDING_HITL");
    assert_eq!(hitl.match_type, MatchType::ManualHitl);

    // Verify cryptographic UUID v4 token
    let token_str = hitl.hitl_token.as_ref().expect("HITL token must exist");
    let parsed_uuid = Uuid::parse_str(token_str).expect("HITL token must be a valid UUID");
    assert_eq!(
        parsed_uuid.get_version(),
        Some(uuid::Version::Random),
        "HITL token must be RFC 4122 UUID v4"
    );

    // 1.4 Mixed Direction Attack on N-to-1 Split:
    // Ledger entry is CREDIT of 80,000,000 VND.
    // Bank transactions:
    // - Tx 1: CREDIT 50,000,000 VND
    // - Tx 2: CREDIT 10,000,000 VND
    // - Tx 3: DEBIT 20,000,000 VND (Opposite direction!)
    // Total sum = 80,000,000 VND if signs are ignored.
    let led_target = make_ledger_entry(
        "led_target_80m",
        TransactionType::Credit,
        80_000_000,
        base_time + 100,
        "HD-TARGET",
        Some("CONG TY XYZ"),
        "Hop dong 80m",
    );
    let tx1 = make_bank_tx(
        "tx1_c_50m",
        TransactionType::Credit,
        50_000_000,
        base_time,
        base_time,
        None,
        "CONG TY XYZ TT HD-TARGET DOT 1",
        Some("CONG TY XYZ"),
    );
    let tx2 = make_bank_tx(
        "tx2_c_10m",
        TransactionType::Credit,
        10_000_000,
        base_time,
        base_time,
        None,
        "CONG TY XYZ TT HD-TARGET DOT 2",
        Some("CONG TY XYZ"),
    );
    let tx3 = make_bank_tx(
        "tx3_d_20m",
        TransactionType::Debit, // MALICIOUS: Opposite direction!
        20_000_000,
        base_time,
        base_time,
        None,
        "CONG TY XYZ TT HD-TARGET DOT 3",
        Some("CONG TY XYZ"),
    );

    let (split_matches_n1, hitl_matches_n1) = SplitSolver::match_tier3(
        &[tx1.clone(), tx2.clone(), tx3.clone()],
        &[led_target.clone()],
        &[0, 1, 2],
        &[0],
    );

    assert!(
        split_matches_n1.is_empty(),
        "N-to-1 solver MUST NOT combine Credit and Debit bank payments"
    );
    assert_eq!(hitl_matches_n1.len(), 3, "All 3 bank txs must be in HITL");
}

// ===========================================================================
// 2. Value Date 24h Boundary Stress & Trace Extraction
// ===========================================================================

#[test]
fn test_challenger2_value_date_boundary_and_trace_extraction() {
    let t0 = 1_726_100_000i64;
    const WINDOW_24H: i64 = 86_400;

    let mk_tx = |id: &str, tx_d: i64, val_d: i64, narration: &str| {
        make_bank_tx(
            id,
            TransactionType::Credit,
            15_000_000,
            tx_d,
            val_d,
            None,
            narration,
            None,
        )
    };

    let mk_led = |id: &str, ent_d: i64, doc_no: &str| {
        make_ledger_entry(
            id,
            TransactionType::Credit,
            15_000_000,
            ent_d,
            doc_no,
            None,
            "Reconciliation test",
        )
    };

    // 2.1 Boundary verification: 86,400s vs 86,401s
    let led = mk_led("led_exact", t0, "HD100");

    // Exactly +86,400s: Must match
    let tx_plus_86400 = mk_tx(
        "tx_p_86400",
        t0 + WINDOW_24H,
        t0 + WINDOW_24H,
        "Thanh toan HD100",
    );
    let (m1, _, _) = HashMatcher::match_tier1(&[tx_plus_86400], std::slice::from_ref(&led));
    assert_eq!(m1.len(), 1, "Exactly +86,400s MUST match in Tier 1");

    // Exactly -86,400s: Must match
    let tx_minus_86400 = mk_tx(
        "tx_m_86400",
        t0 - WINDOW_24H,
        t0 - WINDOW_24H,
        "Thanh toan HD100",
    );
    let (m2, _, _) = HashMatcher::match_tier1(&[tx_minus_86400], std::slice::from_ref(&led));
    assert_eq!(m2.len(), 1, "Exactly -86,400s MUST match in Tier 1");

    // Boundary +86,401s: Must REJECT
    let tx_plus_86401 = mk_tx(
        "tx_p_86401",
        t0 + WINDOW_24H + 1,
        t0 + WINDOW_24H + 1,
        "Thanh toan HD100",
    );
    let (m3, _, _) = HashMatcher::match_tier1(&[tx_plus_86401], std::slice::from_ref(&led));
    assert_eq!(m3.len(), 0, "+86,401s MUST be rejected from Tier 1");

    // Boundary -86,401s: Must REJECT
    let tx_minus_86401 = mk_tx(
        "tx_m_86401",
        t0 - WINDOW_24H - 1,
        t0 - WINDOW_24H - 1,
        "Thanh toan HD100",
    );
    let (m4, _, _) = HashMatcher::match_tier1(&[tx_minus_86401], std::slice::from_ref(&led));
    assert_eq!(m4.len(), 0, "-86,401s MUST be rejected from Tier 1");

    // 2.2 Value Date Priority vs tx_date Fallback
    // Case A: value_date = 0, falls back to tx_date
    let tx_fallback_ok = mk_tx("tx_fb_ok", t0 + 10_000, 0, "Thanh toan HD100");
    let (m5, _, _) = HashMatcher::match_tier1(&[tx_fallback_ok], std::slice::from_ref(&led));
    assert_eq!(
        m5.len(),
        1,
        "value_date == 0 must fallback to tx_date within 24h"
    );

    let tx_fallback_fail = mk_tx("tx_fb_fail", t0 + WINDOW_24H + 10, 0, "Thanh toan HD100");
    let (m6, _, _) = HashMatcher::match_tier1(&[tx_fallback_fail], std::slice::from_ref(&led));
    assert_eq!(
        m6.len(),
        0,
        "value_date == 0 fallback to tx_date > 24h must reject"
    );

    // Case B: value_date > 0 overrides stale tx_date (e.g., interbank weekend lag)
    // tx_date is 5 days earlier (432,000s earlier), but value_date is within 1 hour of ledger
    let tx_weekend_val_ok = mk_tx("tx_wk_ok", t0 - 432_000, t0 + 3_600, "Thanh toan HD100");
    let (m7, _, _) = HashMatcher::match_tier1(&[tx_weekend_val_ok], std::slice::from_ref(&led));
    assert_eq!(
        m7.len(),
        1,
        "Non-zero value_date must override stale tx_date"
    );

    // Case C: value_date > 0 exceeds 24h even though tx_date was identical to ledger
    let tx_val_exceed = mk_tx("tx_val_exc", t0, t0 + WINDOW_24H + 500, "Thanh toan HD100");
    let (m8, _, _) = HashMatcher::match_tier1(&[tx_val_exceed], std::slice::from_ref(&led));
    assert_eq!(
        m8.len(),
        0,
        "value_date exceeding 24h must reject even if tx_date matches"
    );

    // 2.3 Trace Code Extraction in Tier 1 (FT, NPS, VN, and zero-padded doc refs)
    // FT Trace
    let tx_ft = mk_tx("tx_ft", t0, t0, "MBVCB FT240915123456 chuyen tien");
    let led_ft = mk_led("led_ft", t0, "FT240915123456");
    let (m_ft, _, _) = HashMatcher::match_tier1(&[tx_ft], std::slice::from_ref(&led_ft));
    assert_eq!(m_ft.len(), 1, "FT trace code must be extracted and matched");

    // NPS Napas Trace
    let tx_nps = mk_tx("tx_nps", t0, t0, "Napas VietQR TT NPS2625689001 thanh toan");
    let led_nps = mk_led("led_nps", t0, "NPS2625689001");
    let (m_nps, _, _) = HashMatcher::match_tier1(&[tx_nps], std::slice::from_ref(&led_nps));
    assert_eq!(
        m_nps.len(),
        1,
        "NPS trace code must be extracted and matched"
    );

    // VN Trace
    let tx_vn = mk_tx("tx_vn", t0, t0, "QRIBFT VN998877665544 noi dung tt");
    let led_vn = mk_led("led_vn", t0, "VN998877665544");
    let (m_vn, _, _) = HashMatcher::match_tier1(&[tx_vn], std::slice::from_ref(&led_vn));
    assert_eq!(m_vn.len(), 1, "VN trace code must be extracted and matched");

    // Zero-padded invoice reference normalization: HD0000456 vs HD456
    let tx_padded = mk_tx("tx_pad", t0, t0, "Thanh toan tien hang HD0000456");
    let led_normalized = mk_led("led_norm", t0, "HD456");
    let (m_norm, _, _) =
        HashMatcher::match_tier1(&[tx_padded], std::slice::from_ref(&led_normalized));
    assert_eq!(
        m_norm.len(),
        1,
        "Normalized HD0000456 and HD456 must match in Tier 1"
    );
}

// ===========================================================================
// 3. Vietnamese Bank Noise Stripping & Unicode NFC/NFD Equivalence
// ===========================================================================

#[test]
fn test_challenger2_vietnamese_bank_noise_stripping_and_unicode_resilience() {
    // 3.1 Complex multi-bank commercial narration strings
    let vcb_noisy = "MBVCB.1234567890.CONG TY TNHH MINH ANH.CT TU TK 0011001234567 DEN TK 0071007654321 NOIDUNG THANH TOAN TIEN HANG HD 105";
    let stripped_vcb = strip_bank_narration_noise(vcb_noisy);
    assert!(
        stripped_vcb.contains("cong ty tnhh minh anh"),
        "Company name must be preserved: got '{}'",
        stripped_vcb
    );
    assert!(
        !stripped_vcb.contains("mbvcb"),
        "MBVCB prefix must be stripped"
    );
    assert!(
        !stripped_vcb.contains("1234567890"),
        "Account number sequence must be stripped"
    );
    assert!(
        !stripped_vcb.contains("0011001234567"),
        "Sender account must be stripped"
    );

    let tcb_noisy = "Napas VietQR TT FT2409158888 Tu: NGUYEN VAN A sang tk CONG TY CO PHAN XYZ nd chuyen khoan hoa don 999";
    let stripped_tcb = strip_bank_narration_noise(tcb_noisy);
    assert!(
        stripped_tcb.contains("nguyen van a"),
        "Sender name preserved: got '{}'",
        stripped_tcb
    );
    assert!(
        stripped_tcb.contains("cong ty co phan xyz"),
        "Recipient name preserved: got '{}'",
        stripped_tcb
    );
    assert!(
        !stripped_tcb.contains("napas"),
        "Napas prefix must be stripped"
    );
    assert!(
        !stripped_tcb.contains("vietqr"),
        "VietQR prefix must be stripped"
    );
    assert!(
        !stripped_tcb.contains("ft2409158888"),
        "FT trace must be stripped from party narration"
    );

    let bidv_noisy =
        "BIDV IBFT chuyen tien tu tk 123456789999 den tk 987654321111 CTY MINH DUC ck hd 202";
    let stripped_bidv = strip_bank_narration_noise(bidv_noisy);
    assert!(
        stripped_bidv.contains("cty minh duc"),
        "Company preserved: got '{}'",
        stripped_bidv
    );
    assert!(
        !stripped_bidv.contains("123456789999"),
        "BIDV account stripped"
    );

    // 3.2 Unicode NFC vs NFD Equivalence & Diacritics Normalization
    // NFC composed: "CÔNG TY CỔ PHẦN ĐẦU TƯ MINH ANH"
    let nfc_text = "CÔNG TY CỔ PHẦN ĐẦU TƯ MINH ANH";
    // NFD decomposed: 'O' + Combining Circumflex (\u{0302}) + Combining Acute (\u{0301}) etc.
    let nfd_text = "CO\u{0302}\u{0301}NG TY CO\u{0309} PHA\u{0302}\u{0300}N ĐA\u{0302}\u{0300}U TU\u{031B} MINH ANH";

    let norm_nfc = normalize_vietnamese_text(nfc_text);
    let norm_nfd = normalize_vietnamese_text(nfd_text);

    assert_eq!(
        norm_nfc, norm_nfd,
        "NFC and NFD Vietnamese strings must normalize to identical ASCII tokens"
    );

    let sim_score = compare_party_names(nfc_text, nfd_text);
    assert!(
        (sim_score - 1.0).abs() < 1e-6,
        "NFC vs NFD party names must produce identical 1.0 similarity score: got {}",
        sim_score
    );

    // 3.3 Digit filtering boundary: 5 digits kept vs 6 digits removed
    let text_with_codes = "THANH TOAN 12345 VA 123456 CHO CTY ABC";
    let stripped_codes = strip_bank_narration_noise(text_with_codes);
    assert!(
        stripped_codes.contains("12345"),
        "5-digit sequence must NOT be stripped: got '{}'",
        stripped_codes
    );
    assert!(
        !stripped_codes.contains("123456"),
        "6-digit sequence MUST be stripped: got '{}'",
        stripped_codes
    );
}

// ===========================================================================
// 4. Bidirectional Split Solving & BnB Bounds Stress
// ===========================================================================

#[test]
fn test_challenger2_bidirectional_split_and_bnb_depth_stress() {
    let t0 = 1_726_200_000i64;

    // 4.1 Phase A: 1 Bank Transaction -> 3 Ledger Invoices (Composite)
    let tx_composite = make_bank_tx(
        "tx_comp_100",
        TransactionType::Credit,
        100_000_000,
        t0,
        t0,
        None,
        "THANH TOAN HD-C1 HD-C2 HD-C3 CHO CONG TY PHAT DAT",
        Some("CONG TY PHAT DAT"),
    );
    let led1 = make_ledger_entry(
        "led_c1",
        TransactionType::Credit,
        25_000_000,
        t0 + 10,
        "HD-C1",
        Some("Công ty Phát Đạt"),
        "Dot 1",
    );
    let led2 = make_ledger_entry(
        "led_c2",
        TransactionType::Credit,
        35_000_000,
        t0 + 20,
        "HD-C2",
        Some("Công ty Phát Đạt"),
        "Dot 2",
    );
    let led3 = make_ledger_entry(
        "led_c3",
        TransactionType::Credit,
        40_000_000,
        t0 + 30,
        "HD-C3",
        Some("Công ty Phát Đạt"),
        "Dot 3",
    );

    let (matches_comp, hitl_comp) =
        SplitSolver::match_tier3(&[tx_composite], &[led1, led2, led3], &[0], &[0, 1, 2]);
    assert_eq!(matches_comp.len(), 1, "1-to-3 split must be matched");
    assert!(
        hitl_comp.is_empty(),
        "No HITL items expected for clean split"
    );
    let m = &matches_comp[0];
    assert_eq!(m.matched_amount, 100_000_000);
    assert_eq!(m.discrepancy_amount, 0);
    assert!(m.is_split_one_to_n());
    assert_eq!(m.ledger_entry_ids.len(), 3);

    // 4.2 Phase B: 4 Bank Transactions -> 1 Ledger Invoice (Multi-Installment)
    let led_single = make_ledger_entry(
        "led_inst_120",
        TransactionType::Debit,
        120_000_000,
        t0 + 10,
        "HD-INST-120",
        Some("CONG TY VINAHOUSE"),
        "Hop dong tong",
    );
    let tx_inst1 = make_bank_tx(
        "tx_inst_1",
        TransactionType::Debit,
        30_000_000,
        t0 + 100,
        t0 + 100,
        None,
        "CTY VINAHOUSE TT HD-INST-120 KY 1",
        Some("CONG TY VINAHOUSE"),
    );
    let tx_inst2 = make_bank_tx(
        "tx_inst_2",
        TransactionType::Debit,
        20_000_000,
        t0 + 200,
        t0 + 200,
        None,
        "CTY VINAHOUSE TT HD-INST-120 KY 2",
        Some("CONG TY VINAHOUSE"),
    );
    let tx_inst3 = make_bank_tx(
        "tx_inst_3",
        TransactionType::Debit,
        50_000_000,
        t0 + 300,
        t0 + 300,
        None,
        "CTY VINAHOUSE TT HD-INST-120 KY 3",
        Some("CONG TY VINAHOUSE"),
    );
    let tx_inst4 = make_bank_tx(
        "tx_inst_4",
        TransactionType::Debit,
        20_000_000,
        t0 + 400,
        t0 + 400,
        None,
        "CTY VINAHOUSE TT HD-INST-120 KY 4",
        Some("CONG TY VINAHOUSE"),
    );

    let (matches_inst, hitl_inst) = SplitSolver::match_tier3(
        &[tx_inst1, tx_inst2, tx_inst3, tx_inst4],
        &[led_single],
        &[0, 1, 2, 3],
        &[0],
    );
    assert_eq!(matches_inst.len(), 1, "4-to-1 split must be matched");
    assert!(hitl_inst.is_empty(), "No HITL items expected");
    let m_inst = &matches_inst[0];
    assert_eq!(m_inst.matched_amount, 120_000_000);
    assert_eq!(m_inst.discrepancy_amount, 0);
    assert!(m_inst.is_split_n_to_one());
    assert_eq!(m_inst.bank_tx_ids.len(), 4);

    // 4.3 Default Max Depth 4 Contract Test:
    // 5 items of 10,000,000 summing to 50,000,000 VND
    let tx_5items = make_bank_tx(
        "tx_5i",
        TransactionType::Credit,
        50_000_000,
        t0,
        t0,
        None,
        "THANH TOAN 5 HOA DON HD-K1 HD-K2 HD-K3 HD-K4 HD-K5",
        Some("CONG TY KHOI NGUYEN"),
    );
    let led_items: Vec<InternalLedgerEntry> = (1..=5)
        .map(|i| {
            make_ledger_entry(
                &format!("led_k{i}"),
                TransactionType::Credit,
                10_000_000,
                t0 + i * 10,
                &format!("HD-K{i}"),
                Some("CONG TY KHOI NGUYEN"),
                "Tung dot",
            )
        })
        .collect();

    // Default depth 4: MUST FAIL-CLOSED TO HITL!
    let (matches_def, hitl_def) =
        SplitSolver::match_tier3(&[tx_5items.clone()], &led_items, &[0], &[0, 1, 2, 3, 4]);
    assert_eq!(
        matches_def.len(),
        0,
        "5-item combination MUST NOT auto-match under default max depth 4"
    );
    assert_eq!(hitl_def.len(), 1, "5-item transaction must route to HITL");

    // Configurable depth 5 via match_tier3_with_depth: MUST SUCCEED!
    let (matches_d5, hitl_d5) =
        SplitSolver::match_tier3_with_depth(&[tx_5items], &led_items, &[0], &[0, 1, 2, 3, 4], 5);
    assert_eq!(
        matches_d5.len(),
        1,
        "5-item combination must succeed when max_depth >= 5"
    );
    assert!(hitl_d5.is_empty());
    assert_eq!(matches_d5[0].ledger_entry_ids.len(), 5);

    // 4.4 Branch-and-Bound Subset-Sum Bounds & Extremes (depth <= 8)
    // 8 items of 10,000,000 VND -> Target 80,000,000 VND
    let candidates_8: Vec<(usize, u64)> = (0..8).map(|i| (i, 10_000_000)).collect();
    let res8 = solve_exact_subset_sum_bnb(&candidates_8, 80_000_000, 8);
    assert!(
        res8.is_some(),
        "8-item exact subset sum must succeed at depth 8"
    );
    assert_eq!(res8.unwrap().len(), 8);

    // 9 items of 10,000,000 VND -> Target 90,000,000 VND with max_depth 8: MUST FAIL
    let candidates_9: Vec<(usize, u64)> = (0..9).map(|i| (i, 10_000_000)).collect();
    let res9 = solve_exact_subset_sum_bnb(&candidates_9, 90_000_000, 8);
    assert!(
        res9.is_none(),
        "9-item subset sum must return None when max depth is capped at 8"
    );

    // 4.5 Adversarial Subset-Sum Hardness:
    // Greedy heuristic trap: Target 100M VND. Candidates: [60M, 50M, 50M, 30M].
    // Greedy picks 60M, then cannot form 40M from remaining.
    // Optimal BnB explores and finds 50M + 50M = 100M!
    let trap_candidates = vec![
        (0, 60_000_000),
        (1, 50_000_000),
        (2, 50_000_000),
        (3, 30_000_000),
    ];
    let trap_res = solve_exact_subset_sum_bnb(&trap_candidates, 100_000_000, 4);
    assert!(trap_res.is_some(), "BnB must solve greedy trap");
    let sol = trap_res.unwrap();
    let sol_sum: u64 = sol.iter().map(|&idx| trap_candidates[idx].1).sum();
    assert_eq!(sol_sum, 100_000_000);

    // Off-by-one 1 VND drift: Target 100,000,000. Sum of candidates: 99,999,999 VND.
    let off_by_one = vec![(0, 50_000_000), (1, 49_999_999)];
    let off_res = solve_exact_subset_sum_bnb(&off_by_one, 100_000_000, 4);
    assert!(
        off_res.is_none(),
        "1 VND drift must strictly return None (Delta == 0 only)"
    );

    // Boundary edge cases: target = 0, empty list, all zero
    assert!(solve_exact_subset_sum_bnb(&[], 100, 4).is_none());
    assert!(solve_exact_subset_sum_bnb(&[(0, 50)], 0, 4).is_none());
    assert!(solve_exact_subset_sum_bnb(&[(0, 0), (1, 0)], 50, 4).is_none());
}

// ===========================================================================
// 5. Balance Invariants Runtime Check & Zero Float Drift Audit
// ===========================================================================

#[test]
fn test_challenger2_balance_invariants_and_zero_float_drift() {
    let t0 = 1_726_300_000i64;

    // 5.1 Account 1: Strictly consistent running balances
    // Bal0 = 100M
    // Tx 1: Credit 20M -> Bal1 = 120M
    // Tx 2: Debit  30M -> Bal2 =  90M
    // Tx 3: Credit 15M -> Bal3 = 105M
    let txs_acc1 = vec![
        BankTransactionRow {
            id: "tx_b1".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: t0 + 100,
            value_date: t0 + 100,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 20_000_000,
            balance_after: Some(120_000_000),
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Thu tien hang".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: t0 + 100,
        },
        BankTransactionRow {
            id: "tx_b2".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: t0 + 200,
            value_date: t0 + 200,
            doc_ref: None,
            tx_type: TransactionType::Debit,
            amount: 30_000_000,
            balance_after: Some(90_000_000),
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Chi tien hang".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: t0 + 200,
        },
        BankTransactionRow {
            id: "tx_b3".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: t0 + 300,
            value_date: t0 + 300,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 15_000_000,
            balance_after: Some(105_000_000),
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Thu tien hd".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: t0 + 300,
        },
    ];

    let (checked1, passed1, disc1) = ReconciliationEngine::check_balance_invariants(&txs_acc1);
    assert!(checked1, "Should have checked account 1 transitions");
    assert!(passed1, "Account 1 running balances must pass");
    assert_eq!(disc1, 0, "Account 1 discrepancy must be 0");

    // 5.2 Out-of-Order Timestamp Robustness:
    // Shuffle txs_acc1 and verify check_balance_invariants still sorts and passes!
    let mut shuffled = txs_acc1.clone();
    shuffled.swap(0, 2);
    let (checked_shuf, passed_shuf, disc_shuf) =
        ReconciliationEngine::check_balance_invariants(&shuffled);
    assert!(checked_shuf);
    assert!(
        passed_shuf,
        "Internal chronological sort must handle out-of-order input rows"
    );
    assert_eq!(disc_shuf, 0);

    // 5.3 Account 2: Introduces a hidden 5,000,000 VND discrepancy
    // Tx 2 expected balance is 90M, but bank record claims 95M (+5M unrecorded deposit or bank error)
    let tx_corrupt = BankTransactionRow {
        id: "tx_b2_corrupt".to_string(),
        statement_id: "stmt_2".to_string(),
        account_id: "acc_2".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: t0 + 200,
        value_date: t0 + 200,
        doc_ref: None,
        tx_type: TransactionType::Debit,
        amount: 30_000_000,
        balance_after: Some(95_000_000), // Expected 90_000_000! Diff = +5,000,000
        counterparty_account: None,
        counterparty_name: None,
        counterparty_bank: None,
        narration: "Chi sai so du".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: t0 + 200,
    };

    let mut txs_acc2 = txs_acc1.clone();
    for t in &mut txs_acc2 {
        t.statement_id = "stmt_2".to_string();
        t.account_id = "acc_2".to_string();
    }
    txs_acc2[1] = tx_corrupt;

    let (checked2, passed2, disc2) = ReconciliationEngine::check_balance_invariants(&txs_acc2);
    assert!(checked2);
    assert!(
        !passed2,
        "Discrepancy in Account 2 must fail invariant check"
    );
    assert_eq!(disc2, 5_000_000, "Must report exact 5,000,000 discrepancy");

    // 5.4 Multi-Account Batch Check:
    // Combining Account 1 (valid) and Account 2 (corrupted) into a single batch
    let mut combined_batch = txs_acc1.clone();
    combined_batch.extend(txs_acc2);

    let (checked_comb, passed_comb, disc_comb) =
        ReconciliationEngine::check_balance_invariants(&combined_batch);
    assert!(checked_comb);
    assert!(
        !passed_comb,
        "Combined batch containing corrupted account must fail"
    );
    assert_eq!(disc_comb, 5_000_000);

    // 5.5 End-to-End Reconciliation Summary Invariant Reflection:
    let (_matches, summary) = ReconciliationEngine::reconcile(&combined_batch, &[]);
    assert!(summary.balance_invariant_checked);
    assert!(!summary.balance_invariant_passed);
    assert_eq!(summary.balance_discrepancy_amount, 5_000_000);
    assert!(
        summary.discrepancy_count >= 1,
        "Failed balance invariant must increment discrepancy_count in summary"
    );
}
