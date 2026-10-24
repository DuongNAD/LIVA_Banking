//! Branch-and-Bound Subset-Sum Solver with Suffix Pruning
//!
//! Solves composite payments in banking reconciliation:
//! - 1:N: 1 bank transaction settling $N$ invoices ($2 \le N \le 8$).
//! - N:1: $N$ bank transactions settling 1 invoice ($2 \le N \le 8$).
//!
//! Invariants:
//! - Strict mathematical balance conservation: $\Delta \equiv \sum \text{Allocated} - \text{Target} = 0$.
//! - Pure integer checked arithmetic (`i64`/`i128`).
//! - Zero floating-point arithmetic (`#![deny(clippy::float_arithmetic)]`).
//! - Exponential subtree pruning using precomputed descending suffix sums:
//!   If $\text{current\_sum} + \text{suffix\_sum}[i] < \text{target} \implies \text{ABORT}$.
//! - Resource bounds: max depth $k \le 8$, max iterations ($100,000$), timeout budget ($100\text{ ms}$).

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Configuration for the Subset-Sum solver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsetSolverConfig {
    /// Maximum number of items in a subset ($2 \le k \le 8$). Default is 8.
    pub max_depth: usize,
    /// Maximum recursive exploration iterations before halting (default 100,000).
    pub max_iterations: usize,
    /// Timeout limit in milliseconds (default 100ms).
    pub timeout_ms: u64,
    /// Requires exact zero difference between sum of subset and target.
    pub require_zero_delta: bool,
    /// Base confidence score in basis points (default 9500 = 95.00%).
    pub base_confidence_bps: u32,
    /// Confidence penalty per additional item in basis points (default 200 = 2.00%).
    pub item_penalty_bps: u32,
}

impl Default for SubsetSolverConfig {
    fn default() -> Self {
        Self {
            max_depth: 8,
            max_iterations: 100_000,
            timeout_ms: 100,
            require_zero_delta: true,
            base_confidence_bps: 9_500,
            item_penalty_bps: 200,
        }
    }
}

/// A candidate element participating in subset sum resolution.
#[derive(Debug, Clone)]
pub struct SubsetCandidate {
    pub id: String,
    pub amount_vnd: i64,
    pub original_index: usize,
}

/// Solution result from the branch-and-bound solver.
#[derive(Debug, Clone)]
pub struct SubsetSolution {
    pub candidate_ids: Vec<String>,
    pub candidate_indices: Vec<usize>,
    pub total_amount: i64,
    pub iterations_used: usize,
    pub timed_out: bool,
}

struct SearchState {
    iterations: usize,
    timed_out: bool,
    start_time: Instant,
    found_solution: Option<(Vec<String>, Vec<usize>)>,
}

