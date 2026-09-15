import { describe, it, expect } from 'vitest';
import { matchTier1, normalizeDocRef } from '../src/engine/reconciliation/tier1ExactMatcher';
import type { RawStatementRow, LedgerEntry } from '../src/types/banking';

describe('Tier1ExactMatcher', () => {
  it('should normalize document references correctly', () => {
    expect(normalizeDocRef('HD-2026-88')).toBe('HD202688');
    expect(normalizeDocRef('HD-00102')).toBe('HD102');
    expect(normalizeDocRef('INV-002')).toBe('INV2');
  });

  it('should match exact reference and amount within 24h window', () => {
    const bankTxs: Partial<RawStatementRow>[] = [
      {
        id: 'tx1',
        txCode: 'VCB01',
        date: '2026-08-01',
        txDate: 1785544252,
        amount: 145000000,
        txType: 'CREDIT',
        narration: 'CT TT TIEN HANG HOP DONG SO HD-2026-88 CONG TY AN PHAT',
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
        partnerName: 'CONG TY TNHH TM DV AN PHAT',
      },
    ];

    const result = matchTier1(bankTxs as RawStatementRow[], ledgerEntries as LedgerEntry[]);
    expect(result.matches.length).toBe(1);
    expect(result.matches[0].tier).toBe('TIER1_EXACT');
    expect(result.matches[0].confidence).toBe(1.0);
    expect(result.matches[0].discrepancyAmount).toBe(0);
    expect(result.unallocatedBankIndices.length).toBe(0);
    expect(result.unallocatedLedgerIndices.length).toBe(0);
  });

  it('should not match if direction differs (Debit vs Credit)', () => {
    const bankTxs: Partial<RawStatementRow>[] = [
      {
        id: 'tx1',
        txCode: 'VCB01',
        date: '2026-08-01',
        txDate: 1785544252,
        amount: 145000000,
        txType: 'DEBIT', // Bank outgoing debit
        narration: 'Chi tra HD-2026-88',
      },
    ];

    const ledgerEntries: Partial<LedgerEntry>[] = [
      {
        id: 'l1',
        docNo: 'HD-2026-88',
        entryDate: '2026-08-01',
        entryTimestamp: 1785544252,
        amount: 145000000,
        entryType: 'CREDIT', // Ledger incoming receipt
        partnerName: 'CONG TY AN PHAT',
      },
    ];

    const result = matchTier1(bankTxs as RawStatementRow[], ledgerEntries as LedgerEntry[]);
    expect(result.matches.length).toBe(0);
    expect(result.unallocatedBankIndices.length).toBe(1);
  });
});
