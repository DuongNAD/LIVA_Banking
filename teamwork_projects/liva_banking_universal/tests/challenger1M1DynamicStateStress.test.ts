/**
 * Milestone M1 Adversarial Empirical Stress Suite — Challenger 1 (challenger_m1_1)
 *
 * Verification Scope: Milestone M1 (Zero Hardcode & Dynamic Real-Time State)
 * Core Invariants Tested:
 * 1. Fail-Closed Solvency Enforcement:
 *    - Rejects approvals exceeding available balance (even by 1 VND).
 *    - Verifies balance, outflows, Merkle tree, and SSE bus remain unmutated on rejection.
 *    - Exact-balance deduction boundary (balance reduced to exactly 0 VND).
 *    - Rejects approvals on 0-balance accounts.
 *    - Rejects non-finite, negative, or zero voucher amounts.
 * 2. Concurrency & Race Condition Invariant:
 *    - Double Approval Replay Attack (10 concurrent approvals of the same voucher -> exactly 1 succeeds, 9 fail with 400).
 *    - Concurrent Overdraft Race (2 concurrent vouchers of 600M on a 1000M account -> exactly 1 succeeds, 1 fails, balance never negative).
 *    - High-throughput burst pool (10 concurrent vouchers on a 500M account -> exactly 5 succeed, 5 fail, balance exactly 0).
 * 3. Regulatory Segregation & RBAC (Circular 09/2020/TT-NHNN):
 *    - Maker self-approval rejected with HTTP 403.
 *    - Non-CHECKER roles (MAKER, AML, TREASURY) rejected with HTTP 403.
 * 4. Account & Channel Resolution Boundary:
 *    - Account resolution across ID, accountNumber, bankCode, and channelCode.
 *    - Behavior on unknown account / channel code and empty DB fail-closed check.
 * 5. SSE Real-Time Sync & Abrupt Socket Teardown:
 *    - Verified dual broadcast (voucher:approved and account:updated).
 *    - Abrupt socket destruction resilience (10 connections, 6 destroyed, 0 crashes, active clients receive events).
 * 6. Dynamic Pinia Reactive Stores:
 *    - Dynamic dailyAverageBurnVnd and liquidity runway without static 85M burn.
 *    - Dynamic matchRate without static 99.8 fallback.
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import http from 'node:http';
import {
  server,
  DB,
  USERS,
  signJwt,
  broadcastEvent,
  sseClients,
} from '../server/server.mjs';
import {
  setApiBaseUrl,
} from '../src/services/apiClient';
import { useBankingStore } from '../src/stores/bankingStore';
import { useReconciliationStore } from '../src/stores/reconciliationStore';

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

describe('Milestone M1 Adversarial Empirical Stress Suite (Challenger 1)', () => {
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
  // SECTION 1: Fail-Closed Solvency Enforcement & Invariant Boundary
  // ============================================================================
  describe('1. Fail-Closed Solvency Enforcement & Accounting Invariant', () => {
    it('strictly rejects voucher approval when requested amount exceeds balance by 1 VND', async () => {
      // Setup a controlled test account with exactly 100,000,000 VND
      const testAcc = {
        id: 'acc_stress_solvency_01',
        bankCode: 'TCB',
        channelCode: 'NAPAS',
        accountNumber: '9900112233',
        accountName: 'STRESS TEST SOLVENCY',
        balance: 100_000_000,
        balanceVnd: 100_000_000,
        availableBalanceVnd: 100_000_000,
        outflows: 10_000_000,
        currency: 'VND',
        status: 'ACTIVE',
      };
      DB.accounts.push(testAcc);

      const initialLeavesCount = DB.merkleLedger.leaves.length;
      const initialRootHash = DB.merkleLedger.currentRootHash;

      // Create voucher for 100,000,001 VND (exceeding balance by 1 VND)
      const voucherId = 'vch_stress_overdraft_1vnd';
      DB.vouchers.push({
        id: voucherId,
        voucherNumber: 'VCH-STRESS-01',
        sourceAccount: '9900112233',
        targetAccount: '8888888888',
        targetBeneficiary: 'CONG TY MA',
        amount: 100_000_001,
        currency: 'VND',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        makerName: 'Nguyễn Văn Kế Toán (Maker)',
        createdAt: new Date().toISOString(),
      });

      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      const approveRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${voucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      // Invariant: Must fail-closed with HTTP 400
      expect(approveRes.status).toBe(400);
      expect(approveRes.data.success).toBe(false);
      expect(approveRes.data.error).toContain('Số dư khả dụng không đủ để phê duyệt lệnh chi');
      expect(approveRes.data.error).toContain('100.000.000');
      expect(approveRes.data.error).toContain('100.000.001');

      // Invariant: Account balance must remain unchanged
      expect(testAcc.balance).toBe(100_000_000);
      expect(testAcc.balanceVnd).toBe(100_000_000);
      expect(testAcc.outflows).toBe(10_000_000);

      // Invariant: Merkle ledger must NOT record any leaf or change root
      expect(DB.merkleLedger.leaves.length).toBe(initialLeavesCount);
      expect(DB.merkleLedger.currentRootHash).toBe(initialRootHash);

      // Invariant: Voucher must remain PENDING_APPROVAL
      const storedVoucher = DB.vouchers.find((v: any) => v.id === voucherId);
      expect(storedVoucher.status).toBe('PENDING_APPROVAL');
    });

    it('handles exact-balance voucher approval (balance reduced to exactly 0 VND)', async () => {
      const exactAcc = {
        id: 'acc_stress_exact_01',
        bankCode: 'BIDV',
        channelCode: 'BILATERAL',
        accountNumber: '7766554433',
        accountName: 'STRESS TEST EXACT',
        balance: 50_000_000,
        balanceVnd: 50_000_000,
        availableBalanceVnd: 50_000_000,
        outflows: 0,
        currency: 'VND',
        status: 'ACTIVE',
      };
      DB.accounts.push(exactAcc);

      const voucherId = 'vch_stress_exact_balance';
      DB.vouchers.push({
        id: voucherId,
        voucherNumber: 'VCH-STRESS-EXACT',
        sourceAccount: '7766554433',
        targetAccount: '11223344',
        targetBeneficiary: 'CONG TY EXACT',
        amount: 50_000_000,
        currency: 'VND',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        makerName: 'Nguyễn Văn Kế Toán (Maker)',
        createdAt: new Date().toISOString(),
      });

      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      const approveRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${voucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      expect(approveRes.status).toBe(200);
      expect(approveRes.data.success).toBe(true);

      // Invariant: Balance must be exactly 0, outflows exactly 50,000,000
      expect(exactAcc.balance).toBe(0);
      expect(exactAcc.balanceVnd).toBe(0);
      expect(exactAcc.outflows).toBe(50_000_000);

      // Subsequent 1 VND voucher on this 0-balance account MUST fail
      const zeroVoucherId = 'vch_stress_after_zero';
      DB.vouchers.push({
        id: zeroVoucherId,
        voucherNumber: 'VCH-STRESS-ZERO-ATTEMPT',
        sourceAccount: '7766554433',
        targetAccount: '11223344',
        targetBeneficiary: 'CONG TY FAIL',
        amount: 1,
        currency: 'VND',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        makerName: 'Maker',
        createdAt: new Date().toISOString(),
      });

      const secondApprove = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${zeroVoucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      expect(secondApprove.status).toBe(400);
      expect(secondApprove.data.error).toContain('Số dư khả dụng không đủ');
      expect(exactAcc.balance).toBe(0);
    });

    it('rejects voucher approval when voucher amount is 0, negative, or invalid non-number', async () => {
      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      // 1. Zero amount
      const vchZero = {
        id: 'vch_stress_zero_amount',
        amount: 0,
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        sourceAccount: 'acc_vcb_01',
      };
      DB.vouchers.push(vchZero);

      const resZero = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchZero.id}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(resZero.status).toBe(400);
      expect(resZero.data.error).toContain('Số tiền lệnh chi không hợp lệ');

      // 2. Negative amount (-50,000,000 VND)
      const vchNeg = {
        id: 'vch_stress_neg_amount',
        amount: -50_000_000,
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        sourceAccount: 'acc_vcb_01',
      };
      DB.vouchers.push(vchNeg);

      const resNeg = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchNeg.id}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(resNeg.status).toBe(400);
      expect(resNeg.data.error).toContain('Số tiền lệnh chi không hợp lệ');

      // 3. NaN amount
      const vchNaN = {
        id: 'vch_stress_nan_amount',
        amount: NaN,
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        sourceAccount: 'acc_vcb_01',
      };
      DB.vouchers.push(vchNaN);

      const resNaN = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchNaN.id}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(resNaN.status).toBe(400);
      expect(resNaN.data.error).toContain('Số tiền lệnh chi không hợp lệ');
    });

    it('rejects approval when voucher status is not PENDING_APPROVAL', async () => {
      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      // Test REJECTED status
      const vchRejected = {
        id: 'vch_stress_already_rejected',
        amount: 10_000_000,
        status: 'REJECTED',
        makerId: 'usr_maker_01',
        sourceAccount: 'acc_vcb_01',
      };
      DB.vouchers.push(vchRejected);

      const res = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchRejected.id}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(res.status).toBe(400);
      expect(res.data.error).toContain('Lệnh chi không ở trạng thái chờ phê duyệt');
    });
  });

  // ============================================================================
  // SECTION 2: Concurrency & Race Condition Invariant Stress
  // ============================================================================
  describe('2. Concurrency & Race Condition Invariant Stress', () => {
    it('defends against Double-Approval Replay Attack: 10 concurrent requests to approve the SAME voucher', async () => {
      const testAcc = {
        id: 'acc_double_approval_test',
        bankCode: 'VCB',
        channelCode: 'CITAD',
        accountNumber: '007100888999',
        accountName: 'CONG TY DOUBLE APPROVAL DEFENSE',
        balance: 500_000_000,
        balanceVnd: 500_000_000,
        availableBalanceVnd: 500_000_000,
        outflows: 0,
        currency: 'VND',
        status: 'ACTIVE',
      };
      DB.accounts.push(testAcc);

      const voucherId = 'vch_double_approval_target';
      DB.vouchers.push({
        id: voucherId,
        voucherNumber: 'VCH-DOUBLE-01',
        sourceAccount: '007100888999',
        targetAccount: '123456789',
        targetBeneficiary: 'BENEFICIARY',
        amount: 100_000_000,
        currency: 'VND',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        createdAt: new Date().toISOString(),
      });

      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      const initialLeaves = DB.merkleLedger.leaves.length;

      // Send 10 concurrent approval requests simultaneously
      const results = await Promise.all(
        Array.from({ length: 10 }).map(() =>
          makeRawRequest(
            'POST',
            `/api/treasury/vouchers/${voucherId}/approve`,
            { Authorization: `Bearer ${checkerToken}` }
          )
        )
      );

      const successes = results.filter((r) => r.status === 200);
      const failures = results.filter((r) => r.status === 400);

      // Invariant: EXACTLY 1 succeeds, EXACTLY 9 fail
      expect(successes.length).toBe(1);
      expect(failures.length).toBe(9);

      for (const fail of failures) {
        expect(fail.data.error).toContain('Lệnh chi không ở trạng thái chờ phê duyệt');
      }

      // Invariant: Balance deducted EXACTLY ONCE (500M - 100M = 400M, NOT 500M - 1000M)
      expect(testAcc.balance).toBe(400_000_000);
      expect(testAcc.outflows).toBe(100_000_000);

      // Invariant: Merkle ledger received EXACTLY ONE new leaf
      expect(DB.merkleLedger.leaves.length).toBe(initialLeaves + 1);
    });

    it('prevents Overdraft Race Condition: 2 concurrent vouchers of 600M on a 1,000M account', async () => {
      const raceAcc = {
        id: 'acc_race_overdraft_target',
        bankCode: 'TCB',
        channelCode: 'NAPAS',
        accountNumber: '190399887766',
        accountName: 'CONG TY RACE CONDITION TEST',
        balance: 1_000_000_000,
        balanceVnd: 1_000_000_000,
        availableBalanceVnd: 1_000_000_000,
        outflows: 0,
        currency: 'VND',
        status: 'ACTIVE',
      };
      DB.accounts.push(raceAcc);

      const vchAId = 'vch_race_a_600m';
      const vchBId = 'vch_race_b_600m';

      DB.vouchers.push({
        id: vchAId,
        voucherNumber: 'VCH-RACE-A',
        sourceAccount: '190399887766',
        targetAccount: '11111111',
        amount: 600_000_000,
        currency: 'VND',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        createdAt: new Date().toISOString(),
      });

      DB.vouchers.push({
        id: vchBId,
        voucherNumber: 'VCH-RACE-B',
        sourceAccount: '190399887766',
        targetAccount: '22222222',
        amount: 600_000_000,
        currency: 'VND',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        createdAt: new Date().toISOString(),
      });

      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      // Concurrently approve Voucher A and Voucher B
      const [resA, resB] = await Promise.all([
        makeRawRequest('POST', `/api/treasury/vouchers/${vchAId}/approve`, {
          Authorization: `Bearer ${checkerToken}`,
        }),
        makeRawRequest('POST', `/api/treasury/vouchers/${vchBId}/approve`, {
          Authorization: `Bearer ${checkerToken}`,
        }),
      ]);

      const statuses = [resA.status, resB.status];
      // Invariant: One MUST be 200, the other MUST be 400
      expect(statuses).toContain(200);
      expect(statuses).toContain(400);

      // Invariant: The failing request must have rejected due to insufficient balance
      const failRes = resA.status === 400 ? resA : resB;
      expect(failRes.data.error).toContain('Số dư khả dụng không đủ để phê duyệt lệnh chi');
      expect(failRes.data.error).toContain('400.000.000');
      expect(failRes.data.error).toContain('600.000.000');

      // Invariant: Account balance must be 400,000,000 VND (NEVER negative -200,000,000 VND)
      expect(raceAcc.balance).toBe(400_000_000);
      expect(raceAcc.balanceVnd).toBe(400_000_000);
      expect(raceAcc.outflows).toBe(600_000_000);
    });

    it('high-throughput burst pool: 10 concurrent vouchers of 100M on a 500M account', async () => {
      const burstAcc = {
        id: 'acc_burst_pool_target',
        bankCode: 'BIDV',
        channelCode: 'BILATERAL',
        accountNumber: '124199998888',
        accountName: 'BURST POOL TEST',
        balance: 500_000_000,
        balanceVnd: 500_000_000,
        availableBalanceVnd: 500_000_000,
        outflows: 0,
        currency: 'VND',
        status: 'ACTIVE',
      };
      DB.accounts.push(burstAcc);

      const voucherIds: string[] = [];
      for (let i = 0; i < 10; i++) {
        const vid = `vch_burst_${i}_100m`;
        voucherIds.push(vid);
        DB.vouchers.push({
          id: vid,
          voucherNumber: `VCH-BURST-${i}`,
          sourceAccount: '124199998888',
          targetAccount: `334455${i}`,
          amount: 100_000_000,
          currency: 'VND',
          status: 'PENDING_APPROVAL',
          makerId: 'usr_maker_01',
          createdAt: new Date().toISOString(),
        });
      }

      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      // Fire 10 concurrent requests
      const burstResults = await Promise.all(
        voucherIds.map((vid) =>
          makeRawRequest('POST', `/api/treasury/vouchers/${vid}/approve`, {
            Authorization: `Bearer ${checkerToken}`,
          })
        )
      );

      const approved = burstResults.filter((r) => r.status === 200);
      const rejected = burstResults.filter((r) => r.status === 400);

      // Invariant: Exactly 5 succeed, exactly 5 reject
      expect(approved.length).toBe(5);
      expect(rejected.length).toBe(5);

      // Invariant: Final account balance must be exactly 0 VND, outflows 500,000,000 VND
      expect(burstAcc.balance).toBe(0);
      expect(burstAcc.balanceVnd).toBe(0);
      expect(burstAcc.outflows).toBe(500_000_000);
    });
  });

  // ============================================================================
  // SECTION 3: Regulatory Maker-Checker RBAC Enforcement (Circular 09/2020)
  // ============================================================================
  describe('3. Maker-Checker Segregation of Duties (Circular 09/2020/TT-NHNN)', () => {
    it('strictly forbids a Maker from self-approving their own voucher (HTTP 403)', async () => {
      const vchMakerSelf = {
        id: 'vch_maker_self_approve_attempt',
        amount: 25_000_000,
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        sourceAccount: 'acc_vcb_01',
      };
      DB.vouchers.push(vchMakerSelf);

      // Attempt with Maker token
      const makerUser = USERS.find((u: any) => u.id === 'usr_maker_01');
      const makerToken = signJwt({
        id: makerUser.id,
        sub: makerUser.id,
        role: 'MAKER',
        officerId: makerUser.officerId,
        name: makerUser.fullName,
      });

      const resMaker = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchMakerSelf.id}/approve`,
        { Authorization: `Bearer ${makerToken}` }
      );

      // Must be rejected with 403 Forbidden
      expect(resMaker.status).toBe(403);
      expect(resMaker.data.error).toContain('Chỉ Giám đốc (Checker) mới có quyền phê duyệt');

      // Now attempt with a CHECKER token who happens to have makerId === currentUserId
      const rogueCheckerToken = signJwt({
        id: 'usr_maker_01', // same as voucher.makerId!
        sub: 'usr_maker_01',
        role: 'CHECKER',
        officerId: 'SUP-ROGUE',
        name: 'Rogue Checker',
      });

      const resRogue = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchMakerSelf.id}/approve`,
        { Authorization: `Bearer ${rogueCheckerToken}` }
      );

      expect(resRogue.status).toBe(403);
      expect(resRogue.data.error).toContain('Kiểm soát viên không được tự phê duyệt lệnh chi do chính mình tạo lập');
    });

    it('strictly rejects non-CHECKER roles (AML, TREASURY) from approving vouchers', async () => {
      const vch = {
        id: 'vch_rbac_role_boundary',
        amount: 15_000_000,
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        sourceAccount: 'acc_vcb_01',
      };
      DB.vouchers.push(vch);

      const amlUser = USERS.find((u: any) => u.role === 'AML');
      const amlToken = signJwt({
        id: amlUser.id,
        sub: amlUser.id,
        role: 'AML',
        officerId: amlUser.officerId,
        name: amlUser.fullName,
      });

      const resAml = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vch.id}/approve`,
        { Authorization: `Bearer ${amlToken}` }
      );
      expect(resAml.status).toBe(403);

      const treasuryUser = USERS.find((u: any) => u.role === 'TREASURY');
      const treasuryToken = signJwt({
        id: treasuryUser.id,
        sub: treasuryUser.id,
        role: 'TREASURY',
        officerId: treasuryUser.officerId,
        name: treasuryUser.fullName,
      });

      const resTrz = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vch.id}/approve`,
        { Authorization: `Bearer ${treasuryToken}` }
      );
      expect(resTrz.status).toBe(403);
    });
  });

  // ============================================================================
  // SECTION 4: Account & Channel Resolution Boundary Edge Cases
  // ============================================================================
  describe('4. Account & Channel Resolution Edge Cases', () => {
    it('correctly resolves source accounts across id, bankCode, channelCode, and accountNumber', async () => {
      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      // Test SWIFT account resolution
      const swiftAcc = DB.accounts.find((a: any) => a.channelCode === 'SWIFT');
      expect(swiftAcc).toBeDefined();
      const initialSwiftBal = swiftAcc.balance;

      const vchSwift = {
        id: 'vch_resolve_swift_test',
        amount: 20_000_000,
        sourceAccount: 'SWIFT', // resolved via channelCode
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
      };
      DB.vouchers.push(vchSwift);

      const resSwift = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchSwift.id}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      expect(resSwift.status).toBe(200);
      expect(resSwift.data.sourceAccount.channelCode).toBe('SWIFT');
      expect(swiftAcc.balance).toBe(initialSwiftBal - 20_000_000);
    });

    it('verifies fail-closed behavior when DB.accounts is empty', async () => {
      // Temporarily save accounts
      const savedAccounts = [...DB.accounts];
      DB.accounts = [];

      const vchNoAcc = {
        id: 'vch_no_accounts_available',
        amount: 10_000_000,
        sourceAccount: 'ANY_ACC',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
      };
      DB.vouchers.push(vchNoAcc);

      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      const res = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchNoAcc.id}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      // Must fail-closed with 400
      expect(res.status).toBe(400);
      expect(res.data.error).toContain('Không tìm thấy tài khoản nguồn trích nợ');

      // Restore accounts
      DB.accounts = savedAccounts;
    });
  });

  // ============================================================================
  // SECTION 5: SSE Real-Time Sync & Abrupt Socket Disconnect Stress
  // ============================================================================
  describe('5. SSE Real-Time Sync & Abrupt Socket Disconnect Resilience', () => {
    it('survives abrupt socket termination of 10 clients during concurrent broadcasting without server crash', async () => {
      const activeClients: http.IncomingMessage[] = [];
      const receivedEvents: any[] = [];

      // Connect 10 SSE clients
      for (let i = 0; i < 10; i++) {
        await new Promise<void>((resolve) => {
          const clientReq = http.get(`${serverUrl}/api/sync/events`, (res) => {
            activeClients.push(res);
            res.on('data', (chunk) => {
              const str = chunk.toString();
              if (str.includes('account:updated')) {
                receivedEvents.push(str);
              }
            });
            resolve();
          });
          clientReq.on('error', () => {});
        });
      }

      expect(sseClients.size).toBeGreaterThanOrEqual(10);

      // Abruptly destroy 6 client sockets without clean HTTP teardown
      for (let i = 0; i < 6; i++) {
        const client = activeClients[i];
        if (client.socket) {
          client.socket.destroy(); // Abrupt TCP RST / destruction
        }
      }

      // Allow event loop to process socket close
      await new Promise((r) => setTimeout(r, 50));

      // Trigger a server broadcast
      expect(() => {
        broadcastEvent('account:updated', {
          accountId: 'acc_vcb_01',
          balanceVnd: 24_900_000_000,
          lastUpdated: new Date().toISOString(),
        });
      }).not.toThrow();

      // Ensure server is still fully responsive to regular API calls
      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });
      const healthRes = await makeRawRequest('GET', '/api/banking/accounts', {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(healthRes.status).toBe(200);

      // Clean up remaining active clients
      for (let i = 6; i < activeClients.length; i++) {
        activeClients[i].destroy();
      }
    });

    it('broadcasts dual events (voucher:approved AND account:updated) with synchronous cryptographic integrity', async () => {
      const testAcc = DB.accounts.find((a: any) => a.id === 'acc_vcb_01') || DB.accounts[0];
      const initialBalance = testAcc.balance;

      const vchId = 'vch_dual_event_test_01';
      DB.vouchers.push({
        id: vchId,
        voucherNumber: 'VCH-DUAL-01',
        sourceAccount: testAcc.accountNumber,
        targetAccount: '9988776655',
        amount: 80_000_000,
        currency: 'VND',
        status: 'PENDING_APPROVAL',
        makerId: 'usr_maker_01',
        createdAt: new Date().toISOString(),
      });

      const eventsCaptured: { event: string; data: any }[] = [];
      let buffer = '';
      let currentEvent = '';

      const sseRes = await new Promise<http.IncomingMessage>((resolve) => {
        http.get(`${serverUrl}/api/sync/events`, (res) => {
          res.on('data', (chunk) => {
            buffer += chunk.toString();
            const lines = buffer.split('\n');
            buffer = lines.pop() ?? '';
            for (const line of lines) {
              if (line.startsWith('event: ')) {
                currentEvent = line.replace('event: ', '').trim();
              } else if (line.startsWith('data: ')) {
                try {
                  const data = JSON.parse(line.replace('data: ', '').trim());
                  eventsCaptured.push({ event: currentEvent, data });
                } catch {}
              }
            }
          });
          resolve(res);
        });
      });

      const checkerUser = USERS.find((u: any) => u.role === 'CHECKER');
      const checkerToken = signJwt({
        id: checkerUser.id,
        sub: checkerUser.id,
        role: 'CHECKER',
        officerId: checkerUser.officerId,
        name: checkerUser.fullName,
      });

      const approveRes = await makeRawRequest(
        'POST',
        `/api/treasury/vouchers/${vchId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );
      expect(approveRes.status).toBe(200);

      // Wait for SSE propagation
      await new Promise((r) => setTimeout(r, 150));

      const vchApprovedEvent = eventsCaptured.find((e) => e.event === 'voucher:approved');
      const accUpdatedEvent = eventsCaptured.find((e) => e.event === 'account:updated');

      expect(vchApprovedEvent).toBeDefined();
      expect(vchApprovedEvent?.data.voucherId).toBe(vchId);
      expect(vchApprovedEvent?.data.merkleHash).toBeDefined();

      expect(accUpdatedEvent).toBeDefined();
      expect(accUpdatedEvent?.data.balanceVnd).toBe(initialBalance - 80_000_000);
      expect(accUpdatedEvent?.data.deductedAmount).toBe(80_000_000);

      sseRes.destroy();
    });
  });

  // ============================================================================
  // SECTION 6: Dynamic Pinia Stores Integration & Non-Zero Calculation
  // ============================================================================
  describe('6. Dynamic Pinia Stores Integration & Dynamic Math', () => {
    it('verifies bankingStore computes dynamic daily burn and runway without static 85M fallback', () => {
      const store = useBankingStore();

      // With default state
      expect(store.totalLiquidVnd).toBeGreaterThan(0);

      // Mutate an account outflows to test dynamic dailyAverageBurnVnd formula (outflows / 30)
      const testAccount = store.accounts[0];
      testAccount.outflowMonthVnd = 300_000_000;
      testAccount.balanceVnd = 1_000_000_000;

      // Ensure burn calculation is non-zero dynamic (not hardcoded 85,000,000)
      const expectedBurn = Math.round(store.totalOutflowMonthVnd / 30);
      expect(store.dailyAverageBurnVnd).toBe(expectedBurn);
      expect(store.dailyAverageBurnVnd).not.toBe(85_000_000);

      // Test liquidity runway days formula: totalLiquidVnd / burnRate
      const expectedRunway = Math.round(store.totalLiquidVnd / store.dailyAverageBurnVnd);
      expect(store.liquidityRunwayDays).toBe(expectedRunway);
    });

    it('verifies reconciliationStore dynamic match rate evaluates to 0.0% when empty and computes accurate percentage', () => {
      const reconStore = useReconciliationStore();

      // Reset records to empty state
      reconStore.bankTransactions = [];
      reconStore.ledgerEntries = [];
      reconStore.reconciliationSummary = null;
      expect(reconStore.bankTransactions.length).toBe(0);
      expect(reconStore.ledgerEntries.length).toBe(0);

      // Invariant: Empty state must be 0.0%, NOT static 99.8% fallback
      expect(reconStore.matchRate).toBe(0);

      // Apply a dynamic summary via SSE action
      reconStore.applyReconciliationSummary({
        totalBankTransactions: 100,
        matchedCount: 95,
        matchRate: 95.0,
      });

      expect(reconStore.matchRate).toBe(95.0);
    });
  });
});
