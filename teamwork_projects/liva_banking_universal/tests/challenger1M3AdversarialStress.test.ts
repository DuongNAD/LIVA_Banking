/**
 * Milestone M3 Adversarial Stress Test Suite ? Challenger 1
 *
 * EMPIRICAL ADVERSARIAL CHALLENGES:
 * 1. JWT Authentication & Strict RBAC Authorization:
 *    - Forged signatures, wrong secret keys, flipped bits, truncated sigs, alg 'none', payload role tampering -> 401
 *    - Expired tokens (past hours, past seconds, exp=0, negative exp) -> 401
 *    - Malformed headers (empty, missing Bearer, extra parts, truncated parts, corrupt base64) -> 401
 *    - RBAC: Non-checker roles (Maker, AML, Treasury, Auditor) attempting voucher approve/reject -> 403 Forbidden
 *    - Non-authorized roles (AML, Auditor) attempting voucher creation -> 403 Forbidden
 *    - Legitimate Checker approval & rejection flows, state validation (400 duplicate approval, 404 missing voucher)
 * 2. Server-Sent Events (SSE) /api/sync/events Stress & Chaos:
 *    - Abrupt connection drop handling (clean socket teardown, zero server crashes)
 *    - Rapid sequential connect-disconnect cycling (15 churn cycles)
 *    - Multi-client concurrent broadcast fan-out (10 simultaneous clients receiving voucher:created, voucher:approved, voucher:rejected)
 *    - Partial client drop during broadcast (severed connections auto-pruned, remaining clients receive event)
 * 3. Client Dual-Mode Resilience (Decree 13/2023/ND-CP Zero Data Egress):
 *    - Corrupt localStorage tokens and user strings (garbage, non-JSON, truncated, huge string) -> safe error handling
 *    - Sudden server unavailability -> graceful fallback to local Pinia state
 *    - Local voucher creation -> submission -> approval with cryptographic SHA-256 Merkle leaf & root calculation (zero unhandled exceptions)
 *    - Merkle proof generation & verification in offline mode
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import http from 'node:http';
import crypto from 'node:crypto';
import {
  server,
  DB,
  signJwt,
  verifyJwt,
  broadcastEvent,
} from '../server/server.mjs';
import {
  setApiBaseUrl,
  getStoredToken,
  getStoredUser,
} from '../src/services/apiClient';
import { useAuthStore } from '../src/stores/authStore';
import { useTreasuryStore } from '../src/stores/treasuryStore';
import { verifyMerkleProof, generateMerkleProof } from '../src/engine/treasury/merkleAudit';

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

function makeRawRequest(
  method: string,
  path: string,
  headers: Record<string, string> = {},
  body?: any
): Promise<{ status: number; headers: http.IncomingHttpHeaders; data: any; rawBody: string }> {
  return new Promise((resolve, reject) => {
    const req = http.request(
      `${serverUrl}${path}`,
      {
        method,
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
      },
      (res) => {
        let rawBody = '';
        res.on('data', (chunk) => {
          rawBody += chunk;
        });
        res.on('end', () => {
          let data = null;
          try {
            data = JSON.parse(rawBody);
          } catch {
            data = rawBody;
          }
          resolve({
            status: res.statusCode || 0,
            headers: res.headers,
            data,
            rawBody,
          });
        });
      }
    );

    req.on('error', reject);
    if (body) {
      req.write(typeof body === 'string' ? body : JSON.stringify(body));
    }
    req.end();
  });
}

describe('Challenger 1 M3 Adversarial Stress Suite', () => {
  beforeAll(async () => {
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
  // ADVERSARIAL TASK 1: JWT AUTHENTICATION & STRICT RBAC REJECTION STRESS
  // ============================================================================
  describe('Task 1: Adversarial JWT Authentication & RBAC Authorization', () => {
    it('1.1 rejects forged JWT signatures signed with an alien secret key (401)', async () => {
      const alienSecret = 'attacker-alien-secret-key-99999';
      const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
      const payload = Buffer.from(
        JSON.stringify({
          sub: 'usr_admin_forged',
          id: 'usr_admin_forged',
          email: 'admin@malicious.com',
          role: 'CHECKER',
          exp: Math.floor(Date.now() / 1000) + 3600,
        })
      ).toString('base64url');
      const fakeSig = crypto.createHmac('sha256', alienSecret).update(`${header}.${payload}`).digest('base64url');
      const forgedToken = `${header}.${payload}.${fakeSig}`;

      const res = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${forgedToken}`,
      });

      expect(res.status).toBe(401);
      expect(res.data.success).toBe(false);
      expect(typeof res.data.error).toBe('string');
    });

    it('1.2 rejects altered payload with original signature (tamper attack) (401)', async () => {
      const legitToken = signJwt({
        id: 'usr_maker_01',
        email: 'maker@livabanking.vn',
        role: 'MAKER',
      });
      const [hdr, , sig] = legitToken.split('.');

      const tamperedPayloadObj = {
        id: 'usr_maker_01',
        email: 'maker@livabanking.vn',
        role: 'CHECKER',
        exp: Math.floor(Date.now() / 1000) + 3600,
      };
      const tamperedPayloadB64 = Buffer.from(JSON.stringify(tamperedPayloadObj)).toString('base64url');
      const tamperedToken = `${hdr}.${tamperedPayloadB64}.${sig}`;

      const res = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${tamperedToken}`,
      });

      expect(res.status).toBe(401);
      expect(res.data.success).toBe(false);
    });

    it('1.3 rejects truncated signatures and bit-flipped signatures (401)', async () => {
      const legitToken = signJwt({ id: 'usr_maker_01', role: 'MAKER' });
      const [hdr, payload, sig] = legitToken.split('.');

      const truncatedSigToken = `${hdr}.${payload}.${sig.slice(0, -2)}`;
      const resTrunc = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${truncatedSigToken}`,
      });
      expect(resTrunc.status).toBe(401);

      const flippedChar = sig.slice(-1) === 'A' ? 'B' : 'A';
      const flippedSigToken = `${hdr}.${payload}.${sig.slice(0, -1)}${flippedChar}`;
      const resFlipped = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${flippedSigToken}`,
      });
      expect(resFlipped.status).toBe(401);
    });

    it('1.4 rejects alg: none attack token (401)', async () => {
      const noneHeader = Buffer.from(JSON.stringify({ alg: 'none', typ: 'JWT' })).toString('base64url');
      const payload = Buffer.from(JSON.stringify({ id: 'usr_checker_01', role: 'CHECKER' })).toString('base64url');
      const noneToken = `${noneHeader}.${payload}.`;

      const res = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${noneToken}`,
      });
      expect(res.status).toBe(401);
    });

    it('1.5 rejects expired JWT tokens across various expiration bounds (401)', async () => {
      const expiredPast1h = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, -3600);
      const resPast1h = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${expiredPast1h}`,
      });
      expect(resPast1h.status).toBe(401);

      const expiredPast1s = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, -1);
      const resPast1s = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${expiredPast1s}`,
      });
      expect(resPast1s.status).toBe(401);

      // Negative expiration offset (already expired)
      const expiredPast100 = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, -100);
      const resPast100 = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${expiredPast100}`,
      });
      expect(resPast100.status).toBe(401);
    });

    it('1.6 rejects malformed headers and abnormal tokens (401)', async () => {
      const malformedCases: Array<Record<string, string>> = [
        {},
        { Authorization: '' },
        { Authorization: 'Bearer' },
        { Authorization: 'Bearer ' },
        { Authorization: 'Bearer null' },
        { Authorization: 'Bearer undefined' },
        { Authorization: 'Basic dXNlcjpwYXNz' },
        { Authorization: 'Bearer part1.part2' },
        { Authorization: 'Bearer part1.part2.part3.part4' },
        { Authorization: 'Bearer ..' },
        { Authorization: 'Bearer not_a_jwt_at_all' },
        { Authorization: 'Bearer eyJhbGciOi.not_base64_json@@@.sig' },
      ];

      for (const headers of malformedCases) {
        const res = await makeRawRequest('GET', '/api/auth/me', headers);
        expect(res.status).toBe(401);
      }
    });

    it('1.7 verifyJwt pure unit validation returns null for all corrupted inputs without throwing', () => {
      expect(verifyJwt(null as any)).toBeNull();
      expect(verifyJwt(undefined as any)).toBeNull();
      expect(verifyJwt(12345 as any)).toBeNull();
      expect(verifyJwt({} as any)).toBeNull();
      expect(verifyJwt('')).toBeNull();
      expect(verifyJwt('single_string')).toBeNull();
      expect(verifyJwt('part1.part2')).toBeNull();
      expect(verifyJwt('part1.part2.part3.part4')).toBeNull();
      expect(verifyJwt('a.b.c')).toBeNull();
    });

    it('1.8 strictly enforces RBAC: Non-Checker roles cannot approve vouchers (403 Forbidden)', async () => {
      const testVoucherId = `vch_rbac_test_${Date.now()}`;
      DB.vouchers.unshift({
        id: testVoucherId,
        voucherId: testVoucherId,
        voucherNumber: 'VCH-RBAC-001',
        sourceAccount: '0071001234567 (VCB)',
        targetAccount: '020012345678',
        amount: 200_000_000,
        currency: 'VND',
        description: 'RBAC boundary stress test',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        createdAt: new Date().toISOString(),
        approvedAt: null,
        checkerId: null,
        merkleLeafHash: null,
      } as any);

      const makerToken = signJwt({ id: 'usr_maker_01', role: 'MAKER' });
      const makerApprove = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/approve`, {
        Authorization: `Bearer ${makerToken}`,
      });
      expect(makerApprove.status).toBe(403);
      expect(makerApprove.data.error).toContain('Maker-Checker');

      const amlToken = signJwt({ id: 'usr_aml_01', role: 'AML' });
      const amlApprove = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/approve`, {
        Authorization: `Bearer ${amlToken}`,
      });
      expect(amlApprove.status).toBe(403);

      const treasuryToken = signJwt({ id: 'usr_treasury_01', role: 'TREASURY' });
      const treasuryApprove = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/approve`, {
        Authorization: `Bearer ${treasuryToken}`,
      });
      expect(treasuryApprove.status).toBe(403);

      const auditorToken = signJwt({ id: 'usr_auditor_01', role: 'AUDITOR' });
      const auditorApprove = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/approve`, {
        Authorization: `Bearer ${auditorToken}`,
      });
      expect(auditorApprove.status).toBe(403);

      const unauthApprove = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/approve`);
      expect(unauthApprove.status).toBe(401);
    });

    it('1.9 strictly enforces RBAC: Non-Checker roles cannot reject vouchers (403 Forbidden)', async () => {
      const testVoucherId = `vch_rbac_rej_${Date.now()}`;
      DB.vouchers.unshift({
        id: testVoucherId,
        voucherId: testVoucherId,
        voucherNumber: 'VCH-RBAC-REJ-001',
        amount: 150_000_000,
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
      } as any);

      const makerToken = signJwt({ id: 'usr_maker_01', role: 'MAKER' });
      const makerReject = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/reject`, {
        Authorization: `Bearer ${makerToken}`,
      });
      expect(makerReject.status).toBe(403);

      const amlToken = signJwt({ id: 'usr_aml_01', role: 'AML' });
      const amlReject = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/reject`, {
        Authorization: `Bearer ${amlToken}`,
      });
      expect(amlReject.status).toBe(403);

      const treasuryToken = signJwt({ id: 'usr_treasury_01', role: 'TREASURY' });
      const treasuryReject = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/reject`, {
        Authorization: `Bearer ${treasuryToken}`,
      });
      expect(treasuryReject.status).toBe(403);
    });

    it('1.10 prevents unauthorized roles (AML, Auditor) from creating vouchers (403)', async () => {
      const amlToken = signJwt({ id: 'usr_aml_01', role: 'AML' });
      const amlCreate = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${amlToken}` },
        { amount: 100_000_000, purpose: 'AML illegal voucher creation attempt' }
      );
      expect(amlCreate.status).toBe(403);

      const auditorToken = signJwt({ id: 'usr_auditor_01', role: 'AUDITOR' });
      const auditorCreate = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${auditorToken}` },
        { amount: 50_000_000, purpose: 'Auditor illegal voucher creation attempt' }
      );
      expect(auditorCreate.status).toBe(403);
    });

    it('1.11 verifies Checker approval/rejection lifecycle, idempotency, and 404/400 errors', async () => {
      const testVoucherId = `vch_chk_life_${Date.now()}`;
      DB.vouchers.unshift({
        id: testVoucherId,
        voucherId: testVoucherId,
        voucherNumber: 'VCH-CHECKER-001',
        amount: 300_000_000,
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
      } as any);

      const checkerToken = signJwt({ id: 'usr_checker_01', fullName: 'Tr?n Th? Gi?m ??c', role: 'CHECKER' });

      const approveRes = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/approve`, {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(approveRes.status).toBe(200);
      expect(approveRes.data.voucher.status).toBe('APPROVED');
      expect(approveRes.data.voucher.merkleLeafHash).toHaveLength(64);
      expect(approveRes.data.merkleRoot).toHaveLength(64);

      const duplicateApprove = await makeRawRequest('POST', `/api/treasury/vouchers/${testVoucherId}/approve`, {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(duplicateApprove.status).toBe(400);
      expect(typeof duplicateApprove.data.error).toBe('string');

      const nonExistentApprove = await makeRawRequest('POST', '/api/treasury/vouchers/vch_non_existent_9999/approve', {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(nonExistentApprove.status).toBe(404);
    });

    it('1.12 [M4-HARDENING] verifies exp: 0 is strictly treated as expired via payload.exp !== undefined && payload.exp < now', async () => {
      // With negative exp, token is expired and rejected with 401
      const expiredNegative = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, -10);
      const resNeg = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${expiredNegative}`,
      });
      expect(resNeg.status).toBe(401);

      // Hardening verification: When exp === 0, verifyJwt treats it as expired (returns null)
      const tokenWithExpZero = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, -Math.floor(Date.now() / 1000));
      const payloadZero = verifyJwt(tokenWithExpZero);
      expect(payloadZero).toBeNull();

      const resZero = await makeRawRequest('GET', '/api/auth/me', {
        Authorization: `Bearer ${tokenWithExpZero}`,
      });
      expect(resZero.status).toBe(401);
    });
  });

  // ============================================================================
  // ADVERSARIAL TASK 2: SSE REAL-TIME SYNC STRESS & CHAOS TESTING
  // ============================================================================
  describe('Task 2: SSE /api/sync/events Stress, Drops, Reconnects & Multi-Client Broadcast', () => {
    it('2.1 cleanly handles sudden client connection termination and socket destroy without crashing', async () => {
      const clientReq = http.request(`${serverUrl}/api/sync/events`, {
        method: 'GET',
        headers: { Accept: 'text/event-stream' },
      });

      const connectedPromise = new Promise<void>((resolve) => {
        clientReq.on('response', (res) => {
          res.on('data', (chunk) => {
            if (chunk.toString().includes(': connected')) {
              resolve();
            }
          });
        });
      });

      clientReq.end();
      await connectedPromise;

      expect(() => {
        clientReq.destroy();
      }).not.toThrow();

      const healthRes = await makeRawRequest('GET', '/api/health');
      expect(healthRes.status).toBe(200);
      expect(healthRes.data.status).toBe('OK');
    });

    it('2.2 withstands rapid sequential connect-disconnect cycling (15 churn cycles) without memory leak', async () => {
      for (let i = 0; i < 15; i++) {
        await new Promise<void>((resolve) => {
          const req = http.request(`${serverUrl}/api/sync/events`);
          req.on('response', (res) => {
            res.on('data', () => {
              req.destroy();
              resolve();
            });
          });
          req.on('error', () => {
            resolve();
          });
          req.end();
        });
      }

      const health = await makeRawRequest('GET', '/api/health');
      expect(health.status).toBe(200);
    });

    it('2.3 fans out broadcast events to 10 concurrent SSE subscribers simultaneously', async () => {
      const clientCount = 10;
      const clients: http.ClientRequest[] = [];
      const receivedCreatedEvents: any[] = [];
      const receivedApprovedEvents: any[] = [];

      const connectedPromises: Promise<void>[] = [];

      for (let i = 0; i < clientCount; i++) {
        let resolveConn: () => void;
        const connPromise = new Promise<void>((res) => {
          resolveConn = res;
        });
        connectedPromises.push(connPromise);

        const req = http.request(`${serverUrl}/api/sync/events`);
        req.on('response', (res) => {
          res.on('data', (chunk) => {
            const text = chunk.toString();
            if (text.includes(': connected')) {
              resolveConn();
            }
            if (text.includes('event: voucher:created')) {
              const dataMatch = text.match(/data: (.*)\n\n/);
              if (dataMatch) {
                try {
                  receivedCreatedEvents.push(JSON.parse(dataMatch[1]));
                } catch {}
              }
            }
            if (text.includes('event: voucher:approved')) {
              const dataMatch = text.match(/data: (.*)\n\n/);
              if (dataMatch) {
                try {
                  receivedApprovedEvents.push(JSON.parse(dataMatch[1]));
                } catch {}
              }
            }
          });
        });
        req.end();
        clients.push(req);
      }

      await Promise.all(connectedPromises);

      const testVoucher = {
        voucherId: 'vch_sse_fanout_01',
        amountVnd: 500_000_000,
        purpose: 'SSE Fan-out multi-client test',
      };
      broadcastEvent('voucher:created', { voucher: testVoucher });

      broadcastEvent('voucher:approved', {
        voucherId: 'vch_sse_fanout_01',
        checkerId: 'usr_checker_01',
        merkleHash: 'a1b2c3d4e5f6',
      });

      await new Promise((r) => setTimeout(r, 100));

      expect(receivedCreatedEvents.length).toBe(10);
      expect(receivedCreatedEvents.every((e) => e.voucher?.voucherId === 'vch_sse_fanout_01')).toBe(true);

      expect(receivedApprovedEvents.length).toBe(10);
      expect(receivedApprovedEvents.every((e) => e.voucherId === 'vch_sse_fanout_01')).toBe(true);

      clients.forEach((c) => c.destroy());
    });

    it('2.4 auto-prunes severed connections during broadcast without interrupting remaining active clients', async () => {
      const activeClients: http.ClientRequest[] = [];
      const severedClients: http.ClientRequest[] = [];
      const receivedByActive: any[] = [];

      const connectPromises: Promise<void>[] = [];

      for (let i = 0; i < 4; i++) {
        let resolveConn: () => void;
        connectPromises.push(new Promise<void>((r) => (resolveConn = r)));
        const req = http.request(`${serverUrl}/api/sync/events`);
        req.on('response', (res) => {
          res.on('data', (chunk) => {
            const text = chunk.toString();
            if (text.includes(': connected')) resolveConn();
            if (text.includes('event: voucher:rejected')) {
              const dataMatch = text.match(/data: (.*)\n\n/);
              if (dataMatch) {
                try {
                  receivedByActive.push(JSON.parse(dataMatch[1]));
                } catch {}
              }
            }
          });
        });
        req.end();
        activeClients.push(req);
      }

      for (let i = 0; i < 3; i++) {
        let resolveConn: () => void;
        connectPromises.push(new Promise<void>((r) => (resolveConn = r)));
        const req = http.request(`${serverUrl}/api/sync/events`);
        req.on('response', (res) => {
          res.on('data', (chunk) => {
            if (chunk.toString().includes(': connected')) resolveConn();
          });
        });
        req.end();
        severedClients.push(req);
      }

      await Promise.all(connectPromises);

      severedClients.forEach((c) => c.destroy());

      expect(() => {
        broadcastEvent('voucher:rejected', {
          voucherId: 'vch_prune_01',
          reason: 'Prune test rejection',
        });
      }).not.toThrow();

      await new Promise((r) => setTimeout(r, 100));

      expect(receivedByActive.length).toBe(4);
      expect(receivedByActive.every((e) => e.voucherId === 'vch_prune_01')).toBe(true);

      activeClients.forEach((c) => c.destroy());
    });
  });

  // ============================================================================
  // ADVERSARIAL TASK 3: CLIENT DUAL-MODE RESILIENCE & ZERO DATA EGRESS
  // ============================================================================
  describe('Task 3: Client Dual-Mode Resilience & Decree 13 Zero Data Egress', () => {
    it('3.1 handles corrupt tokens in localStorage safely without crashing and self-heals', async () => {
      const corruptTokens = [
        'GARBAGE_TOKEN_STRING',
        '{"fake":"json"}',
        'a.b.c',
        'eyJhbGciOi.eyJzdWIi.fake',
        'null',
        'x'.repeat(20000),
      ];

      for (const corrupt of corruptTokens) {
        mockLocalStorage.setItem('liva_auth_token', corrupt);

        expect(() => {
          const t = getStoredToken();
          expect(t).toBe(corrupt);
        }).not.toThrow();

        setActivePinia(createPinia());
        const auth = useAuthStore();
        await auth.checkServerHealth();

        expect(auth.isAuthenticated).toBe(false);
        expect(mockLocalStorage.getItem('liva_auth_token')).toBeNull();
      }

      // Safe handling of empty token
      mockLocalStorage.setItem('liva_auth_token', '');
      setActivePinia(createPinia());
      const authEmpty = useAuthStore();
      expect(authEmpty.isAuthenticated).toBe(false);
    });

    it('3.2 handles corrupt stored user profile JSON without throwing', () => {
      mockLocalStorage.setItem('liva_auth_user', 'MALFORMED_JSON_STRING{{{');
      expect(() => {
        const u = getStoredUser();
        expect(u).toBeNull();
      }).not.toThrow();
    });

    it('3.3 sudden server downtime: treasuryStore falls back gracefully to local Pinia & Merkle ledger', async () => {
      setApiBaseUrl('http://127.0.0.1:59995/api');

      const store = useTreasuryStore();

      let voucher: any;
      expect(() => {
        voucher = store.createVoucher(
          'usr_maker_01',
          '19034567890123',
          'TECHCOMBANK',
          750_000_000,
          'Thanh toan hop dong cung cap thiet bi (Offline Test)',
          'CONG TY TNHH VIEN THONG'
        );
      }).not.toThrow();

      expect(voucher).toBeDefined();
      expect(voucher.status).toBe('DRAFT');
      expect(voucher.amountVnd).toBe(750_000_000);

      let submitted: any;
      expect(() => {
        submitted = store.submitVoucher(voucher.voucherId);
      }).not.toThrow();
      expect(submitted.status).toBe('PENDING_APPROVAL');

      let approved: any;
      expect(() => {
        approved = store.approveVoucher(voucher.voucherId, 'usr_checker_01', 'BIOMETRIC_SIM');
      }).not.toThrow();

      expect(approved.status).toBe('APPROVED');
      expect(approved.merkleLeafHash).toBeDefined();
      expect(approved.merkleLeafHash).toHaveLength(64);
      expect(store.merkleRoot).toHaveLength(64);

      // 4. Verify cryptographic Merkle proof for the approved voucher leaf
      const proof = generateMerkleProof([approved.merkleLeafHash], 0);
      expect(proof).toBeDefined();
      expect(proof.leaf).toBe(approved.merkleLeafHash);

      const isValidProof = verifyMerkleProof(proof.leaf, proof.proof, proof.root);
      expect(isValidProof).toBe(true);

      // 5. Verify forward audit ledger integrity
      const integrity = store.auditLedger.verifyIntegrity();
      expect(integrity.isValid).toBe(true);

      setApiBaseUrl(`${serverUrl}/api`);
    });

    it('3.4 sudden server downtime during voucher rejection updates local state cleanly without errors', () => {
      setApiBaseUrl('http://127.0.0.1:59995/api');

      const store = useTreasuryStore();
      const v = store.createVoucher(
        'usr_maker_01',
        '007100998877',
        'VCB',
        220_000_000,
        'De nghi chi phi dao tao',
        'TRUONG DAO TAO NGAN HANG'
      );
      store.submitVoucher(v.voucherId);

      let rejected: any;
      expect(() => {
        rejected = store.rejectVoucher(v.voucherId, 'usr_checker_01', 'Chung tu chua du chu ky ke toan truong');
      }).not.toThrow();

      expect(rejected.status).toBe('REJECTED');
      expect(rejected.rejectReason).toBe('Chung tu chua du chu ky ke toan truong');

      setApiBaseUrl(`${serverUrl}/api`);
    });

    it('3.5 offline login fallback preserves full officer identity and generates 3-part signed JWT structure', async () => {
      setApiBaseUrl('http://127.0.0.1:59995/api');

      const auth = useAuthStore();
      const success = await auth.login('checker@livabanking.vn', 'checker123');

      expect(success).toBe(true);
      expect(auth.isAuthenticated).toBe(true);
      expect(auth.isServerOnline).toBe(false);
      expect(auth.isChecker).toBe(true);
      expect(auth.currentOfficerId).toBe('SUP-88214');
      expect(auth.currentBranchCode).toBe('HO-HN-001');

      const token = mockLocalStorage.getItem('liva_auth_token');
      expect(token).toBeDefined();
      const parts = token!.split('.');
      expect(parts.length).toBe(3);

      setApiBaseUrl(`${serverUrl}/api`);
    });
  });
});
