//! Zero-Hallucination Deterministic Reconciliation Matching Engine.
//!
//! 3-Tier Hierarchy:
//! - Tier 1: O(1) Hash Matcher (Exact amount, document reference, 24h window).
//! - Tier 2: Fuzzy Heuristic Matcher (Party Jaro-Winkler >= 0.85, fee deduction tolerance, 72h window).
//! - Tier 3: Constraint Split Solver (Exact integer subset sum Delta=0).
//! - Fail-Closed: Residual items routed into 0.2% HITL review queue with Two-Phase Confirmation UUID token.

pub mod fuzzy_matcher;
pub mod hash_matcher;
pub mod jaro_winkler;
pub mod split_solver;

use std::collections::HashMap;

use crate::banking::models::{
    BalanceInvariantReport, BankStatement, BankTransactionRow, InternalLedgerEntry,
    ReconciliationMatch, ReconciliationSummaryDto, TransactionRecord, TransactionType,
};
use fuzzy_matcher::FuzzyMatcher;
use hash_matcher::HashMatcher;
use split_solver::SplitSolver;

pub struct ReconciliationEngine;

impl ReconciliationEngine {
    /// Evaluates the double-entry balance invariant Closing = Opening + Credit - Debit
    /// on a bank statement.
    pub fn verify_statement_balance(statement: &BankStatement) -> BalanceInvariantReport {
        statement.verify_balance_invariants()
    }

    /// Evaluates balance invariants on a transaction list with opening and closing balances.
    pub fn verify_balance_invariants(
        opening_balance: Option<u64>,
        closing_balance: Option<u64>,
        transactions: &[TransactionRecord],
    ) -> BalanceInvariantReport {
        BalanceInvariantReport::verify(opening_balance, closing_balance, transactions)
    }

    /// Checks consecutive running balance transitions for bank transactions grouped by account.
    pub fn check_balance_invariants(bank_txs: &[BankTransactionRow]) -> (bool, bool, i64) {
        let mut accounts: HashMap<(&str, &str), Vec<&BankTransactionRow>> = HashMap::new();
        for tx in bank_txs {
            accounts
                .entry((&tx.statement_id, &tx.account_id))
                .or_default()
                .push(tx);
        }

        let mut total_checked = 0;
        let mut max_discrepancy: i64 = 0;
        let mut all_passed = true;

        for ((_stmt, _acc), mut txs) in accounts {
            txs.sort_by_key(|t| t.tx_date);
            let rows_with_balance: Vec<&&BankTransactionRow> =
                txs.iter().filter(|t| t.balance_after.is_some()).collect();

            if rows_with_balance.len() >= 2 {
                total_checked += 1;
                for pair in rows_with_balance.windows(2) {
                    let prev = pair[0];
                    let curr = pair[1];
                    let prev_bal = prev.balance_after.unwrap();
                    let curr_bal = curr.balance_after.unwrap();
                    let expected = match curr.tx_type {
                        TransactionType::Credit => (prev_bal as i128) + (curr.amount as i128),
                        TransactionType::Debit => (prev_bal as i128) - (curr.amount as i128),
                    };
                    let diff = (curr_bal as i128) - expected;
                    if diff != 0 {
                        all_passed = false;
                        if diff.abs() > (max_discrepancy as i128).abs() {
                            max_discrepancy = diff as i64;
                        }
                    }
                }
            }
        }

        if total_checked > 0 {
            (true, all_passed, max_discrepancy)
        } else {
            (false, true, 0)
        }
    }

