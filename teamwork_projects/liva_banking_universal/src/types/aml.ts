/**
 * AML & STR Fraud Surveillance Type Definitions
 * Strict compliance with:
 * - Law on Anti-Money Laundering No. 14/2022/QH15
 * - Decision No. 11/2023/QĐ-TTg (Mandatory high-value reporting threshold >= 400M VND)
 * - Circular No. 09/2023/TT-NHNN & Phụ lục II (Statutory Form STR)
 * - Decree 13/2023/NĐ-CP (Personal Data Protection / Zero Data Egress)
 */

export type AmlAnomalyType =
  | 'STRUCTURING_SMURFING'
  | 'HIGH_VALUE'
  | 'RAPID_PASS_THROUGH'
  | 'NIGHT_VELOCITY'
  | 'WATCHLIST_HIT';

export type AmlSeverity = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';

export interface AmlAlert {
  alertId: string;
  anomalyType: AmlAnomalyType;
  severity: AmlSeverity;
  involvedTransactionIds: string[];
  totalAmount: number; // Non-negative integer (VND)
  detectedAt: string; // ISO 8601 date string
  reasoning: string;
  statutoryRuleRef: string; // Statutory citation
  suggestedStrReport: boolean;
}

export interface AmlScreeningConfig {
  highValueThreshold: number; // default 400_000_000 VND
  criticalValueThreshold: number; // default 1_000_000_000 VND
  structuringWindowHours: number; // default 24 hours
  structuringMinTxCount: number; // default 3
  structuringSubLimit: number; // default 400_000_000 VND
  structuringTotalThreshold: number; // default 400_000_000 VND
  passThroughWindowMinutes: number; // default 30 minutes
  passThroughMinDrainRate: number; // default 0.90 (90%)
  passThroughMaxDrainRate: number; // default 1.05 (105%)
  passThroughMinInflow: number; // default 100_000_000 VND
  nightStartHour: number; // default 23
  nightEndHour: number; // default 5
  nightMinAmount: number; // default 50_000_000 VND
  watchlistKeywords: string[];
}

export const DEFAULT_AML_CONFIG: AmlScreeningConfig = {
  highValueThreshold: 400_000_000,
  criticalValueThreshold: 1_000_000_000,
  structuringWindowHours: 24,
  structuringMinTxCount: 3,
  structuringSubLimit: 400_000_000,
  structuringTotalThreshold: 400_000_000,
  passThroughWindowMinutes: 30,
  passThroughMinDrainRate: 0.90,
  passThroughMaxDrainRate: 1.05,
  passThroughMinInflow: 100_000_000,
  nightStartHour: 23,
  nightEndHour: 5,
  nightMinAmount: 50_000_000,
  watchlistKeywords: ['bet88', 'kubet', 'cá độ', 'crypto', 'usdt', 'binance', 'p2p', 'rút ví'],
};

export interface FormStrData {
  reportingEntity: string;
  reportDate: string; // YYYY-MM-DD
  alertType: AmlAnomalyType;
  severity?: AmlSeverity;
  suspectAccount: string;
  suspectName: string;
  transactionCount: number;
  totalVndAmount: number;
  narrativeSummary: string;
  statutoryRuleRef?: string;
  complianceOfficerNotes: string;
  formTemplate: string;
}

export interface SystemPromptInspectionResult {
  systemPrompt: string;
  parameters: AmlScreeningConfig;
  evalSandbox: (txs: any[]) => AmlAlert[];
}
