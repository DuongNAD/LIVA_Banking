//! Tier 1: O(1) Hash Matcher.
//!
//! Matches transactions where:
//! - Scaled amount matches exactly (`u64 == u64`).
//! - Document reference / invoice code matches exactly (`normalize(doc_ref) == normalize(doc_no)`).
//! - Time window $|t_{bank} - t_{ledger}| \le 24$ hours (86,400 seconds).

use std::collections::HashMap;

use crate::banking::models::{
    BankTransactionRow, InternalLedgerEntry, MatchType, ReconciliationMatch,
};

/// Normalizes a document reference for exact hash comparison (e.g. "HD-00102" -> "HD102").
pub fn normalize_doc_ref(s: &str) -> String {
    let clean: String = s
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_uppercase();

    // Strip leading zeros after prefix (e.g. HD000102 -> HD102)
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

pub struct HashMatcher;

impl HashMatcher {
    /// Executes Tier 1 matching on unallocated transactions and ledger entries.
    /// Returns: (matched results, remaining unallocated bank tx indices, remaining unallocated ledger indices).
    pub fn match_tier1(
        bank_txs: &[BankTransactionRow],
        ledger_entries: &[InternalLedgerEntry],
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<usize>) {
        let mut matches = Vec::new();

        // 1. Build O(1) Hash Map index for ledger entries: (amount, entry_type, normalized_doc_ref) -> Vec<ledger_idx>
        // Enforces strict direction invariance: Debit only matches Debit, Credit only matches Credit.
        let mut ledger_index: HashMap<
            (u64, crate::banking::models::TransactionType, String),
            Vec<usize>,
        > = HashMap::new();
        for (idx, entry) in ledger_entries.iter().enumerate() {
            let norm_ref = normalize_doc_ref(&entry.doc_no);
            if !norm_ref.is_empty() {
                ledger_index
                    .entry((entry.amount, entry.entry_type, norm_ref))
                    .or_default()
                    .push(idx);
            }
        }

        let mut matched_bank_txs = vec![false; bank_txs.len()];
        let mut matched_ledger_entries = vec![false; ledger_entries.len()];

        const WINDOW_24H_SECS: i64 = 86_400;

        // 2. O(1) Probe for each bank transaction
        for (b_idx, tx) in bank_txs.iter().enumerate() {
            // Find possible doc ref candidates from tx.doc_ref or tx.narration
            let mut candidates = Vec::new();
            if let Some(ref r) = tx.doc_ref {
                candidates.push(normalize_doc_ref(r));
            }

            // Extract invoice & banking trace tokens (HD, INV, PC, PT, FT, NPS, VN) from narration
            // Preserves '-' and '/' so hyphenated and slashed references (e.g. HD-2026-001, INV-2026-001, PC-001, HD102/2026)
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
                    || (clean.starts_with("FT")
                        && clean.len() >= 6
                        && clean.chars().skip(2).any(|c| c.is_ascii_digit()))
                    || (clean.starts_with("NPS") && clean.len() >= 5)
                    || (clean.starts_with("VN")
                        && clean.len() >= 6
                        && clean.chars().skip(2).any(|c| c.is_ascii_digit()))
                {
                    candidates.push(normalize_doc_ref(&clean));
                }
            }

            candidates.sort();
            candidates.dedup();

            for cand_ref in candidates {
                let key = (tx.amount, tx.tx_type, cand_ref);
                if let Some(ledger_indices) = ledger_index.get(&key) {
                    for &l_idx in ledger_indices {
                        if matched_ledger_entries[l_idx] {
                            continue;
                        }
                        let ledger_entry = &ledger_entries[l_idx];

                        // Direction Invariance Check
                        if tx.tx_type != ledger_entry.entry_type {
                            continue;
                        }

                        // Value Date Check: |t_eff - t_ledger| <= 24h
                        // Uses value_date if present (> 0), falling back to tx_date.
                        let eff_date = tx.effective_date();
                        let time_diff = (eff_date - ledger_entry.entry_date).abs();
                        if time_diff <= WINDOW_24H_SECS {
                            matched_bank_txs[b_idx] = true;
                            matched_ledger_entries[l_idx] = true;

                            matches.push(ReconciliationMatch {
                                id: format!("match_{}_{}", tx.id, ledger_entry.id),
                                bank_tx_id: tx.id.clone(),
                                bank_tx_ids: vec![tx.id.clone()],
                                ledger_entry_ids: vec![ledger_entry.id.clone()],
                                match_type: MatchType::Exact1To1,
                                confidence_score: 1.0,
                                matched_amount: tx.amount,
                                discrepancy_amount: 0,
                                status: "APPROVED".to_string(),
                                matched_by: "ENGINE_AUTOMATIC".to_string(),
                                matched_at: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64,
                                notes: Some(format!(
                                    "Tier 1 Exact Hash Match: Amount={} Ref={}",
                                    tx.amount, ledger_entry.doc_no
                                )),
                                hitl_token: None,
                            });
                            break;
                        }
                    }
                    if matched_bank_txs[b_idx] {
                        break;
                    }
                }
            }
        }

        let remaining_bank_indices: Vec<usize> = (0..bank_txs.len())
            .filter(|&i| !matched_bank_txs[i])
            .collect();
        let remaining_ledger_indices: Vec<usize> = (0..ledger_entries.len())
            .filter(|&i| !matched_ledger_entries[i])
            .collect();

        (matches, remaining_bank_indices, remaining_ledger_indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banking::models::{ReconciliationStatus, TransactionType};

    #[test]
    fn test_tier1_exact_hash_match() {
        let bank_tx = BankTransactionRow {
            id: "tx_1".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: 100_000,
            value_date: 100_000,
            doc_ref: Some("HD-00102".to_string()),
            tx_type: TransactionType::Credit,
            amount: 25_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Thanh toan HD102".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: 100_000,
        };

        let ledger_entry = InternalLedgerEntry {
            id: "led_1".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD102".to_string(),
            entry_date: 100_500, // within 24h
            entry_type: TransactionType::Credit,
            amount: 25_000_000,
            partner_code: Some("CUST01".to_string()),
            partner_name: Some("ABC Corp".to_string()),
            description: "Hoa don 102".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let (matches, rem_tx, rem_led) = HashMatcher::match_tier1(&[bank_tx], &[ledger_entry]);

        assert_eq!(matches.len(), 1);
        assert_eq!(rem_tx.len(), 0);
        assert_eq!(rem_led.len(), 0);
        assert_eq!(matches[0].match_type, MatchType::Exact1To1);
        assert_eq!(matches[0].confidence_score, 1.0);
        assert_eq!(matches[0].discrepancy_amount, 0);
    }

    #[test]
    fn test_tier1_direction_mismatch_rejected() {
        let bank_tx = BankTransactionRow {
            id: "tx_credit".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: 100_000,
            value_date: 100_000,
            doc_ref: Some("HD-102".to_string()),
            tx_type: TransactionType::Credit, // Credit inflow
            amount: 25_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Thanh toan HD102".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: 100_000,
        };

        let ledger_entry = InternalLedgerEntry {
            id: "led_debit".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "HD102".to_string(),
            entry_date: 100_500,
            entry_type: TransactionType::Debit, // Debit outflow - Direction Mismatch!
            amount: 25_000_000,
            partner_code: Some("CUST01".to_string()),
            partner_name: Some("ABC Corp".to_string()),
            description: "Hoa don 102".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: 100_000,
        };

        let (matches, rem_tx, rem_led) = HashMatcher::match_tier1(&[bank_tx], &[ledger_entry]);

        assert_eq!(matches.len(), 0, "Direction mismatch MUST NOT match");
        assert_eq!(rem_tx.len(), 1);
        assert_eq!(rem_led.len(), 1);
    }

    #[test]
    fn test_tier1_value_date_24h_window_fallback() {
        let friday_pm = 1_725_000_000i64; // Friday 17:00
        let monday_am = friday_pm + (72 * 3600); // Monday 17:00 (+72h from tx_date)

        let bank_tx = BankTransactionRow {
            id: "tx_weekend".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: friday_pm,
            value_date: monday_am, // value_date is Monday!
            doc_ref: Some("INV-999".to_string()),
            tx_type: TransactionType::Credit,
            amount: 15_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Thanh toan INV-999".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: friday_pm,
        };

        // Ledger entry posted on Monday 19:00 (2h after value_date, but 74h after tx_date)
        let ledger_entry = InternalLedgerEntry {
            id: "led_monday".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "INV999".to_string(),
            entry_date: monday_am + 7200, // within 2h of value_date!
            entry_type: TransactionType::Credit,
            amount: 15_000_000,
            partner_code: Some("SUPP01".to_string()),
            partner_name: Some("Supplier XYZ".to_string()),
            description: "Mua vat tu".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: monday_am,
        };

        let (matches, rem_tx, rem_led) = HashMatcher::match_tier1(&[bank_tx], &[ledger_entry]);

        assert_eq!(matches.len(), 1, "Must match using effective value_date");
        assert_eq!(rem_tx.len(), 0);
        assert_eq!(rem_led.len(), 0);
    }

    #[test]
    fn test_tier1_napas_trace_and_ft_extraction() {
        let now = 1_725_000_000i64;

        let bank_tx = BankTransactionRow {
            id: "tx_ft".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 30_000_000,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "Napas VietQR TT FT262568912345 Tu: CONG TY ABC".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let ledger_entry = InternalLedgerEntry {
            id: "led_ft".to_string(),
            account_id: "acc_1".to_string(),
            doc_no: "FT262568912345".to_string(),
            entry_date: now + 1200,
            entry_type: TransactionType::Credit,
            amount: 30_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Cong ty ABC".to_string()),
            description: "Thanh toan don hang".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };

        let (matches, rem_tx, rem_led) = HashMatcher::match_tier1(&[bank_tx], &[ledger_entry]);

        assert_eq!(matches.len(), 1, "Must extract and match FT number");
        assert_eq!(rem_tx.len(), 0);
        assert_eq!(rem_led.len(), 0);
    }

    #[test]
    fn test_tier1_hyphenated_and_slashed_doc_refs() {
        let now = 1_725_000_000i64;

        // Test cases: HD-2026-001, INV-2026-001, PC-001, HD102/2026
        let test_cases = [
            (
                "tx_hd",
                "THANH TOAN TIEN HANG HD-2026-001",
                "HD-2026-001",
                10_000_000u64,
            ),
            (
                "tx_inv",
                "CHUYEN KHOAN HOA DON INV-2026-001",
                "INV-2026-001",
                20_000_000u64,
            ),
            ("tx_pc", "CHI PHI TIEN MAT PC-001", "PC-001", 5_000_000u64),
            (
                "tx_slash",
                "THANH TOAN HOP DONG HD102/2026",
                "HD102/2026",
                15_000_000u64,
            ),
        ];

        for (id, narration, doc_no, amount) in test_cases {
            let bank_tx = BankTransactionRow {
                id: id.to_string(),
                statement_id: "stmt_1".to_string(),
                account_id: "acc_1".to_string(),
                bank_code: "VCB".to_string(),
                tx_date: now,
                value_date: now,
                doc_ref: None,
                tx_type: TransactionType::Credit,
                amount,
                balance_after: None,
                counterparty_account: None,
                counterparty_name: None,
                counterparty_bank: None,
                narration: narration.to_string(),
                reconciled_status: ReconciliationStatus::Unmatched,
                reconciled_match_id: None,
                created_at: now,
            };

            let ledger_entry = InternalLedgerEntry {
                id: format!("led_{id}"),
                account_id: "acc_1".to_string(),
                doc_no: doc_no.to_string(),
                entry_date: now,
                entry_type: TransactionType::Credit,
                amount,
                partner_code: Some("PARTNER".to_string()),
                partner_name: Some("Partner Company".to_string()),
                description: format!("Ledger for {doc_no}"),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: now,
            };

            let (matches, rem_tx, rem_led) = HashMatcher::match_tier1(&[bank_tx], &[ledger_entry]);

            assert_eq!(
                matches.len(),
                1,
                "Failed to match hyphenated/slashed doc ref: narration='{narration}', doc_no='{doc_no}'"
            );
            assert_eq!(rem_tx.len(), 0);
            assert_eq!(rem_led.len(), 0);
        }
    }
}
