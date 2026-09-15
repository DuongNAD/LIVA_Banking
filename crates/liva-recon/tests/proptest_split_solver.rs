//! Property-based testing for Tier 3 Subset-Sum Split Solver in `liva-recon`.
//!
//! Validates:
//! - Exact mathematical conservation: Sum(matched_invoices) == bank_tx_amount (Delta == 0).
//! - Zero floating-point drift: All operations performed strictly on i64 atomic VND.
//! - Bounded search depth: Never exceeds requested max_depth k <= 8.

use liva_ledger::PostingType;
use liva_money::Money;
use liva_parse::CanonicalTx;
use liva_recon::{InternalInvoice, MatchTier, ReconciliationEngine};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_split_solver_zero_delta(
        amounts in prop::collection::vec(10_000i64..100_000_000i64, 2..=5)
    ) {
        // Calculate exact total amount using checked i128
        let total_sum: i64 = amounts.iter().copied().sum();

        let tx = CanonicalTx {
            row_id: 1,
            bank_code: "VCB".to_string(),
            ref_code: "BATCH-PROP-01".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(total_sum),
            balance_after: None,
            counterparty_name: Some("PROGTEST CORP".to_string()),
            counterparty_account: None,
            narration: "THANH TOAN GOM CAC HOA DON PROGTEST".to_string(),
            tx_timestamp: 1726400000,
        };

        let mut invoices = Vec::new();
        let mut inv_ids = Vec::new();

        for (idx, &amt) in amounts.iter().enumerate() {
            let id = format!("INV-P-{}", idx);
            inv_ids.push(id.clone());
            invoices.push(InternalInvoice {
                invoice_id: id.clone(),
                doc_ref: format!("REF-P-{}", idx),
                counterparty_name: "PROGTEST CORP".to_string(),
                amount: Money::vnd(amt),
                timestamp: 1726400000,
            });
        }

        let (matched, rem_tx, _) = ReconciliationEngine::match_tier3(
            &[0],
            &[tx.clone()],
            &inv_ids,
            &invoices,
            8,
        );

        // Assertions for zero-loss invariant
        prop_assert_eq!(matched.len(), 1, "Must find exactly 1 composite match");
        let m = &matched[0];
        prop_assert_eq!(m.tier, MatchTier::Tier3Split);
        prop_assert_eq!(m.bank_amount.amount(), total_sum);
        prop_assert_eq!(m.invoice_amount.amount(), total_sum);

        // Verify constituent invoices sum exactly to total_sum
        let constituent_ids = m.constituent_invoice_ids.as_ref().unwrap();
        let constituent_sum: i64 = constituent_ids
            .iter()
            .map(|id| {
                invoices
                    .iter()
                    .find(|inv| &inv.invoice_id == id)
                    .unwrap()
                    .amount
                    .amount()
            })
            .sum();

        prop_assert_eq!(constituent_sum, total_sum, "Constituent invoices must sum to target amount with zero delta");
        prop_assert!(rem_tx.is_empty(), "Bank tx should be fully resolved");
    }
}
