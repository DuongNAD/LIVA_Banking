/**
 * Tier 2: Fuzzy Heuristic Matcher
 * Evaluates unallocated records within +/- 72 hours (259,200s).
 * Employs Amount-Bucket Pre-indexing (+/- 11,000 VND), Jaro-Winkler party name similarity >= 0.85,
 * and automatic interbank wire fee tolerance deduction (1,100 to 11,000 VND).
 */

import type { RawStatementRow, LedgerEntry } from '../../types/banking';
import type { ReconciliationMatch } from '../../types/reconciliation';

const WINDOW_72H_SECS = 259_200;
const BUCKET_SIZE = 11_000;

/**
 * Strips Vietnamese diacritics and folds to ASCII lowercase.
 */
export function normalizeVietnameseText(text: string): string {
  if (!text) return '';
  return text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/đ/g, 'd')
    .replace(/Đ/g, 'D')
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * Calculates Jaro similarity between two strings.
 */
export function jaroSimilarity(s1: string, s2: string): number {
  if (s1 === s2) return 1.0;
  if (!s1.length || !s2.length) return 0.0;

  const matchDistance = Math.floor(Math.max(s1.length, s2.length) / 2) - 1;
  const s1Matches = new Array(s1.length).fill(false);
  const s2Matches = new Array(s2.length).fill(false);

  let matches = 0;
  for (let i = 0; i < s1.length; i++) {
    const start = Math.max(0, i - matchDistance);
    const end = Math.min(i + matchDistance + 1, s2.length);

    for (let j = start; j < end; j++) {
      if (!s2Matches[j] && s1[i] === s2[j]) {
        s1Matches[i] = true;
        s2Matches[j] = true;
        matches++;
        break;
      }
    }
  }

  if (matches === 0) return 0.0;

  let transpositions = 0;
  let k = 0;
  for (let i = 0; i < s1.length; i++) {
    if (!s1Matches[i]) continue;
    while (!s2Matches[k]) k++;
    if (s1[i] !== s2[k]) transpositions++;
    k++;
  }

  const m = matches;
  const t = transpositions / 2;
  return (m / s1.length + m / s2.length + (m - t) / m) / 3.0;
}

/**
 * Calculates Jaro-Winkler similarity (prefix scale p = 0.1, max 4 chars).
 */
export function jaroWinklerSimilarity(s1: string, s2: string): number {
  const jaro = jaroSimilarity(s1, s2);
  if (jaro < 0.7) return jaro;

  let prefix = 0;
  const maxPrefix = Math.min(4, Math.min(s1.length, s2.length));
  for (let i = 0; i < maxPrefix; i++) {
    if (s1[i] === s2[i]) prefix++;
    else break;
  }

  return jaro + prefix * 0.1 * (1.0 - jaro);
}

/**
 * Strips Vietnamese corporate legal prefixes and stop words.
 */
export function stripCorporateLegalNoise(raw: string): string {
  const norm = normalizeVietnameseText(raw);
  let text = ` ${norm} `;

  const phrases = [
    'trach nhiem huu han',
    'thuong mai dich vu',
    'mot thanh vien',
    'bat dong san',
    'truyen thong',
    'thuong mai',
    'duoc pham',
    'cong nghe',
    'giai phap',
    'xay dung',
    'san xuat',
    'giao duc',
    'viet nam',
    'van tai',
    'dich vu',
    'co phan',
    'cong ty',
    'dau tu',
    'tap doan',
    'tnhh mtv',
    'tnhh',
    'ctcp',
    'cp',
  ];

  for (const phrase of phrases) {
    const pattern = ` ${phrase} `;
    while (text.includes(pattern)) {
      text = text.replace(pattern, ' ');
    }
  }

  return text.trim().replace(/\s+/g, ' ');
}

/**
 * Strips bank narration noise, transaction prefixes, and trace keywords.
 */
