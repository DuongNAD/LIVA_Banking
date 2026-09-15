import { describe, it, expect } from 'vitest';
import { solveExactSubsetSumBnb, matchTier3 } from '../src/engine/reconciliation/tier3SplitSolver';
import type { RawStatementRow, LedgerEntry } from '../src/types/banking';

describe('Tier3SplitSolver', () => {
  it('should solve exact subset-sum using branch-and-bound', () => {
    const candidates = [
      { index: 0, amount: 20000000 },
      { index: 1, amount: 45000000 },
      { index: 2, amount: 55000000 },
      { index: 3, amount: 30000000 },
    ];

    // Target = 100M VND (45M + 55M)
    const res = solveExactSubsetSumBnb(candidates, 100000000, 4);
    expect(res).toBeDefined();
    expect(res).toContain(1);
    expect(res).toContain(2);
    expect(res?.length).toBe(2);
  });

  it('should match 1-to-N composite settlement with zero residual (Delta == 0)', () => {
    const bankTxs: Partial<RawStatementRow>[] = [
      {
        id: 'tx_comp',
        txCode: 'VCB_COMP',
        amount: 100000000,
        txType: 'CREDIT',
        narration: 'Thanh toan gom 2 hoa don',
      },
    ];

    const ledgerEntries: Partial<LedgerEntry>[] = [
      { id: 'inv1', docNo: 'HD-01', amount: 45000000, entryType: 'CREDIT' },
      { id: 'inv2', docNo: 'HD-02', amount: 55000000, entryType: 'CREDIT' },
      { id: 'inv3', docNo: 'HD-03', amount: 80000000, entryType: 'CREDIT' },
    ];

    const result = matchTier3(
      bankTxs as RawStatementRow[],
      ledgerEntries as LedgerEntry[],
      [0],
      [0, 1, 2],
      4
    );

    expect(result.matches.length).toBe(1);
    expect(result.matches[0].matchType).toBe('COMPOSITE_1_TO_N');
    expect(result.matches[0].matchedAmount).toBe(100000000);
    expect(result.matches[0].ledgerEntryIds).toContain('inv1');
    expect(result.matches[0].ledgerEntryIds).toContain('inv2');
    expect(result.quarantined.length).toBe(0);
  });

  it('should quarantine ambiguous residual transactions with single-use 15-min UUIDv4 tokens', () => {
    const bankTxs: Partial<RawStatementRow>[] = [
      {
        id: 'tx_unknown',
        txCode: 'VCB_ODD',
        amount: 99999999,
        txType: 'CREDIT',
        narration: 'Giao dich la khong co hoa don',
      },
    ];

    const ledgerEntries: Partial<LedgerEntry>[] = [];

    const result = matchTier3(
      bankTxs as RawStatementRow[],
      ledgerEntries as LedgerEntry[],
      [0],
      [],
      4
    );

    expect(result.matches.length).toBe(0);
    expect(result.quarantined.length).toBe(1);
    expect(result.quarantined[0].hitlToken).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    expect(result.quarantined[0].expiresAt).toBeGreaterThan(result.quarantined[0].createdAt);
  });

  it('should skip duplicate candidate amounts in Branch 2 and avoid combinatorial stall', () => {
    // 50 duplicate candidates of 100M VND, impossible target
    const candidates = Array.from({ length: 50 }, (_, i) => ({
      index: i,
      amount: 100000000,
    }));

    const t0 = performance.now();
    const res = solveExactSubsetSumBnb(candidates, 700000001, 8);
    const duration = performance.now() - t0;

    expect(res).toBeNull();
    expect(duration).toBeLessThan(50);
  });
});
