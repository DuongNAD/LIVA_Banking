/**
 * LIVA Treasury Workbench — Dual-Platform IPC & Web Bridge
 * =========================================================
 * Connects Vue 3 Pinia stores to backend engine via IPlatformAdapter.
 * Automatically selects native Tauri v2 IPC on desktop or REST/WebSocket on web.
 */
import { getPlatformAdapter, isTauriEnvironment } from '../platform';

export function isTauri(): boolean {
  return isTauriEnvironment();
}

export async function invokeBackend<T = unknown>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const adapter = getPlatformAdapter();
  return await adapter.invokeBackend<T>(command, args);
}
