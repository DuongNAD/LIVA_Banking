/**
 * Banking Ingestion & Entity Type Definitions
 * Compliant with Circular 09/2020/TT-NHNN and Decree 13/2023/NĐ-CP.
 * All monetary amounts are 64-bit integer values in VND (0 float drift).
 */

export type InterbankChannelCode = 'CITAD' | 'NAPAS' | 'BILATERAL' | 'SWIFT';

export type BankCode =
  | InterbankChannelCode
  | 'VCB'
  | 'TCB'
  | 'BIDV'
  | 'CTG'
  | 'MBB'
  | 'VBA'
  | 'GENERIC';

export interface LiquidityChannelProfile {
  channelCode: InterbankChannelCode | BankCode;
  bankCode: BankCode;
  channelName: string;
  clearingAuthority: string;
  accountNumber: string;
  accountName: string;
  currency: string;
  balanceVnd: number;
  availableBalanceVnd: number;
  inflowMonthVnd: number;
  outflowMonthVnd: number;
  brandColor: string;
  status: 'ACTIVE' | 'DORMANT' | 'FROZEN';
  lastReconciledDate: string;
  clearingMechanism: string;
  cutoffTime: string;
}

export type BankAccountProfile = LiquidityChannelProfile;

export type TransactionType = 'DEBIT' | 'CREDIT';

export interface RawStatementRow {
  id: string;
  date: string; // ISO date 'YYYY-MM-DD'
  time?: string; // Time string 'HH:MM:SS' or 'HH:MM AM/PM'
  txDate: number; // Unix epoch seconds
  valueDate?: number; // Unix epoch seconds
  txCode: string;
  docRef?: string;
  debit: number; // Non-negative integer (VND)
  credit: number; // Non-negative integer (VND)
  netAmount: number; // Signed integer (VND): negative for debit, positive for credit
  amount: number; // Unsigned absolute integer (VND)
  txType: TransactionType;
  balance: number; // Running balance after transaction (VND)
  balanceAfter?: number;
  narration: string;
  counterparty?: string;
  counterpartyAccount?: string;
  bankCode: BankCode;
  rawRef?: string;
  ftNumber?: string;
  traceId?: string;
}

export interface StatementParseResult {
  bankCode: BankCode;
  bankName?: string;
  accountNumber?: string;
  accountName?: string;
  currency: string;
  openingBalance: number;
  closingBalance: number;
  totalDebit: number;
  totalCredit: number;
  transactions: RawStatementRow[];
  balanceInvariantPassed: boolean;
  balanceDiscrepancy: number;
  rawRowCount: number;
  parseDurationMs?: number;
}

export interface LedgerEntry {
  id: string;
  docNo: string; // e.g. "HD-2026-88"
  entryDate: string; // "YYYY-MM-DD"
  entryTimestamp: number; // Unix epoch seconds
  partnerCode?: string;
  partnerName: string;
  amount: number; // Unsigned integer (VND)
  entryType: TransactionType;
  description: string;
  status: 'UNMATCHED' | 'MATCHED' | 'PENDING_HITL';
}

export interface BalanceInvariantReport {
  isValid: boolean;
  isBalanced: boolean;
  openingBalance: number;
  totalCredit: number;
  totalDebit: number;
  closingBalance: number;
  calculatedClosing: number;
  discrepancy: number;
  errorReason?: string;
}
