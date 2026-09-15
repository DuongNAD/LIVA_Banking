//! Tier 3: Constraint Split Solver.
//!
//! Solves bidirectional composite/split payment matching:
//! 1. 1 Bank Transaction -> N Ledger Invoices (composite invoice settlement).
//! 2. 1 Ledger Invoice -> N Bank Payments (multi-installment payment settlement).
//!
//! Algorithm:
//! - Branch-and-bound subset-sum algorithm with capacity and suffix-sum pruning (depth up to 8).
//! - Pre-indexes candidate entries by partner code, counterparty name, and document references.
//! - Direction Invariance: strictly enforces `tx.tx_type == entry.entry_type`.
//! - Mathematical verification proves `Sum == target_amount`, strictly `Delta == 0` (zero float drift).
//! - Fail-closed: Any remaining unallocated transactions with `|Delta| > 0` are routed
//!   into the HITL (Human-in-the-Loop) quarantine queue with a secure UUID v4 token.

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::hash_matcher::normalize_doc_ref;
use super::jaro_winkler::compare_party_names;
use crate::banking::models::{
    BankTransactionRow, InternalLedgerEntry, MatchType, ReconciliationMatch,
};

/// Default maximum number of items in an automatic split combination.
pub const DEFAULT_MAX_SPLIT_DEPTH: usize = 4;
/// Maximum supported search depth for branch-and-bound solver.
pub const MAX_SUPPORTED_SPLIT_DEPTH: usize = 8;

pub struct SplitSolver;

impl SplitSolver {
    /// Executes Tier 3 constraint split solving using the default max depth (4 items).
    /// Matched combinations have `Delta == 0`. Unresolved transactions fail-closed to HITL.
    pub fn match_tier3(
        bank_txs: &[BankTransactionRow],
        ledger_entries: &[InternalLedgerEntry],
        unallocated_bank_indices: &[usize],
        unallocated_ledger_indices: &[usize],
    ) -> (Vec<ReconciliationMatch>, Vec<ReconciliationMatch>) {
        Self::match_tier3_with_depth(
            bank_txs,
            ledger_entries,
            unallocated_bank_indices,
            unallocated_ledger_indices,
            DEFAULT_MAX_SPLIT_DEPTH,
        )
    }

    /// Executes Tier 3 constraint split solving with configurable depth (up to 8 items).
    pub fn match_tier3_with_depth(
        bank_txs: &[BankTransactionRow],
        ledger_entries: &[InternalLedgerEntry],
        unallocated_bank_indices: &[usize],
        unallocated_ledger_indices: &[usize],
        max_depth: usize,
    ) -> (Vec<ReconciliationMatch>, Vec<ReconciliationMatch>) {
        let depth_limit = max_depth.min(MAX_SUPPORTED_SPLIT_DEPTH);
        let mut auto_matches = Vec::new();
        let mut hitl_queue = Vec::new();

        let mut available_ledger = vec![false; ledger_entries.len()];
        for &idx in unallocated_ledger_indices {
            available_ledger[idx] = true;
        }

        let mut matched_bank = vec![false; bank_txs.len()];
        let mut available_bank = vec![false; bank_txs.len()];
        for &b_idx in unallocated_bank_indices {
            available_bank[b_idx] = true;
        }

        // Pre-index ledger doc numbers for rapid candidate extraction
        let mut ledger_by_doc_no: HashMap<String, usize> = HashMap::new();
        for &l_idx in unallocated_ledger_indices {
            let norm_doc = normalize_doc_ref(&ledger_entries[l_idx].doc_no);
            if !norm_doc.is_empty() {
                ledger_by_doc_no.insert(norm_doc, l_idx);
            }
        }

        // =========================================================================
        // Phase A: 1 Bank Transaction -> N Ledger Invoices (Composite Settlement)
        // =========================================================================
        for &b_idx in unallocated_bank_indices {
            if matched_bank[b_idx] {
                continue;
            }
            let tx = &bank_txs[b_idx];

            let mut candidate_indices = Vec::new();
            let mut mentioned_refs = Vec::new();

            if let Some(ref r) = tx.doc_ref {
                let norm = normalize_doc_ref(r);
                if !norm.is_empty() {
                    mentioned_refs.push(norm.clone());
                    if let Some(&l_idx) = ledger_by_doc_no.get(&norm)
                        && available_ledger[l_idx]
                        && ledger_entries[l_idx].entry_type == tx.tx_type
                    {
                        candidate_indices.push(l_idx);
                    }
                }
            }

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
                    let norm = normalize_doc_ref(&clean);
                    mentioned_refs.push(norm.clone());
                    if let Some(&l_idx) = ledger_by_doc_no.get(&norm)
                        && available_ledger[l_idx]
                        && ledger_entries[l_idx].entry_type == tx.tx_type
                    {
                        candidate_indices.push(l_idx);
                    }
                }
            }

