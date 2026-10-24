//! Empirical Adversarial Challenge Test Suite for Milestone M3
//!
//! Author: Challenger 1
//! Targets:
//! 1. Newton-Raphson integer square root boundary conditions (0, 1, powers of 2, u64::MAX, u128::MAX).
//! 2. Acronym pattern recognition: known corporate acronyms ("HAGL", "VCS", "BIDV", "TCB", "VCB", "CTG"),
//!    informal abbreviations ("ck tien may bom hd 88"), and disjoint/adversarial negative cases.
//! 3. Deterministic fallback: verify that enable_semantic_recon = false completely bypasses Gate 2 without errors.
//! 4. Batch telemetry: verify matched_semantic_count accurately tracks semantic elevations.
//! 5. Integer arithmetic invariant: strictly float-free #![deny(clippy::float_arithmetic)].

#![deny(clippy::float_arithmetic)]

use liva_ledger::PostingType;
use liva_money::Money;
use liva_parse::CanonicalTx;
use liva_recon::{
    compute_semantic_similarity_bps, integer_sqrt_u128, integer_sqrt_u64,
    integer_token_cosine_similarity, is_acronym_pattern, normalize_vietnamese_text,
    DeclarativeReconEngine, InternalInvoice, MatchTier, ReconRuleConfig, SemanticReconContext,
};

fn make_tx(
    row_id: usize,
    amount_vnd: i64,
    ref_code: &str,
    narration: &str,
    timestamp: i64,
) -> CanonicalTx {
    CanonicalTx {
        row_id,
        bank_code: "VCB".to_string(),
        ref_code: ref_code.to_string(),
        posting_type: PostingType::Credit,
        amount: Money::vnd(amount_vnd),
        balance_after: None,
        counterparty_name: None,
        counterparty_account: None,
        narration: narration.to_string(),
        tx_timestamp: timestamp,
    }
}

fn make_invoice(
    invoice_id: &str,
    doc_ref: &str,
    counterparty_name: &str,
    amount_vnd: i64,
    timestamp: i64,
) -> InternalInvoice {
    InternalInvoice {
        invoice_id: invoice_id.to_string(),
        doc_ref: doc_ref.to_string(),
        counterparty_name: counterparty_name.to_string(),
        amount: Money::vnd(amount_vnd),
        timestamp,
    }
}

#[test]
fn test_stress_isqrt_boundaries_and_properties() {
    // 1. Exact boundaries: 0, 1, 2, 3, 4
    assert_eq!(integer_sqrt_u64(0), 0);
    assert_eq!(integer_sqrt_u64(1), 1);
    assert_eq!(integer_sqrt_u64(2), 1);
    assert_eq!(integer_sqrt_u64(3), 1);
    assert_eq!(integer_sqrt_u64(4), 2);

    assert_eq!(integer_sqrt_u128(0), 0);
    assert_eq!(integer_sqrt_u128(1), 1);
    assert_eq!(integer_sqrt_u128(2), 1);
    assert_eq!(integer_sqrt_u128(3), 1);
    assert_eq!(integer_sqrt_u128(4), 2);

    // 2. Exact perfect squares for u64
    for root in [10u64, 100, 1_000, 10_000, 1_000_000, 1_000_000_000] {
        let sq = root * root;
        assert_eq!(integer_sqrt_u64(sq), root);
        assert_eq!(integer_sqrt_u64(sq - 1), root - 1);
        assert_eq!(integer_sqrt_u64(sq + 1), root);
    }

    // 3. Extremes: u64::MAX
    let max_u64_root = integer_sqrt_u64(u64::MAX);
    // sqrt(2^64 - 1) = 2^32 - 1 = 4,294,967,295
    assert_eq!(max_u64_root, 4_294_967_295);
    let sq_u64 = (max_u64_root as u128) * (max_u64_root as u128);
    assert!(sq_u64 <= u64::MAX as u128);
    let next_root_u64 = (max_u64_root as u128) + 1;
    assert!(next_root_u64 * next_root_u64 > u64::MAX as u128);

    // 4. Exact perfect squares for u128
    for root in [
        10u128,
        1_000,
        1_000_000,
        1_000_000_000,
        1_000_000_000_000,
        1_000_000_000_000_000_000,
    ] {
        let sq = root * root;
        assert_eq!(integer_sqrt_u128(sq), root);
        assert_eq!(integer_sqrt_u128(sq - 1), root - 1);
        assert_eq!(integer_sqrt_u128(sq + 1), root);
    }

    // 5. Extremes: u128::MAX
    let max_u128_root = integer_sqrt_u128(u128::MAX);
    // sqrt(2^128 - 1) = 2^64 - 1 = 18,446,744,073,709,551,615
    assert_eq!(max_u128_root, 18_446_744_073_709_551_615u128);
}

