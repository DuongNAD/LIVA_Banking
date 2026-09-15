//! Milestone 4 (M4) Empirical Adversarial Stress Test Suite (Challenger 1)
//!
//! White-box coverage and empirical adversarial challenge for Milestone 4 deliverables:
//! 1. Empirical Test 1: Multiple compound sector descriptors in single company name.
//!    - Confirms similarity < 0.70 on unrelated entities packed with multiple corporate/sector phrases.
//!    - Confirms Tier 2 FuzzyMatcher and full ReconciliationEngine correctly reject false matching.
//! 2. Empirical Test 2: Mixed abbreviations and case variations ("BDS", "bds", "TMDV", "tmdv").
//!    - Confirms uniform stripping and correct matching (score >= 0.85) for genuine company variants.
//!    - Confirms negative control: unrelated companies with identical abbreviations still score < 0.70.
//! 3. Empirical Test 3: Complex multi-part hyphenated and slashed document codes.
//!    - ("HD-2026/08/HN-001", "INV-2026-Q3-999", "PC-001/A1", "PT-2026-0042/DN").
//!    - Confirms token preservation in narration splitters, uniform normalization, and O(1) matching.
//! 4. Empirical Test 4: Direction invariance under adversarial multi-installment matching.
//!    - Credit installments must NEVER match Debit invoice, even with identical doc ref and exact sum.
//!    - Debit installments must NEVER match Credit invoice.
//! 5. Empirical Test 5: Double-allocation immunity in Tier 3.
//!    - Confirms that under complex subset sums and competing permutations, no bank transaction
//!      or ledger invoice is ever allocated more than once (strictly disjoint allocations).

use std::collections::HashSet;

use liva_native_core::banking::models::*;
use liva_native_core::banking::reconciliation::ReconciliationEngine;
use liva_native_core::banking::reconciliation::fuzzy_matcher::FuzzyMatcher;
use liva_native_core::banking::reconciliation::hash_matcher::{HashMatcher, normalize_doc_ref};
use liva_native_core::banking::reconciliation::jaro_winkler::{
    compare_party_names, normalize_vietnamese_text, strip_bank_narration_noise,
    strip_corporate_legal_noise,
};
use liva_native_core::banking::reconciliation::split_solver::SplitSolver;

// Helper: Generates a BankTransactionRow fixture with customizable fields.
fn make_bank_tx(
    id: &str,
    amount: u64,
    tx_type: TransactionType,
    doc_ref: Option<&str>,
    cp_name: Option<&str>,
    narration: &str,
    time: i64,
) -> BankTransactionRow {
    BankTransactionRow {
        id: id.to_string(),
        statement_id: "stmt_m4_test".to_string(),
        account_id: "acc_m4_test".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: time,
        value_date: time,
        doc_ref: doc_ref.map(|s| s.to_string()),
        tx_type,
        amount,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: cp_name.map(|s| s.to_string()),
        counterparty_bank: None,
        narration: narration.to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: time,
    }
}

// Helper: Generates an InternalLedgerEntry fixture with customizable fields.
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
        account_id: "acc_m4_test".to_string(),
        doc_no: doc_no.to_string(),
        entry_date: time,
        entry_type,
        amount,
        partner_code: Some("PARTNER_TEST".to_string()),
        partner_name: partner_name.map(|s| s.to_string()),
        description: format!("Ledger invoice {doc_no}"),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: time,
    }
}

// ===========================================================================
// Empirical Test 1: Multiple Compound Sector Descriptors in Single Company Name
// ===========================================================================

