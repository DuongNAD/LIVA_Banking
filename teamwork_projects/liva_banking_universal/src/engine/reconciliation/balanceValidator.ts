/**
 * Mathematical Balance Invariant Validator
 * Strictly verifies the double-entry accounting identity:
 *   Closing Balance = Opening Balance + Total Credits - Total Debits
 * Ensures zero floating-point drift and validates step-by-step running balance continuity.
 */

import type { BalanceInvariantReport, RawStatementRow } from '../../types/banking';

/**
 * Validates macro statement balance integrity.
 * Closing = Opening + TotalCredit - TotalDebit
 */
export function verifyBalanceInvariants(
  openingBalance: number,
  closingBalance: number,
  totalCredit: number,
  totalDebit: number
): BalanceInvariantReport {
  // Use integer rounding to eliminate any potential float representations
  const open = Math.round(openingBalance || 0);
  const close = Math.round(closingBalance || 0);
  const credit = Math.round(totalCredit || 0);
  const debit = Math.round(totalDebit || 0);

  const calculatedClosing = open + credit - debit;
  const discrepancy = close - calculatedClosing;
  const isBalanced = discrepancy === 0;

  let errorReason: string | undefined;
  if (!isBalanced) {
    errorReason = `Macro balance invariant failed: Closing balance (${close.toLocaleString('vi-VN')} VND) differs from calculated balance (${calculatedClosing.toLocaleString('vi-VN')} VND) by ${discrepancy.toLocaleString('vi-VN')} VND.`;
  }

  return {
    isValid: isBalanced,
    isBalanced,
    openingBalance: open,
    totalCredit: credit,
    totalDebit: debit,
    closingBalance: close,
    calculatedClosing,
    discrepancy,
    errorReason,
  };
}

/**
 * Validates step-by-step continuity of running balance across transaction rows.
 */
export function verifyRunningBalanceContinuity(
  transactions: RawStatementRow[],
  openingBalance = 0
): {
  isContinuous: boolean;
  brokenRowIndex?: number;
  expectedBalance?: number;
  actualBalance?: number;
  errorReason?: string;
} {
  let prevBalance = Math.round(openingBalance);

  for (let i = 0; i < transactions.length; i++) {
    const tx = transactions[i];
    const net = tx.credit - tx.debit;
    const expected = prevBalance + net;

    // If transaction explicitly provides balance or balanceAfter
    if (tx.balanceAfter !== undefined && tx.balanceAfter > 0) {
      const actual = Math.round(tx.balanceAfter);
      if (actual !== expected) {
        return {
          isContinuous: false,
          brokenRowIndex: i,
          expectedBalance: expected,
          actualBalance: actual,
          errorReason: `Running balance continuity broken at transaction row ${i + 1} (${tx.txCode}): Expected ${expected.toLocaleString('vi-VN')} VND but recorded ${actual.toLocaleString('vi-VN')} VND (diff: ${(actual - expected).toLocaleString('vi-VN')} VND).`,
        };
      }
      prevBalance = actual;
    } else {
      prevBalance = expected;
    }
  }

  return {
    isContinuous: true,
  };
}