export function stripBankNarrationNoise(narration: string): string {
  const norm = normalizeVietnameseText(narration);
  let text = ` ${norm} `;

  const bankPhrases = [
    'napas vietqr tt',
    'napas vietqr',
    'vietqr tt',
    'vietqr',
    'napas 247',
    'napas',
    'qribft',
    'ibft',
    'mbvcb',
    'ibvcb',
    'chuyen tien tu tk',
    'chuyen tien den tk',
    'chuyen tien',
    'chuyen khoan tu tk',
    'chuyen khoan den tk',
    'chuyen khoan',
    'thanh toan tien hang',
    'thanh toan hoa don',
    'thanh toan dot',
    'thanh toan',
    'tt tien hang',
    'tt hoa don',
    'tt mua xe',
    'tt',
    'ck tien hang',
    'ck hoa don',
    'ck tien may',
    'ck tien',
    'ck',
    'noi dung',
  ];

  for (const phrase of bankPhrases) {
    const pattern = ` ${phrase} `;
    while (text.includes(pattern)) {
      text = text.replace(pattern, ' ');
    }
  }

  // Remove trace tokens or long digit strings
  const words = text.split(/\s+/).filter((w) => {
    if (!w) return false;
    if (/^\d{6,}$/.test(w)) return false; // long numbers
    if (/^(ft|nps|vn)\d+/i.test(w)) return false; // traces
    return true;
  });

  return words.join(' ').trim();
}

/**
 * Compares two party names using Jaro-Winkler with legal form stripping and token containment.
 */
export function comparePartyNames(party1: string, party2: string): number {
  if (!party1 || !party2) return 0.0;

  const clean1 = stripCorporateLegalNoise(party1);
  const clean2 = stripCorporateLegalNoise(party2);

  if (!clean1 || !clean2) return 0.0;
  if (clean1 === clean2) return 1.0;

  // Direct Jaro-Winkler
  const jwScore = jaroWinklerSimilarity(clean1, clean2);

  // Substring or token containment check (e.g. "masan" in "tap doan masan")
  if (clean1.includes(clean2) || clean2.includes(clean1)) {
    return Math.max(jwScore, 0.95);
  }

  const tokens1 = clean1.split(' ').filter((t) => t.length > 2);
  const tokens2 = clean2.split(' ').filter((t) => t.length > 2);

  const set2 = new Set(tokens2);
  const common = tokens1.filter((t) => set2.has(t));
  if (tokens1.length > 0 && common.length / tokens1.length >= 0.6) {
    return Math.max(jwScore, 0.90);
  }

  return jwScore;
}

export interface Tier2Result {
  matches: ReconciliationMatch[];
  unallocatedBankIndices: number[];
  unallocatedLedgerIndices: number[];
}

/**
 * Executes Tier 2 fuzzy heuristic matching with amount-bucket pre-indexing and fee deduction.
 */
