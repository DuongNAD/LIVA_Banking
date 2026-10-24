//! Tier 2 Hybrid Semantic Reconciliation & Deterministic Fallback Test Suite
//!
//! Validates:
//! 1. 2-Gate hybrid matching in `DeclarativeReconEngine`:
//!    - Gate 1 (Fast Path): Integer Jaro-Winkler >= 8,500 bps.
//!    - Gate 2 (Semantic Elevation): Uncertainty band (5,000..8,499 bps) or acronym pattern
//!      with semantic similarity >= 8,200 bps elevated to Tier 2 with combined score.
//! 2. Vietnamese corporate acronym & abbreviation matching:
//!    - "HAGL" vs "Hoàng Anh Gia Lai"
//!    - "VCS" vs "Vinacomin"
//!    - "ck tien may bom hd 88" vs "Công ty TNHH Máy Bơm Hải Đăng"
//! 3. Pre-computed vector cosine similarity inputs via `SemanticReconContext`.
//! 4. Deterministic fallback when `enable_semantic_recon = false`.
//! 5. Tracking `matched_semantic_count` in batch reconciliation results.
//! 6. Absolute integer arithmetic invariant: zero floats (`#![deny(clippy::float_arithmetic)]`).

#![deny(clippy::float_arithmetic)]

