import { describe, it, expect } from 'vitest';
import { verifyBalanceInvariants, verifyRunningBalanceContinuity } from '../src/engine/reconciliation/balanceValidator';
import type { RawStatementRow } from '../src/types/banking';

describe('BalanceValidator', () => {
  it('should validate exact macro balance identity (Closing = Opening + Credit - Debit)', () => {
    const report = verifyBalanceInvariants(1000000, 1500000, 700000, 200000);
    expect(report.isValid).toBe(true);
    expect(report.discrepancy).toBe(0);
    expect(report.calculatedClosing).toBe(1500000);
  });

  it('should detect discrepancy when closing balance does not match calculation', () => {
    const report = verifyBalanceInvariants(1000000, 1400000, 700000, 200000);
    expect(report.isValid).toBe(false);
    expect(report.discrepancy).toBe(-100000);
    expect(report.errorReason).toBeDefined();
  });

  it('should verify step-by-step running balance continuity across transactions', () => {
    const txs: Partial<RawStatementRow>[] = [
      { id: '1', txCode: 'T1', debit: 0, credit: 500000, balanceAfter: 1500000 },
      { id: '2', txCode: 'T2', debit: 200000, credit: 0, balanceAfter: 1300000 },
      { id: '3', txCode: 'T3', debit: 0, credit: 300000, balanceAfter: 1600000 },
    ];

    const res = verifyRunningBalanceContinuity(txs as RawStatementRow[], 1000000);
    expect(res.isContinuous).toBe(true);
  });

  it('should flag broken running balance step', () => {
    const txs: Partial<RawStatementRow>[] = [
      { id: '1', txCode: 'T1', debit: 0, credit: 500000, balanceAfter: 1500000 },
      { id: '2', txCode: 'T2', debit: 200000, credit: 0, balanceAfter: 1250000 }, // Discrepancy of 50k!
    ];

    const res = verifyRunningBalanceContinuity(txs as RawStatementRow[], 1000000);
    expect(res.isContinuous).toBe(false);
    expect(res.brokenRowIndex).toBe(1);
  });

  it('should strictly flag 1 VND discrepancy (zero tolerance)', () => {
    const txs: Partial<RawStatementRow>[] = [
      { id: '1', txCode: 'T1', debit: 0, credit: 500000, balanceAfter: 1500000 },
      { id: '2', txCode: 'T2', debit: 200000, credit: 0, balanceAfter: 1300001 }, // 1 VND discrepancy
    ];

    const res = verifyRunningBalanceContinuity(txs as RawStatementRow[], 1000000);
    expect(res.isContinuous).toBe(false);
    expect(res.brokenRowIndex).toBe(1);
    expect(res.errorReason).toContain('1 VND');
  });
});
