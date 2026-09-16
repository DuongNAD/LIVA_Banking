//! Tier 3: Bounded Subset-Sum Split Solver & Fair Allocation Engine.
//!
//! Features:
//! - 1:N composite matching (Single lump-sum bank deposit paying multiple invoices)
//! - N:1 partial payment matching (Multiple installment payments paying one invoice)
//! - Bounded search (k <= 8, fail-closed if multiple solutions found)
//! - Fair allocation invariant: Sum of allocated amounts == Target amount (Zero over-allocation)
//! - Optimistic concurrency control (OCC version check)

use crate::models::{ErpDocument, MatchProposal, MatchType, TransactionRecord};

pub struct SplitSolver;

impl SplitSolver {
    /// Finds combinations of ERP documents that sum exactly to a single bank transaction amount (1:N).
    pub fn solve_1_to_n(
        tx: &TransactionRecord,
        candidate_docs: &[ErpDocument],
    ) -> Option<MatchProposal> {
        let target = tx.amount;
        // Bound candidates to at most 8 to prevent exponential blowup
        let docs: Vec<&ErpDocument> = candidate_docs
            .iter()
            .filter(|d| d.doc_type == tx.tx_type && d.open_amount > 0 && d.open_amount <= target)
            .take(8)
            .collect();

        if docs.is_empty() {
            return None;
        }

        let n = docs.len();
        let mut solutions = Vec::new();

        // Check power set (2^n combinations, n <= 8 -> max 256 checks, instantaneous)
        for mask in 1..(1 << n) {
            let mut sum: u64 = 0;
            let mut subset = Vec::new();

            for i in 0..n {
                if (mask & (1 << i)) != 0 {
                    sum = sum.saturating_add(docs[i].open_amount);
                    subset.push(docs[i]);
                }
            }

            if sum == target {
                solutions.push(subset);
                if solutions.len() > 1 {
                    // Ambiguity: multiple combinations sum to the same amount! Fail-closed.
                    return None;
                }
            }
        }

        if solutions.len() == 1 {
            let matched_subset = &solutions[0];
            let doc_ids: Vec<String> = matched_subset.iter().map(|d| d.id.clone()).collect();
            Some(MatchProposal {
                proposal_id: format!("PROP-T3-1N-{}", tx.row_id),
                bank_tx_id: tx.row_id,
                erp_doc_ids: doc_ids,
                match_type: MatchType::CompositeSplit,
                confidence_score: 85,
                matched_amount: target,
                fee_amount: 0,
                fee_account: None,
                notes: format!(
                    "Tier 3 1:N Composite Match: 1 bank transaction pays {} ERP invoices exactly",
                    matched_subset.len()
                ),
                status: "DRAFT".to_string(), // Requires Maker-Checker approval
                version: 1,
                created_at: tx.tx_date,
            })
        } else {
            None
        }
    }

    /// Performs fair partial allocation ensuring:
    /// allocated_amount <= remaining_transaction_amount AND allocated_amount <= open_invoice_amount.
    pub fn allocate_partial(
        tx_amount_remaining: &mut u64,
        doc_open_remaining: &mut u64,
        doc_version: u64,
        expected_version: u64,
    ) -> Result<u64, &'static str> {
        // Optimistic Concurrency Control
        if doc_version != expected_version {
            return Err("OptimisticLockError: Document version mismatch, concurrent modification detected");
        }

        if *tx_amount_remaining == 0 {
            return Err("AllocationError: Transaction amount already exhausted");
        }

        if *doc_open_remaining == 0 {
            return Err("AllocationError: Invoice is already fully settled");
        }

        let alloc = (*tx_amount_remaining).min(*doc_open_remaining);
        *tx_amount_remaining -= alloc;
        *doc_open_remaining -= alloc;

        Ok(alloc)
    }
}