            if candidate_indices.len() < 2 {
                for &l_idx in unallocated_ledger_indices {
                    if !available_ledger[l_idx] {
                        continue;
                    }
                    let ledger = &ledger_entries[l_idx];

                    // Direction Invariance: incoming credit cannot match outgoing debit
                    if ledger.entry_type != tx.tx_type {
                        continue;
                    }

                    if ledger.amount > tx.amount {
                        continue;
                    }

                    let norm_doc = normalize_doc_ref(&ledger.doc_no);
                    let is_ref_mentioned = mentioned_refs.iter().any(|r| r == &norm_doc);

                    let is_partner_matched = if is_ref_mentioned {
                        false
                    } else if let Some(ref partner) = ledger.partner_name {
                        if let Some(ref cp_name) = tx.counterparty_name {
                            compare_party_names(partner, cp_name) >= 0.70
                        } else {
                            compare_party_names(partner, &tx.narration) >= 0.70
                        }
                    } else {
                        false
                    };

                    if is_ref_mentioned || is_partner_matched {
                        candidate_indices.push(l_idx);
                    }
                }
            }
            candidate_indices.sort_unstable();
            candidate_indices.dedup();

            // Solve subset-sum: Sum(invoices) == tx.amount
            let cand_pairs: Vec<(usize, u64)> = candidate_indices
                .iter()
                .filter(|&&idx| available_ledger[idx])
                .map(|&idx| (idx, ledger_entries[idx].amount))
                .collect();

            let solved_subset = solve_exact_subset_sum_bnb(&cand_pairs, tx.amount, depth_limit);