#[test]
fn test_m4_challenger1_multiple_compound_sector_descriptors_rejection() {
    // 1. Primary Empirical Case from Specification:
    // Company A: "CÔNG TY CỔ PHẦN ĐẦU TƯ XÂY DỰNG VÀ BẤT ĐỘNG SẢN MINH PHÚ"
    // Company B: "CÔNG TY TNHH ĐẦU TƯ XÂY DỰNG SÔNG HỒNG"
    let name_a1 = "CÔNG TY CỔ PHẦN ĐẦU TƯ XÂY DỰNG VÀ BẤT ĐỘNG SẢN MINH PHÚ";
    let name_b1 = "CÔNG TY TNHH ĐẦU TƯ XÂY DỰNG SÔNG HỒNG";

    let score1 = compare_party_names(name_a1, name_b1);
    assert!(
        score1 < 0.70,
        "Empirical Case 1 Failed: Similarity must be < 0.70, got {score1}"
    );

    // White-box verification of stripped core tokens
    let norm_a1 = normalize_vietnamese_text(name_a1);
    let norm_b1 = normalize_vietnamese_text(name_b1);
    let (tokens_a1, core_a1) = strip_corporate_legal_noise(&norm_a1);
    let (tokens_b1, core_b1) = strip_corporate_legal_noise(&norm_b1);
    assert!(!tokens_a1.is_empty(), "Tokens A1 should not be empty");
    assert!(!tokens_b1.is_empty(), "Tokens B1 should not be empty");

    assert!(
        !core_a1.contains("dau tu")
            && !core_a1.contains("xay dung")
            && !core_a1.contains("bat dong san"),
        "Sector descriptors must be stripped from core_a1: '{core_a1}'"
    );
    assert!(
        !core_b1.contains("dau tu") && !core_b1.contains("xay dung"),
        "Sector descriptors must be stripped from core_b1: '{core_b1}'"
    );
    assert!(
        core_a1.contains("minh phu"),
        "Core brand tokens 'minh phu' must remain in core_a1: '{core_a1}'"
    );
    assert!(
        core_b1.contains("song hong"),
        "Core brand tokens 'song hong' must remain in core_b1: '{core_b1}'"
    );

    // 2. Compound Multi-Sector Cluster 2:
    // "TẬP ĐOÀN ĐẦU TƯ XÂY DỰNG THƯƠNG MẠI DỊCH VỤ VẬN TẢI THÀNH CÔNG" vs "CÔNG TY CỔ PHẦN ĐẦU TƯ XÂY DỰNG THƯƠNG MẠI DỊCH VỤ PHÁT ĐẠT"
    let name_a2 = "TẬP ĐOÀN ĐẦU TƯ XÂY DỰNG THƯƠNG MẠI DỊCH VỤ VẬN TẢI THÀNH CÔNG";
    let name_b2 = "CÔNG TY CỔ PHẦN ĐẦU TƯ XÂY DỰNG THƯƠNG MẠI DỊCH VỤ PHÁT ĐẠT";
    let score2 = compare_party_names(name_a2, name_b2);
    assert!(
        score2 < 0.70,
        "Empirical Case 2 Failed: Similarity must be < 0.70, got {score2}"
    );

    // 3. Compound Multi-Sector Cluster 3 (Pharmaceutical + Healthcare + Manufacturing):
    // "CÔNG TY TNHH SẢN XUẤT THƯƠNG MẠI DƯỢC PHẨM Y TẾ BÌNH MINH" vs "CÔNG TY CỔ PHẦN DƯỢC PHẨM Y TẾ PHÚ HƯNG"
    let name_a3 = "CÔNG TY TNHH SẢN XUẤT THƯƠNG MẠI DƯỢC PHẨM Y TẾ BÌNH MINH";
    let name_b3 = "CÔNG TY CỔ PHẦN DƯỢC PHẨM Y TẾ PHÚ HƯNG";
    let score3 = compare_party_names(name_a3, name_b3);
    assert!(
        score3 < 0.70,
        "Empirical Case 3 Failed: Similarity must be < 0.70, got {score3}"
    );

    // 4. Compound Multi-Sector Cluster 4 (Media + Tech + Education):
    // "TỔNG CÔNG TY TRUYỀN THÔNG CÔNG NGHỆ GIÁO DỤC SAO MAI" vs "CÔNG TY CỔ PHẦN TRUYỀN THÔNG CÔNG NGHỆ GIÁO DỤC THĂNG LONG"
    let name_a4 = "TỔNG CÔNG TY TRUYỀN THÔNG CÔNG NGHỆ GIÁO DỤC SAO MAI";
    let name_b4 = "CÔNG TY CỔ PHẦN TRUYỀN THÔNG CÔNG NGHỆ GIÁO DỤC THĂNG LONG";
    let score4 = compare_party_names(name_a4, name_b4);
    assert!(
        score4 < 0.70,
        "Empirical Case 4 Failed: Similarity must be < 0.70, got {score4}"
    );

    // Also verify strip_bank_narration_noise protects compound sector phrases "van tai" and "dau tu":
    let clean_narr =
        strip_bank_narration_noise("CK TIEN HANG CTY VAN TAI VA DAU TU MINH PHU TK 10293848493");
    assert!(
        clean_narr.contains("van tai"),
        "Narration noise stripper must preserve compound phrase 'van tai', got '{clean_narr}'"
    );
    assert!(
        clean_narr.contains("dau tu"),
        "Narration noise stripper must preserve compound phrase 'dau tu', got '{clean_narr}'"
    );

    // 5. Tier 2 FuzzyMatcher End-to-End Rejection Test:
    // Bank tx and ledger invoice have identical amounts (88_000_000 VND), same timestamp,
    // NO document reference in bank narration, but parties share multiple sector descriptors.
    let now = 1_726_000_000i64;
    let bank_tx = make_bank_tx(
        "tx_sector_clash",
        88_000_000,
        TransactionType::Credit,
        None,
        Some(name_a1),
        "THANH TOAN TIEN HANG THANG 8",
        now,
    );
    let ledger_entry = make_ledger_entry(
        "led_sector_clash",
        88_000_000,
        TransactionType::Credit,
        "INV-9999",
        Some(name_b1),
        now,
    );

    let (fuzzy_matches, unalloc_b, unalloc_l) =
        FuzzyMatcher::match_tier2(&[bank_tx.clone()], &[ledger_entry.clone()], &[0], &[0]);

    assert_eq!(
        fuzzy_matches.len(),
        0,
        "Tier 2 FuzzyMatcher must NOT match unrelated companies sharing sector descriptors"
    );
    assert_eq!(
        unalloc_b,
        vec![0],
        "Bank transaction must remain unallocated in Tier 2"
    );
    assert_eq!(
        unalloc_l,
        vec![0],
        "Ledger entry must remain unallocated in Tier 2"
    );

    // Also verify through full ReconciliationEngine:
    let (all_matches, summary) = ReconciliationEngine::reconcile(&[bank_tx], &[ledger_entry]);
    assert_eq!(
        summary.matched_fuzzy_count, 0,
        "ReconciliationEngine must report 0 fuzzy matches"
    );
    assert_eq!(
        summary.matched_exact_count, 0,
        "ReconciliationEngine must report 0 exact matches"
    );
    assert_eq!(
        summary.matched_split_count, 0,
        "ReconciliationEngine must report 0 split matches"
    );
    // Residue must route to HITL queue or remain unallocated
    for m in &all_matches {
        assert_ne!(
            m.match_type,
            MatchType::FuzzyHeuristic,
            "No match should be categorized as FuzzyHeuristic auto-approval"
        );
    }
}

