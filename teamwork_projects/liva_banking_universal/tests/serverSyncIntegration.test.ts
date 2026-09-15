/**
 * LIVA Banking Universal — Server, JWT Auth & Real-Time Sync Integration Tests (Milestone M3)
 * Testing Features F11 (Server Expansion), F12 (SSE Sync), F13 (JWT Auth),
 * F14 (Multi-Machine Real-Time Sync), and F15 (Dual-Mode Resilience).
 * Adhering to Circular 09/2020/TT-NHNN, Decree 13/2023/ND-CP Zero Data Egress.
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import http from 'node:http';
import {
  server,
  USERS,
  verifyJwt,
  broadcastEvent,
} from '../server/server.mjs';
import {
  apiRequest,
  setApiBaseUrl,
  setStoredToken,
} from '../src/services/apiClient';
import { useAuthStore } from '../src/stores/authStore';
import { useTreasuryStore } from '../src/stores/treasuryStore';

// In-memory mock localStorage
const storageMap = new Map<string, string>();
const mockLocalStorage = {
  getItem: (k: string) => storageMap.get(k) ?? null,
  setItem: (k: string, v: string) => storageMap.set(k, String(v)),
  removeItem: (k: string) => storageMap.delete(k),
  clear: () => storageMap.clear(),
  get length() {
    return storageMap.size;
  },
  key: (i: number) => Array.from(storageMap.keys())[i] ?? null,
};

(globalThis as any).localStorage = mockLocalStorage;
(globalThis as any).window = {
  location: { hostname: 'localhost' },
  localStorage: mockLocalStorage,
};

let serverPort: number;
let serverUrl: string;

describe('Milestone M3: Backend Server, JWT Auth & Real-Time Sync Integration', () => {
  beforeAll(async () => {
    // Start backend server on an ephemeral OS port to prevent port conflicts
    await new Promise<void>((resolve) => {
      server.listen(0, () => {
        const addr = server.address();
        if (typeof addr === 'object' && addr) {
          serverPort = addr.port;
          serverUrl = `http://127.0.0.1:${serverPort}`;
          setApiBaseUrl(`${serverUrl}/api`);
        }
        resolve();
      });
    });
  });

  afterAll(async () => {
    // Reset API client base URL and close server
    setApiBaseUrl(null);
    await new Promise<void>((resolve) => {
      server.close(() => resolve());
    });
  });

  beforeEach(() => {
    setActivePinia(createPinia());
    mockLocalStorage.clear();
  });

  // ============================================================================
  // 1. Feature F11: Backend Server Expansion & JWT Auth Roles
  // ============================================================================
  describe('Feature F11: Backend Server Expansion & 4-Role JWT Authentication', () => {
    it('verifies pre-seeded user database contains all 4 commercial bank staff roles', () => {
      const roles = USERS.map((u: any) => u.role);
      expect(roles).toContain('MAKER');
      expect(roles).toContain('CHECKER');
      expect(roles).toContain('AML');
      expect(roles).toContain('TREASURY');

      const maker = USERS.find((u: any) => u.role === 'MAKER');
      expect(maker.email).toBe('maker@livabanking.vn');
      expect(maker.officerId).toBe('OPR-77092');
      expect(maker.branchCode).toBe('HO-HN-001');

      const checker = USERS.find((u: any) => u.role === 'CHECKER');
      expect(checker.email).toBe('checker@livabanking.vn');
      expect(checker.officerId).toBe('SUP-88214');
      expect(checker.branchCode).toBe('HO-HN-001');

      const aml = USERS.find((u: any) => u.role === 'AML');
      expect(aml.email).toBe('aml@livabanking.vn');
      expect(aml.officerId).toBe('CMP-99015');
      expect(aml.branchCode).toBe('HO-HN-001');

      const treasury = USERS.find((u: any) => u.role === 'TREASURY');
      expect(treasury.email).toBe('treasury@livabanking.vn');
      expect(treasury.officerId).toBe('TRZ-55038');
      expect(treasury.branchCode).toBe('HO-HN-001');
    });

    it('authenticates Maker (usr_maker_01) and issues JWT containing officerId and branchCode', async () => {
      const res = await apiRequest('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email: 'maker@livabanking.vn', password: 'maker123' }),
      });

      expect(res.success).toBe(true);
      expect(res.data?.token).toBeDefined();
      expect(res.data?.user?.role).toBe('MAKER');
      expect(res.data?.user?.officerId).toBe('OPR-77092');
      expect(res.data?.user?.branchCode).toBe('HO-HN-001');

      const decoded = verifyJwt(res.data.token);
      expect(decoded).not.toBeNull();
      expect(decoded.sub).toBe('usr_maker_01');
      expect(decoded.role).toBe('MAKER');
      expect(decoded.officerId).toBe('OPR-77092');
      expect(decoded.branchCode).toBe('HO-HN-001');
      expect(decoded.exp).toBeGreaterThan(Math.floor(Date.now() / 1000));
    });

    it('authenticates Checker (usr_checker_01) with SUP-88214', async () => {
      const res = await apiRequest('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email: 'checker@livabanking.vn', password: 'checker123' }),
      });

      expect(res.success).toBe(true);
      expect(res.data?.user?.role).toBe('CHECKER');
      expect(res.data?.user?.officerId).toBe('SUP-88214');

      const decoded = verifyJwt(res.data.token);
      expect(decoded.role).toBe('CHECKER');
      expect(decoded.officerId).toBe('SUP-88214');
    });

    it('authenticates AML Specialist (usr_aml_01) with CMP-99015', async () => {
      const res = await apiRequest('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email: 'aml@livabanking.vn', password: 'aml123' }),
      });

      expect(res.success).toBe(true);
      expect(res.data?.user?.role).toBe('AML');
      expect(res.data?.user?.officerId).toBe('CMP-99015');

      const decoded = verifyJwt(res.data.token);
      expect(decoded.role).toBe('AML');
      expect(decoded.officerId).toBe('CMP-99015');
    });

    it('authenticates Treasury Officer (usr_treasury_01) with TRZ-55038', async () => {
      const res = await apiRequest('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email: 'treasury@livabanking.vn', password: 'treasury123' }),
      });

      expect(res.success).toBe(true);
      expect(res.data?.user?.role).toBe('TREASURY');
      expect(res.data?.user?.officerId).toBe('TRZ-55038');

      const decoded = verifyJwt(res.data.token);
      expect(decoded.role).toBe('TREASURY');
      expect(decoded.officerId).toBe('TRZ-55038');
    });

    it('rejects invalid password with 401 Unauthorized', async () => {
      const res = await apiRequest('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email: 'maker@livabanking.vn', password: 'wrongpassword' }),
      });

      expect(res.success).toBe(false);
      expect(res.error).toContain('Email hoặc mật khẩu không chính xác');
    });

    it('validates OPTIONS CORS pre-flight headers', async () => {
      const optionsRes = await fetch(`${serverUrl}/api/auth/login`, {
        method: 'OPTIONS',
      });

      expect(optionsRes.status).toBe(204);
      expect(optionsRes.headers.get('access-control-allow-origin')).toBe('*');
      expect(optionsRes.headers.get('access-control-allow-methods')).toContain('POST');
      expect(optionsRes.headers.get('access-control-allow-headers')).toContain('Authorization');
      expect(optionsRes.headers.get('access-control-allow-headers')).toContain('Content-Type');
    });
  });

  // ============================================================================
  // 2. Feature F12: Server-Sent Events (SSE) Sync Channel
  // ============================================================================
  describe('Feature F12: Server-Sent Events (SSE) Real-Time Channel', () => {
    it('establishes text/event-stream connection at /api/sync/events', async () => {
      const client = await new Promise<http.IncomingMessage>((resolve, reject) => {
        const req = http.get(`${serverUrl}/api/sync/events`, (res) => {
          resolve(res);
        });
        req.on('error', reject);
      });

      expect(client.statusCode).toBe(200);
      expect(client.headers['content-type']).toBe('text/event-stream');
      expect(client.headers['cache-control']).toBe('no-cache');
      expect(client.headers['connection']).toBe('keep-alive');

      client.destroy();
    });

    it('broadcasts voucher:created event to connected SSE subscribers', async () => {
      let receivedEvent = '';
      let receivedData: any = null;

      const client = await new Promise<http.IncomingMessage>((resolve) => {
        http.get(`${serverUrl}/api/sync/events`, (res) => {
          res.on('data', (chunk) => {
            const str = chunk.toString();
            if (str.includes('event: voucher:created')) {
              receivedEvent = 'voucher:created';
              const match = str.match(/data: (.*)\n\n/);
              if (match) {
                try {
                  receivedData = JSON.parse(match[1]);
                } catch {}
              }
            }
          });
          resolve(res);
        });
      });

      // Trigger a broadcast
      const testVoucher = {
        voucherId: 'vch_test_sse_01',
        amountVnd: 75_000_000,
        purpose: 'Thanh toan hop dong he thong',
        status: 'PENDING_APPROVAL',
      };
      broadcastEvent('voucher:created', { voucher: testVoucher });

      // Allow small async propagation
      await new Promise((r) => setTimeout(r, 50));

      expect(receivedEvent).toBe('voucher:created');
      expect(receivedData?.voucher?.voucherId).toBe('vch_test_sse_01');
      expect(receivedData?.voucher?.amountVnd).toBe(75_000_000);

      client.destroy();
    });

    it('broadcasts voucher:approved and voucher:rejected events with cryptographic Merkle proof', async () => {
      const receivedMessages: string[] = [];

      const client = await new Promise<http.IncomingMessage>((resolve) => {
        http.get(`${serverUrl}/api/sync/events`, (res) => {
          res.on('data', (chunk) => {
            receivedMessages.push(chunk.toString());
          });
          resolve(res);
        });
      });

      broadcastEvent('voucher:approved', {
        voucherId: 'vch_test_sse_01',
        checkerId: 'usr_checker_01',
        merkleHash: 'abc123merklehash',
      });

      broadcastEvent('voucher:rejected', {
        voucherId: 'vch_test_sse_02',
        checkerId: 'usr_checker_01',
        remarks: 'Sai lệch số tài khoản thụ hưởng',
      });

      await new Promise((r) => setTimeout(r, 50));

      const allOutput = receivedMessages.join('');
      expect(allOutput).toContain('event: voucher:approved');
      expect(allOutput).toContain('abc123merklehash');
      expect(allOutput).toContain('event: voucher:rejected');
      expect(allOutput).toContain('Sai lệch số tài khoản thụ hưởng');

      client.destroy();
    });
  });

  // ============================================================================
  // 3. Feature F13: Client JWT Auth & Bearer Dispatch
  // ============================================================================
  describe('Feature F13: Client JWT Storage & Bearer Token Interceptor', () => {
    it('stores JWT in localStorage and automatically dispatches Authorization: Bearer <token>', async () => {
      const auth = useAuthStore();
      const loginSuccess = await auth.login('maker@livabanking.vn', 'maker123');

      expect(loginSuccess).toBe(true);
      expect(auth.isAuthenticated).toBe(true);
      expect(auth.isServerOnline).toBe(true);

      const storedToken = mockLocalStorage.getItem('liva_auth_token');
      expect(storedToken).toBeDefined();
      expect(storedToken).toBe(auth.token);

      // Verify /api/auth/me receives and validates the Bearer token
      const meRes = await apiRequest('/auth/me');
      expect(meRes.success).toBe(true);
      expect(meRes.data?.user?.role).toBe('MAKER');
      expect(meRes.data?.user?.officerId).toBe('OPR-77092');
    });

    it('switches role to Checker and sends requests with Checker Bearer token', async () => {
      const auth = useAuthStore();
      await auth.switchRole('CHECKER');

      expect(auth.isChecker).toBe(true);
      expect(auth.currentOfficerId).toBe('SUP-88214');

      const meRes = await apiRequest('/auth/me');
      expect(meRes.success).toBe(true);
      expect(meRes.data?.user?.role).toBe('CHECKER');
    });

    it('switches role to AML and Treasury desks seamlessly', async () => {
      const auth = useAuthStore();

      await auth.switchRole('AML');
      expect(auth.isAml).toBe(true);
      expect(auth.currentOfficerId).toBe('CMP-99015');

      await auth.switchRole('TREASURY');
      expect(auth.isTreasury).toBe(true);
      expect(auth.currentOfficerId).toBe('TRZ-55038');
    });
  });

  // ============================================================================
  // 4. Feature F14: Multi-Machine Real-Time Sync Simulation
  // ============================================================================
  describe('Feature F14: Multi-Machine Real-Time Synchronization', () => {
    it('Maker creates voucher on Machine A -> Checker approves on Machine B via server API', async () => {
      // Machine A: Maker logs in
      setStoredToken('');
      const authA = useAuthStore();
      await authA.login('maker@livabanking.vn', 'maker123');
      const makerToken = authA.token!;

      // Machine A creates voucher via POST /api/treasury/vouchers
      const createRes = await apiRequest('/treasury/vouchers', {
        method: 'POST',
        headers: { Authorization: `Bearer ${makerToken}` },
        body: JSON.stringify({
          voucherId: 'vch_interbank_multi_01',
          targetAccount: '01-CITAD-SBV',
          targetBeneficiary: 'SỞ GIAO DỊCH NHNN',
          targetBank: 'CITAD_SBV',
          amount: 880_000_000,
          purpose: 'Điều chuyển thanh khoản phiên giao dịch chiều',
        }),
      });

      expect(createRes.success).toBe(true);
      expect(createRes.data?.voucher?.status).toBe('PENDING_APPROVAL');
      const createdVoucherId = createRes.data.voucher.id;

      // Machine B: Checker logs in
      const authB = useAuthStore();
      await authB.login('checker@livabanking.vn', 'checker123');
      const checkerToken = authB.token!;

      // Machine B fetches vouchers from server
      const vouchersRes = await apiRequest('/treasury/vouchers', {
        headers: { Authorization: `Bearer ${checkerToken}` },
      });
      expect(vouchersRes.success).toBe(true);
      const serverVoucher = vouchersRes.data.vouchers.find(
        (v: any) => v.id === createdVoucherId || v.voucherId === createdVoucherId
      );
      expect(serverVoucher).toBeDefined();
      expect(serverVoucher.amount).toBe(880_000_000);

      // Machine B approves voucher
      const approveRes = await apiRequest(`/treasury/vouchers/${createdVoucherId}/approve`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${checkerToken}` },
      });

      expect(approveRes.success).toBe(true);
      expect(approveRes.data?.voucher?.status).toBe('APPROVED');
      expect(approveRes.data?.merkleRoot).toHaveLength(64);

      // Verify immutable Merkle audit ledger updated
      const ledgerRes = await apiRequest('/treasury/merkle-ledger');
      expect(ledgerRes.success).toBe(true);
      expect(ledgerRes.data?.ledger?.history.some((h: any) => h.voucherId === createdVoucherId)).toBe(true);
    });

    it('updates treasuryStore reactive state when receiving SSE sync events', () => {
      const store = useTreasuryStore();

      // Simulate incoming voucher:created SSE event from remote machine
      store.processSseEvent('voucher:created', {
        voucher: {
          voucherId: 'vch_remote_maker_99',
          makerId: 'usr_maker_remote',
          beneficiaryAccount: '19033445566',
          beneficiaryBank: 'TECHCOMBANK',
          beneficiaryName: 'CONG TY FPT',
          amountVnd: 250_000_000,
          purpose: 'Quyet toan hoa don phan mem',
          status: 'PENDING_APPROVAL',
        },
      });

      const received = store.vouchers.find((v) => v.voucherId === 'vch_remote_maker_99');
      expect(received).toBeDefined();
      expect(received?.amountVnd).toBe(250_000_000);
      expect(received?.status).toBe('PENDING_APPROVAL');
      expect(store.pendingVouchers.some((v) => v.voucherId === 'vch_remote_maker_99')).toBe(true);

      // Simulate incoming voucher:approved SSE event from remote checker
      store.processSseEvent('voucher:approved', {
        voucherId: 'vch_remote_maker_99',
        checkerId: 'usr_checker_01',
        merkleHash: 'f4e3d2c1b0a987654321fedcba',
      });

      expect(received?.status).toBe('APPROVED');
      expect(received?.checkerId).toBe('usr_checker_01');
      expect(received?.merkleLeafHash).toBe('f4e3d2c1b0a987654321fedcba');
      expect(store.approvedVouchers.some((v) => v.voucherId === 'vch_remote_maker_99')).toBe(true);
    });
  });

  // ============================================================================
  // 5. Feature F15: Dual-Mode Resilience & Zero Cloud Egress Offline Fallback
  // ============================================================================
  describe('Feature F15: Dual-Mode Resilience (Decree 13/2023/ND-CP Zero Data Egress)', () => {
    it('gracefully reports isOnline: false without throwing when server is unreachable', async () => {
      // Point client to an unreachable port
      setApiBaseUrl('http://127.0.0.1:59999/api');

      const res = await apiRequest('/health');
      expect(res.success).toBe(false);
      expect(res.isOnline).toBe(false);
      expect(res.error).toContain('Offline Mode');

      // Restore valid API url
      setApiBaseUrl(`${serverUrl}/api`);
    });

    it('authStore gracefully falls back to local session simulation when offline', async () => {
      setApiBaseUrl('http://127.0.0.1:59999/api');

      const auth = useAuthStore();
      const loginResult = await auth.login('maker@livabanking.vn', 'maker123');

      // Login succeeds in offline mode using authentic local JWT simulation
      expect(loginResult).toBe(true);
      expect(auth.isAuthenticated).toBe(true);
      expect(auth.isServerOnline).toBe(false);
      expect(auth.currentOfficerId).toBe('OPR-77092');

      const token = mockLocalStorage.getItem('liva_auth_token');
      expect(token).toBeDefined();
      expect(token?.split('.').length).toBe(3);

      setApiBaseUrl(`${serverUrl}/api`);
    });

    it('treasuryStore executes local voucher creation and Merkle hashing without server dependency', () => {
      const store = useTreasuryStore();

      // Standalone local voucher creation
      const localVoucher = store.createVoucher(
        'usr_maker_offline',
        '007100998877',
        'VCB',
        150_000_000,
        'Chi tieu offline noi bo',
        'CONG TY VIETTEL'
      );
      expect(localVoucher.status).toBe('DRAFT');

      const submitted = store.submitVoucher(localVoucher.voucherId);
      expect(submitted.status).toBe('PENDING_APPROVAL');

      const approved = store.approveVoucher(localVoucher.voucherId, 'usr_checker_offline', 'BIOMETRIC_SIM');
      expect(approved.status).toBe('APPROVED');
      expect(approved.merkleLeafHash).toBeDefined();
      expect(store.merkleRoot).toHaveLength(64);
    });
  });
});
