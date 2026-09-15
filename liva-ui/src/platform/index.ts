import { TauriAdapter } from './TauriAdapter';
import { MockWebAdapter } from './MockWebAdapter';
import type { IPlatformAdapter } from './IPlatformAdapter';
import { logger } from '../utils/logger';

let instance: IPlatformAdapter | null = null;

export function isTauriEnvironment(): boolean {
  return (
    typeof window !== 'undefined' &&
    Boolean((window as unknown as Record<string, unknown>).__TAURI_INTERNALS__)
  );
}

export function detectPlatform(): IPlatformAdapter {
  // 1. Detect Tauri
  if (isTauriEnvironment()) {
    logger.info('[PlatformBridge]', '🚀 Detected Tauri Environment');
    return new TauriAdapter();
  }

  // 2. Fallback (Chrome/Browser Dev Mode)
  logger.info('[PlatformBridge]', '🌐 Detected Browser Environment (Mock Mode)');
  return new MockWebAdapter();
}

export function getPlatformAdapter(): IPlatformAdapter {
  if (!instance) {
    instance = detectPlatform();
  }
  return instance;
}

export function setPlatformAdapter(adapter: IPlatformAdapter): void {
  instance = adapter;
}

export * from './IPlatformAdapter';
export * from './TauriAdapter';
export * from './WebAdapter';
export * from './MockWebAdapter';
