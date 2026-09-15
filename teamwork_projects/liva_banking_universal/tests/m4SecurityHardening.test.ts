/**
 * LIVA Banking Universal — Milestone M4 Security Hardening Verification Suite
 * Defense-in-depth security verification for:
 * 1. verifyJwt: crypto.timingSafeEqual length checks & strict payload.exp !== undefined checks.
 * 2. POST /api/treasury/vouchers: Number.isFinite(amount) && amount > 0 positive amount validation.
 * 3. POST /api/treasury/vouchers/:id/approve: Circular 09/2020/TT-NHNN anti-self-approval enforcement.
 * 4. POST /api/treasury/vouchers/:id/reject: Status validation (PENDING_APPROVAL required).
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import http from 'node:http';
import {
  server,
  DB,
  signJwt,
  verifyJwt,
} from '../server/server.mjs';

let serverPort: number;
let serverUrl: string;

function makeRequest(
  method: string,
  path: string,
  headers: Record<string, string> = {},
  body?: any
): Promise<{ status: number; headers: http.IncomingHttpHeaders; data: any }> {
  return new Promise((resolve, reject) => {
    const payload = body ? JSON.stringify(body) : '';
    const reqHeaders: Record<string, string> = {
      ...headers,
    };
    if (body) {
      reqHeaders['Content-Type'] = 'application/json';
      reqHeaders['Content-Length'] = String(Buffer.byteLength(payload));
    }

    const req = http.request(`${serverUrl}${path}`, { method, headers: reqHeaders }, (res) => {
      let raw = '';
      res.on('data', (chunk) => (raw += chunk));
      res.on('end', () => {
        try {
          const parsed = JSON.parse(raw);
          resolve({ status: res.statusCode || 0, headers: res.headers, data: parsed });
        } catch {
          resolve({ status: res.statusCode || 0, headers: res.headers, data: raw });
        }
      });
    });

    req.on('error', reject);
    if (body) req.write(payload);
    req.end();
  });
}

describe('Milestone M4: Final Verification & Security Hardening Suite', () => {
  beforeAll(async () => {
    await new Promise<void>((resolve) => {
      server.listen(0, () => {
        const addr = server.address();
        if (typeof addr === 'object' && addr) {
          serverPort = addr.port;
          serverUrl = `http://127.0.0.1:${serverPort}`;
        }
        resolve();
      });
    });
  });

  afterAll(async () => {
    await new Promise<void>((resolve) => {
      server.close(() => resolve());
    });
  });

  // ============================================================================
  // SUITE 1: verifyJwt Security Hardening
  // ============================================================================
  describe('Task 1.1: verifyJwt Defense-in-Depth Hardening', () => {
    it('timingSafeEqual safely handles mismatched signature buffer lengths without throwing', () => {
      const validToken = signJwt({ id: 'usr_test_01', role: 'MAKER' });
      const parts = validToken.split('.');

      // 1. Shorter signature (1 character)
      const shortSigToken = `${parts[0]}.${parts[1]}.a`;
      expect(() => verifyJwt(shortSigToken)).not.toThrow();
      expect(verifyJwt(shortSigToken)).toBeNull();

      // 2. Longer signature (100 characters)
      const longSigToken = `${parts[0]}.${parts[1]}.${'x'.repeat(100)}`;
      expect(() => verifyJwt(longSigToken)).not.toThrow();
      expect(verifyJwt(longSigToken)).toBeNull();

      // 3. Same length signature but wrong bytes
      const sameLenBogusSig = `${parts[0]}.${parts[1]}.${'a'.repeat(parts[2].length)}`;
      expect(() => verifyJwt(sameLenBogusSig)).not.toThrow();
      expect(verifyJwt(sameLenBogusSig)).toBeNull();
    });

    it('timingSafeEqual verifies genuine signatures properly', () => {
      const validToken = signJwt({ id: 'usr_checker_01', role: 'CHECKER', branchCode: 'HO-HN-001' });
      const payload = verifyJwt(validToken);
      expect(payload).not.toBeNull();
      expect(payload?.id).toBe('usr_checker_01');
      expect(payload?.role).toBe('CHECKER');
      expect(payload?.branchCode).toBe('HO-HN-001');
    });

    it('strictly checks payload.exp !== undefined && payload.exp < now to prevent exp === 0 falsy bypass', () => {
      // Craft a token where exp === 0
      const now = Math.floor(Date.now() / 1000);
      const tokenWithExpZero = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, -now);
      
      // verifyJwt must return null because 0 < now
      const verified = verifyJwt(tokenWithExpZero);
      expect(verified).toBeNull();

      // Craft a token where exp is past (now - 50)
      const tokenExpired = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, -50);
      expect(verifyJwt(tokenExpired)).toBeNull();

      // Craft a token where exp is future (now + 3600)
      const tokenFuture = signJwt({ id: 'usr_checker_01', role: 'CHECKER' }, 3600);
      expect(verifyJwt(tokenFuture)).not.toBeNull();
    });
  });

  // ============================================================================
  // SUITE 2: POST /api/treasury/vouchers Amount Validation Hardening
  // ============================================================================
  describe('Task 1.2: POST /api/treasury/vouchers Amount Validation', () => {
    const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });

    it('rejects negative amount with 400 Bad Request', async () => {
      const res = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          amount: -50_000_000,
          purpose: 'Negative amount payment attempt',
        }
      );

      expect(res.status).toBe(400);
      expect(res.data.success).toBe(false);
      expect(res.data.error).toBe('Số tiền lệnh chi phải là số dương hợp lệ.');
    });

    it('rejects zero amount with 400 Bad Request', async () => {
      const res = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          amount: 0,
          purpose: 'Zero amount payment attempt',
        }
      );

      expect(res.status).toBe(400);
      expect(res.data.success).toBe(false);
      expect(res.data.error).toBe('Số tiền lệnh chi phải là số dương hợp lệ.');
    });

    it('rejects non-numeric and NaN amount with 400 Bad Request', async () => {
      const res1 = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          amount: 'invalid_number_string',
          purpose: 'Invalid number payment attempt',
        }
      );

      expect(res1.status).toBe(400);
      expect(res1.data.error).toBe('Số tiền lệnh chi phải là số dương hợp lệ.');

      const res2 = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          purpose: 'Missing amount payment attempt',
        }
      );

      expect(res2.status).toBe(400);
      expect(res2.data.error).toBe('Số tiền lệnh chi phải là số dương hợp lệ.');
    });

    it('accepts valid positive finite amount with 201 Created', async () => {
      const res = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          amount: 250_000_000,
          purpose: 'Valid positive amount payment order',
          targetAccount: '0071009988776',
          targetBeneficiary: 'CÔNG TY TNHH CÔNG NGHỆ LIVA',
        }
      );

      expect(res.status).toBe(201);
      expect(res.data.success).toBe(true);
      expect(res.data.voucher.amount).toBe(250_000_000);
      expect(res.data.voucher.amountVnd).toBe(250_000_000);
      expect(res.data.voucher.status).toBe('PENDING_APPROVAL');
    });
  });

  // ============================================================================
  // SUITE 3: Circular 09/2020/TT-NHNN Anti-Self-Approval on Route 7
  // ============================================================================
  describe('Task 1.3: POST /api/treasury/vouchers/:id/approve Anti-Self-Approval Enforcement', () => {
    it('strictly forbids a Checker from approving a voucher created by themselves (Circular 09/2020)', async () => {
      const checkerToken = signJwt({
        id: 'usr_checker_01',
        email: 'checker@livabanking.vn',
        role: 'CHECKER',
        fullName: 'Trần Thị Giám Đốc',
      });

      // 1. Checker creates a voucher
      const selfVoucherId = `vch_self_m4_${Date.now()}`;
      const createRes = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${checkerToken}` },
        {
          id: selfVoucherId,
          voucherId: selfVoucherId,
          amount: 600_000_000,
          purpose: 'Self approval test voucher',
        }
      );
      expect(createRes.status).toBe(201);
      expect(createRes.data.voucher.makerId).toBe('usr_checker_01');

      // 2. Checker attempts to approve their own voucher
      const approveRes = await makeRequest(
        'POST',
        `/api/treasury/vouchers/${selfVoucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      expect(approveRes.status).toBe(403);
      expect(approveRes.data.success).toBe(false);
      expect(approveRes.data.error).toBe(
        'Kiểm soát viên không được tự phê duyệt lệnh chi do chính mình tạo lập (Thông tư 09/2020/TT-NHNN).'
      );

      // Verify voucher remains PENDING_APPROVAL
      const checkVoucher = DB.vouchers.find((v: any) => v.id === selfVoucherId);
      expect(checkVoucher?.status).toBe('PENDING_APPROVAL');
      expect(checkVoucher?.merkleLeafHash).toBeNull();
    });

    it('allows a Checker to approve a voucher created by another user (Maker)', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', role: 'MAKER' });
      const checkerToken = signJwt({ id: 'usr_checker_01', role: 'CHECKER', fullName: 'Trần Thị Giám Đốc' });

      const voucherId = `vch_valid_approval_${Date.now()}`;
      const createRes = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        {
          id: voucherId,
          voucherId: voucherId,
          amount: 450_000_000,
          purpose: 'Legitimate Maker-Checker segregation test',
        }
      );
      expect(createRes.status).toBe(201);

      const approveRes = await makeRequest(
        'POST',
        `/api/treasury/vouchers/${voucherId}/approve`,
        { Authorization: `Bearer ${checkerToken}` }
      );

      expect(approveRes.status).toBe(200);
      expect(approveRes.data.success).toBe(true);
      expect(approveRes.data.voucher.status).toBe('APPROVED');
      expect(approveRes.data.voucher.checkerId).toBe('usr_checker_01');
      expect(approveRes.data.voucher.merkleLeafHash).toMatch(/^[a-f0-9]{64}$/);
    });

    it('returns 400 Bad Request when approving an already approved voucher', async () => {
      const makerToken = signJwt({ id: 'usr_maker_01', role: 'MAKER' });
      const checkerToken = signJwt({ id: 'usr_checker_01', role: 'CHECKER' });

      const voucherId = `vch_double_app_${Date.now()}`;
      await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        { id: voucherId, voucherId, amount: 120_000_000, purpose: 'Double approval test' }
      );

      // First approval succeeds
      const app1 = await makeRequest('POST', `/api/treasury/vouchers/${voucherId}/approve`, {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(app1.status).toBe(200);

      // Second approval must fail with 400
      const app2 = await makeRequest('POST', `/api/treasury/vouchers/${voucherId}/approve`, {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(app2.status).toBe(400);
      expect(app2.data.error).toBe('Lệnh chi không ở trạng thái chờ phê duyệt.');
    });
  });

  // ============================================================================
  // SUITE 4: POST /api/treasury/vouchers/:id/reject Status Validation Hardening
  // ============================================================================
  describe('Task 1.4: POST /api/treasury/vouchers/:id/reject Status Validation', () => {
    const makerToken = signJwt({ id: 'usr_maker_01', role: 'MAKER' });
    const checkerToken = signJwt({ id: 'usr_checker_01', role: 'CHECKER' });

    it('successfully rejects a voucher in PENDING_APPROVAL status', async () => {
      const voucherId = `vch_reject_pending_${Date.now()}`;
      await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        { id: voucherId, voucherId, amount: 300_000_000, purpose: 'To be rejected' }
      );

      const rejectRes = await makeRequest(
        'POST',
        `/api/treasury/vouchers/${voucherId}/reject`,
        { Authorization: `Bearer ${checkerToken}` },
        { remarks: 'Thiếu chứng từ chứng minh nguồn gốc' }
      );

      expect(rejectRes.status).toBe(200);
      expect(rejectRes.data.success).toBe(true);
      expect(rejectRes.data.voucher.status).toBe('REJECTED');
      expect(rejectRes.data.voucher.rejectionReason).toBe('Thiếu chứng từ chứng minh nguồn gốc');
    });

    it('returns 400 Bad Request when attempting to reject an already APPROVED voucher', async () => {
      const voucherId = `vch_reject_approved_${Date.now()}`;
      await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        { id: voucherId, voucherId, amount: 500_000_000, purpose: 'Approved then reject attempt' }
      );

      // 1. Approve first
      const approveRes = await makeRequest('POST', `/api/treasury/vouchers/${voucherId}/approve`, {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(approveRes.status).toBe(200);

      // 2. Reject attempt
      const rejectRes = await makeRequest(
        'POST',
        `/api/treasury/vouchers/${voucherId}/reject`,
        { Authorization: `Bearer ${checkerToken}` },
        { remarks: 'Attempting illegal rejection on approved voucher' }
      );

      expect(rejectRes.status).toBe(400);
      expect(rejectRes.data.success).toBe(false);
      expect(rejectRes.data.error).toBe('Lệnh chi không ở trạng thái chờ phê duyệt.');

      // Verify voucher remains APPROVED
      const voucher = DB.vouchers.find((v: any) => v.id === voucherId);
      expect(voucher?.status).toBe('APPROVED');
    });

    it('returns 400 Bad Request when attempting to reject an already REJECTED voucher', async () => {
      const voucherId = `vch_double_reject_${Date.now()}`;
      await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        { id: voucherId, voucherId, amount: 150_000_000, purpose: 'Double reject attempt' }
      );

      // 1. First reject
      const reject1 = await makeRequest('POST', `/api/treasury/vouchers/${voucherId}/reject`, {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(reject1.status).toBe(200);

      // 2. Second reject
      const reject2 = await makeRequest('POST', `/api/treasury/vouchers/${voucherId}/reject`, {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(reject2.status).toBe(400);
      expect(reject2.data.error).toBe('Lệnh chi không ở trạng thái chờ phê duyệt.');
    });

    it('returns 404 Not Found for non-existent voucher on reject route', async () => {
      const res = await makeRequest('POST', '/api/treasury/vouchers/vch_non_existent_xyz/reject', {
        Authorization: `Bearer ${checkerToken}`,
      });
      expect(res.status).toBe(404);
      expect(res.data.error).toBe('Không tìm thấy lệnh chi.');
    });

    it('returns 403 Forbidden when non-checker attempts to reject', async () => {
      const voucherId = `vch_unauth_reject_${Date.now()}`;
      await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        { id: voucherId, voucherId, amount: 200_000_000, purpose: 'Unauthorized reject attempt' }
      );

      const res = await makeRequest('POST', `/api/treasury/vouchers/${voucherId}/reject`, {
        Authorization: `Bearer ${makerToken}`, // Maker role
      });
      expect(res.status).toBe(403);
      expect(res.data.error).toBe('Chỉ Checker mới có quyền từ chối lệnh chi.');
    });
  });
});
