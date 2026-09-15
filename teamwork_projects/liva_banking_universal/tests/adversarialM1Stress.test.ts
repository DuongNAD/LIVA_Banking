/**
 * Empirical Adversarial Challenge Suite: Milestone 1
 * Universal Statement Ingestion & 3-Tier Reconciliation Engine
 *
 * Stress-tests:
 * 1. Extreme amount scaling up to 500 Billion VND (500e9) and boundary conditions.
 * 2. Floating-point drift fuzzing and Vietnamese/European/US amount string parsing.
 * 3. Combinatorial branch-and-bound subset-sum cutoff (k <= 8), non-converging pools, and identical items.
 * 4. Tie-breaking determinism across Tier 1 and Tier 2.
 * 5. Balance invariant edge cases (+/- 1 VND discrepancy) and running balance continuity behavior.
 * 6. HITL quarantine fail-closed enforcement and 15-minute token TTL (Circular 09/2020).
 */

import { describe, it, expect } from 'vitest';
import { parseVietnameseAmount } from '../src/engine/ingestion/universalParser';
import { verifyBalanceInvariants, verifyRunningBalanceContinuity } from '../src/engine/reconciliation/balanceValidator';
import { matchTier1 } from '../src/engine/reconciliation/tier1ExactMatcher';
import { matchTier2 } from '../src/engine/reconciliation/tier2FuzzyMatcher';
import {
  solveExactSubsetSumBnb,
  generateUuidV4
} from '../src/engine/reconciliation/tier3SplitSolver';
import { runReconciliationEngine } from '../src/engine/reconciliation/reconciliationEngine';
import type { RawStatementRow, LedgerEntry } from '../src/types/banking';

describe('Adversarial Challenge 1: Extreme Amount Scaling (up to 500B VND)', () => {
  const FIVE_HUNDRED_BILLION = 500_000_000_000;

  it('correctly parses 500 Billion VND in various international and Vietnamese formats', () => {
    // Standard Vietnamese dot-thousand format
    expect(parseVietnameseAmount('500.000.000.000')).toBe(FIVE_HUNDRED_BILLION);
    expect(parseVietnameseAmount('500.000.000.000,00')).toBe(FIVE_HUNDRED_BILLION);
    expect(parseVietnameseAmount('500.000.000.000 VND')).toBe(FIVE_HUNDRED_BILLION);
    expect(parseVietnameseAmount('500.000.000.000 VNĐ')).toBe(FIVE_HUNDRED_BILLION);

    // US comma-thousand format
    expect(parseVietnameseAmount('500,000,000,000')).toBe(FIVE_HUNDRED_BILLION);
    expect(parseVietnameseAmount('500,000,000,000.00')).toBe(FIVE_HUNDRED_BILLION);

    // Raw unformatted number & string
    expect(parseVietnameseAmount('500000000000')).toBe(FIVE_HUNDRED_BILLION);
    expect(parseVietnameseAmount(500000000000)).toBe(FIVE_HUNDRED_BILLION);

    // Negative amounts (prefix minus and standalone paren)
    expect(parseVietnameseAmount('-500.000.000.000')).toBe(-FIVE_HUNDRED_BILLION);
    expect(parseVietnameseAmount('(500.000.000.000)')).toBe(-FIVE_HUNDRED_BILLION);
    expect(parseVietnameseAmount('(-500.000.000.000)')).toBe(-FIVE_HUNDRED_BILLION);

    // Single VND boundary
    expect(parseVietnameseAmount('1')).toBe(1);
    expect(parseVietnameseAmount('1 VND')).toBe(1);
    expect(parseVietnameseAmount('1,00')).toBe(1);
    expect(parseVietnameseAmount('1.00')).toBe(1);
  });

  it('validates macro balance invariant with 500 Billion VND figures with zero precision loss', () => {
    const opening = FIVE_HUNDRED_BILLION;
    const credit = 120_000_000_000;
    const debit = 70_000_000_000;
    const expectedClosing = FIVE_HUNDRED_BILLION + credit - debit; // 550B VND

    const report = verifyBalanceInvariants(opening, expectedClosing, credit, debit);
    expect(report.isValid).toBe(true);
    expect(report.discrepancy).toBe(0);
    expect(report.calculatedClosing).toBe(expectedClosing);
  });

  it('executes Tier 1 exact match with 500 Billion VND transaction', () => {
    const bankTxs: RawStatementRow[] = [
      {
        id: 'tx-vcb-500b',
        date: '2026-08-15',
        time: '10:00:00',
        txDate: 1786762800,
        valueDate: 1786762800,
        txCode: 'VCB500B',
        docRef: 'HD-2026-MEGA',
        debit: 0,
        credit: FIVE_HUNDRED_BILLION,
        netAmount: FIVE_HUNDRED_BILLION,
        amount: FIVE_HUNDRED_BILLION,
        txType: 'CREDIT',
        balance: 600_000_000_000,
        balanceAfter: 600_000_000_000,
        narration: 'Thanh toan hop dong mega HD-2026-MEGA',
        bankCode: 'VCB',
      },
    ];

    const ledgerEntries: LedgerEntry[] = [
      {
        id: 'ledger-mega',
        docNo: 'HD-2026-MEGA',
        entryDate: '2026-08-15',
        entryTimestamp: 1786762800,
        partnerName: 'TAP DOAN DAU KHI QUOC GIA',
        amount: FIVE_HUNDRED_BILLION,
        entryType: 'CREDIT',
        description: 'Doanh thu cung cap vat tu',
        status: 'UNMATCHED',
      },
    ];

    const result = matchTier1(bankTxs, ledgerEntries);
    expect(result.matches.length).toBe(1);
    expect(result.matches[0].matchedAmount).toBe(FIVE_HUNDRED_BILLION);
    expect(result.matches[0].confidence).toBe(1.0);
    expect(result.unallocatedBankIndices.length).toBe(0);
    expect(result.unallocatedLedgerIndices.length).toBe(0);
  });

  it('executes Tier 3 split solving with 500 Billion VND split into 4 invoices of 125B each', () => {
    const quarterBillion = 125_000_000_000;
    const candidates = [
      { index: 0, amount: quarterBillion },
      { index: 1, amount: quarterBillion },
      { index: 2, amount: quarterBillion },
      { index: 3, amount: quarterBillion },
    ];

    const solution = solveExactSubsetSumBnb(candidates, FIVE_HUNDRED_BILLION, 4);
    expect(solution).not.toBeNull();
    expect(solution!.length).toBe(4);
    const sum = solution!.reduce((acc, idx) => acc + candidates[idx].amount, 0);
    expect(sum).toBe(FIVE_HUNDRED_BILLION);
  });
});