#[test]
fn test_stress_acronym_positive_and_negative_disjoint() {
    // 1. Positive: Known corporate abbreviations (using normalized uppercase as DeclarativeReconEngine does)
    assert!(is_acronym_pattern(&normalize_vietnamese_text("CÔNG TY CỔ PHẦN HOÀNG ANH GIA LAI"), "THANH TOAN TIEN HANG HAGL"));
    assert!(is_acronym_pattern(&normalize_vietnamese_text("TẬP ĐOÀN CÔNG NGHIỆP THAN KHOÁNG SẢN VINACOMIN"), "VCS NOP TIEN"));
    assert!(is_acronym_pattern(&normalize_vietnamese_text("NGÂN HÀNG ĐẦU TƯ VÀ PHÁT TRIỂN"), "CHUYEN TIEN BIDV"));
    assert!(is_acronym_pattern("TECHCOMBANK", "NOP TIEN TCB"));
    assert!(is_acronym_pattern("VIETCOMBANK", "CHUYEN TIEN VCB Q3"));
    assert!(is_acronym_pattern("VIETINBANK", "THANH TOAN CTG"));

    // 2. Positive: Sub-phrase initials
    assert!(is_acronym_pattern(&normalize_vietnamese_text("CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG"), "ck tien may bom hd 88"));
    assert!(is_acronym_pattern(&normalize_vietnamese_text("CÔNG TY CỔ PHẦN THĂNG LONG"), "TIEN HANG TL"));

    // 3. Negative: Disjoint / Unrelated strings must NOT trigger acronym pattern
    assert!(!is_acronym_pattern(&normalize_vietnamese_text("CÔNG TY ĐỨC PHÁT"), "THANH TOAN TIEN HANG HAGL"));
    assert!(!is_acronym_pattern(&normalize_vietnamese_text("CÔNG TY CƠ KHÍ PHÚ HẢI"), "THANH TOAN TIEN NUOC"));
    assert!(!is_acronym_pattern(&normalize_vietnamese_text("CÔNG TY SỮA VIỆT NAM"), "TIEN THUE NHA THANG 9"));
    assert!(!is_acronym_pattern("DOANH NGHIEP TU NHAN MINH PHUONG", "CHUYEN TIEN VCS DONG HANG"));
    assert!(!is_acronym_pattern("CONG TY BAO BI THIEN LONG", "BIDV VCB TCB CHUYEN KHOAN"));

    // 4. Semantic similarity scores comparison
    let sim_hagl = compute_semantic_similarity_bps(&normalize_vietnamese_text("CÔNG TY CỔ PHẦN HOÀNG ANH GIA LAI"), "THANH TOAN HAGL");
    let sim_disjoint = compute_semantic_similarity_bps(&normalize_vietnamese_text("CÔNG TY ĐỨC PHÁT"), "THANH TOAN HAGL");

    assert!(sim_hagl >= 8_200, "HAGL should score >= 8,200 bps, got {}", sim_hagl);
    assert!(sim_disjoint < 5_000, "Disjoint pair should score < 5,000 bps, got {}", sim_disjoint);

    let sim_mb_hd = compute_semantic_similarity_bps(&normalize_vietnamese_text("CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG"), "ck tien may bom hd 88");
    let sim_mb_unrelated = compute_semantic_similarity_bps(&normalize_vietnamese_text("CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG"), "ck tien ca phe trung nguyen");

    assert!(sim_mb_hd >= 8_200, "May bom HD should score >= 8,200 bps, got {}", sim_mb_hd);
    assert!(sim_mb_unrelated < 5_000, "Unrelated narration should score < 5,000 bps, got {}", sim_mb_unrelated);
}