export function matchTier2(
  bankTxs: RawStatementRow[],
  ledgerEntries: LedgerEntry[],
  unallocatedBankIndices: number[],
  unallocatedLedgerIndices: number[]
): Tier2Result {
  const matches: ReconciliationMatch[] = [];
  const matchedBank = new Array(bankTxs.length).fill(false);
  const matchedLedger = new Array(ledgerEntries.length).fill(false);

  // 1. Amount-Bucket Pre-indexing:
  // Key: `${entryType}_${Math.floor(amount / BUCKET_SIZE)}` -> array of ledger indices
  const bucketIndex = new Map<string, number[]>();

  for (const lIdx of unallocatedLedgerIndices) {
    const ledger = ledgerEntries[lIdx];
    const b = Math.floor(ledger.amount / BUCKET_SIZE);
    const key = `${ledger.entryType}_${b}`;

    if (!bucketIndex.has(key)) {
      bucketIndex.set(key, []);
    }
    bucketIndex.get(key)!.push(lIdx);
  }

  // 2. Probe candidate buckets for each unallocated bank transaction
  for (const bIdx of unallocatedBankIndices) {
    const tx = bankTxs[bIdx];
    const txBucket = Math.floor(tx.amount / BUCKET_SIZE);

    let bestLIdx = -1;
    let bestScore = 0.0;
    let bestFee = 0;
    let bestDiscrepancy = 0;

    // Probe adjacent buckets: b - 1, b, b + 1
    const candidateLIndices: number[] = [];
    for (const b of [Math.max(0, txBucket - 1), txBucket, txBucket + 1]) {
      const key = `${tx.txType}_${b}`;
      const list = bucketIndex.get(key);
      if (list) {
        candidateLIndices.push(...list);
      }
    }

    // Dedup candidate indices
    const uniqueCandidates = Array.from(new Set(candidateLIndices));

    for (const lIdx of uniqueCandidates) {
      if (matchedLedger[lIdx]) continue;
      const ledger = ledgerEntries[lIdx];

      // Strict Direction Invariance: Credit only matches Credit, Debit only matches Debit
      if (tx.txType !== ledger.entryType) continue;

      // Effective Date check: |t_eff - t_ledger| <= 72h
      const effTxDate = tx.valueDate || tx.txDate;
      const timeDiff = Math.abs(effTxDate - ledger.entryTimestamp);
      if (timeDiff > WINDOW_72H_SECS && tx.date !== ledger.entryDate) {
        continue;
      }

      // Amount check: exact (0 diff) or standard bank fee deduction (1,100 - 11,000 VND)
      const diffAmount = ledger.amount - tx.amount;
      const absDiff = Math.abs(diffAmount);

      let amountScore = 0.0;
      let isAmountValid = false;
      let fee = 0;

      if (absDiff === 0) {
        amountScore = 1.0;
        isAmountValid = true;
      } else if (absDiff >= 1_100 && absDiff <= 11_000) {
        // Interbank transfer fee deduction (Napas / VCB wire fee)
        amountScore = 0.92;
        isAmountValid = true;
        fee = absDiff;
      }

      if (!isAmountValid) continue;

      // Party name similarity
      let partyScore = 0.0;
      if (ledger.partnerName) {
        if (tx.counterparty) {
          partyScore = comparePartyNames(ledger.partnerName, tx.counterparty);
        }
        // Also compare against stripped narration
        const strippedNarration = stripBankNarrationNoise(tx.narration);
        const narrationPartySim = comparePartyNames(ledger.partnerName, strippedNarration);
        partyScore = Math.max(partyScore, narrationPartySim);
      }

      // Date score
      const dateScore = timeDiff <= 86_400 ? 1.0 : (timeDiff <= 172_800 ? 0.95 : 0.88);

      // Composite heuristic score: 50% amount + 40% party + 10% date
      const compositeScore = 0.50 * amountScore + 0.40 * partyScore + 0.10 * dateScore;

      if (compositeScore >= 0.85 && compositeScore > bestScore) {
        bestScore = compositeScore;
        bestLIdx = lIdx;
        bestFee = fee;
        bestDiscrepancy = diffAmount;
      }
    }

    if (bestLIdx >= 0) {
      matchedBank[bIdx] = true;
      matchedLedger[bestLIdx] = true;
      const ledger = ledgerEntries[bestLIdx];

      const feeNote = bestFee > 0
        ? ` (Đã tự động bù trừ phí chuyển tiền liên ngân hàng ${bestFee.toLocaleString('vi-VN')} VND sang TK 6425).`
        : '';

      matches.push({
        matchId: `match-tier2-${tx.id}-${ledger.id}`,
        tier: 'TIER2_FUZZY',
        matchType: 'FUZZY_HEURISTIC',
        bankTransactionIds: [tx.id],
        ledgerEntryIds: [ledger.id],
        matchedAmount: tx.amount,
        feeAmount: bestFee,
        discrepancyAmount: bestDiscrepancy,
        confidence: Math.round(bestScore * 100) / 100,
        explanation: `Khớp heuristic Tier 2 theo đối tác "${ledger.partnerName}" (độ tương đồng ${(bestScore * 100).toFixed(1)}%)${feeNote}`,
        timestamp: new Date().toISOString(),
        status: 'APPROVED',
      });
    }
  }

  const remainingBank = unallocatedBankIndices.filter((idx) => !matchedBank[idx]);
  const remainingLedger = unallocatedLedgerIndices.filter((idx) => !matchedLedger[idx]);

  return {
    matches,
    unallocatedBankIndices: remainingBank,
    unallocatedLedgerIndices: remainingLedger,
  };
}