            if let Some(subset) = solved_subset {
                // Invariant verification: zero floating-point drift
                let verified_sum: u64 = subset.iter().map(|&idx| ledger_entries[idx].amount).sum();
                assert_eq!(
                    verified_sum, tx.amount,
                    "Arithmetic Invariant Violation: Sum of allocated invoices must equal tx amount"
                );

                for &idx in &subset {
                    available_ledger[idx] = false;
                }
                matched_bank[b_idx] = true;

                let matched_ids: Vec<String> = subset
                    .iter()
                    .map(|&idx| ledger_entries[idx].id.clone())
                    .collect();

                let doc_refs: Vec<String> = subset
                    .iter()
                    .map(|&idx| ledger_entries[idx].doc_no.clone())
                    .collect();

                auto_matches.push(ReconciliationMatch {
                    id: format!("match_split_{}_{}", tx.id, Uuid::new_v4().simple()),
                    bank_tx_id: tx.id.clone(),
                    bank_tx_ids: vec![tx.id.clone()],
                    ledger_entry_ids: matched_ids,
                    match_type: MatchType::CompositeSplit,
                    confidence_score: 0.98,
                    matched_amount: tx.amount,
                    discrepancy_amount: 0,
                    status: "APPROVED".to_string(),
                    matched_by: "ENGINE_AUTOMATIC".to_string(),
                    matched_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                    notes: Some(format!(
                        "Tier 3 Composite Split Match (1-to-N): Invoices [{}] exactly equal {} VND (Delta=0)",
                        doc_refs.join(", "),
                        tx.amount
                    )),
                    hitl_token: None,
                });
            }
        }

        // =========================================================================
        // Phase B: 1 Ledger Invoice -> N Bank Transactions (Multi-Installment Settlement)
        // =========================================================================
        // Pre-index bank transactions by mentioned document references for O(1) candidate lookup
        let mut bank_by_doc_ref: HashMap<String, Vec<usize>> = HashMap::new();
        for &b_idx in unallocated_bank_indices {
            if matched_bank[b_idx] || !available_bank[b_idx] {
                continue;
            }
            let tx = &bank_txs[b_idx];
            let mut tx_refs = HashSet::new();
            if let Some(ref r) = tx.doc_ref {
                let norm = normalize_doc_ref(r);
                if !norm.is_empty() {
                    tx_refs.insert(norm);
                }
            }
            // Preserve '-' and '/' so hyphenated and slashed references (e.g. HD-2026-001, INV-2026-001, PC-001, HD102/2026)
            // are not fragmented into prefix and bare numbers.
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
                    let norm = normalize_doc_ref(&clean);
                    if !norm.is_empty() {
                        tx_refs.insert(norm);
                    }
                }
            }
            for norm in tx_refs {
                bank_by_doc_ref.entry(norm).or_default().push(b_idx);
            }
        }

        for &l_idx in unallocated_ledger_indices {
            if !available_ledger[l_idx] {
                continue;
            }
            let ledger = &ledger_entries[l_idx];
            let norm_ledger_doc = normalize_doc_ref(&ledger.doc_no);

            let mut cand_bank_indices = Vec::new();
            if !norm_ledger_doc.is_empty()
                && let Some(b_indices) = bank_by_doc_ref.get(&norm_ledger_doc)
            {
                for &b_idx in b_indices {
                    if available_bank[b_idx]
                        && !matched_bank[b_idx]
                        && bank_txs[b_idx].tx_type == ledger.entry_type
                        && bank_txs[b_idx].amount <= ledger.amount
                    {
                        cand_bank_indices.push(b_idx);
                    }
                }
            }

            if cand_bank_indices.len() < 2 {
                for &b_idx in unallocated_bank_indices {
                    if matched_bank[b_idx] || !available_bank[b_idx] {
                        continue;
                    }
                    let tx = &bank_txs[b_idx];

                    // Direction Invariance
                    if tx.tx_type != ledger.entry_type || tx.amount > ledger.amount {
                        continue;
                    }

                    let is_doc_mentioned = if !norm_ledger_doc.is_empty() {
                        let narration_upper = tx.narration.to_uppercase();
                        narration_upper.contains(&norm_ledger_doc)
                            || tx
                                .narration
                                .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '/')
                                .map(|tok| tok.trim_matches(|c: char| !c.is_alphanumeric()))
                                .any(|tok| normalize_doc_ref(tok) == norm_ledger_doc)
                    } else {
                        false
                    };

                    let is_partner_matched = if is_doc_mentioned {
                        false
                    } else if let Some(ref partner) = ledger.partner_name {
                        if let Some(ref cp_name) = tx.counterparty_name {
                            compare_party_names(partner, cp_name) >= 0.70
                        } else {
                            compare_party_names(partner, &tx.narration) >= 0.70
                        }
                    } else {
                        false
                    };

                    if is_doc_mentioned || is_partner_matched {
                        cand_bank_indices.push(b_idx);
                    }
                }
            }
            cand_bank_indices.sort_unstable();
            cand_bank_indices.dedup();

            if cand_bank_indices.len() >= 2 {
                let cand_pairs: Vec<(usize, u64)> = cand_bank_indices
                    .iter()
                    .map(|&idx| (idx, bank_txs[idx].amount))
                    .collect();

                let solved_subset =
                    solve_exact_subset_sum_bnb(&cand_pairs, ledger.amount, depth_limit);
                if let Some(subset) = solved_subset
                    && subset.len() >= 2
                {
                    let verified_sum: u64 = subset.iter().map(|&idx| bank_txs[idx].amount).sum();
                    assert_eq!(
                        verified_sum, ledger.amount,
                        "Arithmetic Invariant Violation: Sum of bank payments must equal invoice amount"
                    );

                    available_ledger[l_idx] = false;
                    for &b_idx in &subset {
                        matched_bank[b_idx] = true;
                    }

                    let bank_ids: Vec<String> =
                        subset.iter().map(|&idx| bank_txs[idx].id.clone()).collect();

                    auto_matches.push(ReconciliationMatch {
                            id: format!("match_split_n1_{}_{}", ledger.id, Uuid::new_v4().simple()),
                            bank_tx_id: bank_ids[0].clone(),
                            bank_tx_ids: bank_ids.clone(),
                            ledger_entry_ids: vec![ledger.id.clone()],
                            match_type: MatchType::CompositeSplit,
                            confidence_score: 0.98,
                            matched_amount: ledger.amount,
                            discrepancy_amount: 0,
                            status: "APPROVED".to_string(),
                            matched_by: "ENGINE_AUTOMATIC".to_string(),
                            matched_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64,
                            notes: Some(format!(
                                "Tier 3 Multi-Installment Split Match (N-to-1): {} bank payments [{}] exactly equal {} VND for invoice {} (Delta=0)",
                                subset.len(),
                                bank_ids.join(", "),
                                ledger.amount,
                                ledger.doc_no
                            )),
                            hitl_token: None,
                        });
                }
            }
        }

        // =========================================================================
        // Phase C: Fail-closed into HITL Review Queue with Two-Phase Confirmation UUID
        // =========================================================================
        for &b_idx in unallocated_bank_indices {
            if matched_bank[b_idx] {
                continue;
            }
            let tx = &bank_txs[b_idx];
            let hitl_token = Uuid::new_v4().to_string();

            // Find closest partial candidate for recommendation
            let mut best_single_cand: Option<usize> = None;
            let mut min_diff = i64::MAX;

            for &l_idx in unallocated_ledger_indices {
                if available_ledger[l_idx] {
                    let diff = (ledger_entries[l_idx].amount as i64) - (tx.amount as i64);
                    if diff.abs() < min_diff.abs() {
                        min_diff = diff;
                        best_single_cand = Some(l_idx);
                    }
                }
            }

            let suggested_ledger_ids = best_single_cand
                .map(|idx| vec![ledger_entries[idx].id.clone()])
                .unwrap_or_default();

            let disc_amt = if !suggested_ledger_ids.is_empty() {
                min_diff
            } else {
                -(tx.amount as i64)
            };

            hitl_queue.push(ReconciliationMatch {
                id: format!("hitl_{}_{}", tx.id, Uuid::new_v4().simple()),
                bank_tx_id: tx.id.clone(),
                bank_tx_ids: vec![tx.id.clone()],
                ledger_entry_ids: suggested_ledger_ids,
                match_type: MatchType::ManualHitl,
                confidence_score: 0.50,
                matched_amount: tx.amount,
                discrepancy_amount: disc_amt,
                status: "PENDING_HITL".to_string(),
                matched_by: "USER_HITL".to_string(),
                matched_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                notes: Some(format!(
                    "Residual discrepancy of {} VND. Requires Human-in-the-Loop review.",
                    disc_amt
                )),
                hitl_token: Some(hitl_token),
            });
        }

        (auto_matches, hitl_queue)
    }
}

