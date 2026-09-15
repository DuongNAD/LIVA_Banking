/**
 * Tier 1: O(1) Hash-Map Exact Matcher
 * Matches bank transactions to ledger entries with:
 * - Exact scaled integer amount (0 VND drift)
 * - Strict direction invariance (Debit matches Debit, Credit matches Credit)
 * - Exact normalized document reference (e.g. HD-2026-88 matches HD202688 or HD-88)
 * - Timestamp within +/- 24 hours (86,400s)
 */

import type { RawStatementRow, LedgerEntry } from '../../types/banking';
import type { ReconciliationMatch } from '../../types/reconciliation';

const WINDOW_24H_SECS = 86_400;

/**
 * Normalizes document reference for exact hash comparison.
 * e.g. "HD-2026-88" -> "HD202688"
 *      "HD-000102"  -> "HD102"
 *      "INV-002"    -> "INV2"
 */
export function normalizeDocRef(s: string): string {
  if (!s) return '';
  const clean = s.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();

  if (clean.startsWith('HD') && clean.length > 2) {
    const suffix = clean.slice(2).replace(/^0+/, '');
    return `HD${suffix}`;
  }
  if (clean.startsWith('INV') && clean.length > 3) {
    const suffix = clean.slice(3).replace(/^0+/, '');
    return `INV${suffix}`;
  }

  return clean;
}

/**
 * Extracts candidate reference tokens from bank narration or docRef.
 */
export function extractDocRefCandidates(tx: RawStatementRow): string[] {
  const candidates = new Set<string>();

  if (tx.docRef) {
    const norm = normalizeDocRef(tx.docRef);
    if (norm) candidates.add(norm);
  }
  if (tx.txCode) {
    const norm = normalizeDocRef(tx.txCode);
    if (norm) candidates.add(norm);
  }

  // Scan narration for invoice patterns (HD..., INV..., PC..., PT...)
  const tokens = tx.narration.split(/[^a-zA-Z0-9\-_/]/);
  for (const token of tokens) {
    const clean = token.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();
    if (
      (clean.startsWith('HD') && clean.length >= 3) ||
      (clean.startsWith('INV') && clean.length >= 4) ||
      (clean.startsWith('PC') && clean.length >= 3) ||
      (clean.startsWith('PT') && clean.length >= 3)
    ) {
      candidates.add(normalizeDocRef(clean));
    }
  }

  // Also check for numeric suffix matching (e.g. "hd 88" -> "HD88")
  const numMatches = tx.narration.matchAll(/\b(?:hd|inv|hoa\s*don|so)\s*[:#\-]?\s*(\d+)\b/gi);
  for (const m of numMatches) {
    if (m[1]) {
      candidates.add(`HD${m[1].replace(/^0+/, '')}`);
      candidates.add(`INV${m[1].replace(/^0+/, '')}`);
    }
  }

  return Array.from(candidates);
}

export interface Tier1Result {
  matches: ReconciliationMatch[];
  unallocatedBankIndices: number[];
  unallocatedLedgerIndices: number[];
}

/**
 * Executes Tier 1 exact matching.
 */
export function matchTier1(
  bankTxs: RawStatementRow[],
  ledgerEntries: LedgerEntry[]
): Tier1Result {
  const matches: ReconciliationMatch[] = [];
  const matchedBank = new Array(bankTxs.length).fill(false);
  const matchedLedger = new Array(ledgerEntries.length).fill(false);

  // 1. Build O(1) hash map for ledger entries:
  // Key: `${amount}_${entryType}_${normalizedDocNo}` -> array of ledger indices
  const ledgerIndex = new Map<string, number[]>();

  for (let lIdx = 0; lIdx < ledgerEntries.length; lIdx++) {
    const entry = ledgerEntries[lIdx];
    const normRef = normalizeDocRef(entry.docNo);
    const key = `${entry.amount}_${entry.entryType}_${normRef}`;

    if (!ledgerIndex.has(key)) {
      ledgerIndex.set(key, []);
    }
    ledgerIndex.get(key)!.push(lIdx);

    // Also index without year if format is HD-YYYY-XX (e.g. HD-2026-88 -> also index as HD88)
    const yearMatch = normRef.match(/^(HD|INV)(?:20\d{2})(\d+)$/);
    if (yearMatch) {
      const shortKey = `${entry.amount}_${entry.entryType}_${yearMatch[1]}${yearMatch[2]}`;
      if (!ledgerIndex.has(shortKey)) {
        ledgerIndex.set(shortKey, []);
      }
      ledgerIndex.get(shortKey)!.push(lIdx);
    }
  }

  // 2. O(1) Probe for each bank transaction
  for (let bIdx = 0; bIdx < bankTxs.length; bIdx++) {
    const tx = bankTxs[bIdx];
    const candidates = extractDocRefCandidates(tx);

    let matchedLIdx = -1;

    for (const ref of candidates) {
      const key = `${tx.amount}_${tx.txType}_${ref}`;
      const candidateList = ledgerIndex.get(key);

      if (candidateList && candidateList.length > 0) {
        for (const lIdx of candidateList) {
          if (matchedLedger[lIdx]) continue;

          const ledger = ledgerEntries[lIdx];
          // Time window check: |t_bank - t_ledger| <= 24h
          const timeDiff = Math.abs(tx.txDate - ledger.entryTimestamp);
          // If dates match in YYYY-MM-DD or within 24h window
          if (timeDiff <= WINDOW_24H_SECS || tx.date === ledger.entryDate) {
            matchedLIdx = lIdx;
            break;
          }
        }
      }

      if (matchedLIdx >= 0) break;
    }

    if (matchedLIdx >= 0) {
      matchedBank[bIdx] = true;
      matchedLedger[matchedLIdx] = true;
      const ledger = ledgerEntries[matchedLIdx];

      matches.push({
        matchId: `match-tier1-${tx.id}-${ledger.id}`,
        tier: 'TIER1_EXACT',
        matchType: 'EXACT_1_TO_1',
        bankTransactionIds: [tx.id],
        ledgerEntryIds: [ledger.id],
        matchedAmount: tx.amount,
        feeAmount: 0,
        discrepancyAmount: 0,
        confidence: 1.0,
        explanation: `Khớp chính xác 1:1 theo mã chứng từ ${ledger.docNo} và số tiền ${tx.amount.toLocaleString('vi-VN')} VND (chênh lệch 0 VND).`,
        timestamp: new Date().toISOString(),
        status: 'APPROVED',
      });
    }
  }

  const unallocatedBankIndices = bankTxs
    .map((_, idx) => idx)
    .filter((idx) => !matchedBank[idx]);

  const unallocatedLedgerIndices = ledgerEntries
    .map((_, idx) => idx)
    .filter((idx) => !matchedLedger[idx]);

  return {
    matches,
    unallocatedBankIndices,
    unallocatedLedgerIndices,
  };
}