describe('Adversarial Challenge 2: Floating-Point Drift Fuzzing & String Parser Boundaries', () => {
  it('fuzzes parseVietnameseAmount with fractional cents and integer truncation', () => {
    // Truncates fractional cents to integer VND without floating-point artifacts
    expect(parseVietnameseAmount('15.000.000,49')).toBe(15_000_000);
    expect(parseVietnameseAmount('15.000.000,99')).toBe(15_000_000);
    expect(parseVietnameseAmount('15,000,000.85')).toBe(15_000_000);

    // Numeric inputs with decimals
    expect(parseVietnameseAmount(15_000_000.2)).toBe(15_000_000);
    expect(parseVietnameseAmount(15_000_000.7)).toBe(15_000_001);

    // Non-numeric, dates, and file paths
    expect(parseVietnameseAmount('INV/2026/01')).toBe(0);
    expect(parseVietnameseAmount('15/08/2026')).toBe(0);
    expect(parseVietnameseAmount('C:\\Invoices\\88.pdf')).toBe(0);
    expect(parseVietnameseAmount('14:30:00')).toBe(0);
    expect(parseVietnameseAmount('NaN')).toBe(0);
    expect(parseVietnameseAmount(NaN)).toBe(0);
    expect(parseVietnameseAmount(null)).toBe(0);
    expect(parseVietnameseAmount(undefined)).toBe(0);
    expect(parseVietnameseAmount('')).toBe(0);
    expect(parseVietnameseAmount('   ')).toBe(0);

    // Text with currency symbols (without colon)
    expect(parseVietnameseAmount('Cộng 50.000.000 VNĐ')).toBe(50_000_000);
    expect(parseVietnameseAmount('-12.500.000 Đ')).toBe(-12_500_000);
    expect(parseVietnameseAmount('(35.000.000)')).toBe(-35_000_000);
  });

  it('correctly parses negative parenthesized amounts even when trailing currency suffix is present', () => {
    // Verified fix: currency tokens stripped before checking parens
    const resultWithCurrencySuffix = parseVietnameseAmount('(35.000.000) VND');
    const resultWithoutSuffix = parseVietnameseAmount('(35.000.000)');

    expect(resultWithoutSuffix).toBe(-35_000_000);
    expect(resultWithCurrencySuffix).toBe(-35_000_000);
  });

  it('correctly parses amounts with colons in labeled cells while rejecting timestamps', () => {
    // Colon in labeled amounts like "Cộng: 50.000.000" extracts the amount
    expect(parseVietnameseAmount('Cộng: 50.000.000 VNĐ')).toBe(50_000_000);
    expect(parseVietnameseAmount('Cộng 50.000.000 VNĐ')).toBe(50_000_000);
    expect(parseVietnameseAmount('14:30:00')).toBe(0);
  });

  it('guarantees zero IEEE-754 floating-point drift over 10,000 accumulated increments', () => {
    let opening = 0;
    let expectedTotal = 0;
    const iterations = 10_000;
    const stepVnd = 12_345;

    for (let i = 0; i < iterations; i++) {
      expectedTotal += stepVnd;
    }

    const report = verifyBalanceInvariants(opening, expectedTotal, expectedTotal, 0);
    expect(report.isValid).toBe(true);
    expect(report.discrepancy).toBe(0);
    expect(report.calculatedClosing).toBe(123_450_000);
  });
});