/// Solves the subset-sum problem: finds a subset of at least 2 candidates
/// whose sum equals `target` exactly ($\Delta = 0$).
pub fn solve_subset_sum(
    target: i64,
    candidates: &[SubsetCandidate],
    config: &SubsetSolverConfig,
) -> Option<SubsetSolution> {
    if target <= 0 {
        return None;
    }

    let depth_limit = config.max_depth.clamp(2, 8);

    // Filter valid positive candidates strictly less than target
    let mut pool: Vec<SubsetCandidate> = candidates
        .iter()
        .filter(|c| c.amount_vnd > 0 && c.amount_vnd < target)
        .cloned()
        .collect();

    if pool.len() < 2 {
        return None;
    }

    // Sort descending by amount for tighter branch-and-bound pruning
    pool.sort_by_key(|a| std::cmp::Reverse(a.amount_vnd));

    let n = pool.len();

    // Precompute suffix sums using checked i128 to prevent overflow
    // suffix[i] = sum_{j=i}^{n-1} pool[j].amount_vnd
    let mut suffix_sums = vec![0i128; n + 1];
    for i in (0..n).rev() {
        suffix_sums[i] = suffix_sums[i + 1]
            .checked_add(pool[i].amount_vnd as i128)
            .expect("Suffix sum overflow");
    }

    // Quick check: if total sum of all pool items is less than target, impossible
    if suffix_sums[0] < target as i128 {
        return None;
    }

    let mut state = SearchState {
        iterations: 0,
        timed_out: false,
        start_time: Instant::now(),
        found_solution: None,
    };

    let mut current_ids = Vec::with_capacity(depth_limit);
    let mut current_indices = Vec::with_capacity(depth_limit);

    search_recursive(
        0,
        target,
        0,
        &pool,
        &suffix_sums,
        depth_limit,
        config,
        &mut current_ids,
        &mut current_indices,
        &mut state,
    );

    if let Some((ids, indices)) = state.found_solution {
        Some(SubsetSolution {
            candidate_ids: ids,
            candidate_indices: indices,
            total_amount: target,
            iterations_used: state.iterations,
            timed_out: state.timed_out,
        })
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn search_recursive(
    start_idx: usize,
    target: i64,
    current_sum: i64,
    pool: &[SubsetCandidate],
    suffix_sums: &[i128],
    depth_limit: usize,
    config: &SubsetSolverConfig,
    current_ids: &mut Vec<String>,
    current_indices: &mut Vec<usize>,
    state: &mut SearchState,
) {
    if state.found_solution.is_some() || state.timed_out {
        return;
    }

    // Check goal condition
    if current_sum == target && current_ids.len() >= 2 {
        state.found_solution = Some((current_ids.clone(), current_indices.clone()));
        return;
    }

    // Check depth limit
    if current_ids.len() >= depth_limit {
        return;
    }

    // Budget checks
    state.iterations = state.iterations.saturating_add(1);
    if state.iterations >= config.max_iterations {
        state.timed_out = true;
        return;
    }
    if state.iterations.is_multiple_of(1024)
        && state.start_time.elapsed().as_millis() as u64 >= config.timeout_ms
    {
        state.timed_out = true;
        return;
    }

    let n = pool.len();
    for i in start_idx..n {
        let item_amt = pool[i].amount_vnd;

        // Check overshoot with checked addition
        let next_sum = match current_sum.checked_add(item_amt) {
            Some(s) => s,
            None => continue,
        };

        if next_sum > target {
            // Since items are sorted descending, subsequent items might still fit, so continue
            continue;
        }

        // CRITICAL OPTIMIZATION: Suffix-Sum Undershoot Pruning
        // If current_sum + sum of ALL remaining items is strictly less than target,
        // no combination of remaining items can ever reach target!
        let remaining_potential = (current_sum as i128) + suffix_sums[i];
        if remaining_potential < target as i128 {
            // Abort entire subtree
            break;
        }

        current_ids.push(pool[i].id.clone());
        current_indices.push(pool[i].original_index);

        search_recursive(
            i + 1,
            target,
            next_sum,
            pool,
            suffix_sums,
            depth_limit,
            config,
            current_ids,
            current_indices,
            state,
        );

        current_ids.pop();
        current_indices.pop();

        if state.found_solution.is_some() || state.timed_out {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subset_sum_2_items() {
        let candidates = vec![
            SubsetCandidate { id: "INV-1".to_string(), amount_vnd: 10_000_000, original_index: 0 },
            SubsetCandidate { id: "INV-2".to_string(), amount_vnd: 20_000_000, original_index: 1 },
            SubsetCandidate { id: "INV-3".to_string(), amount_vnd: 35_000_000, original_index: 2 },
        ];

        let config = SubsetSolverConfig::default();
        let res = solve_subset_sum(30_000_000, &candidates, &config);

        assert!(res.is_some());
        let sol = res.unwrap();
        assert_eq!(sol.candidate_ids.len(), 2);
        assert!(sol.candidate_ids.contains(&"INV-1".to_string()));
        assert!(sol.candidate_ids.contains(&"INV-2".to_string()));
        assert_eq!(sol.total_amount, 30_000_000);
    }

    #[test]
    fn test_subset_sum_5_items() {
        let candidates = vec![
            SubsetCandidate { id: "I1".to_string(), amount_vnd: 1_000_000, original_index: 0 },
            SubsetCandidate { id: "I2".to_string(), amount_vnd: 2_000_000, original_index: 1 },
            SubsetCandidate { id: "I3".to_string(), amount_vnd: 3_000_000, original_index: 2 },
            SubsetCandidate { id: "I4".to_string(), amount_vnd: 4_000_000, original_index: 3 },
            SubsetCandidate { id: "I5".to_string(), amount_vnd: 5_000_000, original_index: 4 },
        ];

        let config = SubsetSolverConfig::default();
        let res = solve_subset_sum(15_000_000, &candidates, &config);

        assert!(res.is_some());
        let sol = res.unwrap();
        assert_eq!(sol.total_amount, 15_000_000);
        assert_eq!(sol.candidate_ids.len(), 5);
    }

    #[test]
    fn test_suffix_sum_early_abort() {
        // Total available is 6M, target is 100M
        let candidates = vec![
            SubsetCandidate { id: "I1".to_string(), amount_vnd: 1_000_000, original_index: 0 },
            SubsetCandidate { id: "I2".to_string(), amount_vnd: 2_000_000, original_index: 1 },
            SubsetCandidate { id: "I3".to_string(), amount_vnd: 3_000_000, original_index: 2 },
        ];

        let config = SubsetSolverConfig::default();
        let res = solve_subset_sum(100_000_000, &candidates, &config);
        assert!(res.is_none());
    }

    #[test]
    fn test_depth_limit_enforced() {
        // Needs 9 items of 1M to make 9M, but max_depth is 8
        let mut candidates = Vec::new();
        for i in 0..10 {
            candidates.push(SubsetCandidate {
                id: format!("I{}", i),
                amount_vnd: 1_000_000,
                original_index: i,
            });
        }

        let config = SubsetSolverConfig {
            max_depth: 8,
            ..Default::default()
        };

        let res = solve_subset_sum(9_000_000, &candidates, &config);
        assert!(res.is_none(), "Should not find solution requiring 9 items when max_depth is 8");
    }
}