#[test]
fn test_stress_deterministic_fallback_isolation() {
    let mut config = ReconRuleConfig::default();
    config.tier2.enable_semantic_recon = false; // Disable Gate 2 completely

    let engine = DeclarativeReconEngine::new(config).unwrap();

    // Transactions with informal acronyms that require Gate 2 elevation
    // Use non-additive amounts (11M, 23M, 37M) to prevent Tier 3 subset sum from matching them
    let txs = vec![
        make_tx(1, 11_000_000, "FT001", "THANH TOAN HAGL VIETQR", 1726400000),
        make_tx(2, 23_000_000, "FT002", "CHUYEN TIEN VCS DONG HANG", 1726400000),
        make_tx(3, 37_000_000, "FT003", "ck tien may bom hd 88", 1726400000),
    ];

    let invoices = vec![
        make_invoice("INV-01", "HD-01", "CÔNG TY CỔ PHẦN HOÀNG ANH GIA LAI", 11_000_000, 1726400000),
        make_invoice("INV-02", "HD-02", "TẬP ĐOÀN CÔNG NGHIỆP THAN KHOÁNG SẢN VINACOMIN", 23_000_000, 1726400000),
        make_invoice("INV-03", "HD-03", "CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG", 37_000_000, 1726400000),
    ];

    // 1. Direct Tier 2 evaluation: Gate 2 must be completely bypassed
    let (t2_matches, rem_tx, rem_inv) = engine.match_tier2(
        &[0, 1, 2],
        &txs,
        &["INV-01".to_string(), "INV-02".to_string(), "INV-03".to_string()],
        &invoices,
    );
    assert!(t2_matches.is_empty(), "When semantic recon is disabled, Tier 2 must yield 0 matches");
    assert_eq!(rem_tx.len(), 3, "All 3 txs must remain unresolved in Tier 2");
    assert_eq!(rem_inv.len(), 3, "All 3 invoices must remain unresolved in Tier 2");

    // 2. Full Batch execution: Zero matches across all tiers, all routed to HITL quarantine
    let res = engine.reconcile_batch(&txs, &invoices);

    assert_eq!(res.matched_semantic_count, 0, "Semantic count must be 0");
    assert_eq!(res.matched_fuzzy_count, 0, "Fuzzy count must be 0");
    assert_eq!(res.matched_exact_count, 0, "Exact count must be 0");
    assert_eq!(res.matched_split_count, 0, "Split count must be 0");
    assert_eq!(res.total_matched_count, 0, "Total matched must be 0");
    assert_eq!(res.hitl_quarantine.len(), 3, "All 3 ambiguous txs must route to quarantine");
    assert_eq!(res.unmatched_tx_count, 3);
    assert_eq!(res.unmatched_invoice_count, 3);
}