describe('Adversarial Challenge 3: Combinatorial Branch-and-Bound Subset-Sum Cutoff (k <= 8)', () => {
  it('strictly limits search depth to MAX_SUPPORTED_SPLIT_DEPTH (k <= 8) even when higher requested', () => {
    const candidates = [
      { index: 0, amount: 10_000 },
      { index: 1, amount: 10_000 },
      { index: 2, amount: 10_000 },
      { index: 3, amount: 10_000 },
      { index: 4, amount: 10_000 },
      { index: 5, amount: 10_000 },
      { index: 6, amount: 10_000 },
      { index: 7, amount: 10_000 },
      { index: 8, amount: 10_000 }, // 9 items total = 90,000
    ];

    // Target = 90,000 requires 9 items
    // Request depth 12: solver MUST enforce depthLimit = min(12, 8) = 8, so 9 items cannot be matched
    const resultDepth12 = solveExactSubsetSumBnb(candidates, 90_000, 12);
    expect(resultDepth12).toBeNull();

    // Target = 80,000 requires exactly 8 items (k = 8)
    const resultDepth8 = solveExactSubsetSumBnb(candidates, 80_000, 8);
    expect(resultDepth8).not.toBeNull();
    expect(resultDepth8!.length).toBe(8);
  });

  it('BENCHMARK: verifies duplicate candidate pruning in Branch 2 eliminates combinatorial stall', () => {
    // In tier3SplitSolver.ts: Branch 2 skips subsequent candidates with identical amounts
    const candidates = Array.from({ length: 60 }, (_, i) => ({
      index: i,
      amount: 100_000_000,
    }));

    // Target 700M+1 (impossible) at depth 8 triggers worst-case recursive search
    const t0 = performance.now();
    const solution = solveExactSubsetSumBnb(candidates, 700_000_001, 8);
    const elapsedMs = performance.now() - t0;

    expect(solution).toBeNull();
    // Solver terminates near instantaneously (< 50ms) instead of stalling 4+ seconds
    expect(elapsedMs).toBeLessThan(50);
  });

  it('discards negative amounts, zero amounts, and candidates larger than target', () => {
    const candidates = [
      { index: 0, amount: -50_000 },
      { index: 1, amount: 0 },
      { index: 2, amount: 1_000_000 }, // greater than target 500_000
      { index: 3, amount: 200_000 },
      { index: 4, amount: 300_000 },
    ];

    const solution = solveExactSubsetSumBnb(candidates, 500_000, 4);
    expect(solution).not.toBeNull();
    expect(solution!.sort()).toEqual([3, 4]);
  });

  it('returns null immediately when total sum of candidates is less than target', () => {
    const candidates = [
      { index: 0, amount: 100_000 },
      { index: 1, amount: 200_000 },
    ];

    const solution = solveExactSubsetSumBnb(candidates, 500_000, 4);
    expect(solution).toBeNull();
  });
});

