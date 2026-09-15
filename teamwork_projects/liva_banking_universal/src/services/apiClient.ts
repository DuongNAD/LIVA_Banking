/**
 * LIVA Banking Universal — API Client & Network Interceptor
 * Manages JWT Auth Token in localStorage and synchronizes with Backend Server
 */

const TOKEN_KEY = 'liva_auth_token';
const USER_KEY = 'liva_auth_user';

let customApiBaseUrl: string | null = null;

export function setApiBaseUrl(url: string | null): void {
  customApiBaseUrl = url;
}

// Determine API Base URL
// 1. Custom URL if set (e.g. during integration tests with ephemeral ports)
// 2. VITE_API_URL if configured
// 3. http://localhost:3001/api in local dev / test runner
// 4. Relative /api for cloud deployments
export function getApiBaseUrl(): string {
  if (customApiBaseUrl) return customApiBaseUrl;
  if (typeof process !== 'undefined' && process.env?.VITE_API_URL) {
    return process.env.VITE_API_URL;
  }
  if (typeof window !== 'undefined') {
    if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
      return 'http://localhost:3001/api';
    }
    return '/api';
  }
  return 'http://localhost:3001/api';
}

export function getStoredToken(): string | null {
  try {
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem(TOKEN_KEY);
    }
    if (typeof window !== 'undefined' && window.localStorage) {
      return window.localStorage.getItem(TOKEN_KEY);
    }
  } catch {
    return null;
  }
  return null;
}

export function setStoredToken(token: string): void {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(TOKEN_KEY, token);
    } else if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.setItem(TOKEN_KEY, token);
    }
  } catch {
    // Ignore storage errors in restricted contexts
  }
}

export function clearStoredToken(): void {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem(TOKEN_KEY);
      localStorage.removeItem(USER_KEY);
    } else if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.removeItem(TOKEN_KEY);
      window.localStorage.removeItem(USER_KEY);
    }
  } catch {
    // Ignore
  }
}

// Proactively purge any residual decoded user PII from localStorage on load
if (typeof localStorage !== 'undefined') {
  try {
    localStorage.removeItem(USER_KEY);
  } catch {
    // Ignore
  }
} else if (typeof window !== 'undefined' && window.localStorage) {
  try {
    window.localStorage.removeItem(USER_KEY);
  } catch {
    // Ignore
  }
}

export function getStoredUser(): any | null {
  // In-Memory User Security Protocol (Decree 13/2023/ND-CP):
  // Plaintext officer claims are kept strictly in-memory during active sessions.
  return null;
}

export function setStoredUser(_user: any): void {
  // Never persist user PII into browser localStorage. Always purge.
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem(USER_KEY);
    } else if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.removeItem(USER_KEY);
    }
  } catch {
    // Ignore
  }
}

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  isOnline: boolean;
}

/**
 * Robust HTTP client that attaches Bearer token from localStorage
 */
export async function apiRequest<T = any>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const baseUrl = getApiBaseUrl();
  const url = `${baseUrl}${endpoint.startsWith('/') ? '' : '/'}${endpoint}`;
  const token = getStoredToken();

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  try {
    const res = await fetch(url, {
      ...options,
      headers,
    });

    if (res.status === 401) {
      // Token expired or invalid
      clearStoredToken();
      const errData = await res.json().catch(() => ({}));
      return {
        success: false,
        error: errData.error || 'Phiên làm việc đã hết hạn. Vui lòng đăng nhập lại.',
        isOnline: true,
      };
    }

    if (!res.ok) {
      const errData = await res.json().catch(() => ({}));
      const isEndpointMissing = res.status === 404 || res.status === 502 || res.status === 503;
      return {
        success: false,
        error: errData.error || `HTTP error ${res.status}`,
        isOnline: !isEndpointMissing,
      };
    }

    const data = await res.json();
    return {
      success: true,
      data,
      isOnline: true,
    };
  } catch (err: any) {
    // Server unreachable -> Gracefully report offline
    return {
      success: false,
      error: 'Không thể kết nối đến máy chủ Backend (Offline Mode).',
      isOnline: false,
    };
  }
}
