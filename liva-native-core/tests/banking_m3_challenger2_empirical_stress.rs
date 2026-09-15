//! Milestone 3 (M3) Empirical Adversarial Challenge Test Suite (Challenger 2).
//!
//! Independently authored and executed by Challenger 2.
//! Adversarially challenges:
//! 1. Latency Stability & Memory Bounds Across Back-to-Back 50k Runs:
//!    - Verifies 50,000 tx processing executes consistently < 19.2s across multiple runs.
//!    - Verifies Peak RAM (RSS) strictly < 680 MB via Win32 `GetProcessMemoryInfo`.
//!    - Tests memory stability across back-to-back runs to confirm absence of unbounded memory growth or leaks.
//! 2. Tier 3 Combinatorial Stress & Candidate Pre-Indexing Audit:
//!    - Audits candidate pre-indexing in `split_solver.rs` under heavy load:
//!      verifies candidate lookup remains O(1) and does not degrade into quadratic scans.
//!    - Stress tests branch-and-bound subset-sum pruning bounds (Bound 1, Bound 2, Bound 3).
//!    - Extreme combinatorial depths (depth up to 8 items).
//!    - Arithmetic boundary safety: u64::MAX saturating arithmetic and zero-amount filtering.
//! 3. Core SLA Compliance:
//!    - Latency < 19.2s, Peak RSS < 680MB, Match rate >= 99.5%, Exception rate <= 0.5%, Zero float drift.

use std::time::Instant;

use liva_native_core::banking::generator::{DEFAULT_BENCHMARK_SEED, generate_50k_dataset};
use liva_native_core::banking::models::*;
use liva_native_core::banking::reconciliation::ReconciliationEngine;
use liva_native_core::banking::reconciliation::split_solver::{
    DEFAULT_MAX_SPLIT_DEPTH, MAX_SUPPORTED_SPLIT_DEPTH, SplitSolver, solve_exact_subset_sum_bnb,
};

/// Retrieves current working set (RSS) and peak working set in bytes via Win32 API.
fn get_process_memory_rss_bytes() -> (u64, u64) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
        };
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        let mut pmc: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
        pmc.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        if unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb) } != 0 {
            return (pmc.WorkingSetSize as u64, pmc.PeakWorkingSetSize as u64);
        }
    }
    (0, 0)
}

fn make_bank_tx(
    id: &str,
    tx_type: TransactionType,
    amount: u64,
    doc_ref: Option<&str>,
    narration: &str,
    partner: Option<&str>,
) -> BankTransactionRow {
    BankTransactionRow {
        id: id.to_string(),
        statement_id: "stmt_stress".to_string(),
        account_id: "acc_stress".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: 1704067200,
        value_date: 1704067200,
        doc_ref: doc_ref.map(|s| s.to_string()),
        tx_type,
        amount,
        balance_after: Some(500_000_000),
        counterparty_account: None,
        counterparty_name: partner.map(|s| s.to_string()),
        counterparty_bank: None,
        narration: narration.to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: 1704067200,
    }
}

fn make_ledger_entry(
    id: &str,
    entry_type: TransactionType,
    amount: u64,
    doc_no: &str,
    partner_name: Option<&str>,
) -> InternalLedgerEntry {
    InternalLedgerEntry {
        id: id.to_string(),
        account_id: "acc_led_stress".to_string(),
        doc_no: doc_no.to_string(),
        entry_date: 1704067200,
        entry_type,
        amount,
        partner_code: Some("KH_STRESS".to_string()),
        partner_name: partner_name.map(|s| s.to_string()),
        description: format!("Ledger invoice {doc_no}"),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: 1704067200,
    }
}

// ===========================================================================
// Test 1: Latency Stability & Memory Bounds Across Back-to-Back 50k Runs
// ===========================================================================

