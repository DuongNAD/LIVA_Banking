#!/usr/bin/env node
/**
 * Master E2E Test Suite Runner
 * Universal Corporate Banking & Treasury Harness (liva_banking_universal)
 * Executes all 4 Tiers (236 tests):
 * - Tier 1: Feature Coverage (105 tests across F01 - F21)
 * - Tier 2: Boundary & Corner Cases (105 tests across F01 - F21)
 * - Tier 3: Cross-Feature Combinations (21 pairwise integration tests)
 * - Tier 4: Real-World Multi-Bank Workflows (5 end-to-end scenarios)
 *
 * Exit code 0 on 100% pass, non-zero on any failure.
 */

import { runTier1Tests } from './tier1_feature_coverage.test.mjs';
import { runTier2Tests } from './tier2_boundary_corner.test.mjs';
import { runTier3Tests } from './tier3_cross_feature.test.mjs';
import { runTier4Tests } from './tier4_real_world.test.mjs';

const ANSI = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  dim: '\x1b[2m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  magenta: '\x1b[35m',
};

async function main() {
  console.log(`\n${ANSI.bold}${ANSI.cyan}================================================================================${ANSI.reset}`);
  console.log(`${ANSI.bold}${ANSI.cyan}  LIVA BANKING UNIVERSAL — 4-TIER E2E TEST SUITE RUNNER${ANSI.reset}`);
  console.log(`${ANSI.dim}  Adhering to Circular 09/2020, Circular 09/2023, Decision 11/2023, Decree 13/2023${ANSI.reset}`);
  console.log(`${ANSI.bold}${ANSI.cyan}================================================================================${ANSI.reset}\n`);

  const suiteStart = performance.now();
  const summary = {
    tier1: { total: 0, passed: 0, failed: 0, durationMs: 0 },
    tier2: { total: 0, passed: 0, failed: 0, durationMs: 0 },
    tier3: { total: 0, passed: 0, failed: 0, durationMs: 0 },
    tier4: { total: 0, passed: 0, failed: 0, durationMs: 0 },
    totalTests: 0,
    totalPassed: 0,
    totalFailed: 0,
    failures: [],
  };

  async function executeTier(name, runnerFn, key) {
    console.log(`${ANSI.bold}${ANSI.blue}▶ Executing ${name}...${ANSI.reset}`);
    const tStart = performance.now();
    const results = await runnerFn();
    const duration = performance.now() - tStart;

    const passedCount = results.filter((r) => r.passed).length;
    const failedCount = results.filter((r) => !r.passed).length;

    summary[key] = {
      total: results.length,
      passed: passedCount,
      failed: failedCount,
      durationMs: duration,
    };

    summary.totalTests += results.length;
    summary.totalPassed += passedCount;
    summary.totalFailed += failedCount;

    for (const r of results) {
      if (!r.passed) {
        summary.failures.push({ tier: name, ...r });
        console.log(`  ${ANSI.red}✗ [FAIL] ${r.id}: ${r.title}${ANSI.reset}`);
        console.log(`    ${ANSI.dim}${r.error}${ANSI.reset}`);
      }
    }

    const badge = failedCount === 0 ? `${ANSI.green}✓ PASS${ANSI.reset}` : `${ANSI.red}✗ FAIL (${failedCount})${ANSI.reset}`;
    console.log(`  ${badge} ${ANSI.dim}(${passedCount}/${results.length} passed in ${duration.toFixed(1)}ms)${ANSI.reset}\n`);
  }

  // 1. Tier 1: Feature Coverage (105 tests)
  await executeTier('Tier 1: Feature Coverage (F01 - F21)', runTier1Tests, 'tier1');

  // 2. Tier 2: Boundary & Corner Cases (105 tests)
  await executeTier('Tier 2: Boundary & Corner Cases (F01 - F21)', runTier2Tests, 'tier2');

  // 3. Tier 3: Cross-Feature Interactions (21 tests)
  await executeTier('Tier 3: Cross-Feature Combinations', runTier3Tests, 'tier3');

  // 4. Tier 4: Real-World Workflows (5 tests)
  await executeTier('Tier 4: Real-World Multi-Bank Scenarios', runTier4Tests, 'tier4');

  const totalDuration = performance.now() - suiteStart;

  // Final Summary Table
  console.log(`${ANSI.bold}${ANSI.cyan}--------------------------------------------------------------------------------${ANSI.reset}`);
  console.log(`${ANSI.bold}E2E TEST SUITE EXECUTION SUMMARY${ANSI.reset}`);
  console.log(`${ANSI.bold}${ANSI.cyan}--------------------------------------------------------------------------------${ANSI.reset}`);
  console.log(`  ${'Tier'.padEnd(42)} | ${'Pass/Total'.padEnd(12)} | ${'Duration'.padEnd(10)} | Status`);
  console.log(`  ------------------------------------------------------------------------------`);
  
  const printTierRow = (label, data) => {
    const status = data.failed === 0 ? `${ANSI.green}PASS${ANSI.reset}` : `${ANSI.red}FAIL${ANSI.reset}`;
    const ratio = `${data.passed}/${data.total}`;
    const dur = `${data.durationMs.toFixed(1)}ms`;
    console.log(`  ${label.padEnd(42)} | ${ratio.padEnd(12)} | ${dur.padEnd(10)} | ${status}`);
  };

  printTierRow('Tier 1: Feature Coverage (21 features)', summary.tier1);
  printTierRow('Tier 2: Boundary & Corner Cases (21 features)', summary.tier2);
  printTierRow('Tier 3: Cross-Feature Interactions (Pairwise)', summary.tier3);
  printTierRow('Tier 4: Real-World Scenarios (VCB/TCB/BIDV)', summary.tier4);
  console.log(`  ------------------------------------------------------------------------------`);
  console.log(
    `  ${ANSI.bold}${'TOTAL ACROSS ALL 4 TIERS'.padEnd(42)} | ${(`${summary.totalPassed}/${summary.totalTests}`).padEnd(12)} | ${(`${totalDuration.toFixed(1)}ms`).padEnd(10)} | ${summary.totalFailed === 0 ? `${ANSI.green}${ANSI.bold}ALL PASSED${ANSI.reset}` : `${ANSI.red}${ANSI.bold}FAILED${ANSI.reset}`}${ANSI.reset}`
  );
  console.log(`${ANSI.bold}${ANSI.cyan}================================================================================${ANSI.reset}\n`);

  if (summary.totalFailed > 0) {
    console.error(`${ANSI.red}${ANSI.bold}TEST RUN FAILED with ${summary.totalFailed} errors.${ANSI.reset}\n`);
    process.exit(1);
  } else {
    console.log(`${ANSI.green}${ANSI.bold}ALL ${summary.totalTests} TESTS PASSED SUCCESSFULLY! (100% PASS RATE)${ANSI.reset}\n`);
    process.exit(0);
  }
}

main().catch((err) => {
  console.error(`${ANSI.red}Fatal runner error: ${err instanceof Error ? err.stack : String(err)}${ANSI.reset}`);
  process.exit(1);
});
