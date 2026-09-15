use std::collections::HashMap;

use super::jaro_winkler::compare_party_names;
use crate::banking::models::{
    BankTransactionRow, InternalLedgerEntry, MatchType, ReconciliationMatch, TransactionType,
};

const WINDOW_72H_SECS: i64 = 259_200;
const BUCKET_SIZE: u64 = 11_000;

pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// Executes Tier 2 fuzzy heuristic matching on unallocated transactions and ledger entries.
    /// Employs Amount-Bucket Pre-indexing (+- 11,000 VND) and strict Direction Invariance
    /// to eliminate quadratic O(M x N) scans on large statement pools.
    pub fn match_tier2(
        bank_txs: &[BankTransactionRow],
        ledger_entries: &[InternalLedgerEntry],
        unallocated_bank_indices: &[usize],
        unallocated_ledger_indices: &[usize],
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<usize>) {
        let mut matches = Vec::new();
        let mut matched_bank = vec![false; bank_txs.len()];
        let mut matched_ledger = vec![false; ledger_entries.len()];

        // 1. Amount-Bucket Pre-indexing:
        // Key: (entry_type, amount / BUCKET_SIZE) -> Vec<ledger_idx>
        // Enforces strict Direction Invariance by segmenting Debit vs Credit buckets.
        let mut bucket_index: HashMap<(TransactionType, u64), Vec<usize>> = HashMap::new();
        for &l_idx in unallocated_ledger_indices {
            let ledger = &ledger_entries[l_idx];
            let b = ledger.amount / BUCKET_SIZE;
            bucket_index
                .entry((ledger.entry_type, b))
                .or_default()
                .push(l_idx);
        }

        // 2. Probe candidate buckets for each unallocated bank transaction
        for &b_idx in unallocated_bank_indices {
            let tx = &bank_txs[b_idx];

            let mut best_l_idx: Option<usize> = None;
            let mut best_score: f64 = 0.0;
            let mut best_discrepancy: i64 = 0;

            let tx_b = tx.amount / BUCKET_SIZE;
            let mut candidate_l_indices = Vec::new();
            for b in [tx_b.saturating_sub(1), tx_b, tx_b + 1] {
                if let Some(indices) = bucket_index.get(&(tx.tx_type, b)) {
                    candidate_l_indices.extend_from_slice(indices);
                }
            }
            candidate_l_indices.sort_unstable();
            candidate_l_indices.dedup();

            for &l_idx in &candidate_l_indices {
                if matched_ledger[l_idx] {
                    continue;
                }
                let ledger = &ledger_entries[l_idx];

                // Strict Direction Invariance: Credit only matches Credit, Debit only matches Debit
                if tx.tx_type != ledger.entry_type {
                    continue;
                }

                // Effective Date check: |t_eff - t_ledger| <= 72h
                let eff_tx_date = tx.effective_date();
                let time_diff = (eff_tx_date - ledger.entry_date).abs();
                if time_diff > WINDOW_72H_SECS {
                    continue;
                }

                // Amount check: exact (0 diff) or standard bank fee (1,100 - 11,000 VND)
                let diff_amount = (ledger.amount as i64) - (tx.amount as i64);
                let abs_diff = diff_amount.abs();

                let (amount_score, is_amount_valid) = if abs_diff == 0 {
                    (1.0, true)
                } else if (1_100..=11_000).contains(&abs_diff) {
                    // Standard interbank Napas / VCB transfer fee deduction
                    (0.92, true)
                } else {
                    (0.0, false)
                };

                if !is_amount_valid {
                    continue;
                }

                // Party name similarity (with bank prefix and legal noise stripping)
                let mut max_party_sim: f64 = 0.0;
                if let Some(partner_name) = ledger.partner_name.as_deref() {
                    if let Some(cp_name) = tx.counterparty_name.as_deref() {
                        let sim = compare_party_names(partner_name, cp_name);
                        if sim > max_party_sim {
                            max_party_sim = sim;
                        }
                    }

                    // Also compare partner name against narration
                    let narration_sim = compare_party_names(partner_name, &tx.narration);
                    if narration_sim > max_party_sim {
                        max_party_sim = narration_sim;
                    }
                }

                if max_party_sim < 0.85 {
                    continue;
                }

                let date_score = 1.0 - (time_diff as f64 / WINDOW_72H_SECS as f64) * 0.10;
                let combined_score = 0.50 * amount_score + 0.40 * max_party_sim + 0.10 * date_score;

                if combined_score > best_score && combined_score >= 0.85 {
                    best_score = combined_score;
                    best_l_idx = Some(l_idx);
                    best_discrepancy = diff_amount;
                }
            }

            if let Some(l_idx) = best_l_idx {
                let ledger = &ledger_entries[l_idx];
                matched_bank[b_idx] = true;
                matched_ledger[l_idx] = true;

                let is_fee_discrepancy = best_discrepancy != 0;
                let note = if is_fee_discrepancy {
                    format!(
                        "Tier 2 Fuzzy Match: Party='{}' with standard fee diff={} VND",
                        ledger.partner_name.as_deref().unwrap_or(""),
                        best_discrepancy
                    )
                } else {
                    format!(
                        "Tier 2 Fuzzy Match: Party='{}' Jaro-Winkler score={:.2}",
                        ledger.partner_name.as_deref().unwrap_or(""),
                        best_score
                    )
                };

                matches.push(ReconciliationMatch {
                    id: format!("match_fuzz_{}_{}", tx.id, ledger.id),
                    bank_tx_id: tx.id.clone(),
                    bank_tx_ids: vec![tx.id.clone()],
                    ledger_entry_ids: vec![ledger.id.clone()],
                    match_type: MatchType::FuzzyHeuristic,
                    confidence_score: best_score,
                    matched_amount: tx.amount,
                    discrepancy_amount: best_discrepancy,
                    status: "APPROVED".to_string(),
                    matched_by: "ENGINE_AUTOMATIC".to_string(),
                    matched_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                    notes: Some(note),
                    hitl_token: None,
                });
            }
        }

        let remaining_bank_indices: Vec<usize> = unallocated_bank_indices
            .iter()
            .copied()
            .filter(|&i| !matched_bank[i])
            .collect();
        let remaining_ledger_indices: Vec<usize> = unallocated_ledger_indices
            .iter()
            .copied()
            .filter(|&i| !matched_ledger[i])
            .collect();

        (matches, remaining_bank_indices, remaining_ledger_indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banking::models::{ReconciliationStatus, TransactionType};

    #[test]
    fn test_tier2_fuzzy_heuristic_match() {
        let bank_tx = BankTransactionRow {
            id: "tx_2".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: 100_000,
            value_date: 100_000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 49_989_000, // 50M minus 11,000 fee
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY CP THUONG MAI ABC".to_string()),
            counterparty_bank: None,
            narration: "CONG TY ABC CK TIEN HANG".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: 100_000,
        };

        let ledger_entry = InternalLedgerEntry {
            id: "led_2".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD-00200".to_string(),
            entry_date: 102_000, // within 72h
            entry_type: TransactionType::Credit,
            amount: 50_000_000,
            partner_code: Some("ABC01".to_string()),
            partner_name: Some("Công ty Cổ phần Thương mại ABC".to_string()),
            description: "Ban hang ABC".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let (matches, rem_tx, rem_led) =
            FuzzyMatcher::match_tier2(&[bank_tx], &[ledger_entry], &[0], &[0]);

        assert_eq!(matches.len(), 1);
        assert_eq!(rem_tx.len(), 0);
        assert_eq!(rem_led.len(), 0);
        assert_eq!(matches[0].match_type, MatchType::FuzzyHeuristic);
        assert!(matches[0].confidence_score >= 0.85);
        assert_eq!(matches[0].discrepancy_amount, 11_000); // 11,000 fee discrepancy
    }

    #[test]
    fn test_tier2_rejects_unrelated_companies_sharing_legal_prefix() {
        let bank_tx = BankTransactionRow {
            id: "tx_unrelated".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: 100_000,
            value_date: 100_000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 50_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY TNHH MINH ANH".to_string()),
            counterparty_bank: None,
            narration: "CONG TY TNHH MINH ANH THANH TOAN".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: 100_000,
        };

        let ledger_entry = InternalLedgerEntry {
            id: "led_unrelated".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD-00999".to_string(),
            entry_date: 100_000,
            entry_type: TransactionType::Credit,
            amount: 50_000_000,
            partner_code: Some("PD01".to_string()),
            partner_name: Some("CONG TY TNHH PHUONG DONG".to_string()),
            description: "Ban hang Phuong Dong".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let (matches, rem_tx, rem_led) =
            FuzzyMatcher::match_tier2(&[bank_tx], &[ledger_entry], &[0], &[0]);

        assert_eq!(
            matches.len(),
            0,
            "Unrelated companies sharing only legal prefix must not match"
        );
        assert_eq!(rem_tx.len(), 1);
        assert_eq!(rem_led.len(), 1);
    }

    #[test]
    fn test_tier2_direction_mismatch_rejected() {
        let bank_tx = BankTransactionRow {
            id: "tx_credit_fuzz".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: 100_000,
            value_date: 100_000,
            doc_ref: None,
            tx_type: TransactionType::Credit, // Credit
            amount: 50_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY CP THUONG MAI ABC".to_string()),
            counterparty_bank: None,
            narration: "CONG TY ABC CK TIEN HANG".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: 100_000,
        };

        let ledger_entry = InternalLedgerEntry {
            id: "led_debit_fuzz".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD-00200".to_string(),
            entry_date: 102_000,
            entry_type: TransactionType::Debit, // Debit - Direction mismatch!
            amount: 50_000_000,
            partner_code: Some("ABC01".to_string()),
            partner_name: Some("Công ty Cổ phần Thương mại ABC".to_string()),
            description: "Ban hang ABC".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let (matches, rem_tx, rem_led) =
            FuzzyMatcher::match_tier2(&[bank_tx], &[ledger_entry], &[0], &[0]);

        assert_eq!(
            matches.len(),
            0,
            "Tier 2 MUST NOT match opposite transaction directions"
        );
        assert_eq!(rem_tx.len(), 1);
        assert_eq!(rem_led.len(), 1);
    }

    #[test]
    fn test_tier2_amount_bucket_filtering() {
        let bank_tx = BankTransactionRow {
            id: "tx_target".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: 100_000,
            value_date: 100_000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 50_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY ABC".to_string()),
            counterparty_bank: None,
            narration: "CONG TY ABC".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: 100_000,
        };

        // Ledger entries far outside +-11,000 VND bucket
        let ledger_entries = vec![
            InternalLedgerEntry {
                id: "led_too_low".to_string(),
                account_id: "acc_1".to_string(),
                doc_no: "HD1".to_string(),
                entry_date: 100_000,
                entry_type: TransactionType::Credit,
                amount: 49_000_000, // 1M diff (beyond 11k)
                partner_code: Some("ABC".to_string()),
                partner_name: Some("Công ty ABC".to_string()),
                description: "".to_string(),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: 100_000,
            },
            InternalLedgerEntry {
                id: "led_too_high".to_string(),
                account_id: "acc_1".to_string(),
                doc_no: "HD2".to_string(),
                entry_date: 100_000,
                entry_type: TransactionType::Credit,
                amount: 51_000_000, // 1M diff (beyond 11k)
                partner_code: Some("ABC".to_string()),
                partner_name: Some("Công ty ABC".to_string()),
                description: "".to_string(),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: 100_000,
            },
            InternalLedgerEntry {
                id: "led_exact_bucket".to_string(),
                account_id: "acc_1".to_string(),
                doc_no: "HD3".to_string(),
                entry_date: 100_000,
                entry_type: TransactionType::Credit,
                amount: 50_000_000, // Exact in-bucket match
                partner_code: Some("ABC".to_string()),
                partner_name: Some("Công ty ABC".to_string()),
                description: "".to_string(),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: 100_000,
            },
        ];

        let (matches, rem_tx, rem_led) =
            FuzzyMatcher::match_tier2(&[bank_tx], &ledger_entries, &[0], &[0, 1, 2]);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ledger_entry_ids, vec!["led_exact_bucket"]);
        assert_eq!(rem_tx.len(), 0);
        assert_eq!(rem_led.len(), 2);
    }
}