use liva_ledger::PostingType;
use liva_money::Money;
use liva_parse::CanonicalTx;
use liva_recon::{
    DeclarativeReconEngine, InternalInvoice, MatchTier, ReconciliationEngine,
    ReconRuleConfig, SemanticReconContext,
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
fn test_semantic_acronym_hagl_vs_hoang_anh_gia_lai() {
    let engine = DeclarativeReconEngine::default_rules();

    // Bank statement with acronym narration "HAGL" and no doc ref
    let tx = make_tx(
        1,
        25_000_000,
        "FT26001",
        "THANH TOAN TIEN HANG HAGL VIETQR NAPAS247",
        1726400000,
    );

    // ERP Invoice with full Vietnamese corporate name
    let invoice = make_invoice(
        "INV-HAGL-01",
        "HD-8888",
        "CÔNG TY CỔ PHẦN HOÀNG ANH GIA LAI",
        25_000_000,
        1726400000,
    );

    let (matched, rem_tx, rem_inv) = engine.match_tier2(
        &[0],
        &[tx],
        &["INV-HAGL-01".to_string()],
        &[invoice],
    );

    assert_eq!(matched.len(), 1, "HAGL acronym must be matched in Tier 2");
    assert!(rem_tx.is_empty(), "Transaction should be resolved");
    assert!(rem_inv.is_empty(), "Invoice should be resolved");

    let m = &matched[0];
    assert_eq!(m.tier, MatchTier::Tier2Semantic, "Tier must be Tier2Semantic");
    assert!(m.is_semantic, "is_semantic flag must be true");
    assert!(
        m.semantic_similarity_bps.unwrap() >= 8_200,
        "Semantic similarity must be >= 8,200 bps, got {}",
        m.semantic_similarity_bps.unwrap()
    );
    assert!(
        m.confidence_score >= 80,
        "Combined confidence score must be >= 80, got {}",
        m.confidence_score
    );
}

#[test]
fn test_semantic_acronym_vcs_vs_vinacomin_with_fee_deduction() {
    let engine = DeclarativeReconEngine::default_rules();

    // Bank transaction: 50,000,000 VND minus 2,200 VND standard wire fee
    let tx = make_tx(
        2,
        49_997_800,
        "FT26002",
        "CHUYEN TIEN VCS DONG TIEN HANG Q3",
        1726400000,
    );

    // ERP Invoice
    let invoice = make_invoice(
        "INV-VCS-01",
        "HD-7777",
        "TẬP ĐOÀN CÔNG NGHIỆP THAN KHOÁNG SẢN VINACOMIN",
        50_000_000,
        1726400000,
    );

    let (matched, rem_tx, rem_inv) = engine.match_tier2(
        &[0],
        &[tx],
        &["INV-VCS-01".to_string()],
        &[invoice],
    );

    assert_eq!(matched.len(), 1, "VCS vs Vinacomin must be matched in Tier 2");
    assert!(rem_tx.is_empty());
    assert!(rem_inv.is_empty());

    let m = &matched[0];
    assert_eq!(m.tier, MatchTier::Tier2Semantic);
    assert!(m.is_semantic);
    assert_eq!(m.fee_deducted, Some(Money::vnd(2_200)));
    assert!(m.semantic_similarity_bps.unwrap() >= 8_200);
}

#[test]
fn test_semantic_abbreviation_ck_tien_may_bom_hd_88() {
    let engine = DeclarativeReconEngine::default_rules();

    // Highly informal Vietnamese banking narration
    let tx = make_tx(
        3,
        18_500_000,
        "FT26003",
        "ck tien may bom hd 88",
        1726400000,
    );

    // ERP Invoice: Company name "CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG", doc_ref "HD-88"
    let invoice = make_invoice(
        "INV-MB-88",
        "HD-88",
        "CÔNG TY TNHH MÁY BƠM HẢI ĐĂNG",
        18_500_000,
        1726400000,
    );

    let (matched, rem_tx, rem_inv) = engine.match_tier2(
        &[0],
        &[tx],
        &["INV-MB-88".to_string()],
        &[invoice],
    );

    assert_eq!(matched.len(), 1, "ck tien may bom hd 88 must be resolved");
    assert!(rem_tx.is_empty());
    assert!(rem_inv.is_empty());

    let m = &matched[0];
    assert_eq!(m.tier, MatchTier::Tier2Semantic);
    assert!(m.is_semantic);
    assert!(m.semantic_similarity_bps.unwrap() >= 8_200);
}

#[test]
fn test_precomputed_vector_context_elevation() {
    let engine = DeclarativeReconEngine::default_rules();

    let tx = make_tx(
        10,
        30_000_000,
        "FT26010",
        "CHUYEN TIEN THANH TOAN NGUYEN LIEU DET MAY",
        1726400000,
    );

    let invoice = make_invoice(
        "INV-PRE-10",
        "DOC-10",
        "CÔNG TY TNHH SỢI VẢI NAM ĐỊNH",
        30_000_000,
        1726400000,
    );

    // Provide pre-computed semantic similarity of 8,800 bps (88%)
    let context = SemanticReconContext::new().with_similarity(10, "INV-PRE-10", 8_800);

    let (matched, rem_tx, rem_inv) = engine.match_tier2_with_semantic(
        &[0],
        &[tx],
        &["INV-PRE-10".to_string()],
        &[invoice],
        Some(&context),
    );

    assert_eq!(matched.len(), 1);
    assert!(rem_tx.is_empty());
    assert!(rem_inv.is_empty());

    let m = &matched[0];
    assert_eq!(m.tier, MatchTier::Tier2Semantic);
    assert!(m.is_semantic);
    assert_eq!(m.semantic_similarity_bps, Some(8_800));
}

#[test]
fn test_deterministic_fallback_when_semantic_disabled() {
    let mut config = ReconRuleConfig::default();
    config.tier2.enable_semantic_recon = false; // Explicitly disable semantic recon

    let engine = DeclarativeReconEngine::new(config).unwrap();

    // HAGL transaction where character overlap is in uncertainty band (< 8,500 bps)
    let tx = make_tx(
        20,
        25_000_000,
        "FT26020",
        "THANH TOAN TIEN HANG HAGL VIETQR NAPAS247",
        1726400000,
    );

    let invoice = make_invoice(
        "INV-HAGL-20",
        "HD-2020",
        "CÔNG TY CỔ PHẦN HOÀNG ANH GIA LAI",
        25_000_000,
        1726400000,
    );

    // With semantic recon disabled, Gate 2 must NOT elevate
    let (matched, rem_tx, rem_inv) = engine.match_tier2(
        &[0],
        &[tx],
        &["INV-HAGL-20".to_string()],
        &[invoice],
    );

    assert!(
        matched.is_empty(),
        "With semantic recon disabled, candidate in uncertainty band must not match"
    );
    assert_eq!(rem_tx.len(), 1, "Transaction remains unresolved for Tier 3 / HITL");
    assert_eq!(rem_inv.len(), 1, "Invoice remains unresolved");
}

#[test]
fn test_reconcile_batch_tracks_matched_semantic_count() {
    let engine = DeclarativeReconEngine::default_rules();

    let txs = vec![
        // 1. Tier 1 exact match (Amount + DocRef)
        make_tx(1, 10_000_000, "REF-T1-01", "Thanh toan hoa don REF-T1-01", 1726400000),
        // 2. Tier 2 fast path (Name overlap >= 8,500 bps)
        make_tx(2, 20_000_000, "FT-T2-01", "CONG TY TNHH PHAT TRIEN CONG NGHE AN PHAT", 1726400000),
        // 3. Tier 2 semantic elevation (HAGL acronym)
        make_tx(3, 15_000_000, "FT-T2-02", "THANH TOAN HAGL VIETQR", 1726400000),
        // 4. Tier 2 semantic elevation (VCS acronym)
        make_tx(4, 30_000_000, "FT-T2-03", "CHUYEN TIEN VCS DONG TIEN HANG", 1726400000),
        // 5. Unmatched transaction for HITL quarantine
        make_tx(5, 5_000_000, "UNKNOWN-01", "Khong ro noi dung chuyen khoan", 1726400000),
    ];

    let invoices = vec![
        make_invoice("INV-01", "REF-T1-01", "CONG TY DAI VIET", 10_000_000, 1726400000),
        make_invoice("INV-02", "DOC-02", "CONG TY TNHH CONG NGHE AN PHAT", 20_000_000, 1726400000),
        make_invoice("INV-03", "DOC-03", "CÔNG TY CỔ PHẦN HOÀNG ANH GIA LAI", 15_000_000, 1726400000),
        make_invoice("INV-04", "DOC-04", "TẬP ĐOÀN CÔNG NGHIỆP THAN KHOÁNG SẢN VINACOMIN", 30_000_000, 1726400000),
    ];

    let res = engine.reconcile_batch(&txs, &invoices);

    assert_eq!(res.matched_exact_count, 1, "Expected 1 exact match");
    assert_eq!(res.matched_fuzzy_count, 1, "Expected 1 standard fuzzy match");
    assert_eq!(res.matched_semantic_count, 2, "Expected 2 semantic elevated matches");
    assert_eq!(res.total_matched_count, 4, "Expected 4 total matches");
    assert_eq!(res.hitl_quarantine.len(), 1, "Expected 1 quarantined transaction");

    // Also verify via ReconciliationEngine facade
    let facade_res = ReconciliationEngine::reconcile_batch(&txs, &invoices);
    assert_eq!(facade_res.matched_semantic_count, 2);
}
