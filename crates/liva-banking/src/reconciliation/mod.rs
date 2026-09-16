//! 3-Tier Reconciliation Engine.

pub mod tier1;
pub mod tier2;
pub mod tier3;

pub use tier1::{Tier1Matcher, Tier1Result, normalize_ref};
pub use tier2::{Tier2Matcher, jaro_winkler_score};
pub use tier3::SplitSolver;

use crate::models::{ErpDocument, MatchProposal, TransactionRecord};

pub struct FullReconciliationResult {
    pub auto_matched: Vec<MatchProposal>,
    pub proposed_matches: Vec<MatchProposal>,
    pub candidate_conflicts: Vec<MatchProposal>,
    pub unmatched_tx_indices: Vec<usize>,
    pub unmatched_doc_ids: Vec<String>,
}

pub struct ReconciliationEngine;

impl ReconciliationEngine {
    /// Executes the full 3-Tier reconciliation pipeline:
    /// 1. Tier 1 exact match (Single Candidate Rule) -> Auto-matched or Conflicts
    /// 2. Tier 2 fuzzy heuristics & wire fee extraction on remaining items -> Draft Proposals
    /// 3. Tier 3 1:N composite split solver on remaining items -> Split Proposals
    pub fn reconcile(
        txs: &[TransactionRecord],
        docs: &[ErpDocument],
    ) -> FullReconciliationResult {
        // Run Tier 1
        let t1_res = Tier1Matcher::match_tier1(txs, docs);

        // Run Tier 2 on unmatched
        let t2_proposals = Tier2Matcher::match_tier2(
            txs,
            &t1_res.unmatched_tx_indices,
            docs,
            &t1_res.unmatched_doc_ids,
        );

        // Run Tier 3 split solver on remaining transactions
        let mut t3_proposals = Vec::new();
        let remaining_docs: Vec<ErpDocument> = docs
            .iter()
            .filter(|d| t1_res.unmatched_doc_ids.contains(&d.id))
            .cloned()
            .collect();

        for &tx_idx in &t1_res.unmatched_tx_indices {
            let tx = &txs[tx_idx];
            // If already proposed in Tier 2 with high confidence, skip Tier 3
            if t2_proposals.iter().any(|p| p.bank_tx_id == tx.row_id && p.confidence_score >= 85) {
                continue;
            }

            if let Some(split_prop) = SplitSolver::solve_1_to_n(tx, &remaining_docs) {
                t3_proposals.push(split_prop);
            }
        }

        let mut all_proposals = t2_proposals;
        all_proposals.extend(t3_proposals);

        FullReconciliationResult {
            auto_matched: t1_res.auto_matched,
            proposed_matches: all_proposals,
            candidate_conflicts: t1_res.candidate_conflicts,
            unmatched_tx_indices: t1_res.unmatched_tx_indices,
            unmatched_doc_ids: t1_res.unmatched_doc_ids,
        }
    }
}
