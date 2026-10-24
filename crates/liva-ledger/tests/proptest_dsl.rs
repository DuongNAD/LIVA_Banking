//! Property-based testing for DSL Fee Unbundling and Declarative Posting Engine.
//!
//! Validates:
//! 1. Fee unbundling mathematical conservation invariant: Net + Vat == Gross holds 100%.
//! 2. Double-entry balance invariant: Sum(Debit) == Sum(Credit) holds for all synthesized vouchers.

use liva_ledger::{
    load_default_posting_engine, unbundle_gross_fee, voucher_to_journal_entry, ErpTarget,
    PostingContext, PostingType,
};
use liva_money::Money;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    #[test]
    fn prop_fee_unbundling_conservation(
        gross_amount in 1i64..100_000_000_000i64,
        vat_bps in 0i64..2_000i64, // 0% to 20%
    ) {
        let gross = Money::vnd(gross_amount);
        let (net, vat) = unbundle_gross_fee(gross, vat_bps).expect("Unbundling must never fail for positive amounts");

        // Invariant 1: Exact integer conservation (Delta = 0)
        let sum = net.checked_add(vat).expect("Addition must not overflow");
        prop_assert_eq!(sum, gross, "Net ({}) + Vat ({}) != Gross ({})", net, vat, gross);

        // Invariant 2: Net and Vat must not exceed Gross
        prop_assert!(net.amount() <= gross.amount());
        prop_assert!(vat.amount() <= gross.amount());

        // Invariant 3: Net must be strictly positive
        prop_assert!(net.is_positive());

        // Invariant 4: Vat must be non-negative
        prop_assert!(vat.amount() >= 0);
    }

    #[test]
    fn prop_posting_engine_double_entry_balance(
        amount in 1_000i64..10_000_000_000i64,
        is_credit in proptest::bool::ANY,
        scenario_idx in 0usize..4usize,
    ) {
        let engine = load_default_posting_engine().expect("Engine must be available");

        let posting_type = if is_credit {
            PostingType::Credit
        } else {
            PostingType::Debit
        };

        let narration = match scenario_idx {
            0 => "PHI DUY TRI TK DIGIBANK HANG THANG",
            1 => "PHI CHUYEN KHOAN LIEN NGAN HANG NAPAS",
            2 => "TRA LAI TIEN GUI THANG NAY",
            _ => "THANH TOAN TIEN HANG DON HANG 102",
        };

        let ctx = PostingContext::new(
            "VCB",
            "REF_PROP_TEST",
            posting_type,
            Money::vnd(amount),
            narration,
            "1121",
            "DOI TAC PROPTEST",
            "guid-prop",
            "batch-prop",
            "match-prop",
            "2026-09-16",
            "VOUCHER_PROP",
            ErpTarget::MisaAmis,
        );

        let proposal = engine.evaluate(&ctx).expect("Evaluation must succeed");

        // Double-entry equation invariant
        prop_assert_eq!(proposal.total_debit, proposal.total_credit);
        prop_assert!(proposal.lines.len() >= 2);

        // JournalEntry balance validation
        let entry = voucher_to_journal_entry(&proposal, 1, 1726400000).expect("Must convert cleanly");
        prop_assert!(entry.verify_balance().is_ok());
    }
}
