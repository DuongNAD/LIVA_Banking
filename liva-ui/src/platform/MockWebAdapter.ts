import type {
  IPlatformAdapter,
  PlatformCapabilities,
  BankingOverviewResponse,
  ReconciliationMatrixResponse,
  HitlConfirmationPayload,
  HitlResolutionResponse,
  ComplianceStatusResponse,
  StatementIngestResult,
} from './IPlatformAdapter';
import { logger } from '../utils/logger';

export class MockWebAdapter implements IPlatformAdapter {
  readonly platformName = 'web' as const;
  readonly capabilities: PlatformCapabilities = {
    hasNativeFileSystem: false,
    hasNativeDialogs: false,
    hasHardwareKeystore: false,
    hasWindowControls: false,
    hasProcessControl: false,
    supportsStreamingUpload: true,
  };

  private readonly vaultSecretKeys = new Set<string>();

  constructor() {
    if (typeof document !== 'undefined') {
      document.body.classList.add('web-mock-mode');
    }
  }

  async getWindowSize() {
    return {
      width: typeof window !== 'undefined' ? window.innerWidth : 1280,
      height: typeof window !== 'undefined' ? window.innerHeight : 800,
    };
  }

  async toggleGhostMode(enabled: boolean) {
    logger.debug('[MockWebAdapter]', `Toggle Ghost Mode: ${enabled}`);
  }

  async minimizeToTray() {
    logger.debug('[MockWebAdapter]', 'Minimize to tray requested.');
  }

  async quitApp() {
    logger.debug('[MockWebAdapter]', 'Quit app requested. Closing window.');
    if (typeof window !== 'undefined') {
      window.close();
    }
  }

  async minimize() {
    return this.minimizeToTray();
  }

  async maximize() {
    logger.debug('[MockWebAdapter]', 'Maximize requested.');
  }

  async close() {
    return this.quitApp();
  }

  async hasVaultSecret(key: string) {
    return this.vaultSecretKeys.has(key);
  }

  async storeVaultSecret(key: string, value: string) {
    if (!value) throw new Error('vault secret must not be empty');
    this.vaultSecretKeys.add(key);
    logger.debug('[MockWebAdapter]', `Stored mock vault presence: ${key}`);
  }

  async deleteVaultSecret(key: string) {
    this.vaultSecretKeys.delete(key);
  }

  async hasSecret(key: string): Promise<boolean> {
    return this.hasVaultSecret(key);
  }

  async storeSecret(key: string, value: string): Promise<void> {
    return this.storeVaultSecret(key, value);
  }

  async deleteSecret(key: string): Promise<void> {
    return this.deleteVaultSecret(key);
  }

  onGatewayReady(callback: (port: number, token: string | null) => void) {
    logger.info('[MockWebAdapter]', 'Emulating GATEWAY_READY handshake on port 8002');
    setTimeout(() => {
      callback(8002, null);
    }, 1000);
  }

  async invokeBackend<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T> {
    logger.debug('[MockWebAdapter]', `Invoked command: ${command}`, args);
    return null as unknown as T;
  }

  async getOverview(): Promise<BankingOverviewResponse> {
    return {
      vcb_balance: 1450230000,
      tcb_balance: 890400000,
      bidv_balance: 512000000,
      discrepancy_count: 3,
      matched_count: 1492,
      total_count: 1495,
      matched_ratio: 99.8,
      automatic_count: 1485,
      hitl_count: 7,
    };
  }

  async runReconciliation(): Promise<Record<string, unknown>> {
    return { success: true, processed: 1495, matched: 1492, discrepancies: 3 };
  }

  async getReconciliationMatrix(_filter?: string): Promise<ReconciliationMatrixResponse> {
    return { items: [], total_count: 0, unmatched_count: 0 };
  }

  async resolveHitl(payload: HitlConfirmationPayload): Promise<HitlResolutionResponse> {
    return {
      success: true,
      auditRecord: {
        timestamp: new Date().toISOString(),
        txCode: payload.txId,
        tokenUuid: payload.tokenUuid,
        action: payload.action,
        makerId: payload.makerId || 'maker_mock',
        checkerId: payload.checkerId || 'checker_mock',
        hash: `mock_audit_hash_${Date.now()}`,
      },
    };
  }

  async getComplianceStatus(): Promise<ComplianceStatusResponse> {
    return {
      decree_13_compliant: true,
      circular_09_compliant: true,
      zero_egress_verified: true,
      merkle_root_hash: '0xmock_root_hash',
      pii_scrubbed_count: 0,
    };
  }

  async ingestStatement(
    _fileInput: File | { name: string; size: number; path?: string; file?: File; content?: string | ArrayBuffer },
    onProgress?: (progressPercent: number, stage: string) => void
  ): Promise<StatementIngestResult> {
    onProgress?.(30, 'SCANNING');
    onProgress?.(60, 'EXTRACTING');
    onProgress?.(100, 'COMPLETED');
    return {
      success: true,
      total_transactions: 42,
      opening_balance: 1000000000,
      closing_balance: 1250000000,
    };
  }

  async subscribeEvents(
    _onEvent: (event: { event: string; payload: unknown }) => void
  ): Promise<() => void> {
    return () => {};
  }
}