/// Bounded Branch-and-Bound Subset-Sum Solver.
/// Finds an exact subset of candidates whose sum strictly equals `target`.
/// Supports combinations from 1 up to `max_depth` items (depth <= 8).
pub fn solve_exact_subset_sum_bnb(
    candidates: &[(usize, u64)],
    target: u64,
    max_depth: usize,
) -> Option<Vec<usize>> {
    let mut valid: Vec<(usize, u64)> = candidates
        .iter()
        .copied()
        .filter(|&(_, amt)| amt > 0 && amt <= target)
        .collect();

    if valid.is_empty() || max_depth == 0 {
        return None;
    }

    // Fast path: single candidate exact match
    for &(idx, amt) in &valid {
        if amt == target {
            return Some(vec![idx]);
        }
    }

    if max_depth == 1 {
        return None;
    }

    // Sort ascending by amount for optimal bounding and pruning
    valid.sort_by_key(|c| c.1);

    let n = valid.len();
    let mut suffix_sums = vec![0u64; n + 1];
    for i in (0..n).rev() {
        suffix_sums[i] = suffix_sums[i + 1].saturating_add(valid[i].1);
    }

    // Bound check: sum of all candidates cannot reach target
    if suffix_sums[0] < target {
        return None;
    }

    let mut selected = Vec::with_capacity(max_depth);
    if bnb_recurse(0, 0, &mut selected, &valid, &suffix_sums, target, max_depth) {
        Some(selected)
    } else {
        None
    }
}