describe('Adversarial Challenge 4: Tie-Breaking Determinism', () => {
  it('Tier 1 exact match deterministically selects the first candidate when multiple identical exist', () => {
    const tx: RawStatementRow = {
      id: 'tx-dup-1',
      date: '2026-08-15',
      time: '09:00:00',
      txDate: 1786759200,
      valueDate: 1786759200,
      txCode: 'TX001',
      docRef: 'HD-88',
      debit: 0,
      credit: 50_000_000,
      netAmount: 50_000_000,
      amount: 50_000_000,
      txType: 'CREDIT',
      balance: 150_000_000,
      balanceAfter: 150_000_000,
      narration: 'Thanh toan hoa don HD-88',
      bankCode: 'VCB',
    };

    // 3 identical ledger invoices
    const ledgers: LedgerEntry[] = [
      {
        id: 'ledger-idx-0',
        docNo: 'HD-88',
        entryDate: '2026-08-15',
        entryTimestamp: 1786759200,
        partnerName: 'Cong ty Alpha',
        amount: 50_000_000,
        entryType: 'CREDIT',
        description: 'Hoa don 88',
        status: 'UNMATCHED',
      },
      {
        id: 'ledger-idx-1',
        docNo: 'HD-88',
        entryDate: '2026-08-15',
        entryTimestamp: 1786759200,
        partnerName: 'Cong ty Alpha',
        amount: 50_000_000,
        entryType: 'CREDIT',
        description: 'Hoa don 88 ban sao 1',
        status: 'UNMATCHED',
      },
      {
        id: 'ledger-idx-2',
        docNo: 'HD-88',
        entryDate: '2026-08-15',
        entryTimestamp: 1786759200,
        partnerName: 'Cong ty Alpha',
        amount: 50_000_000,
        entryType: 'CREDIT',
        description: 'Hoa don 88 ban sao 2',
        status: 'UNMATCHED',
      },
    ];

    // Run 50 times in a loop to ensure 100% determinism
    for (let run = 0; run < 50; run++) {
      const res = matchTier1([tx], ledgers);
      expect(res.matches.length).toBe(1);
      expect(res.matches[0].ledgerEntryIds[0]).toBe('ledger-idx-0');
    }
  });

  it('Tier 2 fuzzy match deterministically selects the highest-scoring candidate, breaking ties stably', () => {
    const tx: RawStatementRow = {
      id: 'tx-fuzzy-dup',
      date: '2026-08-15',
      time: '09:00:00',
      txDate: 1786759200,
      valueDate: 1786759200,
      txCode: 'TX002',
      docRef: 'TX002',
      debit: 0,
      credit: 50_000_000,
      netAmount: 50_000_000,
      amount: 50_000_000,
      txType: 'CREDIT',
      balance: 150_000_000,
      balanceAfter: 150_000_000,
      narration: 'Chuyen khoan Masan Consumer',
      counterparty: 'Masan Consumer',
      bankCode: 'TCB',
    };

    const ledgers: LedgerEntry[] = [
      {
        id: 'ledger-fuzzy-0',
        docNo: 'INV-A',
        entryDate: '2026-08-15',
        entryTimestamp: 1786759200,
        partnerName: 'Tap doan Masan Consumer',
        amount: 50_000_000,
        entryType: 'CREDIT',
        description: 'Ban hang',
        status: 'UNMATCHED',
      },
      {
        id: 'ledger-fuzzy-1',
        docNo: 'INV-B',
        entryDate: '2026-08-15',
        entryTimestamp: 1786759200,
        partnerName: 'Tap doan Masan Consumer',
        amount: 50_000_000,
        entryType: 'CREDIT',
        description: 'Ban hang dot 2',
        status: 'UNMATCHED',
      },
    ];

    for (let run = 0; run < 50; run++) {
      const res = matchTier2([tx], ledgers, [0], [0, 1]);
      expect(res.matches.length).toBe(1);
      expect(res.matches[0].ledgerEntryIds[0]).toBe('ledger-fuzzy-0');
    }
  });
});