// ===========================================================================
// Empirical Test 2: Mixed Abbreviations and Case Variations
// ===========================================================================

#[test]
fn test_m4_challenger1_abbreviations_and_case_variations_uniform_matching() {
    // 1. Variations of "BDS" / "bds" / "Bất Động Sản":
    let bds_pairs = [
        ("CTY CP BDS AN KHANG", "Công Ty TNHH Bất Động Sản An Khang"),
        ("cty bds an khang", "CÔNG TY BẤT ĐỘNG SẢN AN KHANG"),
        ("BDS AN KHANG", "CONG TY CP BDS AN KHANG"),
        ("CTY BDS AN KHANG", "AN KHANG BDS"),
        ("CONG TY TNHH BDS AN KHANG", "CTY CP BAT DONG SAN AN KHANG"),
    ];

    for (var1, var2) in bds_pairs {
        let score = compare_party_names(var1, var2);
        assert!(
            score >= 0.85,
            "BDS variant pair ('{var1}', '{var2}') must score >= 0.85, got {score}"
        );
    }

    // 2. Variations of "TMDV" / "tmdv" / "Thương Mại Dịch Vụ":
    let tmdv_pairs = [
        (
            "CTY TNHH TMDV THÁI BÌNH",
            "Công Ty Cổ Phần Thương Mại Dịch Vụ Thái Bình",
        ),
        ("cty tmdv thai binh", "CÔNG TY TM DV THÁI BÌNH"),
        ("TMDV THAI BINH", "CONG TY CP THUONG MAI DICH VU THAI BINH"),
        ("CTY TMDV THAI BINH", "THAI BINH TMDV"),
        (
            "CONG TY TNHH TMDV THAI BINH",
            "CTY CP THUONG MAI DICH VU THAI BINH",
        ),
    ];

    for (var1, var2) in tmdv_pairs {
        let score = compare_party_names(var1, var2);
        assert!(
            score >= 0.85,
            "TMDV variant pair ('{var1}', '{var2}') must score >= 0.85, got {score}"
        );
    }

    // 3. Combined Legal & Industry Abbreviations:
    let combined_pairs = [
        (
            "CTY CP TM DV & BDS HOANG GIA",
            "CONG TY TNHH BAT DONG SAN HOANG GIA",
        ),
        ("cty tnhh tmdv & bds hung phat", "CONG TY CO PHAN HUNG PHAT"),
    ];
    for (var1, var2) in combined_pairs {
        let score = compare_party_names(var1, var2);
        assert!(
            score >= 0.85,
            "Combined abbreviation pair ('{var1}', '{var2}') must score >= 0.85, got {score}"
        );
    }

    // 4. Adversarial Negative Control:
    // Unrelated companies that happen to share the abbreviation "BDS" or "TMDV"
    // MUST NOT match falsely and MUST score < 0.70.
    let negative_abbrev_pairs = [
        ("CTY BDS THIÊN LONG", "CÔNG TY BẤT ĐỘNG SẢN SÔNG HỒNG"),
        ("cty bds an gia", "CONG TY BAT DONG SAN PHU MY"),
        ("CTY TMDV KIM LONG", "CONG TY TM DV PHU THINH"),
        ("tmdv hoa sen", "CÔNG TY THƯƠNG MẠI DỊCH VỤ BẠCH ĐẰNG"),
    ];

    for (neg1, neg2) in negative_abbrev_pairs {
        let score = compare_party_names(neg1, neg2);
        assert!(
            score < 0.70,
            "Adversarial Negative Pair ('{neg1}', '{neg2}') must score < 0.70, got {score}"
        );
    }

    // 5. Tier 2 Fuzzy Matching for Genuine Abbreviation Variant:
    let now = 1_726_000_000i64;
    let bank_tx = make_bank_tx(
        "tx_bds_valid",
        45_000_000,
        TransactionType::Credit,
        None,
        Some("CTY BDS AN KHANG"),
        "THANH TOAN TIEN DICH VU",
        now,
    );
    let ledger_entry = make_ledger_entry(
        "led_bds_valid",
        45_000_000,
        TransactionType::Credit,
        "INV-8888",
        Some("CÔNG TY TNHH BẤT ĐỘNG SẢN AN KHANG"),
        now,
    );

    let (fuzzy_matches, _, _) = FuzzyMatcher::match_tier2(&[bank_tx], &[ledger_entry], &[0], &[0]);

    assert_eq!(
        fuzzy_matches.len(),
        1,
        "Tier 2 FuzzyMatcher must auto-match genuine company variants with abbreviations"
    );
    assert!(
        fuzzy_matches[0].confidence_score >= 0.85,
        "Confidence score must be >= 0.85, got {}",
        fuzzy_matches[0].confidence_score
    );
}