    /// Runs the complete 3-Tier deterministic reconciliation process.
    pub fn reconcile(
        bank_txs: &[BankTransactionRow],
        ledger_entries: &[InternalLedgerEntry],
    ) -> (Vec<ReconciliationMatch>, ReconciliationSummaryDto) {
        let start_time = std::time::Instant::now();

        if bank_txs.is_empty() {
            return (
                Vec::new(),
                ReconciliationSummaryDto {
                    total_bank_transactions: 0,
                    matched_exact_count: 0,
                    matched_fuzzy_count: 0,
                    matched_split_count: 0,
                    total_matched_count: 0,
                    pending_hitl_count: 0,
                    discrepancy_count: 0,
                    match_rate: 100.0,
                    execution_duration_ms: 0,
                    balance_invariant_checked: false,
                    balance_invariant_passed: true,
                    balance_discrepancy_amount: 0,
                },
            );
        }

        // --- Double-Entry Balance Invariant Pre-check ---
        let (inv_checked, inv_passed, inv_discrepancy) = Self::check_balance_invariants(bank_txs);

        // --- Tier 1: O(1) Exact Hash Match ---
        let t1_start = std::time::Instant::now();
        let (tier1_matches, unalloc_bank_t1, unalloc_ledger_t1) =
            HashMatcher::match_tier1(bank_txs, ledger_entries);
        let count_tier1 = tier1_matches.len();
        eprintln!(
            "RECONCILE T1: {:.3}s (matches: {count_tier1}, unalloc_bank: {}, unalloc_led: {})",
            t1_start.elapsed().as_secs_f64(),
            unalloc_bank_t1.len(),
            unalloc_ledger_t1.len()
        );

        // --- Tier 2: Fuzzy Heuristic Match ---
        let t2_start = std::time::Instant::now();
        let (tier2_matches, unalloc_bank_t2, unalloc_ledger_t2) = FuzzyMatcher::match_tier2(
            bank_txs,
            ledger_entries,
            &unalloc_bank_t1,
            &unalloc_ledger_t1,
        );
        let count_tier2 = tier2_matches.len();
        eprintln!(
            "RECONCILE T2: {:.3}s (matches: {count_tier2}, unalloc_bank: {}, unalloc_led: {})",
            t2_start.elapsed().as_secs_f64(),
            unalloc_bank_t2.len(),
            unalloc_ledger_t2.len()
        );

        // --- Tier 3: Constraint Split Solver & Fail-Closed HITL ---
        let t3_start = std::time::Instant::now();
        let (tier3_matches, hitl_matches) = SplitSolver::match_tier3(
            bank_txs,
            ledger_entries,
            &unalloc_bank_t2,
            &unalloc_ledger_t2,
        );
        let count_tier3: usize = tier3_matches
            .iter()
            .map(|m| m.bank_tx_ids.len().max(1))
            .sum();
        let count_hitl = hitl_matches.len();
        eprintln!(
            "RECONCILE T3: {:.3}s (split txs: {count_tier3}, matches: {}, hitl: {count_hitl})",
            t3_start.elapsed().as_secs_f64(),
            tier3_matches.len()
        );

        let mut all_matches =
            Vec::with_capacity(count_tier1 + count_tier2 + tier3_matches.len() + count_hitl);
        all_matches.extend(tier1_matches);
        all_matches.extend(tier2_matches);
        all_matches.extend(tier3_matches);
        all_matches.extend(hitl_matches);

        let total_matched = count_tier1 + count_tier2 + count_tier3;
        let mut discrepancy_count = 0;
        for m in &all_matches {
            if m.discrepancy_amount != 0 {
                discrepancy_count += 1;
            }
        }
        if inv_checked && !inv_passed {
            discrepancy_count += 1;
        }

        let total_tx = bank_txs.len();
        let match_rate = if total_tx > 0 {
            (total_matched as f64 / total_tx as f64) * 100.0
        } else {
            100.0
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        let summary = ReconciliationSummaryDto {
            total_bank_transactions: total_tx,
            matched_exact_count: count_tier1,
            matched_fuzzy_count: count_tier2,
            matched_split_count: count_tier3,
            total_matched_count: total_matched,
            pending_hitl_count: count_hitl,
            discrepancy_count,
            match_rate,
            execution_duration_ms: duration_ms,
            balance_invariant_checked: inv_checked,
            balance_invariant_passed: inv_passed,
            balance_discrepancy_amount: inv_discrepancy,
        };

        (all_matches, summary)
    }
}
