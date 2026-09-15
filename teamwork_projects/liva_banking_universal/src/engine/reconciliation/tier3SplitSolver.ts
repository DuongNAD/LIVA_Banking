/**
 * Tier 3: Constraint Split Solver (Subset-Sum) & HITL Quarantine Manager
 * Solves bidirectional composite/split payment matching:
 *   Phase A: 1 Bank Payment -> N Ledger Invoices (Composite settlement)
 *   Phase B: 1 Ledger Invoice -> N Bank Installments (Multi-installment settlement)
 * Bounded branch-and-bound subset-sum search with depth k <= 8 and strict Delta == 0.
 * Residual unallocated transactions fail closed to HITL Quarantine with 15-min UUIDv4 tokens.
 */

import type { RawStatementRow, LedgerEntry } from '../../types/banking';
import type { ReconciliationMatch, HitlQuarantineItem } from '../../types/reconciliation';

export const DEFAULT_MAX_SPLIT_DEPTH = 4;
export const MAX_SUPPORTED_SPLIT_DEPTH = 8;
const HITL_TTL_SECONDS = 900; // 15 minutes

/**
 * Solves exact subset-sum using branch-and-bound with 3 pruning bounds.
 * Returns indices of selected candidates summing exactly to target with Delta == 0.
 */
export function solveExactSubsetSumBnb(
  candidates: { index: number; amount: number }[],
  target: number,
  maxDepth = DEFAULT_MAX_SPLIT_DEPTH
): number[] | null {
  const depthLimit = Math.min(maxDepth, MAX_SUPPORTED_SPLIT_DEPTH);

  const valid = candidates
    .filter((c) => c.amount > 0 && c.amount <= target);

  if (valid.length === 0 || depthLimit === 0) {
    return null;
  }

  // Fast path: single candidate exact match
  for (const c of valid) {
    if (c.amount === target) {
      return [c.index];
    }
  }

  if (depthLimit === 1) return null;

  // Sort ascending by amount for optimal pruning
  valid.sort((a, b) => a.amount - b.amount);

  const n = valid.length;
  const suffixSums = new Array(n + 1).fill(0);
  for (let i = n - 1; i >= 0; i--) {
    suffixSums[i] = suffixSums[i + 1] + valid[i].amount;
  }

  // If sum of all candidates is less than target, impossible
  if (suffixSums[0] < target) {
    return null;
  }

  const selected: number[] = [];

  function recurse(idx: number, currentSum: number): boolean {
    if (currentSum === target) {
      return selected.length > 0;
    }
    if (selected.length >= depthLimit || idx >= valid.length) {
      return false;
    }

    const remainingDepth = depthLimit - selected.length;
    const availableCount = valid.length - idx;

    // Pruning Bound 1 (Overshoot): minimum remaining addition exceeds target
    if (currentSum + valid[idx].amount > target) {
      return false;
    }

    // Pruning Bound 2 (Undershoot): sum of all remaining cannot reach target
    if (currentSum + suffixSums[idx] < target) {
      return false;
    }

    // Pruning Bound 3 (Max depth capacity undershoot): sum of largest remaining_depth cannot reach target
    if (availableCount > remainingDepth) {
      let maxPossible = 0;
      for (let k = valid.length - remainingDepth; k < valid.length; k++) {
        maxPossible += valid[k].amount;
      }
      if (currentSum + maxPossible < target) {
        return false;
      }
    }

    // Branch 1: Include valid[idx]
    selected.push(valid[idx].index);
    if (recurse(idx + 1, currentSum + valid[idx].amount)) {
      return true;
    }

    // Branch 2: Exclude valid[idx] and skip duplicate candidate amounts
    selected.pop();
    let nextIdx = idx + 1;
    while (nextIdx < valid.length && valid[nextIdx].amount === valid[idx].amount) {
      nextIdx++;
    }
    return recurse(nextIdx, currentSum);
  }

  if (recurse(0, 0)) {
    return selected;
  }

  return null;
}

/**
 * Generates a standard UUIDv4 string.
 */
