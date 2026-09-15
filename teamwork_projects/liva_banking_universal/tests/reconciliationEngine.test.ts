import { describe, it, expect } from 'vitest';
import { runReconciliationEngine } from '../src/engine/reconciliation/reconciliationEngine';
import type { RawStatementRow, LedgerEntry } from '../src/types/banking';

describe('ReconciliationEngine', () => {
  it('should run end-to-end reconciliation across all 3 tiers', () => {
    const bankTxs: Partial<RawStatementRow>[] = [
      // Tier 1: Exact
      {
        id: 'b1',
        txCode: 'VCB01',
        date: '2026-08-01',
        txDate: 1785544252,
        amount: 145000000,
        txType: 'CREDIT',
        narration: 'TT HD-2026-88',
      },
      // Tier 2: Fuzzy with fee deduction
      {
        id: 'b2',
        txCode: 'TCB01',
        date: '2026-08-08',
        txDate: 1785800000,
        amount: 119990000, // 10k fee
        txType: 'CREDIT',
        narration: 'ck tien hang Dai Viet',
        counterparty: 'Cong ty Dai Viet',
      },
      // Tier 3: 1:N Composite Split
      {
        id: 'b3',
        txCode: 'VCB02',
        date: '2026-08-10',
        txDate: 1785900000,
        amount: 100000000,
        txType: 'CREDIT',
        narration: 'Thanh toan 2 dot',
      },
    ];

    const ledgerEntries: Partial<LedgerEntry>[] = [
      {
        id: 'l1',
        docNo: 'HD-2026-88',
        entryDate: '2026-08-01',
        entryTimestamp: 1785544252,
        amount: 145000000,
        entryType: 'CREDIT',
        partnerName: 'An Phat',
      },
      {
        id: 'l2',
        docNo: 'HD-2026-92',
        entryDate: '2026-08-08',
        entryTimestamp: 1785800000,
        amount: 120000000,
        entryType: 'CREDIT',
        partnerName: 'Cong ty Dai Viet',
      },
      {
        id: 'l3a',
        docNo: 'HD-03A',
        entryDate: '2026-08-10',
        entryTimestamp: 1785900000,
        amount: 40000000,
        entryType: 'CREDIT',
        partnerName: 'Doi tac C',
      },
      {
        id: 'l3b',
        docNo: 'HD-03B',
        entryDate: '2026-08-10',
        entryTimestamp: 1785900000,
        amount: 60000000,
        entryType: 'CREDIT',
        partnerName: 'Doi tac C',
      },
    ];

    const summary = runReconciliationEngine(
      bankTxs as RawStatementRow[],
      ledgerEntries as LedgerEntry[]
    );

    expect(summary.totalBankTransactions).toBe(3);
    expect(summary.matchedCount).toBe(3);
    expect(summary.matchRate).toBe(100.0);
    expect(summary.tier1Count).toBe(1);
    expect(summary.tier2Count).toBe(1);
    expect(summary.tier3Count).toBe(1);
    expect(summary.hitlQuarantineCount).toBe(0);
    expect(summary.totalFeeDisentangled).toBe(10000);
  });
});