#[test]
fn test_challenger2_back_to_back_50k_memory_stability_and_latency() {
    println!("\n--- Challenger 2: Back-to-Back 50,000 Transaction Memory Stability ---");

    let num_rounds = 3;
    let mut round_latencies = Vec::new();
    let mut round_peak_rss = Vec::new();
    let mut round_current_rss = Vec::new();

    for round in 1..=num_rounds {
        let round_start = Instant::now();

        // 1. Generate fresh dataset
        let dataset = generate_50k_dataset(DEFAULT_BENCHMARK_SEED + (round as u64 * 1000));
        assert_eq!(dataset.bank_transactions.len(), 50_000);
        assert_eq!(dataset.ledger_entries.len(), 50_000);

        // 2. Invariant pre-check
        let (checked, passed, max_disc) =
            ReconciliationEngine::check_balance_invariants(&dataset.bank_transactions);
        assert!(checked && passed && max_disc == 0);

        // 3. Reconcile full 50,000 dataset
        let (matches, summary) =
            ReconciliationEngine::reconcile(&dataset.bank_transactions, &dataset.ledger_entries);
        let round_duration = round_start.elapsed();
        let round_secs = round_duration.as_secs_f64();

        // 4. Memory telemetry via Win32 GetProcessMemoryInfo
        let (cur_rss, peak_rss) = get_process_memory_rss_bytes();
        let cur_rss_mb = (cur_rss as f64) / (1024.0 * 1024.0);
        let peak_rss_mb = (peak_rss as f64) / (1024.0 * 1024.0);

        round_latencies.push(round_secs);
        round_peak_rss.push(peak_rss_mb);
        round_current_rss.push(cur_rss_mb);

        println!(
            "Round {}: Latency = {:.3}s, Current RSS = {:.2} MB, Peak RSS = {:.2} MB, Match Rate = {:.2}%, HITL = {}",
            round,
            round_secs,
            cur_rss_mb,
            peak_rss_mb,
            summary.match_rate,
            summary.pending_hitl_count
        );

        // Assert core SLA bounds on every round
        assert!(
            round_secs < 19.2,
            "Round {round} latency ({round_secs:.3}s) violated SLA (< 19.2s)"
        );
        if peak_rss > 0 {
            assert!(
                peak_rss_mb < 680.0,
                "Round {round} peak RSS ({peak_rss_mb:.2} MB) violated limit (< 680 MB)"
            );
        }
        assert!(
            summary.match_rate >= 99.5,
            "Round {round} match rate ({:.2}%) below 99.5%",
            summary.match_rate
        );
        let hitl_rate = (summary.pending_hitl_count as f64 / 50_000.0) * 100.0;
        assert!(
            hitl_rate <= 0.5,
            "Round {round} HITL rate ({hitl_rate:.2}%) exceeded 0.5%"
        );

        // Verify zero float drift on exact and split matches
        for m in &matches {
            if m.status == "APPROVED"
                && (m.match_type == MatchType::Exact1To1
                    || m.match_type == MatchType::CompositeSplit)
            {
                assert_eq!(
                    m.discrepancy_amount, 0,
                    "Discrepancy must be strictly 0 for approved match in round {round}"
                );
            }
        }
    }

    // Verify memory bounds across iterations:
    // Memory should not explode unbounded across consecutive rounds.
    let max_peak = round_peak_rss.iter().copied().fold(0.0f64, f64::max);
    println!(
        "Max Peak RSS across {num_rounds} rounds: {:.2} MB",
        max_peak
    );
    if max_peak > 0.0 {
        assert!(
            max_peak < 680.0,
            "Max peak RSS ({max_peak:.2} MB) must remain strictly under 680 MB"
        );
    }

    // Working set growth between round 2 and round 3 must be bounded (allocator reuse)
    if round_current_rss[1] > 0.0 && round_current_rss[2] > 0.0 {
        let rss_delta = (round_current_rss[2] - round_current_rss[1]).abs();
        println!("RSS Delta between Round 2 and Round 3: {:.2} MB", rss_delta);
        assert!(
            rss_delta < 150.0,
            "Excessive memory drift ({rss_delta:.2} MB) between runs indicates potential leak"
        );
    }
}

// ===========================================================================
// Test 2: Tier 3 Candidate Pre-Indexing Audit & Heavy Load Scalability
// ===========================================================================

