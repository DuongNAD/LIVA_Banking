export interface PlatformCapabilities {
  hasNativeFileSystem: boolean;
  hasNativeDialogs: boolean;
  hasHardwareKeystore: boolean;
  hasWindowControls: boolean;
  hasProcessControl: boolean;
  supportsStreamingUpload: boolean;
}

export interface BankingOverviewResponse {
  vcb_balance?: number;
  tcb_balance?: number;
  bidv_balance?: number;
  discrepancy_count?: number;
  matched_count?: number;
  total_count?: number;
  matched_ratio?: number;
  automatic_count?: number;
  hitl_count?: number;
  rolling_forecast?: Array<{
    date: string;
    expected_inflow: number;
    expected_outflow: number;
    projected_balance: number;
    is_deficit_risk: boolean;
  }>;
}

export interface ReconciliationMatrixResponse {
  items?: Array<Record<string, unknown>>;
  total_count?: number;
  unmatched_count?: number;
}

export interface HitlConfirmationPayload {
  txId: string;
  tokenUuid: string;
  action: 'ALLOCATE_FEE' | 'MANUAL_MATCH' | 'CREATE_VOUCHER' | 'REJECT' | 'MATCH' | 'ESCALATE' | string;
  targetAccount?: string;
  matchedLedgerId?: string;
  notes?: string;
  makerId?: string;
  checkerId?: string;
}

export interface HitlResolutionResponse {
  success: boolean;
  auditRecord?: {
    timestamp: string;
    txCode: string;
    tokenUuid: string;
    action: string;
    makerId: string;
    checkerId: string;
    hash: string;
  };
}

export interface ComplianceStatusResponse {
  decree_13_compliant: boolean;
  circular_09_compliant: boolean;
  zero_egress_verified: boolean;
  merkle_root_hash?: string;
  pii_scrubbed_count?: number;
}

export interface StatementIngestResult {
  success?: boolean;
  total_transactions?: number;
  opening_balance?: number;
  closing_balance?: number;
  bank_code?: string;
  error?: string;
  [key: string]: unknown;
}

export interface IPlatformAdapter {
  readonly platformName: 'tauri' | 'web';
  readonly capabilities: PlatformCapabilities;

  // Window & Process Lifecycle
  getWindowSize(): Promise<{ width: number; height: number }>;
  toggleGhostMode(enabled: boolean): Promise<void>;
  minimizeToTray(): Promise<void>;
  quitApp(): Promise<void>;
  minimize?(): Promise<void>;
  maximize?(): Promise<void>;
  close?(): Promise<void>;

  // Vault / Keystore Secrets
  hasVaultSecret(key: string): Promise<boolean>;
  storeVaultSecret(key: string, value: string): Promise<void>;
  deleteVaultSecret(key: string): Promise<void>;
  hasSecret?(key: string): Promise<boolean>;
  storeSecret?(key: string, value: string): Promise<void>;
  deleteSecret?(key: string): Promise<void>;

  // Gateway & Events
  onGatewayReady(callback: (port: number, token: string | null) => void): void;
  subscribeEvents?(
    onEvent: (event: { event: string; payload: unknown }) => void,
    onError?: (err: Error) => void
  ): Promise<() => void>;

  // IPC / Generic Invocation
  invokeBackend<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T>;

  // High-Level Banking Domain APIs
  getOverview?(): Promise<BankingOverviewResponse>;
  runReconciliation?(): Promise<Record<string, unknown>>;
  getReconciliationMatrix?(filter?: string): Promise<ReconciliationMatrixResponse>;
  resolveHitl?(payload: HitlConfirmationPayload): Promise<HitlResolutionResponse>;
  getComplianceStatus?(): Promise<ComplianceStatusResponse>;
  ingestStatement?(
    fileInput: File | { name: string; size: number; path?: string; file?: File; content?: string | ArrayBuffer },
    onProgress?: (progressPercent: number, stage: string) => void
  ): Promise<StatementIngestResult>;
}
