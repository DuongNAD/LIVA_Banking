import { describe, it, expect } from 'vitest';
import { matchTier2, comparePartyNames, stripCorporateLegalNoise } from '../src/engine/reconciliation/tier2FuzzyMatcher';
import type { RawStatementRow, LedgerEntry } from '../src/types/banking';

describe('Tier2FuzzyMatcher', () => {
  it('should strip corporate legal noise correctly', () => {
    const clean = stripCorporateLegalNoise('CONG TY TNHH THUONG MAI DICH VU AN PHAT');
    expect(clean).toBe('an phat');
  });

  it('should compute high Jaro-Winkler similarity for variant legal names', () => {
    const sim = comparePartyNames(
      'Công ty TNHH Cơ điện lạnh Đại Việt',
      'CONG TY CO DIEN LANH DAI VIET'
    );
    expect(sim).toBeGreaterThanOrEqual(0.95);
  });

  it('should match with interbank wire fee deduction within standard tolerance (1,100 to 11,000 VND)', () => {
    const bankTxs: Partial<RawStatementRow>[] = [
      {
        id: 'tx2',
        txCode: 'TCB02',
        date: '2026-08-08',
        txDate: 1785800000,
        amount: 119990000, // 10,000 VND fee deducted by bank
        txType: 'CREDIT',
        narration: 'ck tien hang may bien ap hd 99',
        counterparty: 'Công ty TNHH Cơ điện lạnh Đại Việt',
      },
    ];

    const ledgerEntries: Partial<LedgerEntry>[] = [
      {
        id: 'l2',
        docNo: 'HD-2026-92',
        entryDate: '2026-08-08',
        entryTimestamp: 1785800000,
        amount: 120000000,
        entryType: 'CREDIT',
        partnerName: 'Công ty TNHH Cơ điện lạnh Đại Việt',
      },
    ];

    const result = matchTier2(
      bankTxs as RawStatementRow[],
      ledgerEntries as LedgerEntry[],
      [0],
      [0]
    );

    expect(result.matches.length).toBe(1);
    expect(result.matches[0].tier).toBe('TIER2_FUZZY');
    expect(result.matches[0].feeAmount).toBe(10000);
    expect(result.matches[0].confidence).toBeGreaterThanOrEqual(0.85);
  });
});