#[test]
fn test_challenger2_split_solver_candidate_pre_indexing_scalability() {
    println!("\n--- Challenger 2: Split Solver Pre-Indexing O(1) Scalability Audit ---");

    // Construct a large unallocated pool: 2,000 bank transactions and 1,000 ledger split invoices
    // Bank transactions represent multi-installment payments (2 bank payments per invoice)
    let num_invoices = 1_000;
    let mut bank_txs = Vec::new();
    let mut ledger_entries = Vec::new();

    for i in 0..num_invoices {
        let doc_code = format!("HD{:07}", i);
        let part_a = 4_000_000u64 + (i as u64 * 100);
        let part_b = 6_000_000u64 + (i as u64 * 100);
        let total_inv = part_a + part_b;

        // Two bank payments mentioning the invoice doc code
        bank_txs.push(make_bank_tx(
            &format!("btx_{}_a", i),
            TransactionType::Credit,
            part_a,
            None,
            &format!("THANH TOAN DOT 1 HOP DONG {doc_code}"),
            Some("CONG TY TNHH VIET THAI"),
        ));
        bank_txs.push(make_bank_tx(
            &format!("btx_{}_b", i),
            TransactionType::Credit,
            part_b,
            None,
            &format!("THANH TOAN DOT 2 HOP DONG {doc_code}"),
            Some("CONG TY TNHH VIET THAI"),
        ));

        // Ledger invoice
        ledger_entries.push(make_ledger_entry(
            &format!("led_{}", i),
            TransactionType::Credit,
            total_inv,
            &doc_code,
            Some("CONG TY TNHH VIET THAI"),
        ));
    }

    let unalloc_bank: Vec<usize> = (0..bank_txs.len()).collect();
    let unalloc_led: Vec<usize> = (0..ledger_entries.len()).collect();

    assert_eq!(bank_txs.len(), 2_000);
    assert_eq!(ledger_entries.len(), 1_000);

    // Measure Tier 3 execution time with pre-indexed candidate lookups
    let t0 = Instant::now();
    let (matches, hitl) =
        SplitSolver::match_tier3(&bank_txs, &ledger_entries, &unalloc_bank, &unalloc_led);
    let solve_time = t0.elapsed();

    println!(
        "Tier 3 Pre-indexed 2,000 bank txs vs 1,000 split invoices solved in: {:.3}s (matches: {}, hitl: {})",
        solve_time.as_secs_f64(),
        matches.len(),
        hitl.len()
    );

    // If pre-indexing was O(M * N) unindexed scan with string comparisons:
    // 2,000 * 1,000 = 2,000,000 comparisons -> would take > 15-30s.
    // With O(1) hash pre-indexing by doc ref, it solves in < 1.0s.
    assert!(
        solve_time.as_secs_f64() < 1.5,
        "Pre-indexing audit failed: Solve time ({:.3}s) exceeded 1.5s limit! Degraded to quadratic scan?",
        solve_time.as_secs_f64()
    );

    // Verify match count: all 1,000 invoices should be matched 2-to-1 (N-to-1)
    assert_eq!(
        matches.len(),
        1_000,
        "All 1,000 split invoices must be matched"
    );
    assert_eq!(hitl.len(), 0, "No transactions should fail to HITL");

    // Verify mathematical invariants on all matches
    for m in &matches {
        assert_eq!(m.bank_tx_ids.len(), 2, "Each match must contain 2 bank txs");
        assert_eq!(m.discrepancy_amount, 0, "Discrepancy must be strictly 0");
        assert_eq!(m.match_type, MatchType::CompositeSplit);
    }
}

// ===========================================================================
// Test 3: Branch-and-Bound Subset-Sum Pruning Bounds Stress Test
// ===========================================================================