#[test]
fn test_stress_batch_telemetry_accuracy() {
    let engine = DeclarativeReconEngine::default_rules();

    let txs = vec![
        // Exact Tier 1: doc_ref match
        make_tx(1, 5_000_000, "REF-E1", "Thanh toan don hang REF-E1", 1726400000),
        make_tx(2, 7_000_000, "REF-E2", "Chuyen tien REF-E2", 1726400000),
        // Fuzzy Tier 2 fast path (high character overlap)
        make_tx(3, 12_000_000, "FT-F1", "CONG TY TNHH PHAT TRIEN CONG NGHE AN PHAT", 1726400000),
        // Semantic Tier 2 Gate 2 elevations
        make_tx(4, 15_000_000, "FT-S1", "THANH TOAN TIEN HANG HAGL VIETQR", 1726400000),
        make_tx(5, 25_000_000, "FT-S2", "CHUYEN TIEN VCS DONG TIEN HANG", 1726400000),
        make_tx(6, 35_000_000, "FT-S3", "ck tien may bom hd 88", 1726400000),
        // Quarantined / Unmatched
        make_tx(7, 99_000_000, "UN-01", "Giao dich khong xac dinh", 1726400000),
    ];

    let invoices = vec![
        make_invoice("INV-E1", "REF-E1", "CONG TY A", 5_000_000, 1726400000),
        make_invoice("INV-E2", "REF-E2", "CONG TY B", 7_000_000, 1726400000),
        make_invoice("INV-F1", "DOC-F1", "CONG TY TNHH CONG NGHE AN PHAT", 12_000_000, 1726400000),
        make_invoice("INV-S1", "DOC-S1", "CÔNG TY CỔ PHẦN HOÀNG ANH GIA LAI", 15_000_000, 1726400000),
        make_invoice("INV-S2", "DOC-S2", "TẬP ĐOÀN CÔNG NGHIỆP THAN KHOÁNG SẢN VINACOMIN", 25_000_000, 1726400000),
        make_invoice("INV-S3", "DOC-S3", "CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG", 35_000_000, 1726400000),
        make_invoice("INV-U1", "DOC-U1", "CONG TY KHONG KHOP", 88_000_000, 1726400000),
    ];

    let res = engine.reconcile_batch(&txs, &invoices);

    assert_eq!(res.matched_exact_count, 2, "Exact count must be 2");
    assert_eq!(res.matched_fuzzy_count, 1, "Fuzzy count must be 1");
    assert_eq!(res.matched_semantic_count, 3, "Semantic count must be 3");
    assert_eq!(res.matched_split_count, 0, "Split count must be 0");
    assert_eq!(res.total_matched_count, 6, "Total matched must be 6");
    assert_eq!(res.hitl_quarantine.len(), 1, "Quarantine count must be 1");
    assert_eq!(res.unmatched_invoice_count, 1, "Unmatched invoices must be 1");

    // Invariant check: total matches == exact + fuzzy + semantic + split
    assert_eq!(
        res.total_matched_count,
        res.matched_exact_count + res.matched_fuzzy_count + res.matched_semantic_count + res.matched_split_count
    );
}

#[test]
fn test_stress_token_cosine_empty_identical_and_orthogonals() {
    assert_eq!(integer_token_cosine_similarity("", ""), 0);
    assert_eq!(integer_token_cosine_similarity("abc", ""), 0);
    assert_eq!(integer_token_cosine_similarity("", "xyz"), 0);

    // Identical strings must yield exactly 10,000 bps (100.00%)
    assert_eq!(integer_token_cosine_similarity("CONG TY A", "CONG TY A"), 10_000);
    assert_eq!(integer_token_cosine_similarity("HOANG ANH GIA LAI", "HOANG ANH GIA LAI"), 10_000);

    // Orthogonal / Disjoint strings should have very low or 0 score
    let sim_orth = integer_token_cosine_similarity("CONG TY XXXXX", "DON VI YYYYY");
    assert!(sim_orth < 2_000, "Orthogonal strings must have low similarity, got {}", sim_orth);
}

#[test]
#[should_panic(expected = "attempt to add with overflow")]
fn test_empirical_vector_dot_overflow_panic() {
    // Adversarial input: 3 dimensions with i32::MAX
    let ctx = SemanticReconContext::new().with_embeddings(
        1,
        vec![i32::MAX, i32::MAX, i32::MAX],
        "INV-01",
        vec![i32::MAX, i32::MAX, i32::MAX],
    );

    // In debug mode, this panics with arithmetic overflow because (i32::MAX as i64) * (i32::MAX as i64) * 3 > i64::MAX!
    let _ = ctx.get_similarity(1, "INV-01");
}