// ===========================================================================
// Empirical Test 3: Complex Multi-part Hyphenated and Slashed Document Codes
// ===========================================================================

#[test]
fn test_m4_challenger1_complex_multipart_hyphenated_slashed_doc_codes() {
    // 1. Doc Ref Normalization Invariance:
    let test_codes = [
        ("HD-2026/08/HN-001", "HD202608HN001"),
        ("HD-0002026-HN", "HD2026HN"),
        ("INV-2026-Q3-999", "INV2026Q3999"),
        ("INV-00456-A", "INV456A"),
        ("PC-001/A1", "PC001A1"),
        ("PT-2026-0042/DN", "PT20260042DN"),
        ("HD102/2026/TND", "HD1022026TND"),
    ];

    for (raw, expected_norm) in test_codes {
        let norm = normalize_doc_ref(raw);
        assert_eq!(
            norm, expected_norm,
            "normalize_doc_ref('{raw}') must produce '{expected_norm}', got '{norm}'"
        );
    }

    // 2. Token extraction under boundary punctuation and edge characters:
    // Slashes and hyphens inside token are preserved; external brackets, colons, spaces are stripped.
    let complex_narrations = [
        (
            "CTY ABC THANH TOAN (HD-2026/08/HN-001) DOT 1",
            "HD202608HN001",
        ),
        (
            "CHUYEN KHOAN HOA DON: [INV-2026-Q3-999] TIEN HANG",
            "INV2026Q3999",
        ),
        ("PHIEU CHI SO: PC-001/A1/ TAI KHOAN VCB", "PC001A1"),
        ("THANH TOAN PT-2026-0042/DN: TIEN CUOC", "PT20260042DN"),
    ];

    let now = 1_726_000_000i64;

    for (narration, expected_norm) in complex_narrations {
        let tx = make_bank_tx(
            "tx_extract_test",
            10_000_000,
            TransactionType::Credit,
            None,
            None,
            narration,
            now,
        );

        let mut extracted = Vec::new();
        for token in tx
            .narration
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '/')
        {
            let clean = token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_uppercase();
            if (clean.starts_with("HD") && clean.len() >= 3)
                || (clean.starts_with("INV") && clean.len() >= 4)
                || (clean.starts_with("PC") && clean.len() >= 3)
                || (clean.starts_with("PT") && clean.len() >= 3)
            {
                extracted.push(normalize_doc_ref(&clean));
            }
        }

        assert!(
            extracted.contains(&expected_norm.to_string()),
            "Narration '{narration}' must extract normalized token '{expected_norm}', got {extracted:?}"
        );
    }

    // 3. Tier 1 Exact Hash Matching on Complex Hyphenated/Slashed Document Codes:
    let tx1 = make_bank_tx(
        "tx_t1_complex",
        125_000_000,
        TransactionType::Credit,
        None,
        None,
        "THANH TOAN THEO HOP DONG (HD-2026/08/HN-001)",
        now,
    );
    let led1 = make_ledger_entry(
        "led_t1_complex",
        125_000_000,
        TransactionType::Credit,
        "HD-2026/08/HN-001",
        None,
        now,
    );

    let (tier1_matches, unalloc_b1, unalloc_l1) = HashMatcher::match_tier1(&[tx1], &[led1]);
    assert_eq!(
        tier1_matches.len(),
        1,
        "Tier 1 HashMatcher must match exact amount and complex slashed/hyphenated doc ref"
    );
    assert_eq!(tier1_matches[0].match_type, MatchType::Exact1To1);
    assert_eq!(unalloc_b1.len(), 0);
    assert_eq!(unalloc_l1.len(), 0);

    // 4. Tier 3 Multi-Installment Split Matching with Complex References:
    // Invoice: INV-2026-Q3-999, Total 150_000_000 VND
    // 3 Bank Installments: 50M + 60M + 40M VND, each mentioning INV-2026-Q3-999 in narration.
    let inst1 = make_bank_tx(
        "tx_inst_1",
        50_000_000,
        TransactionType::Credit,
        None,
        None,
        "THANH TOAN DOT 1 INV-2026-Q3-999",
        now,
    );
    let inst2 = make_bank_tx(
        "tx_inst_2",
        60_000_000,
        TransactionType::Credit,
        None,
        None,
        "THANH TOAN DOT 2 THEO [INV-2026-Q3-999]",
        now,
    );
    let inst3 = make_bank_tx(
        "tx_inst_3",
        40_000_000,
        TransactionType::Credit,
        None,
        None,
        "THANH TOAN DOT 3 INV-2026-Q3-999 HOAN TAT",
        now,
    );

    let invoice = make_ledger_entry(
        "led_inv_q3",
        150_000_000,
        TransactionType::Credit,
        "INV-2026-Q3-999",
        None,
        now,
    );

    let bank_txs = vec![inst1, inst2, inst3];
    let ledger_entries = vec![invoice];
    let unalloc_b: Vec<usize> = vec![0, 1, 2];
    let unalloc_l: Vec<usize> = vec![0];

    let (split_matches, hitl) =
        SplitSolver::match_tier3(&bank_txs, &ledger_entries, &unalloc_b, &unalloc_l);

    assert_eq!(
        split_matches.len(),
        1,
        "Must resolve 3-to-1 split match via O(1) doc ref indexing"
    );
    assert_eq!(hitl.len(), 0, "No residual items should route to HITL");
    assert_eq!(split_matches[0].matched_amount, 150_000_000);
    assert_eq!(split_matches[0].discrepancy_amount, 0);
    assert_eq!(split_matches[0].bank_tx_ids.len(), 3);
}

