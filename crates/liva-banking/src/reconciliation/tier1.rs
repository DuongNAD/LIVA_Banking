//! Tier 1: Exact Hash Matcher with Single Candidate Rule.
//!
//! Matches transactions where:
//! - Scaled amount matches exactly (`u64 == u64`)
//! - Direction matches (Credit matches Credit / AR, Debit matches Debit / AP)
//! - Normalized reference code matches
//! - Time window |tx_date - doc_date| <= 24h (86,400s)
//! - Single Candidate Rule: If multiple candidates match the same hash key,
//!   auto-matching is FAIL-CLOSED and candidates are diverted to Tier 2/proposals.

use crate::models::{ErpDocument, MatchProposal, MatchType, TransactionRecord, TransactionType};
use std::collections::HashMap;

/// Normalizes reference codes (e.g. "HD-00102" -> "HD102", "INV 2026/01" -> "INV202601")
pub fn normalize_ref(s: &str) -> String {
    let clean: String = s
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_uppercase();

    if clean.starts_with("HD") && clean.len() > 2 {
        let suffix = clean[2..].trim_start_matches('0');
        format!("HD{suffix}")
    } else if clean.starts_with("INV") && clean.len() > 3 {
        let suffix = clean[3..].trim_start_matches('0');
        format!("INV{suffix}")
    } else {
        clean
    }
}

pub struct Tier1Result {
    pub auto_matched: Vec<MatchProposal>,
    pub candidate_conflicts: Vec<MatchProposal>,
    pub unmatched_tx_indices: Vec<usize>,
    pub unmatched_doc_ids: Vec<String>,
}

pub struct Tier1Matcher;

impl Tier1Matcher {
    pub fn match_tier1(
        txs: &[TransactionRecord],
        docs: &[ErpDocument],
    ) -> Tier1Result {
        // Map: (amount, direction, normalized_ref) -> Vec<doc_idx>
        let mut index: HashMap<(u64, TransactionType, String), Vec<usize>> = HashMap::new();

        for (idx, doc) in docs.iter().enumerate() {
            if doc.open_amount == 0 {
                continue;
            }
            let key_ref = doc
                .invoice_no
                .as_deref()
                .unwrap_or(&doc.voucher_no);
            let norm = normalize_ref(key_ref);
            if !norm.is_empty() {
                index.entry((doc.open_amount, doc.doc_type, norm)).or_default().push(idx);
            }
        }

        let mut auto_matched = Vec::new();
        let mut candidate_conflicts = Vec::new();
        let mut matched_tx_set = std::collections::HashSet::new();
        let mut matched_doc_set = std::collections::HashSet::new();

        for (tx_idx, tx) in txs.iter().enumerate() {
            let tx_ref = tx
                .doc_ref
                .as_deref()
                .or(tx.raw_ref.as_deref())
                .unwrap_or("");
            let norm = normalize_ref(tx_ref);

            // Also check if narration contains invoice pattern
            let search_norm = if norm.is_empty() {
                // Try finding voucher or invoice in narration
                let words: Vec<&str> = tx.narration.split_whitespace().collect();
                let mut found = String::new();
                for w in words {
                    let c = normalize_ref(w);
                    if (c.starts_with("HD") || c.starts_with("INV")) && c.len() > 3 {
                        found = c;
                        break;
                    }
                }
                found
            } else {
                norm
            };

            if search_norm.is_empty() {
                continue;
            }

            let key = (tx.amount, tx.tx_type, search_norm);
            if let Some(candidate_indices) = index.get(&key) {
                // Filter available candidates within 24h window
                let valid_candidates: Vec<usize> = candidate_indices
                    .iter()
                    .copied()
                    .filter(|&d_idx| {
                        !matched_doc_set.contains(&docs[d_idx].id)
                            && (tx.tx_date - docs[d_idx].doc_date).abs() <= 86400
                    })
                    .collect();

                if valid_candidates.len() == 1 {
                    // Single Candidate Rule satisfied! Safe exact auto-match.
                    let doc_idx = valid_candidates[0];
                    let doc = &docs[doc_idx];

                    matched_tx_set.insert(tx_idx);
                    matched_doc_set.insert(doc.id.clone());

                    auto_matched.push(MatchProposal {
                        proposal_id: format!("PROP-T1-{}", auto_matched.len() + 1),
                        bank_tx_id: tx.row_id,
                        erp_doc_ids: vec![doc.id.clone()],
                        match_type: MatchType::Exact1To1,
                        confidence_score: 100,
                        matched_amount: tx.amount,
                        fee_amount: 0,
                        fee_account: None,
                        notes: format!("Tier 1 Exact Match: Reference {} matched voucher {}", tx_ref, doc.voucher_no),
                        status: "APPROVED".to_string(), // Tier 1 auto-accepted
                        version: 1,
                        created_at: tx.tx_date,
                    });
                } else if valid_candidates.len() > 1 {
                    // Ambiguity / Collision detected: Divert to human proposals!
                    let doc_ids: Vec<String> = valid_candidates.iter().map(|&i| docs[i].id.clone()).collect();
                    candidate_conflicts.push(MatchProposal {
                        proposal_id: format!("PROP-T1-CONFLICT-{}", candidate_conflicts.len() + 1),
                        bank_tx_id: tx.row_id,
                        erp_doc_ids: doc_ids,
                        match_type: MatchType::Exact1To1,
                        confidence_score: 75,
                        matched_amount: tx.amount,
                        fee_amount: 0,
                        fee_account: None,
                        notes: format!("Tier 1 Ambiguity: {} valid candidates found. Routed to manual review.", valid_candidates.len()),
                        status: "DRAFT".to_string(),
                        version: 1,
                        created_at: tx.tx_date,
                    });
                }
            }
        }

        let unmatched_tx_indices = (0..txs.len())
            .filter(|i| !matched_tx_set.contains(i))
            .collect();

        let unmatched_doc_ids = docs
            .iter()
            .filter(|d| !matched_doc_set.contains(&d.id))
            .map(|d| d.id.clone())
            .collect();

        Tier1Result {
            auto_matched,
            candidate_conflicts,
            unmatched_tx_indices,
            unmatched_doc_ids,
        }
    }
}