describe('Adversarial Challenge 5: Deliberate Balance Invariant Discrepancies (+/- 1 VND)', () => {
  it('detects deliberate +1 VND discrepancy in closing balance with exact delta', () => {
    const opening = 100_000_000;
    const totalCredit = 50_000_000;
    const totalDebit = 20_000_000;
    const calculatedClosing = 130_000_000;

    // Deliberate +1 VND tampering
    const tamperedClosing = calculatedClosing + 1;

    const report = verifyBalanceInvariants(opening, tamperedClosing, totalCredit, totalDebit);
    expect(report.isValid).toBe(false);
    expect(report.isBalanced).toBe(false);
    expect(report.discrepancy).toBe(1);
    expect(report.calculatedClosing).toBe(calculatedClosing);
    expect(report.closingBalance).toBe(tamperedClosing);
    expect(report.errorReason).toContain('differs from calculated balance');
    expect(report.errorReason).toContain('1 VND');
  });

  it('detects deliberate -1 VND discrepancy in closing balance with exact delta', () => {
    const opening = 100_000_000;
    const totalCredit = 50_000_000;
    const totalDebit = 20_000_000;
    const calculatedClosing = 130_000_000;

    // Deliberate -1 VND tampering
    const tamperedClosing = calculatedClosing - 1;

    const report = verifyBalanceInvariants(opening, tamperedClosing, totalCredit, totalDebit);
    expect(report.isValid).toBe(false);
    expect(report.isBalanced).toBe(false);
    expect(report.discrepancy).toBe(-1);
    expect(report.calculatedClosing).toBe(calculatedClosing);
    expect(report.closingBalance).toBe(tamperedClosing);
    expect(report.errorReason).toContain('-1 VND');
  });

  it('strictly flags 1 VND discrepancy in running balance continuity (0 VND tolerance)', () => {
    // Transactions with exactly 1 VND step discrepancy
    const txs: Partial<RawStatementRow>[] = [
      { id: '1', txCode: 'T1', debit: 0, credit: 500_000, balanceAfter: 1_500_000 },
      { id: '2', txCode: 'T2', debit: 200_000, credit: 0, balanceAfter: 1_300_001 }, // 1 VND diff!
    ];

    const res = verifyRunningBalanceContinuity(txs as RawStatementRow[], 1_000_000);
    // Strict equality actual === expected flags 1 VND discrepancy
    expect(res.isContinuous).toBe(false);
    expect(res.brokenRowIndex).toBe(1);
    expect(res.errorReason).toContain('1 VND');

    // Continuous transactions without discrepancy pass
    const continuousTxs: Partial<RawStatementRow>[] = [
      { id: '1', txCode: 'T1', debit: 0, credit: 500_000, balanceAfter: 1_500_000 },
      { id: '2', txCode: 'T2', debit: 200_000, credit: 0, balanceAfter: 1_300_000 },
    ];
    const resContinuous = verifyRunningBalanceContinuity(continuousTxs as RawStatementRow[], 1_000_000);
    expect(resContinuous.isContinuous).toBe(true);
  });
});

describe('Adversarial Challenge 6: HITL Quarantine & Circular 09/2020 Compliance', () => {
  it('quarantines unmatchable transaction with valid UUIDv4 token and exact 15-minute TTL', () => {
    const unmatchableTx: RawStatementRow = {
      id: 'tx-mystery-999',
      date: '2026-08-15',
      time: '14:22:10',
      txDate: 1786778530,
      valueDate: 1786778530,
      txCode: 'MYSTERY999',
      docRef: 'UNKNOWN-REF-999',
      debit: 0,
      credit: 77_777_777,
      netAmount: 77_777_777,
      amount: 77_777_777,
      txType: 'CREDIT',
      balance: 177_777_777,
      balanceAfter: 177_777_777,
      narration: 'Tien chuyen khong ro noi dung va khong co hoa don tuong ung',
      bankCode: 'BIDV',
    };

    const ledgers: LedgerEntry[] = [
      {
        id: 'ledger-unrelated',
        docNo: 'HD-9999',
        entryDate: '2026-08-15',
        entryTimestamp: 1786778530,
        partnerName: 'Nha cung cap khac',
        amount: 10_000_000,
        entryType: 'CREDIT',
        description: 'Vat tu',
        status: 'UNMATCHED',
      },
    ];

    const summary = runReconciliationEngine([unmatchableTx], ledgers);

    expect(summary.totalBankTransactions).toBe(1);
    expect(summary.matchedCount).toBe(0);
    expect(summary.hitlQuarantineCount).toBe(1);
    expect(summary.quarantined.length).toBe(1);

    const quarantinedItem = summary.quarantined[0];
    expect(quarantinedItem.txId).toBe('tx-mystery-999');
    expect(quarantinedItem.amount).toBe(77_777_777);

    // Validate UUIDv4 token format
    const uuidv4Regex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    expect(quarantinedItem.hitlToken).toMatch(uuidv4Regex);

    // Validate 15-minute expiration window (Circular 09/2020: 900 seconds)
    const ttl = quarantinedItem.expiresAt - quarantinedItem.createdAt;
    expect(ttl).toBe(900); // exactly 15 minutes (900 seconds)

    // Suggests available ledger candidates of same direction
    expect(quarantinedItem.candidateLedgerIds).toBeDefined();
    expect(quarantinedItem.candidateLedgerIds).toContain('ledger-unrelated');
  });

  it('generates cryptographically unique UUIDv4 tokens across 1,000 invocations', () => {
    const tokens = new Set<string>();
    const count = 1_000;
    const uuidv4Regex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

    for (let i = 0; i < count; i++) {
      const token = generateUuidV4();
      expect(token).toMatch(uuidv4Regex);
      tokens.add(token);
    }

    expect(tokens.size).toBe(count); // Zero collisions
  });
});
