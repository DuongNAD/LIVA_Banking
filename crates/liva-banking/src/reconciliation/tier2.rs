//! Tier 2: Heuristic Fuzzy Matcher with Fee Separation (TK 6425).
//!
//! Features:
//! - Jaro-Winkler string similarity on partner names and narrations
//! - 72h time window tolerance (|tx_date - doc_date| <= 259,200s)
//! - Bank wire transfer fee detection (e.g. 1,100 - 33,000 VND), allocated to TK 6425
//! - Always outputs MatchProposal with status "DRAFT" / "SUBMITTED" (Never auto-accepted)

use crate::models::{ErpDocument, MatchProposal, MatchType, TransactionRecord};

/// Jaro-Winkler string distance algorithm (returns 0..100)
pub fn jaro_winkler_score(s1: &str, s2: &str) -> u32 {
    let s1_clean = s1.trim().to_lowercase();
    let s2_clean = s2.trim().to_lowercase();

    if s1_clean == s2_clean {
        return 100;
    }
    if s1_clean.is_empty() || s2_clean.is_empty() {
        return 0;
    }

    let s1_chars: Vec<char> = s1_clean.chars().collect();
    let s2_chars: Vec<char> = s2_clean.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);
    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;
    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);
        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0;
    }

    let mut transpositions = 0;
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    let jaro = ((m / len1 as f64) + (m / len2 as f64) + ((m - (transpositions as f64 / 2.0)) / m)) / 3.0;

    // Common prefix length up to 4 characters
    let mut prefix_len = 0;
    for (c1, c2) in s1_chars.iter().zip(s2_chars.iter()) {
        if c1 == c2 && prefix_len < 4 {
            prefix_len += 1;
        } else {
            break;
        }
    }

    let jaro_winkler = jaro + (prefix_len as f64 * 0.1 * (1.0 - jaro));
    (jaro_winkler.clamp(0.0, 1.0) * 100.0) as u32
}

pub struct Tier2Matcher;

impl Tier2Matcher {
    /// Evaluates unmatched transactions against open ERP documents.
    pub fn match_tier2(
        txs: &[TransactionRecord],
        unmatched_tx_indices: &[usize],
        docs: &[ErpDocument],
        unmatched_doc_ids: &[String],
    ) -> Vec<MatchProposal> {
        let mut proposals = Vec::new();
        let active_docs: Vec<&ErpDocument> = docs
            .iter()
            .filter(|d| unmatched_doc_ids.contains(&d.id) && d.open_amount > 0)
            .collect();

        for &tx_idx in unmatched_tx_indices {
            let tx = &txs[tx_idx];

            for doc in &active_docs {
                // Must match direction (Credit -> AR/Inflow, Debit -> AP/Outflow)
                if tx.tx_type != doc.doc_type {
                    continue;
                }

                // Check time window within 72 hours (259,200s)
                if (tx.tx_date - doc.doc_date).abs() > 259_200 {
                    continue;
                }

                // Check exact amount match OR amount with bank fee
                let is_exact_amt = tx.amount == doc.open_amount;
                let (has_fee, fee_amt) = if !is_exact_amt && tx.amount < doc.open_amount {
                    let diff = doc.open_amount - tx.amount;
                    // Standard VN banking wire transfer fees: 1.100đ, 2.200đ, 5.500đ, 7.700đ, 9.900đ, 11.000đ, 22.000đ, 33.000đ
                    if (1_100..=33_000).contains(&diff) {
                        (true, diff)
                    } else {
                        (false, 0)
                    }
                } else {
                    (false, 0)
                };

                if !is_exact_amt && !has_fee {
                    continue;
                }

                // Calculate textual similarity on counterparty name or narration
                let name_score = if let Some(cp) = &tx.counterparty_name {
                    jaro_winkler_score(cp, &doc.partner_name)
                } else {
                    0
                };
                let narration_score = jaro_winkler_score(&tx.narration, &doc.partner_name);
                let text_score = name_score.max(narration_score);

                // Threshold >= 80 for candidate proposals
                if text_score >= 80 || is_exact_amt {
                    let confidence = if is_exact_amt && text_score >= 85 {
                        90
                    } else if has_fee && text_score >= 85 {
                        85
                    } else {
                        75
                    };

                    let (notes, fee_account) = if has_fee {
                        (
                            format!(
                                "Tier 2 Proposal: Partner matched with wire fee deduction (Fee: {} VND to TK 6425)",
                                fee_amt
                            ),
                            Some("6425".to_string()),
                        )
                    } else {
                        (
                            format!("Tier 2 Proposal: Fuzzy partner similarity ({text_score}%)"),
                            None,
                        )
                    };

                    proposals.push(MatchProposal {
                        proposal_id: format!("PROP-T2-{}", proposals.len() + 1),
                        bank_tx_id: tx.row_id,
                        erp_doc_ids: vec![doc.id.clone()],
                        match_type: MatchType::FuzzyHeuristic,
                        confidence_score: confidence,
                        matched_amount: tx.amount,
                        fee_amount: fee_amt,
                        fee_account,
                        notes,
                        status: "DRAFT".to_string(), // Never auto-posts!
                        version: 1,
                        created_at: tx.tx_date,
                    });
                }
            }
        }

        proposals
    }
}
