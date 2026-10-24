import { TauriAdapter } from './TauriAdapter';
import { WebAdapter } from './WebAdapter';
import { MockWebAdapter } from './MockWebAdapter';
import type { IPlatformAdapter } from './IPlatformAdapter';
import { logger } from '../utils/logger';

let instance: IPlatformAdapter | null = null;

export const BACKEND_URL_STORAGE_KEY = 'LIVA_BACKEND_URL';

export function getConfiguredBackendUrl(): string | null {
  if (typeof window === 'undefined') return null;
  try {
    // 1. Check query parameter ?backend=...
    const urlParams = new URLSearchParams(window.location.search);
    const paramBackend = urlParams.get('backend');
    if (paramBackend && paramBackend.trim()) {
      const clean = paramBackend.trim().replace(/\/+$/, '');
      localStorage.setItem(BACKEND_URL_STORAGE_KEY, clean);
      return clean;
    }
    // 2. Check localStorage
    const saved = localStorage.getItem(BACKEND_URL_STORAGE_KEY);
    if (saved && saved.trim()) {
      return saved.trim().replace(/\/+$/, '');
    }
  } catch {
    // Ignore restricted storage contexts
  }
  return null;
}

export function setConfiguredBackendUrl(url: string | null): void {
  if (typeof window === 'undefined') return;
  try {
    if (url && url.trim()) {
      localStorage.setItem(BACKEND_URL_STORAGE_KEY, url.trim().replace(/\/+$/, ''));
    } else {
      localStorage.removeItem(BACKEND_URL_STORAGE_KEY);
    }
  } catch {
    // Ignore storage errors
  }
}

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

  // 2. Check if a custom live Backend Tunnel URL is configured
  const backendUrl = getConfiguredBackendUrl();
  if (backendUrl) {
    logger.info('[PlatformBridge]', `🔗 Connecting to Live Backend: ${backendUrl}`);
    const apiBase = backendUrl.endsWith('/v1') ? backendUrl : `${backendUrl}/v1`;
    return new WebAdapter(apiBase);
  }

  // 3. Fallback (Chrome/Browser Standalone Demo Mode)
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
