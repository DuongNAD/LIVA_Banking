use liva_banking::models::{ErpDocument, TransactionRecord, TransactionType};
use liva_banking::reconciliation::{ReconciliationEngine, SplitSolver, Tier1Matcher};

fn make_tx(
    row_id: usize,
    date: i64,
    ref_no: Option<&str>,
    tx_type: TransactionType,
    amount: u64,
    partner: Option<&str>,
    narration: &str,
) -> TransactionRecord {
    TransactionRecord::new(
        row_id,
        date,
        date,
        ref_no.map(|s| s.to_string()),
        tx_type,
        amount,
        Some(100_000_000),
        None,
        partner.map(|s| s.to_string()),
        Some("VCB".to_string()),
        narration.to_string(),
    )
}

fn make_doc(
    id: &str,
    voucher: &str,
    invoice: Option<&str>,
    partner: &str,
    date: i64,
    doc_type: TransactionType,
    open_amount: u64,
) -> ErpDocument {
    ErpDocument {
        id: id.to_string(),
        voucher_no: voucher.to_string(),
        invoice_no: invoice.map(|s| s.to_string()),
        partner_code: partner.to_string(),
        partner_name: partner.to_string(),
        doc_date: date,
        due_date: None,
        doc_type,
        total_amount: open_amount,
        open_amount,
        currency: "VND".to_string(),
        description: "Invoice payment".to_string(),
        version: 1,
    }
}

#[test]
fn test_tier1_single_candidate_rule() {
    let now = 1755216000; // 2025-08-15
    let txs = vec![
        make_tx(1, now, Some("HD-00102"), TransactionType::Credit, 50_000_000, Some("ABC"), "Thanh toan HD00102"),
    ];

    let docs = vec![
        make_doc("DOC1", "CT001", Some("HD-00102"), "ABC", now + 3600, TransactionType::Credit, 50_000_000),
    ];

    let res = Tier1Matcher::match_tier1(&txs, &docs);
    assert_eq!(res.auto_matched.len(), 1, "Single candidate should auto-match");
    assert_eq!(res.auto_matched[0].matched_amount, 50_000_000);
    assert_eq!(res.candidate_conflicts.len(), 0);
    assert_eq!(res.unmatched_tx_indices.len(), 0);
}

#[test]
fn test_tier1_multi_candidate_fails_closed() {
    let now = 1755216000;
    // Two different invoices share the same reference and amount within 24h
    let txs = vec![
        make_tx(1, now, Some("HD-SAME"), TransactionType::Credit, 20_000_000, Some("ABC"), "Thanh toan HD-SAME"),
    ];

    let docs = vec![
        make_doc("DOC1", "CT001", Some("HD-SAME"), "Chi nhanh 1", now + 1000, TransactionType::Credit, 20_000_000),
        make_doc("DOC2", "CT002", Some("HD-SAME"), "Chi nhanh 2", now + 2000, TransactionType::Credit, 20_000_000),
    ];

    let res = Tier1Matcher::match_tier1(&txs, &docs);
    // Must NOT auto-match! Fails closed and diverts to candidate_conflicts
    assert_eq!(res.auto_matched.len(), 0, "Multiple candidates must NEVER auto-match in Tier 1");
    assert_eq!(res.candidate_conflicts.len(), 1);
    assert_eq!(res.candidate_conflicts[0].erp_doc_ids.len(), 2);
}

#[test]
fn test_tier2_fuzzy_and_fee_separation() {
    let now = 1755216000;
    // Bank deposit is 49,978,000 VND (50,000,000 - 22,000 wire fee)
    let txs = vec![
        make_tx(1, now, None, TransactionType::Credit, 49_978_000, Some("Cong Ty TNHH Phat Trien ABC"), "TT tien hang ABC"),
    ];

    let docs = vec![
        make_doc("DOC1", "CT001", Some("HD999"), "Cong ty TNHH ABC", now + 7200, TransactionType::Credit, 50_000_000),
    ];

    let result = ReconciliationEngine::reconcile(&txs, &docs);
    assert_eq!(result.auto_matched.len(), 0);
    assert_eq!(result.proposed_matches.len(), 1);

    let prop = &result.proposed_matches[0];
    assert_eq!(prop.matched_amount, 49_978_000);
    assert_eq!(prop.fee_amount, 22_000);
    assert_eq!(prop.fee_account.as_deref(), Some("6425"));
    assert_eq!(prop.status, "DRAFT", "Tier 2 proposals must always be DRAFT/pending human review");
}

#[test]
fn test_tier3_1_to_n_composite_matching_and_fair_allocation() {
    let now = 1755216000;
    // 1 Bank deposit of 60,000,000 paying 3 invoices: 10M + 20M + 30M
    let tx = make_tx(1, now, None, TransactionType::Credit, 60_000_000, Some("XYZ"), "Gop 3 hoa don");

    let docs_unique = vec![
        make_doc("D1", "CT1", Some("HD1"), "XYZ", now, TransactionType::Credit, 10_000_000),
        make_doc("D2", "CT2", Some("HD2"), "XYZ", now, TransactionType::Credit, 20_000_000),
        make_doc("D3", "CT3", Some("HD3"), "XYZ", now, TransactionType::Credit, 30_000_000),
        make_doc("D4", "CT4", Some("HD4"), "XYZ", now, TransactionType::Credit, 45_000_000), // Irrelevant (45M != 60M)
    ];

    let prop = SplitSolver::solve_1_to_n(&tx, &docs_unique).expect("Should find unique 1:N solution");
    assert_eq!(prop.matched_amount, 60_000_000);
    assert_eq!(prop.erp_doc_ids.len(), 3);
    assert!(prop.erp_doc_ids.contains(&"D1".to_string()));
    assert!(prop.erp_doc_ids.contains(&"D2".to_string()));
    assert!(prop.erp_doc_ids.contains(&"D3".to_string()));

    // Ambiguity / Multiple solutions test (e.g. 10+20+30=60 and 10+50=60)
    let docs_ambiguous = vec![
        make_doc("D1", "CT1", Some("HD1"), "XYZ", now, TransactionType::Credit, 10_000_000),
        make_doc("D2", "CT2", Some("HD2"), "XYZ", now, TransactionType::Credit, 20_000_000),
        make_doc("D3", "CT3", Some("HD3"), "XYZ", now, TransactionType::Credit, 30_000_000),
        make_doc("D4", "CT4", Some("HD4"), "XYZ", now, TransactionType::Credit, 50_000_000), // 10+50=60!
    ];
    assert!(
        SplitSolver::solve_1_to_n(&tx, &docs_ambiguous).is_none(),
        "Must fail closed when multiple subsets sum to the target amount"
    );

    // Fair Partial Allocation test
    let mut tx_rem = 25_000_000;
    let mut doc_rem = 40_000_000;
    let alloc = SplitSolver::allocate_partial(&mut tx_rem, &mut doc_rem, 1, 1).expect("Allocation should succeed");
    assert_eq!(alloc, 25_000_000);
    assert_eq!(tx_rem, 0);
    assert_eq!(doc_rem, 15_000_000);

    // Concurrency version mismatch test
    let mut tx2 = 10_000_000;
    let mut doc2 = 15_000_000;
    let err = SplitSolver::allocate_partial(&mut tx2, &mut doc2, 2, 1);
    assert!(err.is_err(), "Must reject allocation if version mismatched (OCC conflict)");
}