// ===========================================================================
// Empirical Test 4: Direction Invariance Under Adversarial Multi-Installment Matching
// ===========================================================================

#[test]
fn test_m4_challenger1_direction_invariance_adversarial_multi_installment() {
    let now = 1_726_000_000i64;

    // Case 1: Outgoing Debit Invoice (Accounts Payable, amount 200_000_000)
    // Adversarial Bank Transactions: 2 incoming Credit installments (120M + 80M = 200M)
    // sharing IDENTICAL doc_ref and narration!
    let led_debit = make_ledger_entry(
        "led_deb_invoice",
        200_000_000,
        TransactionType::Debit,
        "HD-DIR-TRAP-001",
        Some("CONG TY CUNG UNG VAT TU"),
        now,
    );

    let btx_cred1 = make_bank_tx(
        "btx_cred_1",
        120_000_000,
        TransactionType::Credit, // CREDIT (incoming) vs DEBIT (outgoing)
        Some("HD-DIR-TRAP-001"),
        Some("CONG TY CUNG UNG VAT TU"),
        "THANH TOAN HD-DIR-TRAP-001 DOT 1",
        now,
    );
    let btx_cred2 = make_bank_tx(
        "btx_cred_2",
        80_000_000,
        TransactionType::Credit, // CREDIT (incoming) vs DEBIT (outgoing)
        Some("HD-DIR-TRAP-001"),
        Some("CONG TY CUNG UNG VAT TU"),
        "THANH TOAN HD-DIR-TRAP-001 DOT 2",
        now,
    );

    let bank_txs_c = vec![btx_cred1, btx_cred2];
    let ledger_entries_d = vec![led_debit];

    // Tier 1 Exact Check: Must NOT match
    let (t1_m, unalloc_b_t1, unalloc_l_t1) =
        HashMatcher::match_tier1(&bank_txs_c, &ledger_entries_d);
    assert_eq!(
        t1_m.len(),
        0,
        "Direction Invariance: Tier 1 must reject Credit tx vs Debit ledger"
    );

    // Tier 2 Fuzzy Check: Must NOT match
    let (t2_m, unalloc_b_t2, unalloc_l_t2) =
        FuzzyMatcher::match_tier2(&bank_txs_c, &ledger_entries_d, &unalloc_b_t1, &unalloc_l_t1);
    assert_eq!(
        t2_m.len(),
        0,
        "Direction Invariance: Tier 2 must reject Credit tx vs Debit ledger"
    );

    // Tier 3 Split Solver Check: Must NEVER match Credit installments with Debit invoice!
    let (t3_m, _) =
        SplitSolver::match_tier3(&bank_txs_c, &ledger_entries_d, &unalloc_b_t2, &unalloc_l_t2);
    assert_eq!(
        t3_m.len(),
        0,
        "Direction Invariance Violation: Tier 3 matched Credit installments with Debit invoice!"
    );

    // Case 2: Incoming Credit Invoice (Accounts Receivable, amount 150_000_000)
    // Adversarial Bank Transactions: 2 outgoing Debit installments (90M + 60M = 150M)
    let led_credit = make_ledger_entry(
        "led_cred_invoice",
        150_000_000,
        TransactionType::Credit,
        "INV-DIR-TRAP-002",
        Some("KHACH HANG MINH ANH"),
        now,
    );

    let btx_deb1 = make_bank_tx(
        "btx_deb_1",
        90_000_000,
        TransactionType::Debit, // DEBIT vs CREDIT
        Some("INV-DIR-TRAP-002"),
        Some("KHACH HANG MINH ANH"),
        "THANH TOAN INV-DIR-TRAP-002 DOT 1",
        now,
    );
    let btx_deb2 = make_bank_tx(
        "btx_deb_2",
        60_000_000,
        TransactionType::Debit, // DEBIT vs CREDIT
        Some("INV-DIR-TRAP-002"),
        Some("KHACH HANG MINH ANH"),
        "THANH TOAN INV-DIR-TRAP-002 DOT 2",
        now,
    );

    let bank_txs_d = vec![btx_deb1, btx_deb2];
    let ledger_entries_c = vec![led_credit];

    let (t3_m_rev, _) = SplitSolver::match_tier3(&bank_txs_d, &ledger_entries_c, &[0, 1], &[0]);
    assert_eq!(
        t3_m_rev.len(),
        0,
        "Direction Invariance Violation: Tier 3 matched Debit installments with Credit invoice!"
    );

    // Case 3: Mixed Direction Poison Pill:
    // Invoice is Credit 100M. Candidate installments are Credit 50M and Debit 50M.
    let led_mixed = make_ledger_entry(
        "led_mixed",
        100_000_000,
        TransactionType::Credit,
        "HD-POISON-003",
        None,
        now,
    );
    let btx_pos1 = make_bank_tx(
        "btx_pos_credit",
        50_000_000,
        TransactionType::Credit,
        Some("HD-POISON-003"),
        None,
        "HD-POISON-003",
        now,
    );
    let btx_pos2 = make_bank_tx(
        "btx_pos_debit",
        50_000_000,
        TransactionType::Debit, // Poison pill opposite direction!
        Some("HD-POISON-003"),
        None,
        "HD-POISON-003",
        now,
    );

    let (t3_mixed, _) =
        SplitSolver::match_tier3(&[btx_pos1, btx_pos2], &[led_mixed], &[0, 1], &[0]);
    assert_eq!(
        t3_mixed.len(),
        0,
        "Direction Invariance: Cannot combine Credit and Debit transactions to satisfy split invoice"
    );

    // Full Pipeline Verification:
    let (all_m, summary) = ReconciliationEngine::reconcile(&bank_txs_c, &ledger_entries_d);
    assert_eq!(
        summary.total_matched_count, 0,
        "No direction-inverted items can be matched"
    );
    for m in all_m {
        assert_eq!(
            m.match_type,
            MatchType::ManualHitl,
            "Residual direction-mismatched items must only route to ManualHitl queue"
        );
    }
}

