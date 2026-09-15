import type { RawStatementRow, LedgerEntry } from './banking';

export type MatchTier = 'TIER1_EXACT' | 'TIER2_FUZZY' | 'TIER3_SPLIT' | 'HITL_QUARANTINE';

export type MatchType = 
  | 'EXACT_1_TO_1' 
  | 'FUZZY_HEURISTIC' 
  | 'COMPOSITE_1_TO_N' 
  | 'SPLIT_N_TO_1' 
  | 'MANUAL_HITL';

export interface ReconciliationMatch {
  matchId: string;
  tier: MatchTier;
  matchType: MatchType;
  bankTransactionIds: string[];
  ledgerEntryIds: string[];
  matchedAmount: number; // Scaled integer (VND)
  feeAmount: number; // Detected interbank wire fee (VND)
  discrepancyAmount: number; // Signed discrepancy (VND)
  confidence: number; // 1.0 (Tier 1), 0.85 - 0.98 (Tier 2), 0.95 - 0.99 (Tier 3), 0.50 (HITL)
  explanation: string;
  timestamp: string; // ISO 8601 string
  status: 'APPROVED' | 'PENDING_HITL' | 'REJECTED';
  hitlToken?: string;
  hitlExpiresAt?: number; // Unix epoch seconds (15-min TTL)
}

export interface HitlQuarantineItem {
  txId: string;
  amount: number;
  reason: string;
  hitlToken: string;
  createdAt: number; // Unix epoch seconds
  expiresAt: number; // Unix epoch seconds
  candidateLedgerIds?: string[];
  rawTransaction: RawStatementRow;
}

export interface ReconciliationSummary {
  totalBankTransactions: number;
  totalLedgerEntries: number;
  matchedCount: number;
  matchRate: number; // Percentage 0 - 100 (e.g. 99.8)
  tier1Count: number;
  tier2Count: number;
  tier3Count: number;
  hitlQuarantineCount: number;
  totalMatchedAmount: number;
  totalFeeDisentangled: number;
  matches: ReconciliationMatch[];
  quarantined: HitlQuarantineItem[];
  unallocatedBankTransactions: RawStatementRow[];
  unallocatedLedgerEntries: LedgerEntry[];
}
