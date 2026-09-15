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

export class TauriAdapter implements IPlatformAdapter {
  readonly platformName = 'tauri' as const;
  readonly capabilities: PlatformCapabilities = {
    hasNativeFileSystem: true,
    hasNativeDialogs: true,
    hasHardwareKeystore: true,
    hasWindowControls: true,
    hasProcessControl: true,
    supportsStreamingUpload: false,
  };

  async init(): Promise<void> {
    logger.info('[TauriAdapter]', 'Initialized Desktop Tauri v2 Platform Adapter');
  }

  async getWindowSize() {
    return { width: window.innerWidth, height: window.innerHeight };
  }

  async toggleGhostMode(enabled: boolean) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('toggle_ghost_mode', { enabled });
    } catch (e) {
      logger.warn('[TauriAdapter] toggleGhostMode not available', e);
    }
  }

  async minimizeToTray() {
    try {
      const { Window } = await import('@tauri-apps/api/window');
      const win = Window.getCurrent();
      await win.hide();
    } catch (e) {
      logger.warn('[TauriAdapter] minimizeToTray not available', e);
    }
  }

  async quitApp() {
    try {
      const { exit } = await import('@tauri-apps/plugin-process');
      await exit(0);
    } catch (e) {
      logger.warn('[TauriAdapter] quitApp not available', e);
    }
  }

  async minimize() {
    return this.minimizeToTray();
  }

  async maximize() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().toggleMaximize();
    } catch (e) {
      logger.warn('[TauriAdapter] maximize failed', e);
    }
  }

  async close() {
    return this.quitApp();
  }

  async hasVaultSecret(key: string): Promise<boolean> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('vault_secret_present', { key });
    } catch (e) {
      logger.warn('[TauriAdapter] hasVaultSecret not available', e);
      return false;
    }
  }

  async storeVaultSecret(key: string, value: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('store_vault_secret', { key, value });
    } catch (e) {
      logger.error('[TauriAdapter] storeVaultSecret failed', e);
      throw e;
    }
  }

  async deleteVaultSecret(key: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_vault_secret', { key });
    } catch (e) {
      logger.error('[TauriAdapter] deleteVaultSecret failed', e);
      throw e;
    }
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
    import('@tauri-apps/api/event')
      .then(({ listen }) => {
        listen('gateway-ready', (event: { payload: { port: number; token: string | null } }) => {
          callback(event.payload.port, event.payload.token);
        });
      })
      .catch((e) => {
        logger.warn('[TauriAdapter] Failed to listen to gateway-ready', e);
      });
  }

  async invokeBackend<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(command, args);
    } catch (e) {
      logger.warn(`[TauriAdapter] invokeBackend(${command}) not available`, e);
      return null as unknown as T;
    }
  }

  async getOverview(): Promise<BankingOverviewResponse> {
    const res = await this.invokeBackend<BankingOverviewResponse>('banking_get_overview');
    return res || {};
  }

  async runReconciliation(): Promise<Record<string, unknown>> {
    const res = await this.invokeBackend<Record<string, unknown>>('banking_run_reconciliation');
    return res || {};
  }

  async getReconciliationMatrix(filter?: string): Promise<ReconciliationMatrixResponse> {
    const res = await this.invokeBackend<ReconciliationMatrixResponse>(
      'banking_get_reconciliation_matrix',
      filter ? { filter } : undefined
    );
    return res || { items: [], total_count: 0, unmatched_count: 0 };
  }

  async resolveHitl(payload: HitlConfirmationPayload): Promise<HitlResolutionResponse> {
    const res = await this.invokeBackend<HitlResolutionResponse>(
      'reconciliation_resolve_hitl',
      payload as unknown as Record<string, unknown>
    );
    if (!res) {
      return { success: false };
    }
    return res;
  }

  async getComplianceStatus(): Promise<ComplianceStatusResponse> {
    const res = await this.invokeBackend<ComplianceStatusResponse>('banking_get_compliance_status');
    return (
      res || {
        decree_13_compliant: true,
        circular_09_compliant: true,
        zero_egress_verified: true,
      }
    );
  }

  async ingestStatement(
    fileInput: File | { name: string; size: number; path?: string; file?: File; content?: string | ArrayBuffer },
    onProgress?: (progressPercent: number, stage: string) => void
  ): Promise<StatementIngestResult> {
    onProgress?.(30, 'SCANNING');
    const filePath = 'path' in fileInput ? fileInput.path : (fileInput as File & { path?: string }).path;
    if (!filePath) {
      throw new Error('TauriAdapter requires native file path for statement ingestion');
    }
    onProgress?.(60, 'EXTRACTING');
    const result = await this.invokeBackend<StatementIngestResult>('statement_ingest_file', {
      file_path: filePath,
    });
    onProgress?.(100, 'COMPLETED');
    return result || { success: true };
  }

  async subscribeEvents(
    onEvent: (event: { event: string; payload: unknown }) => void,
    onError?: (err: Error) => void
  ): Promise<() => void> {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      const unlisten = await listen('banking-event', (e: { payload: unknown }) => {
        onEvent({ event: 'banking-event', payload: e.payload });
      });
      return () => {
        unlisten();
      };
    } catch (e) {
      onError?.(e instanceof Error ? e : new Error(String(e)));
      return () => {};
    }
  }
}