// ===========================================================================
// Empirical Test 5: Double-Allocation Immunity in Tier 3
// ===========================================================================

#[test]
fn test_m4_challenger1_double_allocation_immunity_under_complex_permutations() {
    let now = 1_726_000_000i64;

    // 1. Competing Invoices for Common Bank Installments:
    // L1 = 100_000_000 VND
    // L2 = 100_000_000 VND
    // Both L1 and L2 reference doc "HD-COMPETING".
    // Available bank installments: Tx1 (40M) + Tx2 (60M) = exactly 100M.
    // There is only ONE 100M subset available.
    // ONLY ONE invoice may match; the other MUST NOT consume Tx1 or Tx2.
    let l1 = make_ledger_entry(
        "led_comp_1",
        100_000_000,
        TransactionType::Credit,
        "HD-COMPETING",
        None,
        now,
    );
    let l2 = make_ledger_entry(
        "led_comp_2",
        100_000_000,
        TransactionType::Credit,
        "HD-COMPETING",
        None,
        now,
    );

    let b1 = make_bank_tx(
        "btx_comp_1",
        40_000_000,
        TransactionType::Credit,
        None,
        None,
        "HD-COMPETING DOT 1",
        now,
    );
    let b2 = make_bank_tx(
        "btx_comp_2",
        60_000_000,
        TransactionType::Credit,
        None,
        None,
        "HD-COMPETING DOT 2",
        now,
    );

    let (split_matches1, _hitl1) = SplitSolver::match_tier3(&[b1, b2], &[l1, l2], &[0, 1], &[0, 1]);

    assert_eq!(
        split_matches1.len(),
        1,
        "Exactly one split match should occur"
    );
    assert_eq!(split_matches1[0].matched_amount, 100_000_000);
    assert_eq!(split_matches1[0].bank_tx_ids.len(), 2);
    assert_eq!(split_matches1[0].ledger_entry_ids, vec!["led_comp_1"]);
    // Confirm competing ledger entry was NOT double-allocated:
    let matched_ledgers1: HashSet<String> = split_matches1
        .iter()
        .flat_map(|m| m.ledger_entry_ids.clone())
        .collect();
    assert!(
        !matched_ledgers1.contains("led_comp_2"),
        "Competing invoice led_comp_2 must NOT be matched!"
    );

    // 2. Multi-Subset Intersecting Candidates Permutation:
    // 4 Bank Transactions:
    // B_A1 = 30M, B_A2 = 70M  (Subset A = 100M)
    // B_B1 = 40M, B_B2 = 60M  (Subset B = 100M)
    // 1 Residual Bank Transaction: B_RES = 50M (unmatched)
    // 3 Ledger Invoices:
    // L_1 = 100M, L_2 = 100M, L_3 = 100M.
    // Only 2 subsets of 100M exist. L_3 CANNOT match!
    let b_a1 = make_bank_tx(
        "btx_sub_a1",
        30_000_000,
        TransactionType::Credit,
        None,
        None,
        "INV-BATCH-001",
        now,
    );
    let b_a2 = make_bank_tx(
        "btx_sub_a2",
        70_000_000,
        TransactionType::Credit,
        None,
        None,
        "INV-BATCH-001",
        now,
    );
    let b_b1 = make_bank_tx(
        "btx_sub_b1",
        40_000_000,
        TransactionType::Credit,
        None,
        None,
        "INV-BATCH-002",
        now,
    );
    let b_b2 = make_bank_tx(
        "btx_sub_b2",
        60_000_000,
        TransactionType::Credit,
        None,
        None,
        "INV-BATCH-002",
        now,
    );
    let b_res = make_bank_tx(
        "btx_sub_res",
        50_000_000,
        TransactionType::Credit,
        None,
        None,
        "INV-BATCH-001 DU",
        now,
    );

    let led_1 = make_ledger_entry(
        "led_sub_1",
        100_000_000,
        TransactionType::Credit,
        "INV-BATCH-001",
        None,
        now,
    );
    let led_2 = make_ledger_entry(
        "led_sub_2",
        100_000_000,
        TransactionType::Credit,
        "INV-BATCH-002",
        None,
        now,
    );
    let led_3 = make_ledger_entry(
        "led_sub_3",
        100_000_000,
        TransactionType::Credit,
        "INV-BATCH-001",
        None,
        now,
    ); // Competing on INV-BATCH-001!

    let bank_batch = vec![b_a1, b_a2, b_b1, b_b2, b_res];
    let ledger_batch = vec![led_1, led_2, led_3];

    let (split_matches2, hitl2) =
        SplitSolver::match_tier3(&bank_batch, &ledger_batch, &[0, 1, 2, 3, 4], &[0, 1, 2]);

    assert_eq!(
        split_matches2.len(),
        2,
        "Exactly 2 split matches must be formed"
    );
    assert_eq!(
        hitl2.len(),
        1,
        "Residual bank transaction must be quarantined to HITL"
    );
    assert_eq!(hitl2[0].bank_tx_id, "btx_sub_res");
    assert_eq!(hitl2[0].match_type, MatchType::ManualHitl);

    // Strictly verify disjoint bank allocations:
    let mut allocated_bank_ids = HashSet::new();
    let mut allocated_ledger_ids = HashSet::new();

    for m in &split_matches2 {
        for b_id in &m.bank_tx_ids {
            assert!(
                allocated_bank_ids.insert(b_id.clone()),
                "DOUBLE ALLOCATION DETECTED: Bank transaction {b_id} was allocated multiple times!"
            );
        }
        for l_id in &m.ledger_entry_ids {
            assert!(
                allocated_ledger_ids.insert(l_id.clone()),
                "DOUBLE ALLOCATION DETECTED: Ledger entry {l_id} was allocated multiple times!"
            );
        }
    }

    // 3. High-permutation Stress: 20 Bank Tx vs 20 Ledger Invoices
    // Interleaved 1-to-N, N-to-1, and 1-to-1 permutations with identical amounts and dates.
    let mut high_bank = Vec::new();
    let mut high_ledger = Vec::new();

    // 5 N-to-1 pairs: each ledger is 100M, paid by two 50M bank transactions
    for i in 0..5 {
        let doc = format!("HD-PERM-N1-{i:02}");
        high_bank.push(make_bank_tx(
            &format!("hb_n1_{i}_1"),
            50_000_000,
            TransactionType::Credit,
            None,
            None,
            &format!("THANH TOAN {doc} PART 1"),
            now,
        ));
        high_bank.push(make_bank_tx(
            &format!("hb_n1_{i}_2"),
            50_000_000,
            TransactionType::Credit,
            None,
            None,
            &format!("THANH TOAN {doc} PART 2"),
            now,
        ));
        high_ledger.push(make_ledger_entry(
            &format!("hl_n1_{i}"),
            100_000_000,
            TransactionType::Credit,
            &doc,
            None,
            now,
        ));
    }

    // 5 1-to-N pairs: each bank tx is 100M, paying two 50M ledger invoices
    for i in 0..5 {
        let doc1 = format!("HD-PERM-1N-{i:02}-A");
        let doc2 = format!("HD-PERM-1N-{i:02}-B");
        high_bank.push(make_bank_tx(
            &format!("hb_1n_{i}"),
            100_000_000,
            TransactionType::Credit,
            None,
            None,
            &format!("THANH TOAN {doc1} VA {doc2}"),
            now,
        ));
        high_ledger.push(make_ledger_entry(
            &format!("hl_1n_{i}_1"),
            50_000_000,
            TransactionType::Credit,
            &doc1,
            None,
            now,
        ));
        high_ledger.push(make_ledger_entry(
            &format!("hl_1n_{i}_2"),
            50_000_000,
            TransactionType::Credit,
            &doc2,
            None,
            now,
        ));
    }

    let (reconcile_matches, summary) = ReconciliationEngine::reconcile(&high_bank, &high_ledger);

    // Assert Double-Allocation Immunity across ENTIRE pipeline:
    let mut global_bank_seen = HashSet::new();
    let mut global_ledger_seen = HashSet::new();

    for m in &reconcile_matches {
        if m.match_type == MatchType::ManualHitl {
            continue; // HITL queue holds residuals
        }

        for b_id in &m.bank_tx_ids {
            assert!(
                global_bank_seen.insert(b_id.clone()),
                "DOUBLE ALLOCATION VIOLATION: Bank Tx '{b_id}' was matched in multiple matches!"
            );
        }

        for l_id in &m.ledger_entry_ids {
            assert!(
                global_ledger_seen.insert(l_id.clone()),
                "DOUBLE ALLOCATION VIOLATION: Ledger Entry '{l_id}' was matched in multiple matches!"
            );
        }

        // Arithmetic invariant check: Delta must be strictly 0
        assert_eq!(
            m.discrepancy_amount, 0,
            "Arithmetic Invariant Violation: Match discrepancy must be 0, got {}",
            m.discrepancy_amount
        );
    }

    assert!(
        summary.matched_split_count >= 10,
        "High permutation stress must successfully resolve all split matches without collision"
    );
}
