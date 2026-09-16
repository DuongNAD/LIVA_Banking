use std::collections::HashMap;
use liva_ledger::PostingType;
use liva_money::{Currency, Money};
use crate::models::{BankTransaction, LedgerEntry, MatchConfig, MatchType, ReconciliationMatch};

/// Normalizes document reference tokens into canonical form.
/// E.g. "HD-00102" -> "HD102", "INV/2026/0045" -> "INV45".
pub fn normalize_doc_ref(s: &str) -> String {
    let clean: String = s
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_uppercase();

    if clean.starts_with("HD") && clean.len() > 2 {
        format!("HD{}", clean[2..].trim_start_matches('0'))
    } else if clean.starts_with("INV") && clean.len() > 3 {
        format!("INV{}", clean[3..].trim_start_matches('0'))
    } else {
        clean
    }
}

pub struct HashMatcher;

impl HashMatcher {
    /// Executes Tier 1 O(1) hash matching in a ±24h time window.
    ///
    /// Returns:
    /// - Vector of established 1:1 exact matches.
    /// - Vector of remaining unallocated bank transaction indices.
    /// - Vector of remaining unallocated ledger entry indices.
    pub fn match_tier1(
        bank_txs: &[BankTransaction],
        ledger_entries: &[LedgerEntry],
        config: &MatchConfig,
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<usize>) {
        let mut matches = Vec::new();
        let mut matched_bank = vec![false; bank_txs.len()];
        let mut matched_ledger = vec![false; ledger_entries.len()];

        // 1. Build O(1) index: (amount_minor, currency, direction, normalized_doc_no) -> Vec<ledger_idx>
        let mut ledger_index: HashMap<(i64, Currency, PostingType, String), Vec<usize>> = HashMap::new();
        for (idx, entry) in ledger_entries.iter().enumerate() {
            let norm_ref = normalize_doc_ref(&entry.doc_no);
            if !norm_ref.is_empty() {
                ledger_index
                    .entry((
                        entry.amount.amount(),
                        entry.amount.currency(),
                        entry.direction,
                        norm_ref,
                    ))
                    .or_default()
                    .push(idx);
            }
        }

        // 2. O(1) Probe for each bank transaction
        for (b_idx, tx) in bank_txs.iter().enumerate() {
            let mut candidates = Vec::new();
            if let Some(ref r) = tx.doc_ref {
                candidates.push(normalize_doc_ref(r));
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
                    || (clean.starts_with("FT") && clean.len() >= 6)
                    || (clean.starts_with("NPS") && clean.len() >= 5)
                    || (clean.starts_with("VN") && clean.len() >= 6)
                {
                    candidates.push(normalize_doc_ref(&clean));
                }
            }
            candidates.sort();
            candidates.dedup();

            for cand_ref in candidates {
                let key = (
                    tx.amount.amount(),
                    tx.amount.currency(),
                    tx.direction,
                    cand_ref,
                );
                if let Some(l_indices) = ledger_index.get(&key) {
                    for &l_idx in l_indices {
                        if matched_ledger[l_idx] {
                            continue;
                        }
                        let ledger = &ledger_entries[l_idx];
                        // Strict direction invariance
                        if tx.direction != ledger.direction {
                            continue;
                        }
                        // Strict currency invariance
                        if tx.amount.currency() != ledger.amount.currency() {
                            continue;
                        }
                        let time_diff = (tx.effective_date() - ledger.entry_date).abs();
                        if time_diff <= config.tier1_window_secs {
                            matched_bank[b_idx] = true;
                            matched_ledger[l_idx] = true;
                            matches.push(ReconciliationMatch {
                                match_id: format!("match_exact_{}_{}", tx.id, ledger.id),
                                match_type: MatchType::Exact1To1,
                                bank_tx_ids: vec![tx.id.clone()],
                                ledger_entry_ids: vec![ledger.id.clone()],
                                matched_amount: tx.amount,
                                fee_amount: Money::zero(tx.amount.currency()),
                                discrepancy_amount: Money::zero(tx.amount.currency()),
                                confidence_score: 1.0,
                                explanation: format!(
                                    "Tier 1 Exact Hash Match: Amount={} Ref={}",
                                    tx.amount, ledger.doc_no
                                ),
                                timestamp: tx.effective_date(),
                                status: "APPROVED".to_string(),
                                hitl_token: None,
                            });
                            break;
                        }
                    }
                    if matched_bank[b_idx] {
                        break;
                    }
                }
            }
        }

        let unalloc_bank = (0..bank_txs.len())
            .filter(|&i| !matched_bank[i])
            .collect();
        let unalloc_ledger = (0..ledger_entries.len())
            .filter(|&i| !matched_ledger[i])
            .collect();
        (matches, unalloc_bank, unalloc_ledger)
    }
}
