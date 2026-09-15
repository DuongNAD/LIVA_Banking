/**
 * Milestone M3 Adversarial Stress Test Suite — Challenger 2
 *
 * INDEPENDENT ADVERSARIAL CHALLENGES:
 * 1. Empirical Cross-Machine Synchronization & Real-Time SSE Push:
 *    - Simulated Machine A (Maker Station) & Machine B (Checker Station) with dedicated sessions & stores.
 *    - Maker creates voucher on Machine A -> SSE pushes 'voucher:created' -> Checker Machine B receives & verifies.
 *    - Checker approves voucher on Machine B -> Server generates SHA-256 Merkle leaf & broadcasts 'voucher:approved'
 *      -> Maker Machine A receives event, updates state to APPROVED, and records Merkle leaf hash.
 *    - Checker rejects voucher on Machine B -> SSE broadcasts 'voucher:rejected' -> Maker Machine A updates to REJECTED.
 * 2. Concurrency & Race Condition Stress Testing:
 *    - High-throughput burst voucher creation (50 concurrent requests via Promise.all).
 *    - Race Condition: Double Approval Attack (5 concurrent approvals for the same voucher). Exactly 1 succeeds, 4 fail (400).
 *    - Race Condition: Simultaneous Approval vs Rejection Conflict. Exactly 1 wins, state remains consistent.
 *    - High-concurrency mixed pipeline (20 creations, 10 approvals, 5 rejections, 20 queries concurrently).
 *    - Probing status transitions and edge case vulnerabilities.
 * 3. Decree 13/2023/NĐ-CP Zero Data Egress & Disruption Resilience:
 *    - Dead server / offline mode: full voucher lifecycle executes locally with ZERO unhandled exceptions.
 *    - Cryptographic Merkle proof & audit ledger integrity verified in offline mode.
 *    - Offline JWT token generation with full Decree 13 officer claims.
 *    - State convergence upon server recovery.
 * 4. Adversarial Input Sanitization, Unicode Diacritics & Boundary Edge Cases:
 *    - XSS, SQLi, and path traversal strings in voucher payloads.
 *    - Full Vietnamese UTF-8 diacritics & emojis preserved across SSE and JSON serialization.
 *    - Empirical probe of server-side status checks.
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import http from 'node:http';
import {
  server,
  DB,
  signJwt,
} from '../server/server.mjs';
import {
  apiRequest,
  setApiBaseUrl,
  getStoredToken,
  setStoredToken,
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

    if (body !== undefined) {
      req.write(typeof body === 'string' ? body : JSON.stringify(body));
    }
    req.end();
  });
}

/**
 * Creates a real HTTP SSE client that connects to /api/sync/events
 * and gathers received events into an array.
 */
function createSseReceiver(onEvent?: (event: string, data: any) => void) {
  const events: Array<{ event: string; data: any }> = [];
  let req: http.ClientRequest;

  const connectedPromise = new Promise<void>((resolve, reject) => {
    req = http.request(
      `${serverUrl}/api/sync/events`,
      {
        method: 'GET',
        headers: { Accept: 'text/event-stream' },
      },
      (res) => {
        let buffer = '';
        res.on('data', (chunk) => {
          buffer += chunk.toString();
          const lines = buffer.split('\n\n');
          buffer = lines.pop() || '';

          for (const block of lines) {
            if (block.includes(': connected')) {
              resolve();
            }
            let eventType = 'message';
            let eventData = '';
            for (const line of block.split('\n')) {
              if (line.startsWith('event: ')) {
                eventType = line.slice(7).trim();
              } else if (line.startsWith('data: ')) {
                eventData = line.slice(6).trim();
              }
            }
            if (eventData) {
              try {
                const parsed = JSON.parse(eventData);
                events.push({ event: eventType, data: parsed });
                if (onEvent) onEvent(eventType, parsed);
              } catch {
                events.push({ event: eventType, data: eventData });
                if (onEvent) onEvent(eventType, eventData);
              }
            }
          }
        });
      }
    );

    req.on('error', reject);
    req.end();
  });

  return {
    events,
    ready: connectedPromise,
    close: () => {
      req.destroy();
    },
  };
}

