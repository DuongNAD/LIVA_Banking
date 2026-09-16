#![allow(clippy::float_arithmetic)]

use std::collections::HashMap;
use liva_ledger::PostingType;
use liva_money::{Currency, Money};
use crate::fee_splitter::FeeSplitter;
use crate::jaro_winkler::compare_party_names;
use crate::models::{BankTransaction, LedgerEntry, MatchConfig, MatchType, ReconciliationMatch};

const BUCKET_SIZE: i64 = 11_000;

pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// Executes Tier 2 Amount-Bucketed Fuzzy Jaro-Winkler Matching with Fee Detection in a ±72h window.
    ///
    /// Pre-indexes candidate ledger entries into 11,000 VND buckets to eliminate O(M * N) quadratic complexity.
    pub fn match_tier2(
        bank_txs: &[BankTransaction],
        ledger_entries: &[LedgerEntry],
        unallocated_bank: &[usize],
        unallocated_ledger: &[usize],
        config: &MatchConfig,
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<usize>) {
        let mut matches = Vec::new();
        let mut matched_bank = vec![false; bank_txs.len()];
        let mut matched_ledger = vec![false; ledger_entries.len()];

        // Amount-Bucket Pre-indexing: (currency, direction, amount / BUCKET_SIZE) -> Vec<idx>
        let mut bucket_index: HashMap<(Currency, PostingType, i64), Vec<usize>> = HashMap::new();
        for &l_idx in unallocated_ledger {
            let ledger = &ledger_entries[l_idx];
            let b = ledger.amount.amount() / BUCKET_SIZE;
            bucket_index
                .entry((ledger.amount.currency(), ledger.direction, b))
                .or_default()
                .push(l_idx);
        }

        for &b_idx in unallocated_bank {
            let tx = &bank_txs[b_idx];
            let tx_b = tx.amount.amount() / BUCKET_SIZE;
            let tx_currency = tx.amount.currency();

            // Probe buckets [b-2, b-1, b, b+1, b+2] to cover up to 22,000 VND fee tolerance
            let mut candidate_l_indices = Vec::new();
            for b in [
                tx_b.saturating_sub(2),
                tx_b.saturating_sub(1),
                tx_b,
                tx_b + 1,
                tx_b + 2,
            ] {
                if let Some(indices) = bucket_index.get(&(tx_currency, tx.direction, b)) {
                    candidate_l_indices.extend_from_slice(indices);
                }
            }
            candidate_l_indices.sort_unstable();
            candidate_l_indices.dedup();

            let mut best_l_idx = None;
            let mut best_score = 0.0;
            let mut best_fee = 0i64;

            for &l_idx in &candidate_l_indices {
                if matched_ledger[l_idx] {
                    continue;
                }
                let ledger = &ledger_entries[l_idx];
                if tx.direction != ledger.direction {
                    continue;
                }
                // Strict currency invariance
                if tx.amount.currency() != ledger.amount.currency() {
                    continue;
                }

                let time_diff = (tx.effective_date() - ledger.entry_date).abs();
                if time_diff > config.tier2_window_secs {
                    continue;
                }

                let diff_amount = ledger.amount.amount() - tx.amount.amount();
                let abs_diff = diff_amount.abs();

                let (amount_score, is_amount_valid, fee_val) = if abs_diff == 0 {
                    (1.0, true, 0)
                } else if tx.amount.currency() == Currency::VND
                    && FeeSplitter::is_standard_fee(abs_diff)
                {
                    (0.92, true, abs_diff)
                } else if tx.amount.currency() == Currency::VND
                    && abs_diff >= config.fee_tolerance_min_vnd
                    && abs_diff <= config.fee_tolerance_max_vnd
                {
                    (0.88, true, abs_diff)
                } else {
                    (0.0, false, 0)
                };

                if !is_amount_valid {
                    continue;
                }

                let mut party_sim: f64 = 0.0;
                if let Some(ref partner) = ledger.partner_name {
                    if let Some(ref cp) = tx.counterparty_name {
                        party_sim = party_sim.max(compare_party_names(partner, cp));
                    }
                    party_sim = party_sim.max(compare_party_names(partner, &tx.narration));
                }

                if party_sim < config.tier2_min_party_similarity {
                    continue;
                }

                let date_score =
                    1.0 - (time_diff as f64 / config.tier2_window_secs as f64) * 0.10;
                let combined_score = 0.50 * amount_score + 0.40 * party_sim + 0.10 * date_score;

                if combined_score > best_score && combined_score >= config.tier2_min_combined_score
                {
                    best_score = combined_score;
                    best_l_idx = Some(l_idx);
                    best_fee = fee_val;
                }
            }

            if let Some(l_idx) = best_l_idx {
                let ledger = &ledger_entries[l_idx];
                matched_bank[b_idx] = true;
                matched_ledger[l_idx] = true;

                matches.push(ReconciliationMatch {
                    match_id: format!("match_fuzzy_{}_{}", tx.id, ledger.id),
                    match_type: if best_fee > 0 {
                        MatchType::FeeSplit
                    } else {
                        MatchType::FuzzyHeuristic
                    },
                    bank_tx_ids: vec![tx.id.clone()],
                    ledger_entry_ids: vec![ledger.id.clone()],
                    matched_amount: tx.amount,
                    fee_amount: Money::from_minor(best_fee, tx.amount.currency()),
                    discrepancy_amount: Money::from_minor(
                        ledger.amount.amount() - tx.amount.amount(),
                        tx.amount.currency(),
                    ),
                    confidence_score: best_score,
                    explanation: format!(
                        "Tier 2 Fuzzy Match: Party='{}' Score={:.2} Fee={} {} (Allocated to TK {})",
                        ledger.partner_name.as_deref().unwrap_or(""),
                        best_score,
                        best_fee,
                        tx.amount.currency(),
                        config.account_bank_fee
                    ),
                    timestamp: tx.effective_date(),
                    status: "APPROVED".to_string(),
                    hitl_token: None,
                });
            }
        }

        let unalloc_b = unallocated_bank
            .iter()
            .copied()
            .filter(|&i| !matched_bank[i])
            .collect();
        let unalloc_l = unallocated_ledger
            .iter()
            .copied()
            .filter(|&i| !matched_ledger[i])
            .collect();
        (matches, unalloc_b, unalloc_l)
    }
}
