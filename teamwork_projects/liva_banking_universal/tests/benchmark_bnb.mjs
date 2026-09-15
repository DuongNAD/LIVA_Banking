/**
 * Standalone High-Precision Microbenchmark for Tier 3 Subset-Sum BnB
 * Tests worst-case combinatorial branch-and-bound with duplicate items at k=8
 */

import { performance } from 'node:perf_hooks';
import { solveExactSubsetSumBnb } from '../src/engine/reconciliation/tier3SplitSolver.ts';

console.log('=== TIER 3 SUBSET-SUM BRANCH-AND-BOUND HIGH-PRECISION BENCHMARK ===\n');

// 1. Benchmark 60 identical candidates, impossible target (worst-case full tree traversal)
const candidates60 = Array.from({ length: 60 }, (_, i) => ({
  index: i,
  amount: 100_000_000,
}));

const TARGET_IMPOSSIBLE = 700_000_001;
const DEPTH_K8 = 8;
const ITERATIONS = 1000;

// Warm-up JIT
for (let i = 0; i < 50; i++) {
  solveExactSubsetSumBnb(candidates60, TARGET_IMPOSSIBLE, DEPTH_K8);
}

const times = [];
const tStart = performance.now();

for (let i = 0; i < ITERATIONS; i++) {
  const t0 = performance.now();
  const res = solveExactSubsetSumBnb(candidates60, TARGET_IMPOSSIBLE, DEPTH_K8);
  const t1 = performance.now();
  if (res !== null) throw new Error('Expected null solution for impossible target');
  times.push(t1 - t0);
}

const totalElapsed = performance.now() - tStart;
const avgMs = totalElapsed / ITERATIONS;
const minMs = Math.min(...times);
const maxMs = Math.max(...times);

// Percentiles
times.sort((a, b) => a - b);
const p50 = times[Math.floor(ITERATIONS * 0.5)];
const p95 = times[Math.floor(ITERATIONS * 0.95)];
const p99 = times[Math.floor(ITERATIONS * 0.99)];

console.log(`Scenario: 60 identical items (100M VND each), target=${TARGET_IMPOSSIBLE}, depth k=${DEPTH_K8}`);
console.log(`Iterations: ${ITERATIONS}`);
console.log(`Total Elapsed: ${totalElapsed.toFixed(2)} ms`);
console.log(`Avg Latency:   ${avgMs.toFixed(4)} ms`);
console.log(`Min Latency:   ${minMs.toFixed(4)} ms`);
console.log(`Max Latency:   ${maxMs.toFixed(4)} ms`);
console.log(`P50 Latency:   ${p50.toFixed(4)} ms`);
console.log(`P95 Latency:   ${p95.toFixed(4)} ms`);
console.log(`P99 Latency:   ${p99.toFixed(4)} ms`);

if (avgMs >= 5.0 || maxMs >= 50.0) {
  console.error(`\n[FAIL] Average latency ${avgMs.toFixed(4)} ms exceeds 5.0 ms threshold!`);
  process.exit(1);
} else {
  console.log(`\n[PASS] Meets strict < 5ms requirement by a factor of ${(5.0 / avgMs).toFixed(1)}x!`);
}