#[test]
fn test_empirical_score_bounding_and_integer_math_properties() {
    // 1. Grid stress across full range [0..=10,000] bps for both JW and Semantic similarity
    let jw_weight = 40u32;
    let sem_weight = 60u32;

    for jw in (0..=10_000).step_by(100) {
        for sem in (0..=10_000).step_by(100) {
            let combined_bps = (jw_weight * jw + sem_weight * sem) / 100;
            let conf = ((combined_bps + 50) / 100).min(100);

            assert!(combined_bps <= 10_000, "combined_bps exceeded 10,000: {}", combined_bps);
            assert!(conf <= 100, "conf exceeded 100: {}", conf);
        }
    }

    // Helper closure to compute combined score without literal 0 multiplications
    let calc_combined = |jw: u32, sem: u32, w_jw: u32, w_sem: u32| -> (u32, u32) {
        let combined_bps = (w_jw * jw + w_sem * sem) / 100;
        let conf = ((combined_bps + 50) / 100).min(100);
        (combined_bps, conf)
    };

    // 2. Exact boundary values verification
    // 2a. Zero similarity: (0, 0)
    let (b_0_0, c_0_0) = calc_combined(0, 0, 40, 60);
    assert_eq!(b_0_0, 0);
    assert_eq!(c_0_0, 0);

    // 2b. Perfect similarity: (10_000, 10_000)
    let (b_max, c_max) = calc_combined(10_000, 10_000, 40, 60);
    assert_eq!(b_max, 10_000);
    assert_eq!(c_max, 100);

    // 2c. Asymmetric extremes: (10_000, 0) and (0, 10_000)
    let (b_jw_only, c_jw_only) = calc_combined(10_000, 0, 40, 60);
    assert_eq!(b_jw_only, 4_000);
    assert_eq!(c_jw_only, 40);

    let (b_sem_only, c_sem_only) = calc_combined(0, 10_000, 40, 60);
    assert_eq!(b_sem_only, 6_000);
    assert_eq!(c_sem_only, 60);

    // 2d. Minimum elevation threshold: (5_000, 8_200)
    let (b_min_elev, c_min_elev) = calc_combined(5_000, 8_200, 40, 60);
    assert_eq!(b_min_elev, 6_920);
    assert_eq!(c_min_elev, 69);

    // 3. Weight combinations satisfying jw_weight + sem_weight == 100
    for w_jw in [0u32, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100] {
        let w_sem = 100 - w_jw;
        let (c_max_w, conf_max_w) = calc_combined(10_000, 10_000, w_jw, w_sem);
        assert_eq!(c_max_w, 10_000);
        assert_eq!(conf_max_w, 100);
    }

    // 4. Integer overflow boundary safety check
    // Max sum before division: 40 * 10,000 + 60 * 10,000 = 1,000,000
    // u32::MAX = 4,294,967,295, margin of safety is > 4,294x
    let max_interm: u64 = (40u64 * 10_000) + (60u64 * 10_000);
    assert_eq!(max_interm, 1_000_000);
    assert!(max_interm <= u32::MAX as u64);
}

