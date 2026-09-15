/**
 * Treasury & Maker-Checker Dual Control Types
 * Adhering to Circular 09/2020/TT-NHNN (Articles 16 & 18)
 * Decree 13/2023/ND-CP Personal Data Protection & Zero Data Egress
 */

export type VoucherStatus =
  | 'DRAFT'
  | 'PENDING_APPROVAL'
  | 'APPROVED'
  | 'REJECTED'
  | 'SETTLED'
  | 'EXPIRED';

export type AuthMethod =
  | 'BIOMETRIC_SIM'
  | 'SMS_OTP_SIM'
  | 'HARDWARE_TOKEN';

export interface PaymentVoucher {
  voucherId: string;
  makerId: string;
  checkerId?: string | null;
  beneficiaryAccount: string;
  beneficiaryBank: string;
  beneficiaryName?: string;
  amountVnd: number;
  purpose: string;
  status: VoucherStatus;
  createdAt: string;
  submittedAt?: string;
  approvedAt?: string;
  rejectedAt?: string;
  settledAt?: string;
  rejectReason?: string;
  authMethod?: AuthMethod;
  hitlToken?: string | null;
  tokenExpiresAt?: string | null;
  signatureHmac?: string | null;
  merkleLeafHash?: string;
}

export interface AuditLogEntry {
  entryIndex: number;
  timestamp: string;
  actor: string;
  event: string;
  payload: any;
  prevHash: string;
  currentHash: string;
}

export interface MerkleProofStep {
  position: 'left' | 'right';
  hash: string;
}

export interface MerkleProof {
  leaf: string;
  root: string;
  index: number;
  proof: MerkleProofStep[];
}

export interface MerkleTreeResult {
  root: string;
  leaves: string[];
}

export type CopilotIntent =
  | 'LIQUIDITY_RUNWAY'
  | 'RECONCILIATION_RATE'
  | 'TOP_EXPENSE'
  | 'AML_SUMMARY'
  | 'GENERAL_ASSISTANCE'
  | 'EXECUTE_MAKER_CHECKER'
  | 'QUERY_CASH_POSITION'
  | 'QUERY_CASHFLOW_RISK'
  | 'QUERY_RECONCILIATION_STATUS'
  | 'QUERY_AML_SURVEILLANCE';

export interface CopilotResponse {
  query: string;
  intent: string;
  answer: string;
  metrics: Record<string, any>;
}

export interface TourStep {
  stepIndex: number;
  id: string;
  title: string;
  durationMs: number;
  speakerScript?: string;
  highlightTarget?: string;
}

export interface GuidedTourState {
  steps: TourStep[];
  getCurrentStep: () => TourStep;
  play: () => void;
  pause: () => void;
  nextStep: () => TourStep | null;
  prevStep: () => TourStep;
  isCompleted: () => boolean;
  isPlaying: () => boolean;
  reset?: () => void;
}