export function generateUuidV4(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

export interface Tier3Result {
  matches: ReconciliationMatch[];
  quarantined: HitlQuarantineItem[];
  unallocatedBankIndices: number[];
  unallocatedLedgerIndices: number[];
}

/**
 * Executes Tier 3 split solving (1:N and N:1) followed by HITL quarantine for residuals.
 */
export function matchTier3(
  bankTxs: RawStatementRow[],
  ledgerEntries: LedgerEntry[],
  unallocatedBankIndices: number[],
  unallocatedLedgerIndices: number[],
  maxDepth = DEFAULT_MAX_SPLIT_DEPTH
): Tier3Result {
  const matches: ReconciliationMatch[] = [];
  const quarantined: HitlQuarantineItem[] = [];

  const matchedBank = new Array(bankTxs.length).fill(false);
  const matchedLedger = new Array(ledgerEntries.length).fill(false);

  let activeBankIndices = [...unallocatedBankIndices];
  let activeLedgerIndices = [...unallocatedLedgerIndices];

  // =========================================================================
  // Phase A: 1 Bank Transaction -> N Ledger Invoices (Composite Settlement)
  // =========================================================================
  for (const bIdx of activeBankIndices) {
    if (matchedBank[bIdx]) continue;
    const tx = bankTxs[bIdx];

    // Filter available ledger candidates with same direction
    const candidates = activeLedgerIndices
      .filter((lIdx) => !matchedLedger[lIdx] && ledgerEntries[lIdx].entryType === tx.txType)
      .map((lIdx) => ({ index: lIdx, amount: ledgerEntries[lIdx].amount }));

    const solution = solveExactSubsetSumBnb(candidates, tx.amount, maxDepth);

    if (solution && solution.length > 1) {
      matchedBank[bIdx] = true;
      for (const lIdx of solution) {
        matchedLedger[lIdx] = true;
      }

      const matchedLedgers = solution.map((lIdx) => ledgerEntries[lIdx]);
      const invoiceRefs = matchedLedgers.map((l) => l.docNo).join(', ');

      matches.push({
        matchId: `match-tier3-1toN-${tx.id}`,
        tier: 'TIER3_SPLIT',
        matchType: 'COMPOSITE_1_TO_N',
        bankTransactionIds: [tx.id],
        ledgerEntryIds: matchedLedgers.map((l) => l.id),
        matchedAmount: tx.amount,
        feeAmount: 0,
        discrepancyAmount: 0,
        confidence: 0.98,
        explanation: `Khớp gộp 1-nhiều (Tier 3 Subset-Sum): 1 giao dịch ngân hàng ${tx.amount.toLocaleString('vi-VN')} VND tất toán ${solution.length} hóa đơn [${invoiceRefs}] (sai số tuyệt đối 0 VND).`,
        timestamp: new Date().toISOString(),
        status: 'APPROVED',
      });
    }
  }

  // Refresh remaining
  activeBankIndices = activeBankIndices.filter((idx) => !matchedBank[idx]);
  activeLedgerIndices = activeLedgerIndices.filter((idx) => !matchedLedger[idx]);

  // =========================================================================
  // Phase B: 1 Ledger Invoice -> N Bank Transactions (Multi-Installments)
  // =========================================================================
  for (const lIdx of activeLedgerIndices) {
    if (matchedLedger[lIdx]) continue;
    const ledger = ledgerEntries[lIdx];

    const candidates = activeBankIndices
      .filter((bIdx) => !matchedBank[bIdx] && bankTxs[bIdx].txType === ledger.entryType)
      .map((bIdx) => ({ index: bIdx, amount: bankTxs[bIdx].amount }));

    const solution = solveExactSubsetSumBnb(candidates, ledger.amount, maxDepth);

    if (solution && solution.length > 1) {
      matchedLedger[lIdx] = true;
      for (const bIdx of solution) {
        matchedBank[bIdx] = true;
      }

      const matchedBankTxs = solution.map((bIdx) => bankTxs[bIdx]);
      const txCodes = matchedBankTxs.map((b) => b.txCode).join(', ');

      matches.push({
        matchId: `match-tier3-Nto1-${ledger.id}`,
        tier: 'TIER3_SPLIT',
        matchType: 'SPLIT_N_TO_1',
        bankTransactionIds: matchedBankTxs.map((b) => b.id),
        ledgerEntryIds: [ledger.id],
        matchedAmount: ledger.amount,
        feeAmount: 0,
        discrepancyAmount: 0,
        confidence: 0.98,
        explanation: `Khớp trả góp nhiều-1 (Tier 3 Subset-Sum): ${solution.length} đợt thanh toán [${txCodes}] thanh toán trọn gói hóa đơn ${ledger.docNo} ${ledger.amount.toLocaleString('vi-VN')} VND (sai số tuyệt đối 0 VND).`,
        timestamp: new Date().toISOString(),
        status: 'APPROVED',
      });
    }
  }

  // Refresh remaining
  const finalRemainingBank = unallocatedBankIndices.filter((idx) => !matchedBank[idx]);
  const finalRemainingLedger = unallocatedLedgerIndices.filter((idx) => !matchedLedger[idx]);

  // =========================================================================
  // HITL Quarantine: Fail-closed routing for residual unallocated transactions
  // =========================================================================
  const nowSecs = Math.floor(Date.now() / 1000);

  for (const bIdx of finalRemainingBank) {
    const tx = bankTxs[bIdx];
    const hitlToken = generateUuidV4();
    const expiresAt = nowSecs + HITL_TTL_SECONDS;

    // Suggest possible remaining ledger candidates
    const candidateLedgerIds = finalRemainingLedger
      .filter((lIdx) => ledgerEntries[lIdx].entryType === tx.txType)
      .map((lIdx) => ledgerEntries[lIdx].id);

    quarantined.push({
      txId: tx.id,
      amount: tx.amount,
      reason: `Chưa thể đối khớp tự động qua 3 tầng (chênh lệch số tiền hoặc không có đối ứng sổ cái). Đưa vào hàng đợi kiểm soát 2 vòng Maker-Checker.`,
      hitlToken,
      createdAt: nowSecs,
      expiresAt,
      candidateLedgerIds: candidateLedgerIds.length > 0 ? candidateLedgerIds : undefined,
      rawTransaction: tx,
    });
  }

  return {
    matches,
    quarantined,
    unallocatedBankIndices: finalRemainingBank,
    unallocatedLedgerIndices: finalRemainingLedger,
  };
}