fn bnb_recurse(
    idx: usize,
    current_sum: u64,
    selected: &mut Vec<usize>,
    candidates: &[(usize, u64)],
    suffix_sums: &[u64],
    target: u64,
    max_depth: usize,
) -> bool {
    if current_sum == target {
        return !selected.is_empty();
    }
    if selected.len() >= max_depth || idx >= candidates.len() {
        return false;
    }

    let remaining_depth = max_depth - selected.len();
    let available_count = candidates.len() - idx;

    // Pruning Bound 1: If minimum remaining addition exceeds target, impossible (since sorted ascending)
    if current_sum.saturating_add(candidates[idx].1) > target {
        return false;
    }

    // Pruning Bound 2: If sum of all remaining items cannot reach target, impossible
    if current_sum.saturating_add(suffix_sums[idx]) < target {
        return false;
    }

    // Pruning Bound 3: Sum of largest `remaining_depth` items cannot reach target
    if available_count > remaining_depth {
        let max_possible: u64 = candidates[candidates.len() - remaining_depth..]
            .iter()
            .map(|c| c.1)
            .sum();
        if current_sum.saturating_add(max_possible) < target {
            return false;
        }
    }

    // Branch 1: Include candidates[idx]
    selected.push(candidates[idx].0);
    if bnb_recurse(
        idx + 1,
        current_sum + candidates[idx].1,
        selected,
        candidates,
        suffix_sums,
        target,
        max_depth,
    ) {
        return true;
    }
    selected.pop();

    // Branch 2: Exclude candidates[idx]
    bnb_recurse(
        idx + 1,
        current_sum,
        selected,
        candidates,
        suffix_sums,
        target,
        max_depth,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banking::models::{ReconciliationStatus, TransactionType};

    #[test]
    fn test_tier3_composite_split_solver() {
        let bank_tx = BankTransactionRow {
            id: "tx_composite".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: 100_000,
            value_date: 100_000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 100_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY ABC".to_string()),
            counterparty_bank: None,
            narration: "CK HD 102 va 103 con lai no HD 104".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: 100_000,
        };

        let led102 = InternalLedgerEntry {
            id: "led_102".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD102".to_string(),
            entry_date: 100_000,
            entry_type: TransactionType::Credit,
            amount: 45_000_000,
            partner_code: Some("ABC01".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: "No HD 102".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let led103 = InternalLedgerEntry {
            id: "led_103".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD103".to_string(),
            entry_date: 100_000,
            entry_type: TransactionType::Credit,
            amount: 55_000_000,
            partner_code: Some("ABC01".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: "No HD 103".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let led104 = InternalLedgerEntry {
            id: "led_104".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD104".to_string(),
            entry_date: 100_000,
            entry_type: TransactionType::Credit,
            amount: 80_000_000,
            partner_code: Some("ABC01".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: "No HD 104".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let (auto_matches, hitl) =
            SplitSolver::match_tier3(&[bank_tx], &[led102, led103, led104], &[0], &[0, 1, 2]);

        assert_eq!(auto_matches.len(), 1);
        assert_eq!(hitl.len(), 0);
        assert_eq!(auto_matches[0].match_type, MatchType::CompositeSplit);
        assert_eq!(auto_matches[0].matched_amount, 100_000_000);
        assert_eq!(auto_matches[0].discrepancy_amount, 0);
        assert_eq!(auto_matches[0].ledger_entry_ids.len(), 2);
    }

    #[test]
    fn test_tier3_bidirectional_n_to_1_installments() {
        let now = 1_725_000_000i64;

        // 1 Invoice of 100M VND
        let invoice = InternalLedgerEntry {
            id: "inv_big".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "INV-BIG-100".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 100_000_000,
            partner_code: Some("PARTNER_X".to_string()),
            partner_name: Some("Cong ty Doi Tac X".to_string()),
            description: "Hop dong lon".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };

        // 3 separate installment payments: 30M + 30M + 40M = 100M
        let tx1 = BankTransactionRow {
            id: "tx_inst_1".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 1000,
            value_date: now + 1000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 30_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY DOI TAC X".to_string()),
            counterparty_bank: None,
            narration: "Thanh toan dot 1 INV-BIG-100".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let tx2 = BankTransactionRow {
            id: "tx_inst_2".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 2000,
            value_date: now + 2000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 30_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY DOI TAC X".to_string()),
            counterparty_bank: None,
            narration: "Thanh toan dot 2 INV-BIG-100".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let tx3 = BankTransactionRow {
            id: "tx_inst_3".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 3000,
            value_date: now + 3000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 40_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY DOI TAC X".to_string()),
            counterparty_bank: None,
            narration: "Thanh toan dot 3 INV-BIG-100".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let (matches, hitl) =
            SplitSolver::match_tier3(&[tx1, tx2, tx3], &[invoice], &[0, 1, 2], &[0]);

        assert_eq!(matches.len(), 1, "N-to-1 installment match must be found");
        assert_eq!(hitl.len(), 0, "No remaining items in HITL");
        let m = &matches[0];
        assert_eq!(m.matched_amount, 100_000_000);
        assert_eq!(m.discrepancy_amount, 0);
        assert_eq!(m.bank_tx_ids.len(), 3);
        assert_eq!(m.ledger_entry_ids, vec!["inv_big"]);
        assert!(m.is_split_n_to_one());
    }

    #[test]
    fn test_tier3_branch_and_bound_depth_up_to_8() {
        // 6 items of 10M = 60M
        let amounts: Vec<(usize, u64)> = (0..6).map(|i| (i, 10_000_000)).collect();
        let target = 60_000_000;

        // With depth 4: should fail (too many items)
        let res_d4 = solve_exact_subset_sum_bnb(&amounts, target, 4);
        assert!(res_d4.is_none());

        // With depth 6: should succeed!
        let res_d6 = solve_exact_subset_sum_bnb(&amounts, target, 6);
        assert!(res_d6.is_some());
        let subset = res_d6.unwrap();
        assert_eq!(subset.len(), 6);
        let sum: u64 = subset.iter().map(|&i| amounts[i].1).sum();
        assert_eq!(sum, target);

        // 8 items of varying amounts = 80M
        let var_amounts = vec![
            (0, 5_000_000),
            (1, 5_000_000),
            (2, 10_000_000),
            (3, 10_000_000),
            (4, 10_000_000),
            (5, 10_000_000),
            (6, 15_000_000),
            (7, 15_000_000),
        ];
        let target80 = 80_000_000;
        let res8 = solve_exact_subset_sum_bnb(&var_amounts, target80, 8);
        assert!(res8.is_some());
        assert_eq!(res8.unwrap().len(), 8);
    }

    #[test]
    fn test_tier3_direction_mismatch_rejected() {
        let now = 1_725_000_000i64;

        let tx = BankTransactionRow {
            id: "tx_credit_split".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit, // Credit
            amount: 50_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY ABC".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN HD1 VA HD2".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let i1 = InternalLedgerEntry {
            id: "i1".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD1".to_string(),
            entry_date: now,
            entry_type: TransactionType::Debit, // Debit - Mismatch!
            amount: 25_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty ABC".to_string()),
            description: "".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };
        let i2 = InternalLedgerEntry {
            id: "i2".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD2".to_string(),
            entry_date: now,
            entry_type: TransactionType::Debit, // Debit - Mismatch!
            amount: 25_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty ABC".to_string()),
            description: "".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };

        let (matches, hitl) = SplitSolver::match_tier3(&[tx], &[i1, i2], &[0], &[0, 1]);

        assert_eq!(
            matches.len(),
            0,
            "Direction mismatch in Tier 3 must NOT match"
        );
        assert_eq!(hitl.len(), 1, "Must fail closed to HITL");
    }

    #[test]
    fn test_tier3_hyphenated_and_slashed_doc_refs_bidirectional() {
        let now = 1_725_000_000i64;

        // Subtest 1: 1-to-N (Phase A) with hyphenated references in narration
        let tx_1_to_n = BankTransactionRow {
            id: "tx_1toN".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 90_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY ABC".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN HOP DONG HD-2026-001 VA HD-2026-002".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let inv_a1 = InternalLedgerEntry {
            id: "inv_a1".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD-2026-001".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 40_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty ABC".to_string()),
            description: "Hoa don HD-2026-001".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };

        let inv_a2 = InternalLedgerEntry {
            id: "inv_a2".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD-2026-002".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 50_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty ABC".to_string()),
            description: "Hoa don HD-2026-002".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };

        let (matches_a, hitl_a) =
            SplitSolver::match_tier3(&[tx_1_to_n], &[inv_a1, inv_a2], &[0], &[0, 1]);
        assert_eq!(
            matches_a.len(),
            1,
            "Phase A: Must match 1-to-N with hyphenated doc refs"
        );
        assert_eq!(hitl_a.len(), 0);
        assert_eq!(matches_a[0].discrepancy_amount, 0);
        assert_eq!(matches_a[0].ledger_entry_ids.len(), 2);

        // Subtest 2: N-to-1 (Phase B) with hyphenated references in narration (O(1) pre-indexed)
        let tx_n_to_1_part1 = BankTransactionRow {
            id: "tx_nto1_p1".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 60_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY XYZ".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN DOT 1 INV-2026-001".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let tx_n_to_1_part2 = BankTransactionRow {
            id: "tx_nto1_p2".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 3600,
            value_date: now + 3600,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 40_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY XYZ".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN DOT 2 INV-2026-001".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now + 3600,
        };

        let inv_single = InternalLedgerEntry {
            id: "inv_single".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "INV-2026-001".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 100_000_000,
            partner_code: Some("XYZ".to_string()),
            partner_name: Some("Công ty XYZ".to_string()),
            description: "Hoa don INV-2026-001".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };

        let (matches_b, hitl_b) = SplitSolver::match_tier3(
            &[tx_n_to_1_part1, tx_n_to_1_part2],
            &[inv_single],
            &[0, 1],
            &[0],
        );
        assert_eq!(
            matches_b.len(),
            1,
            "Phase B: Must match N-to-1 with hyphenated doc refs via pre-indexing"
        );
        assert_eq!(hitl_b.len(), 0);
        assert_eq!(matches_b[0].discrepancy_amount, 0);
        assert_eq!(matches_b[0].bank_tx_ids.len(), 2);
    }
}