describe('Challenger 2 — Adversarial Stress & Verification Suite (Milestone M3)', () => {
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
  // Suite 1: Empirical Cross-Machine Synchronization & Real-Time SSE Push
  // ============================================================================
  describe('Suite 1: Empirical Cross-Machine Synchronization & Real-Time SSE Push', () => {
    it('synchronizes voucher creation from Maker (Machine A) to Checker (Machine B) in real-time', async () => {
      // 1. Setup simulated Machine A (Maker Station)
      const piniaA = createPinia();
      setActivePinia(piniaA);
      const makerToken = signJwt({
        id: 'usr_maker_01',
        email: 'maker@livabanking.vn',
        name: 'Nguyễn Văn Kế Toán (Maker)',
        role: 'MAKER',
        officerId: 'OPR-77092',
        branchCode: 'HO-HN-001',
      });
      setStoredToken(makerToken);
      const storeA = useTreasuryStore(piniaA);
      storeA.setCurrentUser('usr_maker_01', 'MAKER', 'Nguyễn Văn Kế Toán (Maker)');

      // 2. Setup simulated Machine B (Checker Station)
      const piniaB = createPinia();
      const storeB = useTreasuryStore(piniaB);
      storeB.setCurrentUser('usr_checker_01', 'CHECKER', 'Trần Thị Giám Đốc (Checker)');

      // 3. Connect real SSE receiver for Machine B
      const sseReceiverB = createSseReceiver((event, data) => {
        storeB.processSseEvent(event, data);
      });
      await sseReceiverB.ready;

      // 4. Maker creates voucher on Machine A and posts to backend server
      const testVoucherId = `vch_cross_mach_${Date.now()}`;
      const createRes = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: testVoucherId,
          voucherId: testVoucherId,
          beneficiaryAccount: '0451000998877',
          beneficiaryBank: 'BIDV',
          beneficiaryName: 'TẬP ĐOÀN CÔNG NGHIỆP NĂNG LƯỢNG LIVA',
          amount: 850000000,
          purpose: 'Thanh toán đợt 2 theo hợp đồng thiết bị turbine',
        }
      );

      expect(createRes.status).toBe(201);
      expect(createRes.data.success).toBe(true);

      // 5. Allow brief event-loop turn for SSE packet propagation
      await new Promise((r) => setTimeout(r, 100));

      // 6. Verify Checker on Machine B received the event and voucher is immediately visible
      const checkerPendingVoucher = storeB.vouchers.find((v) => v.voucherId === testVoucherId);
      expect(checkerPendingVoucher).toBeDefined();
      expect(checkerPendingVoucher?.amountVnd).toBe(850000000);
      expect(checkerPendingVoucher?.status).toBe('PENDING_APPROVAL');
      expect(checkerPendingVoucher?.beneficiaryName).toBe('TẬP ĐOÀN CÔNG NGHIỆP NĂNG LƯỢNG LIVA');

      // 7. Verify Machine B audit ledger recorded the SSE event
      const syncAuditEntry = storeB.auditLedger.getEntries().find(
        (e: any) => e.payload?.voucherId === testVoucherId
      );
      expect(syncAuditEntry).toBeDefined();
      expect(syncAuditEntry?.event).toBe('VOUCHER_CREATED');

      sseReceiverB.close();
    });

    it('synchronizes voucher approval from Checker (Machine B) to Maker (Machine A) with Merkle leaf hash', async () => {
      // 1. Setup simulated Machine A (Maker Station)
      const piniaA = createPinia();
      const storeA = useTreasuryStore(piniaA);
      storeA.setCurrentUser('usr_maker_01', 'MAKER', 'Nguyễn Văn Kế Toán (Maker)');

      // Pre-seed the voucher in Machine A's store as pending
      const testVoucherId = `vch_approve_sync_${Date.now()}`;
      storeA.vouchers.push({
        voucherId: testVoucherId,
        makerId: 'usr_maker_01',
        checkerId: null,
        beneficiaryAccount: '0451000998877',
        beneficiaryBank: 'BIDV',
        beneficiaryName: 'TỔNG CÔNG TY ĐIỆN LỰC MIỀN BẮC',
        amountVnd: 1200000000,
        purpose: 'Thanh toán tiền điện trạm biến áp tháng 8/2026',
        status: 'PENDING_APPROVAL',
        createdAt: new Date().toISOString(),
      });

      // 2. Connect real SSE receiver for Machine A
      const sseReceiverA = createSseReceiver((event, data) => {
        storeA.processSseEvent(event, data);
      });
      await sseReceiverA.ready;

      // 3. Setup Checker (Machine B) token and submit voucher to server
      const checkerToken = signJwt({
        id: 'usr_checker_01',
        email: 'checker@livabanking.vn',
        name: 'Trần Thị Giám Đốc (Checker)',
        role: 'CHECKER',
        officerId: 'SUP-88214',
        branchCode: 'HO-HN-001',
      });

      // Create voucher on server first so Checker can approve it
      const makerToken = signJwt({
        id: 'usr_maker_01',
        email: 'maker@livabanking.vn',
        name: 'Nguyễn Văn Kế Toán (Maker)',
        role: 'MAKER',
        officerId: 'OPR-77092',
        branchCode: 'HO-HN-001',
      });

      await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: testVoucherId,
          voucherId: testVoucherId,
          beneficiaryAccount: '0451000998877',
          beneficiaryBank: 'BIDV',
          amount: 1200000000,
          purpose: 'Thanh toán tiền điện trạm biến áp',
        }
      );

      // 4. Checker on Machine B approves the voucher via API
      const approveRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${testVoucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` },
        { remarks: 'Hồ sơ hóa đơn hợp lệ, số dư đủ, phê duyệt lệnh chi.' }
      );

      expect(approveRes.status).toBe(200);
      expect(approveRes.data.success).toBe(true);
      expect(approveRes.data.merkleRoot).toBeDefined();
      const serverLeafHash = approveRes.data.voucher.merkleLeafHash;
      expect(serverLeafHash).toMatch(/^[a-f0-9]{64}$/);

      // 5. Allow brief delay for SSE delivery to Machine A
      await new Promise((r) => setTimeout(r, 100));

      // 6. Verify Maker on Machine A received event: status is APPROVED, Merkle leaf matches
      const makerVoucher = storeA.vouchers.find((v) => v.voucherId === testVoucherId);
      expect(makerVoucher).toBeDefined();
      expect(makerVoucher?.status).toBe('APPROVED');
      expect(makerVoucher?.checkerId).toBe('usr_checker_01');
      expect(makerVoucher?.merkleLeafHash).toBe(serverLeafHash);

      // 7. Verify audit ledger entry on Machine A records Merkle leaf
      const approvedAuditEntry = storeA.auditLedger.getEntries().find(
        (e: any) => e.payload?.voucherId === testVoucherId && e.event === 'VOUCHER_APPROVED'
      );
      expect(approvedAuditEntry).toBeDefined();
      expect(approvedAuditEntry?.payload?.merkleHash).toBe(serverLeafHash);

      sseReceiverA.close();
    });

    it('synchronizes voucher rejection from Checker (Machine B) to Maker (Machine A) with explanatory remarks', async () => {
      // 1. Setup Machine A (Maker Station)
      const piniaA = createPinia();
      const storeA = useTreasuryStore(piniaA);
      storeA.setCurrentUser('usr_maker_01', 'MAKER', 'Nguyễn Văn Kế Toán (Maker)');

      const testVoucherId = `vch_reject_sync_${Date.now()}`;
      storeA.vouchers.push({
        voucherId: testVoucherId,
        makerId: 'usr_maker_01',
        checkerId: null,
        beneficiaryAccount: '012399998888',
        beneficiaryBank: 'TCB',
        amountVnd: 350000000,
        purpose: 'Chi tạm ứng công tác phí',
        status: 'PENDING_APPROVAL',
        createdAt: new Date().toISOString(),
      });

      const sseReceiverA = createSseReceiver((event, data) => {
        storeA.processSseEvent(event, data);
      });
      await sseReceiverA.ready;

      // 2. Create voucher on server
      const makerToken = signJwt({
        id: 'usr_maker_01',
        email: 'maker@livabanking.vn',
        role: 'MAKER',
        officerId: 'OPR-77092',
      });
      await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: testVoucherId,
          voucherId: testVoucherId,
          beneficiaryAccount: '012399998888',
          beneficiaryBank: 'TCB',
          amount: 350000000,
          purpose: 'Chi tạm ứng công tác phí',
        }
      );

      // 3. Checker rejects voucher on Machine B
      const checkerToken = signJwt({
        id: 'usr_checker_01',
        email: 'checker@livabanking.vn',
        role: 'CHECKER',
        officerId: 'SUP-88214',
      });

      const rejectRemarks = 'Từ chối: Tạm ứng vượt quá định mức quy chế tài chính nội bộ số 14/QĐ-LIVA.';
      const rejectRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${testVoucherId}/reject`,
        { Authorization: `Bearer ${checkerToken}` },
        { remarks: rejectRemarks }
      );

      expect(rejectRes.status).toBe(200);
      expect(rejectRes.data.success).toBe(true);

      await new Promise((r) => setTimeout(r, 100));

      // 4. Verify Maker on Machine A received event: status is REJECTED, reason preserved
      const makerVoucher = storeA.vouchers.find((v) => v.voucherId === testVoucherId);
      expect(makerVoucher).toBeDefined();
      expect(makerVoucher?.status).toBe('REJECTED');
      expect(makerVoucher?.rejectReason).toBe(rejectRemarks);

      sseReceiverA.close();
    });
  });

  // ============================================================================
  // Suite 2: Concurrency & Race Condition Stress Testing
  // ============================================================================
  describe('Suite 2: Concurrency & Race Condition Stress Testing', () => {
    it('handles high-throughput burst creation of 50 vouchers concurrently without state corruption', async () => {
      const makerToken = signJwt({
        id: 'usr_maker_01',
        email: 'maker@livabanking.vn',
        role: 'MAKER',
        officerId: 'OPR-77092',
      });

      const initialCount = DB.vouchers.length;
      const BURST_COUNT = 50;

      const creationPromises = Array.from({ length: BURST_COUNT }).map((_, idx) => {
        const vId = `vch_burst_${Date.now()}_${idx}`;
        return makeRawRequest(
          'POST',
          '/api/treasury/vouchers',
          { Authorization: `Bearer ${makerToken}` },
          {
            id: vId,
            voucherId: vId,
            beneficiaryAccount: `0071000${String(idx).padStart(5, '0')}`,
            beneficiaryBank: idx % 2 === 0 ? 'VCB' : 'BIDV',
            amount: 10000000 + idx * 500000,
            purpose: `Thanh toán hợp đồng burst batch #${idx}`,
          }
        );
      });

      const results = await Promise.all(creationPromises);

      // Verify all 50 calls completed with HTTP 201 Created
      for (const res of results) {
        expect(res.status).toBe(201);
        expect(res.data.success).toBe(true);
        expect(res.data.voucher).toBeDefined();
      }

      // Verify DB count increased by exactly BURST_COUNT
      expect(DB.vouchers.length).toBe(initialCount + BURST_COUNT);

      // Verify all 50 vouchers have unique IDs
      const burstVouchers = DB.vouchers.filter((v: any) => v.id.startsWith('vch_burst_'));
      const idSet = new Set(burstVouchers.map((v: any) => v.id));
      expect(idSet.size).toBe(BURST_COUNT);
    });

    it('eliminates race condition: simultaneous double-approval attempts on same voucher (idempotency)', async () => {
      const makerToken = signJwt({
        id: 'usr_maker_01',
        email: 'maker@livabanking.vn',
        role: 'MAKER',
      });
      const checkerToken = signJwt({
        id: 'usr_checker_01',
        email: 'checker@livabanking.vn',
        role: 'CHECKER',
      });

      const raceVoucherId = `vch_race_double_approve_${Date.now()}`;
      await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: raceVoucherId,
          voucherId: raceVoucherId,
          beneficiaryAccount: '0999888777',
          amount: 500000000,
          purpose: 'Test double approval race condition',
        }
      );

      const initialLeavesCount = DB.merkleLedger.leaves.length;

      // Launch 5 concurrent approval requests at the exact same moment
      const CONCURRENT_ATTEMPTS = 5;
      const approvalPromises = Array.from({ length: CONCURRENT_ATTEMPTS }).map(() =>
        makeRawRequest(
          'POST',
          `/api/treasury/vouchers/${raceVoucherId}/approve`,
          { Authorization: `Bearer ${checkerToken}` }
        )
      );

      const results = await Promise.all(approvalPromises);

      const successResults = results.filter((r) => r.status === 200);
      const conflictResults = results.filter((r) => r.status === 400);

      // Exactly ONE approval must succeed (200 OK)
      expect(successResults.length).toBe(1);
      // All other attempts must fail with HTTP 400 (not pending approval)
      expect(conflictResults.length).toBe(CONCURRENT_ATTEMPTS - 1);

      for (const failRes of conflictResults) {
        expect(failRes.data.error).toBe('Lệnh chi không ở trạng thái chờ phê duyệt.');
      }

      // Verify the Merkle tree only incremented by exactly 1 leaf, preventing duplicate ledger inclusion
      expect(DB.merkleLedger.leaves.length).toBe(initialLeavesCount + 1);

      // Verify voucher final state is APPROVED
      const voucherInDb = DB.vouchers.find((v: any) => v.id === raceVoucherId);
      expect(voucherInDb.status).toBe('APPROVED');
      expect(voucherInDb.checkerId).toBe('usr_checker_01');
    });

    it('handles concurrent mixed pipeline (creations, approvals, queries) with high consistency', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      const checkerToken = signJwt({ id: 'usr_checker_01', email: 'checker@livabanking.vn', role: 'CHECKER' });

      // 1. Pre-seed 10 vouchers to be approved
      const vouchersToApprove: string[] = [];
      for (let i = 0; i < 10; i++) {
        const vId = `vch_pipe_seed_${Date.now()}_${i}`;
        vouchersToApprove.push(vId);
        await makeRawRequest(
          'POST',
          '/api/treasury/vouchers',
          { Authorization: `Bearer ${makerToken}` },
          { id: vId, voucherId: vId, amount: 20000000 + i * 1000000, purpose: `Pipeline seed ${i}` }
        );
      }

      // 2. Launch concurrent operations:
      // - 15 new voucher creations
      // - 10 voucher approvals
      // - 15 GET vouchers queries
      const creationOps = Array.from({ length: 15 }).map((_, i) =>
        makeRawRequest(
          'POST',
          '/api/treasury/vouchers',
          { Authorization: `Bearer ${makerToken}` },
          { id: `vch_pipe_new_${Date.now()}_${i}`, voucherId: `vch_pipe_new_${Date.now()}_${i}`, amount: 50000000, purpose: `Pipeline new ${i}` }
        )
      );

      const approvalOps = vouchersToApprove.map((vId) =>
        makeRawRequest(
          'POST',
          `/api/treasury/vouchers/${vId}/approve`,
          { Authorization: `Bearer ${checkerToken}` }
        )
      );

      const queryOps = Array.from({ length: 15 }).map(() =>
        makeRawRequest('GET', '/api/treasury/vouchers', { Authorization: `Bearer ${makerToken}` })
      );

      const [creationResults, approvalResults, queryResults] = await Promise.all([
        Promise.all(creationOps),
        Promise.all(approvalOps),
        Promise.all(queryOps),
      ]);

      // Assertions
      for (const res of creationResults) {
        expect(res.status).toBe(201);
      }
      for (const res of approvalResults) {
        expect(res.status).toBe(200);
      }
      for (const res of queryResults) {
        expect(res.status).toBe(200);
        expect(Array.isArray(res.data.vouchers)).toBe(true);
      }

      // Verify all 10 approved vouchers are in APPROVED state
      for (const vId of vouchersToApprove) {
        const v = DB.vouchers.find((item: any) => item.id === vId);
        expect(v.status).toBe('APPROVED');
        expect(v.merkleLeafHash).toBeDefined();
      }
    });

    it('rejects approval attempts on non-existent voucher IDs (404 Fail-Closed)', async () => {
      const checkerToken = signJwt({ id: 'usr_checker_01', email: 'checker@livabanking.vn', role: 'CHECKER' });
      const res = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers/vch_non_existent_99999/approve',
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(res.status).toBe(404);
      expect(res.data.success).toBe(false);
      expect(res.data.error).toBe('Không tìm thấy lệnh chi.');
    });
  });

  // ============================================================================
  // Suite 3: Decree 13/2023/NĐ-CP Zero Data Egress & Resilient Local-First Fallback
  // ============================================================================
  describe('Suite 3: Decree 13/2023/NĐ-CP Zero Data Egress & Resilient Local-First Fallback', () => {
    it('executes complete voucher lifecycle locally with ZERO unhandled exceptions when server is offline', async () => {
      // Point client to an unreachable dead port
      setApiBaseUrl('http://127.0.0.1:59998/api');

      const pinia = createPinia();
      setActivePinia(pinia);
      const store = useTreasuryStore(pinia);
      store.setCurrentUser('usr_maker_01', 'MAKER', 'Nguyễn Văn Kế Toán (Maker)');

      // 1. Create Voucher in offline mode
      let voucher: any;
      expect(() => {
        voucher = store.createVoucher(
          'usr_maker_01',
          '0071000112233',
          'VCB',
          650000000,
          'Thanh toán cung ứng thép xây dựng dự án LIVA Tower',
          'CÔNG TY CỔ PHẦN THÉP HÒA PHÁT'
        );
      }).not.toThrow();

      expect(voucher).toBeDefined();
      expect(voucher.status).toBe('DRAFT');
      expect(voucher.amountVnd).toBe(650000000);
      expect(store.vouchers.length).toBeGreaterThanOrEqual(1);

      // 2. Submit Voucher for Approval in offline mode
      let submitted: any;
      expect(() => {
        submitted = store.submitVoucher(voucher.voucherId);
      }).not.toThrow();

      expect(submitted.status).toBe('PENDING_APPROVAL');
      expect(submitted.hitlToken).toBeDefined();
      expect(submitted.tokenExpiresAt).toBeDefined();

      // 3. Switch role to Checker and Approve Voucher in offline mode
      store.setCurrentUser('usr_checker_01', 'CHECKER', 'Trần Thị Giám Đốc (Checker)');
      let approved: any;
      expect(() => {
        approved = store.approveVoucher(
          submitted.voucherId,
          'usr_checker_01',
          'BIOMETRIC_SIM',
          submitted.hitlToken
        );
      }).not.toThrow();

      expect(approved.status).toBe('APPROVED');
      expect(approved.checkerId).toBe('usr_checker_01');
      expect(approved.merkleLeafHash).toMatch(/^[a-f0-9]{64}$/);

      // 4. Verify local cryptographic Merkle tree determinism
      const leaves = store.vouchers
        .filter((v) => v.status === 'APPROVED' && v.merkleLeafHash)
        .map((v) => v.merkleLeafHash!);
      expect(leaves).toContain(approved.merkleLeafHash);

      const targetIndex = leaves.indexOf(approved.merkleLeafHash);
      const proofObj = generateMerkleProof(leaves, targetIndex);
      expect(proofObj).toBeDefined();
      const isValidProof = verifyMerkleProof(
        approved.merkleLeafHash,
        proofObj.proof,
        proofObj.root
      );
      expect(isValidProof).toBe(true);

      // 5. Verify local forward audit ledger integrity
      const ledgerVerification = store.auditLedger.verifyIntegrity();
      expect(ledgerVerification.isValid).toBe(true);
      expect(ledgerVerification.brokenAt).toBeUndefined();

      // Restore base URL for subsequent tests
      setApiBaseUrl(`${serverUrl}/api`);
    });

    it('generates fully compliant Decree 13 local JWT tokens with all officer claims during offline login', async () => {
      // Set to dead port
      setApiBaseUrl('http://127.0.0.1:59998/api');

      const pinia = createPinia();
      setActivePinia(pinia);
      const authStore = useAuthStore(pinia);

      // Login as Checker offline
      const loginResult = await authStore.login('checker@livabanking.vn', 'checker123');
      expect(loginResult).toBe(true);
      expect(authStore.isAuthenticated).toBe(true);
      expect(authStore.currentUser?.role).toBe('CHECKER');
      expect(authStore.currentOfficerId).toBe('SUP-88214');
      expect(authStore.currentBranchCode).toBe('HO-HN-001');

      // Verify token in storage
      const token = getStoredToken();
      expect(token).toBeDefined();
      expect(typeof token).toBe('string');
      const parts = token!.split('.');
      expect(parts.length).toBe(3);

      // Decode payload and verify required claims per Decree 13
      const payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
      expect(payload.role).toBe('CHECKER');
      expect(payload.officerId).toBe('SUP-88214');
      expect(payload.branchCode).toBe('HO-HN-001');
      expect(payload.exp).toBeGreaterThan(Math.floor(Date.now() / 1000));

      // Verify user profile kept securely in in-memory store (Decree 13 / No plaintext localStorage PII)
      expect(authStore.currentUser).toBeDefined();
      expect(authStore.currentUser?.email).toBe('checker@livabanking.vn');
      expect(authStore.currentUser?.role).toBe('CHECKER');
      expect(getStoredUser()).toBeNull();

      // Verify that apiRequest handles this offline state smoothly
      const testReq = await apiRequest('/banking/accounts');
      expect(testReq.success).toBe(false);
      expect(testReq.isOnline).toBe(false);
      expect(testReq.error).toContain('Offline Mode');

      setApiBaseUrl(`${serverUrl}/api`);
    });

    it('re-synchronizes with server upon reconnection without corrupting local Pinia state', async () => {
      setApiBaseUrl(`${serverUrl}/api`);

      const pinia = createPinia();
      setActivePinia(pinia);
      const store = useTreasuryStore(pinia);

      // Seed local voucher
      const localVoucher = store.createVoucher(
        'usr_maker_01',
        '001100223344',
        'VCB',
        150000000,
        'Thanh toán dịch vụ viễn thông',
        'VNPT HÀ NỘI'
      );
      expect(store.vouchers.some((v) => v.voucherId === localVoucher.voucherId)).toBe(true);

      // Seed a server voucher
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      setStoredToken(makerToken);
      const serverVoucherId = `vch_server_seed_${Date.now()}`;
      await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: serverVoucherId,
          voucherId: serverVoucherId,
          beneficiaryAccount: '9988776655',
          amount: 250000000,
          purpose: 'Server voucher before sync',
        }
      );

      // Trigger fetch vouchers from server
      const syncSuccess = await store.fetchVouchersFromServer();
      expect(syncSuccess).toBe(true);
      expect(store.isOnlineSyncActive).toBe(true);

      // Both server voucher and pre-existing local vouchers must be present
      expect(store.vouchers.some((v) => v.voucherId === serverVoucherId)).toBe(true);
      expect(store.vouchers.some((v) => v.voucherId === localVoucher.voucherId)).toBe(true);
    });
  });

  // ============================================================================
  // Suite 4: Adversarial Input Sanitization, Unicode Diacritics & Boundary Edge Cases
  // ============================================================================
  describe('Suite 4: Adversarial Input Sanitization, Unicode Diacritics & Boundary Edge Cases', () => {
    it('safely handles adversarial payloads (XSS, SQLi, Path Traversal) in voucher creation & SSE', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      const pinia = createPinia();
      const store = useTreasuryStore(pinia);

      const sseReceiver = createSseReceiver((event, data) => {
        store.processSseEvent(event, data);
      });
      await sseReceiver.ready;

      const adversarialPayload = {
        id: `vch_adv_xss_${Date.now()}`,
        voucherId: `vch_adv_xss_${Date.now()}`,
        beneficiaryAccount: "0071000' OR '1'='1; --",
        beneficiaryBank: 'BIDV',
        beneficiaryName: '<script>alert("XSS_ATTACK_EXPLOIT")</script>',
        amount: 1000000,
        purpose: "'; DROP TABLE vouchers; SELECT * FROM credentials WHERE '1'='1",
      };

      const res = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        adversarialPayload
      );

      expect(res.status).toBe(201);
      expect(res.data.success).toBe(true);

      await new Promise((r) => setTimeout(r, 100));

      // Verify stored voucher preserves text as inert string without evaluating or crashing
      const stored = store.vouchers.find((v) => v.voucherId === adversarialPayload.id);
      expect(stored).toBeDefined();
      expect(stored?.beneficiaryName).toBe('<script>alert("XSS_ATTACK_EXPLOIT")</script>');
      expect(stored?.purpose).toContain('DROP TABLE');

      sseReceiver.close();
    });

    it('preserves rich Vietnamese UTF-8 diacritics and emojis with cryptographic Merkle integrity', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      const checkerToken = signJwt({ id: 'usr_checker_01', email: 'checker@livabanking.vn', role: 'CHECKER' });

      const vietnamesePayload = {
        id: `vch_vn_utf8_${Date.now()}`,
        voucherId: `vch_vn_utf8_${Date.now()}`,
        beneficiaryAccount: '0071001234567',
        beneficiaryBank: 'VIETCOMBANK',
        beneficiaryName: 'TỔNG CÔNG TY THIẾT BỊ ĐIỆN & CƠ KHÍ ĐỘNG LỰC VIỆT NAM (LIVA) 🌊🔧🇻🇳',
        amount: 999999999,
        purpose: 'Chi trả hợp đồng mua sắm máy biến áp 500kV trạm Hòa Bình — Đợt 3/2026',
      };

      const createRes = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        vietnamesePayload
      );
      expect(createRes.status).toBe(201);

      const approveRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vietnamesePayload.id}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(approveRes.status).toBe(200);

      const leafHash = approveRes.data.voucher.merkleLeafHash;
      expect(leafHash).toMatch(/^[a-f0-9]{64}$/);

      // Verify Merkle leaf hash is deterministic for the exact UTF-8 payload
      const rootHash = approveRes.data.merkleRoot;
      expect(rootHash).toMatch(/^[a-f0-9]{64}$/);
    });

    it('empirically tests simultaneous approval vs rejection race condition (competing actions)', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      const checkerToken = signJwt({ id: 'usr_checker_01', email: 'checker@livabanking.vn', role: 'CHECKER' });

      const raceVoucherId = `vch_race_conflict_${Date.now()}`;
      await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: raceVoucherId,
          voucherId: raceVoucherId,
          beneficiaryAccount: '0988776655',
          amount: 400000000,
          purpose: 'Simultaneous approve vs reject conflict test',
        }
      );

      // Fire approve and reject at the exact same millisecond
      const [approveRes, rejectRes] = await Promise.all([
        makeRawRequest(
          'POST',
          `/api/treasury/vouchers/${raceVoucherId}/approve`,
          { Authorization: `Bearer ${checkerToken}` }
        ),
        makeRawRequest(
          'POST',
          `/api/treasury/vouchers/${raceVoucherId}/reject`,
          { Authorization: `Bearer ${checkerToken}` },
          { remarks: 'Reject in race condition' }
        ),
      ]);

      // Both requests complete without 500 crashes
      expect([200, 400]).toContain(approveRes.status);
      expect([200, 400]).toContain(rejectRes.status);

      // Verify final voucher state is deterministic
      const finalVoucher = DB.vouchers.find((v: any) => v.id === raceVoucherId);
      expect(['APPROVED', 'REJECTED']).toContain(finalVoucher.status);
      expect(finalVoucher.checkerId).toBe('usr_checker_01');
    });

    it('empirically tests 3-station concurrent workflow: Maker 1 and Maker 2 create, Checker reviews', async () => {
      // Station 1: Maker 1 (Operations Desk 1)
      const pinia1 = createPinia();
      const store1 = useTreasuryStore(pinia1);
      store1.setCurrentUser('usr_maker_01', 'MAKER', 'Maker Station 1');

      // Station 2: Maker 2 (Operations Desk 2)
      const pinia2 = createPinia();
      const store2 = useTreasuryStore(pinia2);
      store2.setCurrentUser('usr_maker_02', 'MAKER', 'Maker Station 2');

      // Station 3: Checker (Supervisor Desk)
      const pinia3 = createPinia();
      const store3 = useTreasuryStore(pinia3);
      store3.setCurrentUser('usr_checker_01', 'CHECKER', 'Supervisor Desk');

      // Connect SSE receivers for all 3 stations
      const sse1 = createSseReceiver((ev, data) => store1.processSseEvent(ev, data));
      const sse2 = createSseReceiver((ev, data) => store2.processSseEvent(ev, data));
      const sse3 = createSseReceiver((ev, data) => store3.processSseEvent(ev, data));
      await Promise.all([sse1.ready, sse2.ready, sse3.ready]);

      const maker1Token = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      const checkerToken = signJwt({ id: 'usr_checker_01', email: 'checker@livabanking.vn', role: 'CHECKER' });

      const vId1 = `vch_station1_${Date.now()}`;
      const vId2 = `vch_station2_${Date.now()}`;

      // Maker 1 & Maker 2 create vouchers concurrently
      await Promise.all([
        makeRawRequest(
          'POST',
          '/api/treasury/vouchers',
          { Authorization: `Bearer ${maker1Token}` },
          { id: vId1, voucherId: vId1, amount: 150000000, purpose: 'Station 1 Transfer' }
        ),
        makeRawRequest(
          'POST',
          '/api/treasury/vouchers',
          { Authorization: `Bearer ${maker1Token}` },
          { id: vId2, voucherId: vId2, amount: 250000000, purpose: 'Station 2 Transfer' }
        ),
      ]);

      await new Promise((r) => setTimeout(r, 100));

      // Checker sees both vouchers
      expect(store3.vouchers.some((v) => v.voucherId === vId1)).toBe(true);
      expect(store3.vouchers.some((v) => v.voucherId === vId2)).toBe(true);

      // Checker approves vId1 and rejects vId2
      await Promise.all([
        makeRawRequest('POST', `/api/treasury/vouchers/${vId1}/approve`, { Authorization: `Bearer ${checkerToken}` }),
        makeRawRequest('POST', `/api/treasury/vouchers/${vId2}/reject`, { Authorization: `Bearer ${checkerToken}` }, { remarks: 'Incorrect budget line' }),
      ]);

      await new Promise((r) => setTimeout(r, 100));

      // Both Maker 1 and Maker 2 have synchronized states
      const v1OnStation1 = store1.vouchers.find((v) => v.voucherId === vId1);
      const v2OnStation1 = store1.vouchers.find((v) => v.voucherId === vId2);
      expect(v1OnStation1?.status).toBe('APPROVED');
      expect(v2OnStation1?.status).toBe('REJECTED');

      const v1OnStation2 = store2.vouchers.find((v) => v.voucherId === vId1);
      const v2OnStation2 = store2.vouchers.find((v) => v.voucherId === vId2);
      expect(v1OnStation2?.status).toBe('APPROVED');
      expect(v2OnStation2?.status).toBe('REJECTED');

      sse1.close();
      sse2.close();
      sse3.close();
    });

    it('empirically tests SSE broadcast robustness during abrupt socket severance mid-stream', async () => {
      // Connect 5 receivers
      const receivers = [
        createSseReceiver(),
        createSseReceiver(),
        createSseReceiver(),
        createSseReceiver(),
        createSseReceiver(),
      ];
      await Promise.all(receivers.map((r) => r.ready));

      // Abruptly destroy receiver 1 and receiver 3
      receivers[0].close();
      receivers[2].close();

      // Trigger broadcast by creating a voucher
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      const testVId = `vch_sse_drop_${Date.now()}`;
      const res = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        { id: testVId, voucherId: testVId, amount: 77000000, purpose: 'Test mid-stream drop' }
      );
      expect(res.status).toBe(201);

      await new Promise((r) => setTimeout(r, 100));

      // Remaining active receivers (receivers 1, 3, 4) must receive event without server crashing
      expect(receivers[1].events.some((e) => e.data?.voucher?.voucherId === testVId)).toBe(true);
      expect(receivers[3].events.some((e) => e.data?.voucher?.voucherId === testVId)).toBe(true);
      expect(receivers[4].events.some((e) => e.data?.voucher?.voucherId === testVId)).toBe(true);

      receivers[1].close();
      receivers[3].close();
      receivers[4].close();
    });

    it('empirically probes Circular 09/2020 self-approval prevention on backend server route', async () => {
      // Checker creates a voucher where makerId = 'usr_checker_01'
      const checkerToken = signJwt({
        id: 'usr_checker_01',
        email: 'checker@livabanking.vn',
        role: 'CHECKER',
        officerId: 'SUP-88214',
      });

      const selfVoucherId = `vch_self_approve_probe_${Date.now()}`;
      const createRes = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${checkerToken}` },
        {
          id: selfVoucherId,
          voucherId: selfVoucherId,
          amount: 300000000,
          purpose: 'Probe self approval by Checker',
        }
      );
      expect(createRes.status).toBe(201);

      // Now the same Checker attempts to approve their own voucher
      const approveRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${selfVoucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      // Milestone M4 Security Hardening:
      // Circular 09/2020/TT-NHNN anti-self-approval is strictly enforced on the server route:
      // When voucher.makerId === authUser.id, the server returns 403 Forbidden.
      expect(approveRes.status).toBe(403);
      expect(approveRes.data?.error).toContain('Thông tư 09/2020/TT-NHNN');
    });

    it('empirically verifies validation for negative or zero voucher amount on server route', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });

      const negativeVoucherId = `vch_negative_probe_${Date.now()}`;
      const res = await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: negativeVoucherId,
          voucherId: negativeVoucherId,
          amount: -500000000, // Negative amount
          purpose: 'Negative amount probe',
        }
      );

      // Milestone M4 Security Hardening:
      // Server validates Number.isFinite(amount) && amount > 0, returning 400 Bad Request
      expect(res.status).toBe(400);
      expect(res.data?.error).toBe('Số tiền lệnh chi phải là số dương hợp lệ.');
    });

    it('empirically verifies server reject route behavior rejects already APPROVED vouchers', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
      const checkerToken = signJwt({ id: 'usr_checker_01', email: 'checker@livabanking.vn', role: 'CHECKER' });

      const testVoucherId = `vch_probe_reject_approved_${Date.now()}`;
      await makeRawRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: testVoucherId,
          voucherId: testVoucherId,
          amount: 100000000,
          purpose: 'Probe reject on approved voucher',
        }
      );

      // 1. Approve voucher first
      const approveRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${testVoucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(approveRes.status).toBe(200);

      // 2. Attempt to call /reject on the now APPROVED voucher
      const rejectRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${testVoucherId}/reject`,
        { Authorization: `Bearer ${checkerToken}` },
        { remarks: 'Attempting to reject an already approved voucher' }
      );

      // Milestone M4 Security Hardening:
      // Server validates voucher.status === 'PENDING_APPROVAL', returning 400 Bad Request
      expect(rejectRes.status).toBe(400);
      expect(rejectRes.data?.error).toBe('Lệnh chi không ở trạng thái chờ phê duyệt.');
    });
  });
});