#[test]
fn test_empirical_extreme_vector_similarity_handling() {
    let engine = DeclarativeReconEngine::default_rules();

    // 1. Extreme: 0 bps similarity
    // 1a. Explicit with_similarity
    let ctx_zero = SemanticReconContext::new().with_similarity(1, "INV-EXT-1", 0);
    assert_eq!(ctx_zero.get_similarity(1, "INV-EXT-1"), Some(0));

    // 1b. Orthogonal vectors dot product: [1000, 0, 0] . [0, 1000, 0] = 0
    let ctx_orth = SemanticReconContext::new().with_embeddings(
        1,
        vec![1000, 0, 0],
        "INV-EXT-1",
        vec![0, 1000, 0],
    );
    assert_eq!(ctx_orth.get_similarity(1, "INV-EXT-1"), Some(0));

    // 2. Extreme: 10,000 bps similarity (100%)
    // 2a. Explicit with_similarity
    let ctx_max = SemanticReconContext::new().with_similarity(1, "INV-EXT-1", 10_000);
    assert_eq!(ctx_max.get_similarity(1, "INV-EXT-1"), Some(10_000));

    // 2b. Identical normalized vectors: [1000, 0, 0] . [1000, 0, 0] = 1,000,000 -> clamped_dot / 100 = 10,000 bps
    let ctx_unit = SemanticReconContext::new().with_embeddings(
        1,
        vec![1000, 0, 0],
        "INV-EXT-1",
        vec![1000, 0, 0],
    );
    assert_eq!(ctx_unit.get_similarity(1, "INV-EXT-1"), Some(10_000));

    // 3. Extreme: Negative dot product (opposing vectors)
    // [1000, 0, 0] . [-1000, 0, 0] = -1,000,000
    // Clamping to 0 must prevent integer underflow or wrap-around to u32::MAX
    let ctx_neg = SemanticReconContext::new().with_embeddings(
        1,
        vec![1000, 0, 0],
        "INV-EXT-1",
        vec![-1000, 0, 0],
    );
    assert_eq!(ctx_neg.get_similarity(1, "INV-EXT-1"), Some(0));

    // 4. Extreme: Out-of-bounds positive inputs clamped to 10,000 bps
    let ctx_overflow_score = SemanticReconContext::new().with_similarity(1, "INV-EXT-1", 65_535);
    assert_eq!(ctx_overflow_score.get_similarity(1, "INV-EXT-1"), Some(10_000));

    // Dot product exceeding 1,000,000: [2000, 0, 0] . [2000, 0, 0] = 4,000,000
    let ctx_super_norm = SemanticReconContext::new().with_embeddings(
        1,
        vec![2000, 0, 0],
        "INV-EXT-1",
        vec![2000, 0, 0],
    );
    assert_eq!(ctx_super_norm.get_similarity(1, "INV-EXT-1"), Some(10_000));

    // 5. Verify elevation behavior under 0 bps vs 10,000 bps:
    // Create an ambiguous transaction (JW in uncertainty band 5,000..8,499)
    let amb_tx = make_tx(2, 20_000_000, "FT-AMB", "THANH TOAN TIEN DON HANG PHAN PHOI", 1726400000);
    let amb_inv = make_invoice("INV-AMB", "DOC-AMB", "CONG TY TNHH PHAN PHOI TOAN CAU", 20_000_000, 1726400000);

    // With 0 bps: Gate 2 must NOT elevate
    let (matches_zero, rem_tx, _) = engine.match_tier2_with_semantic(
        &[0],
        std::slice::from_ref(&amb_tx),
        &["INV-AMB".to_string()],
        std::slice::from_ref(&amb_inv),
        Some(&ctx_zero),
    );
    assert!(matches_zero.is_empty(), "0 bps semantic similarity must NOT elevate candidate");
    assert_eq!(rem_tx.len(), 1);

    // With negative dot product: Gate 2 must NOT elevate
    let (matches_neg, _, _) = engine.match_tier2_with_semantic(
        &[0],
        std::slice::from_ref(&amb_tx),
        &["INV-AMB".to_string()],
        std::slice::from_ref(&amb_inv),
        Some(&ctx_neg),
    );
    assert!(matches_neg.is_empty(), "Negative dot product (0 bps) must NOT elevate candidate");

    // With 10,000 bps: Gate 2 MUST elevate candidate
    let ctx_elev = SemanticReconContext::new().with_similarity(2, "INV-AMB", 10_000);
    let (matches_elev, rem_tx_elev, _) = engine.match_tier2_with_semantic(
        &[0],
        &[amb_tx],
        &["INV-AMB".to_string()],
        &[amb_inv],
        Some(&ctx_elev),
    );
    assert_eq!(matches_elev.len(), 1, "10,000 bps semantic similarity MUST elevate candidate");
    assert_eq!(rem_tx_elev.len(), 0);
    assert_eq!(matches_elev[0].tier, MatchTier::Tier2Semantic);
    assert!(matches_elev[0].is_semantic);
    assert_eq!(matches_elev[0].semantic_similarity_bps, Some(10_000));
}