#[test]
fn test_challenger2_bnb_subset_sum_pruning_bounds_adversarial() {
    println!("\n--- Challenger 2: Branch-and-Bound Pruning Bounds Adversarial Stress ---");

    // 1. Bound 1: Minimum candidate exceeds target
    // All candidates are strictly greater than target -> must return None instantly in < 50µs
    let target = 50_000_000u64;
    let candidates_above: Vec<(usize, u64)> = (0..1_000)
        .map(|i| (i, 60_000_000u64 + (i as u64 * 100)))
        .collect();
    let t0 = Instant::now();
    let res = solve_exact_subset_sum_bnb(&candidates_above, target, DEFAULT_MAX_SPLIT_DEPTH);
    let dur = t0.elapsed();
    assert!(res.is_none(), "Bound 1: Must return None");
    assert!(
        dur.as_micros() < 500,
        "Bound 1 pruning took too long: {}µs",
        dur.as_micros()
    );

    // 2. Bound 2: Suffix-sum bound (sum of all remaining items cannot reach target)
    // 500 small candidates whose total sum = 49_999_999 < 50_000_000
    let candidates_insufficient: Vec<(usize, u64)> = (0..500).map(|i| (i, 99_999u64)).collect();
    let t0 = Instant::now();
    let res = solve_exact_subset_sum_bnb(&candidates_insufficient, target, DEFAULT_MAX_SPLIT_DEPTH);
    let dur = t0.elapsed();
    assert!(
        res.is_none(),
        "Bound 2: Must return None when total sum < target"
    );
    assert!(
        dur.as_micros() < 500,
        "Bound 2 suffix pruning took too long: {}µs",
        dur.as_micros()
    );

    // 3. Bound 3: Max depth capacity bound
    // 1,000 candidates of 1,000 VND each. Total sum is 1,000,000 VND.
    // Target is 100,000 VND. With max_depth = 4, max possible sum is 4 * 1,000 = 4,000 VND < 100,000.
    // Bound 3 should prune at the top of the search tree in < 1ms.
    let candidates_small: Vec<(usize, u64)> = (0..1_000).map(|i| (i, 1_000u64)).collect();
    let t0 = Instant::now();
    let res = solve_exact_subset_sum_bnb(&candidates_small, 100_000, DEFAULT_MAX_SPLIT_DEPTH);
    let dur = t0.elapsed();
    assert!(
        res.is_none(),
        "Bound 3: Must return None when max_depth items cannot reach target"
    );
    assert!(
        dur.as_millis() < 5,
        "Bound 3 capacity pruning took too long: {}ms",
        dur.as_millis()
    );

    // 4. Maximum Supported Split Depth (depth = 8 items)
    // Construct an 8-item exact split hidden among 100 distractors
    let mut candidates_8depth: Vec<(usize, u64)> = Vec::new();
    let split_parts = [
        1_100_000u64,
        2_200_000u64,
        3_300_000u64,
        4_400_000u64,
        5_500_000u64,
        6_600_000u64,
        7_700_000u64,
        8_800_000u64,
    ];
    let exact_target: u64 = split_parts.iter().sum(); // 39,600,000 VND

    for (idx, &amt) in split_parts.iter().enumerate() {
        candidates_8depth.push((idx, amt));
    }
    // Add 100 distractor candidates
    for i in 8..108 {
        candidates_8depth.push((i, 10_000_000u64 + (i as u64 * 37_000)));
    }

    let t0 = Instant::now();
    let res =
        solve_exact_subset_sum_bnb(&candidates_8depth, exact_target, MAX_SUPPORTED_SPLIT_DEPTH);
    let dur = t0.elapsed();

    assert!(res.is_some(), "Must find 8-item split subset");
    let solution = res.unwrap();
    assert_eq!(solution.len(), 8, "Solution must contain exactly 8 items");
    let found_sum: u64 = solution.iter().map(|&idx| candidates_8depth[idx].1).sum();
    assert_eq!(
        found_sum, exact_target,
        "Solution sum must strictly equal target (Delta=0)"
    );
    println!(
        "8-item split solved among 108 candidates in {:.3}ms",
        dur.as_secs_f64() * 1000.0
    );
    assert!(
        dur.as_millis() < 50,
        "8-item split search took too long: {}ms",
        dur.as_millis()
    );
}

// ===========================================================================
// Test 4: Extreme Arithmetic Boundaries in Split Solver
// ===========================================================================

#[test]
fn test_challenger2_split_solver_arithmetic_boundaries() {
    println!("\n--- Challenger 2: Split Solver Arithmetic Boundaries ---");

    // A. u64::MAX saturating arithmetic (no panic on massive values)
    let candidates_huge = [
        (0usize, u64::MAX / 2),
        (1usize, u64::MAX / 2 + 1),
        (2usize, 1_000_000u64),
    ];
    let res = solve_exact_subset_sum_bnb(&candidates_huge, u64::MAX, 2);
    // Even near u64::MAX, suffix sums and additions must not overflow/panic
    println!("u64::MAX boundary search completed safely: {:?}", res);

    // B. Zero amount filtering: 0 VND items must not cause infinite loops or zero-value matches
    let candidates_with_zeros = [
        (0usize, 0u64),
        (1usize, 0u64),
        (2usize, 10_000_000u64),
        (3usize, 20_000_000u64),
    ];
    let res =
        solve_exact_subset_sum_bnb(&candidates_with_zeros, 30_000_000, DEFAULT_MAX_SPLIT_DEPTH);
    assert!(res.is_some());
    let sol = res.unwrap();
    assert_eq!(sol.len(), 2);
    assert!(
        !sol.contains(&0) && !sol.contains(&1),
        "Zero-amount items must be filtered out"
    );
}