#[test]
fn test_empirical_acronym_collision_defenses() {
    let engine = DeclarativeReconEngine::default_rules();

    // The counterparty "CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG" has sub-phrase initials "HD" (from "HAI DANG")
    let party_hai_dang = "CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG";
    let cleaned_party = "MAY BOM HAI DANG";

    // 1. Defense Layer 1: Monetary Invariant Defense
    // Narration contains "HD 123" (representing 'hợp đồng 123' for an unrelated company).
    // Even though "HD" matches the sub-phrase initials, different amount (e.g. 52M vs 50M)
    // is REJECTED by amount invariant before any string or acronym matching is evaluated!
    let tx_diff_amount = make_tx(1, 52_000_000, "FT-HD-01", "THANH TOAN THEO HD 123 CHO DNTN TOAN THANG", 1726400000);
    let inv_hai_dang = make_invoice("INV-HD", "DOC-HD", party_hai_dang, 50_000_000, 1726400000);

    let (matches_amt, rem_tx_amt, _) = engine.match_tier2(
        &[0],
        &[tx_diff_amount],
        &["INV-HD".to_string()],
        std::slice::from_ref(&inv_hai_dang),
    );
    assert!(matches_amt.is_empty(), "Monetary invariant must prevent match on amount discrepancy");
    assert_eq!(rem_tx_amt.len(), 1);

    // 2. Defense Layer 2: Temporal Window Defense
    // Amount matches exactly, but timestamp delta exceeds time tolerance (default 72h = 259,200s).
    // Transaction occurs 5 days later (432,000s).
    let tx_diff_time = make_tx(2, 50_000_000, "FT-HD-02", "THANH TOAN THEO HD 123 CHO DNTN TOAN THANG", 1726400000 + 432_000);
    let (matches_time, rem_tx_time, _) = engine.match_tier2(
        &[0],
        &[tx_diff_time],
        &["INV-HD".to_string()],
        std::slice::from_ref(&inv_hai_dang),
    );
    assert!(matches_time.is_empty(), "Temporal window tolerance must prevent match when out-of-window");
    assert_eq!(rem_tx_time.len(), 1);

    // 3. Empirical Vulnerability Demonstration: Coincidental Exact Amount + Same Window
    // Narration has "HD 123" (contract) for "TOAN THANG", coincidental identical amount (50M) and timestamp.
    // Sub-phrase initials of "HAI DANG" is "HD", which triggers is_acronym_pattern = true and acronym_sim = 8,500 bps!
    let tx_coincidental = make_tx(3, 50_000_000, "FT-HD-03", "THANH TOAN THEO HD 123 CHO CONG TY TOAN THANG", 1726400000);
    assert!(is_acronym_pattern(cleaned_party, &normalize_vietnamese_text(&tx_coincidental.narration)));

    let sim = compute_semantic_similarity_bps(cleaned_party, &normalize_vietnamese_text(&tx_coincidental.narration));
    assert!(sim >= 8_200, "Vulnerability: 'HD' sub-phrase initials causes similarity {} >= 8,200 bps", sim);

    // 4. Batch Competition: Demonstration of Greedy Invoice Ordering Bias
    // In a batch with TWO invoices of identical amount (50M):
    // INV-TRUE: "CÔNG TY TNHH TOÀN THẮNG" (true intended recipient)
    // INV-FALSE: "CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG" (false recipient with sub-phrase acronym "HD")
    let inv_true = make_invoice("INV-TRUE", "DOC-TT", "CÔNG TY TNHH TOÀN THẮNG", 50_000_000, 1726400000);
    let inv_false = make_invoice("INV-FALSE", "DOC-HD", party_hai_dang, 50_000_000, 1726400000);

    // Scenario 4a: Order [INV-TRUE, INV-FALSE]
    // INV-TRUE is evaluated first -> matches Gate 1 fast-path (100% containment) cleanly as Tier2Fuzzy!
    let res_a = engine.reconcile_batch(
        std::slice::from_ref(&tx_coincidental),
        &[inv_true.clone(), inv_false.clone()],
    );
    assert_eq!(res_a.total_matched_count, 1);
    assert_eq!(res_a.matches[0].invoice_id, "INV-TRUE");
    assert_eq!(res_a.matches[0].tier, MatchTier::Tier2Fuzzy);
    assert!(!res_a.matches[0].is_semantic);

    // Scenario 4b: Order [INV-FALSE, INV-TRUE]
    // Greedy loop evaluates INV-FALSE first. Because 'HD' triggers Gate 2, it matches INV-FALSE as Tier2Semantic!
    // This empirically confirms Challenge 2 ("Greedy Invoice Ordering Bias") and Challenge 4 ("2-Letter Sub-Phrase Acronym Collision")!
    let res_b = engine.reconcile_batch(
        &[tx_coincidental],
        &[inv_false, inv_true],
    );
    assert_eq!(res_b.total_matched_count, 1);
    assert_eq!(res_b.matches[0].invoice_id, "INV-FALSE", "Empirically demonstrates greedy order bias: INV-FALSE steals match!");
    assert_eq!(res_b.matches[0].tier, MatchTier::Tier2Semantic);
    assert!(res_b.matches[0].is_semantic);
    // Notice that confidence score is lower than Gate 1 fast path
    assert!(res_b.matches[0].confidence_score < res_a.matches[0].confidence_score);
}
