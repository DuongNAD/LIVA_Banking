import http from 'node:http';
import crypto from 'node:crypto';
import {
  server,
  DB,
  signJwt,
  verifyJwt,
  base64UrlEncode,
} from '../server/server.mjs';

const JWT_SECRET = process.env.JWT_SECRET || 'liva-enterprise-super-secret-key-2026-circular-09';

let serverPort;
let serverUrl;

function makeRequest(method, path, headers = {}, body = null) {
  return new Promise((resolve, reject) => {
    const payload = body !== null ? JSON.stringify(body) : '';
    const reqHeaders = { ...headers };
    if (body !== null) {
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
    if (body !== null) req.write(payload);
    req.end();
  });
}

const results = [];
function recordTest(suite, name, passed, details = '') {
  results.push({ suite, name, passed, details });
  const mark = passed ? 'PASS' : 'FAIL';
  console.log(`  [${mark}] ${name}${details ? ` -> ${details}` : ''}`);
  if (!passed) {
    throw new Error(`Test assertion failed: ${suite} :: ${name} (${details})`);
  }
}

async function runAdversarialHarness() {
  console.log('================================================================');
  console.log('STARTING EMPIRICAL ADVERSARIAL STRESS HARNESS — MILESTONE M4');
  console.log('================================================================\n');

  await new Promise((resolve) => {
    server.listen(0, () => {
      const addr = server.address();
      serverPort = addr.port;
      serverUrl = `http://127.0.0.1:${serverPort}`;
      resolve();
    });
  });

  const makerToken = signJwt({ id: 'usr_maker_01', email: 'maker@livabanking.vn', role: 'MAKER' });
  const checkerToken = signJwt({ id: 'usr_checker_01', email: 'checker@livabanking.vn', role: 'CHECKER' });
  const amlToken = signJwt({ id: 'usr_aml_01', email: 'aml@livabanking.vn', role: 'AML' });

  try {
    console.log('--- SECTION 1: verifyJwt Adversarial Hardening ---');
    const validToken = signJwt({ id: 'usr_fuzz_01', role: 'MAKER' });
    const [h, p, validSig] = validToken.split('.');

    // 1.1 Length mismatch attacks
    const lengthAttacks = [
      { desc: 'Empty signature (0 bytes)', sig: '' },
      { desc: 'Single char signature (1 byte)', sig: 'A' },
      { desc: '10-byte signature', sig: '1234567890' },
      { desc: '42-byte signature (1 byte short)', sig: 'A'.repeat(42) },
      { desc: '44-byte signature (1 byte long)', sig: 'A'.repeat(44) },
      { desc: '100-byte signature', sig: 'A'.repeat(100) },
      { desc: '1000-byte signature (buffer overflow probe)', sig: 'A'.repeat(1000) },
    ];
    for (const test of lengthAttacks) {
      const probe = `${h}.${p}.${test.sig}`;
      let res;
      try {
        res = verifyJwt(probe);
        recordTest('verifyJwt', `Length attack: ${test.desc}`, res === null);
      } catch (err) {
        recordTest('verifyJwt', `Length attack: ${test.desc}`, false, `Threw error: ${err.message}`);
      }
    }

    // 1.2 Constant-time equality probe on equal-length corrupted signatures
    const bitFlipPositions = [0, 5, 15, 25, 42];
    for (const pos of bitFlipPositions) {
      const sigArr = validSig.split('');
      sigArr[pos] = sigArr[pos] === 'X' ? 'Y' : 'X';
      const corruptedSig = sigArr.join('');
      const probe = `${h}.${p}.${corruptedSig}`;
      const res = verifyJwt(probe);
      recordTest('verifyJwt', `Bit-flip at position ${pos}`, res === null);
    }

    // 1.3 exp parameter boundary & bypass probes
    const now = Math.floor(Date.now() / 1000);
    const expProbes = [
      { desc: 'exp === 0 (epoch bypass attempt)', exp: 0, expectValid: false },
      { desc: 'exp === -1 (negative timestamp)', exp: -1, expectValid: false },
      { desc: 'exp === -999999999 (deep past)', exp: -999999999, expectValid: false },
      { desc: 'exp === now - 60 (expired 1 min ago)', exp: now - 60, expectValid: false },
      { desc: 'exp === now + 3600 (valid future)', exp: now + 3600, expectValid: true },
    ];
    for (const probe of expProbes) {
      const customPayloadB64 = base64UrlEncode(JSON.stringify({ id: 'usr_exp_probe', role: 'MAKER', exp: probe.exp }));
      const dataToSign = `${h}.${customPayloadB64}`;
      const sig = crypto.createHmac('sha256', JWT_SECRET).update(dataToSign).digest('base64url');
      const token = `${dataToSign}.${sig}`;
      const res = verifyJwt(token);
      if (probe.expectValid) {
        recordTest('verifyJwt', `exp probe: ${probe.desc}`, res !== null && res.id === 'usr_exp_probe');
      } else {
        recordTest('verifyJwt', `exp probe: ${probe.desc}`, res === null);
      }
    }

    // 1.4 Alg none and malformed structures
    const malformedHeaders = [
      JSON.stringify({ alg: 'none', typ: 'JWT' }),
      JSON.stringify({ alg: 'HS256' }),
      JSON.stringify({ typ: 'JWT' }),
    ];
    for (const hdr of malformedHeaders) {
      const hdrB64 = base64UrlEncode(hdr);
      const probe = `${hdrB64}.${p}.${validSig}`;
      const res = verifyJwt(probe);
      recordTest('verifyJwt', `Header attack: ${hdr}`, res === null);
    }

    console.log('\n--- SECTION 2: Amount Validation Hardening ---');
    const invalidAmountProbes = [
      { desc: 'Negative integer (-50,000,000)', amount: -50000000 },
      { desc: 'Negative float (-0.0001)', amount: -0.0001 },
      { desc: 'Zero integer (0)', amount: 0 },
      { desc: 'Zero float (0.0)', amount: 0.0 },
      { desc: 'Negative zero (-0)', amount: -0 },
      { desc: 'Zero string ("0")', amount: '0' },
      { desc: 'Positive Infinity', amount: Infinity },
      { desc: 'Negative Infinity', amount: -Infinity },
      { desc: 'String "Infinity"', amount: 'Infinity' },
      { desc: 'NaN literal', amount: NaN },
      { desc: 'String "NaN"', amount: 'NaN' },
      { desc: 'Empty string ("")', amount: '' },
      { desc: 'Whitespace string ("   ")', amount: '   ' },
      { desc: 'Alphanumeric string ("100M VND")', amount: '100M VND' },
      { desc: 'SQL Injection string ("100; DROP TABLE")', amount: '100; DROP TABLE' },
      { desc: 'Invalid object ({ val: 100 })', amount: { val: 100 } },
      { desc: 'Missing amount field', amount: undefined },
    ];

    for (const probe of invalidAmountProbes) {
      const payload = probe.amount !== undefined ? { amount: probe.amount, purpose: `Stress test ${probe.desc}` } : { purpose: `Stress test ${probe.desc}` };
      const res = await makeRequest(
        'POST',
        '/api/treasury/vouchers',
        { Authorization: `Bearer ${makerToken}` },
        payload
      );
      const is400 = res.status === 400;
      const isExactError = res.data && res.data.error === 'Số tiền lệnh chi phải là số dương hợp lệ.';
      recordTest('AmountValidation', `Reject invalid amount: ${probe.desc}`, is400 && isExactError, `Status: ${res.status}, Error: ${res.data?.error}`);
    }

    // Also probe amountVnd field fallback
    const resVndInvalid = await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${makerToken}` },
      { amountVnd: -100000, purpose: 'Negative amountVnd probe' }
    );
    recordTest('AmountValidation', 'Reject negative amountVnd', resVndInvalid.status === 400);

    // Probe valid positive finite amount
    const resValid = await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${makerToken}` },
      { amount: 150000000, purpose: 'Valid positive amount' }
    );
    recordTest('AmountValidation', 'Accept valid positive amount (150,000,000 VND)', resValid.status === 201 && resValid.data.success === true);

    // Adversarial finding probe: JS Number coercion on boolean
    const resBoolTrue = await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${makerToken}` },
      { amount: true, purpose: 'Boolean true coercion probe' }
    );
    console.log(`  [INFO] JS Boolean true amount coercion result: status ${resBoolTrue.status}, recorded amount: ${resBoolTrue.data?.voucher?.amount} VND (Documented in Caveats & Findings)`);

    console.log('\n--- SECTION 3: Maker-Checker Dual-Control & Replay Attacks ---');

    const selfVchId = `vch_adv_self_${Date.now()}`;
    const createSelfRes = await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${checkerToken}` },
      { id: selfVchId, amount: 800000000, purpose: 'Checker self-created voucher' }
    );
    recordTest('MakerChecker', 'Checker creates voucher successfully', createSelfRes.status === 201);
    recordTest('MakerChecker', 'Voucher makerId recorded as Checker ID', createSelfRes.data.voucher.makerId === 'usr_checker_01');

    const approveSelfRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${selfVchId}/approve`,
      { Authorization: `Bearer ${checkerToken}` }
    );
    const selfAppForbidden = approveSelfRes.status === 403;
    const selfAppMsg = approveSelfRes.data && approveSelfRes.data.error.includes('Thông tư 09/2020/TT-NHNN');
    recordTest('MakerChecker', 'Circular 09/2020 self-approval forbidden (403)', selfAppForbidden && selfAppMsg, `Status: ${approveSelfRes.status}`);

    const selfVchDB = DB.vouchers.find((v) => v.id === selfVchId);
    recordTest('MakerChecker', 'Self-approved voucher remains in PENDING_APPROVAL status', selfVchDB.status === 'PENDING_APPROVAL' && selfVchDB.merkleLeafHash === null);

    const spoofVchId = `vch_adv_spoof_${Date.now()}`;
    const spoofCreateRes = await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${checkerToken}` },
      { id: spoofVchId, makerId: 'usr_maker_01', amount: 500000000, purpose: 'Spoof makerId attempt' }
    );
    recordTest('MakerChecker', 'Server ignores client-supplied makerId spoofing', spoofCreateRes.data.voucher.makerId === 'usr_checker_01');

    const approveSpoofedRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${spoofVchId}/approve`,
      { Authorization: `Bearer ${checkerToken}` }
    );
    recordTest('MakerChecker', 'Self-approval still blocked despite makerId spoof attempt', approveSpoofedRes.status === 403);

    const crossVchId = `vch_adv_cross_${Date.now()}`;
    await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${makerToken}` },
      { id: crossVchId, amount: 350000000, purpose: 'Valid cross-user voucher' }
    );
    const approveCrossRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${crossVchId}/approve`,
      { Authorization: `Bearer ${checkerToken}` }
    );
    recordTest('MakerChecker', 'Cross-user approval succeeds with 200', approveCrossRes.status === 200 && approveCrossRes.data.voucher.status === 'APPROVED');

    const replayAppRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${crossVchId}/approve`,
      { Authorization: `Bearer ${checkerToken}` }
    );
    recordTest('MakerChecker', 'Double approval replay rejected with 400', replayAppRes.status === 400 && replayAppRes.data.error === 'Lệnh chi không ở trạng thái chờ phê duyệt.');

    const burstVchId = `vch_adv_burst_${Date.now()}`;
    await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${makerToken}` },
      { id: burstVchId, amount: 990000000, purpose: 'Burst concurrency test' }
    );
    const burstPromises = Array.from({ length: 8 }, () =>
      makeRequest('POST', `/api/treasury/vouchers/${burstVchId}/approve`, {
        Authorization: `Bearer ${checkerToken}`,
      })
    );
    const burstResponses = await Promise.all(burstPromises);
    const successCount = burstResponses.filter((r) => r.status === 200).length;
    const errorCount = burstResponses.filter((r) => r.status === 400).length;
    recordTest('MakerChecker', 'Burst approval: exactly 1 request succeeds (200)', successCount === 1, `200 count: ${successCount}`);
    recordTest('MakerChecker', 'Burst approval: exactly 7 requests rejected (400)', errorCount === 7, `400 count: ${errorCount}`);

    console.log('\n--- SECTION 4: Rejection State Machine Attacks ---');

    const rejectApprovedRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${crossVchId}/reject`,
      { Authorization: `Bearer ${checkerToken}` },
      { remarks: 'Illegal rejection on approved voucher' }
    );
    recordTest('RejectGuard', 'Rejecting APPROVED voucher returns 400', rejectApprovedRes.status === 400 && rejectApprovedRes.data.error === 'Lệnh chi không ở trạng thái chờ phê duyệt.');

    const pendingToRejectId = `vch_adv_reject_${Date.now()}`;
    await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${makerToken}` },
      { id: pendingToRejectId, amount: 200000000, purpose: 'Legitimate reject target' }
    );
    const validRejectRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${pendingToRejectId}/reject`,
      { Authorization: `Bearer ${checkerToken}` },
      { remarks: 'Sai lệch số tài khoản thụ hưởng' }
    );
    recordTest('RejectGuard', 'Legitimate rejection of pending voucher succeeds (200)', validRejectRes.status === 200 && validRejectRes.data.voucher.status === 'REJECTED');

    const doubleRejectRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${pendingToRejectId}/reject`,
      { Authorization: `Bearer ${checkerToken}` },
      { remarks: 'Second rejection attempt' }
    );
    recordTest('RejectGuard', 'Double rejection replay rejected with 400', doubleRejectRes.status === 400 && doubleRejectRes.data.error === 'Lệnh chi không ở trạng thái chờ phê duyệt.');

    const approveRejectedRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${pendingToRejectId}/approve`,
      { Authorization: `Bearer ${checkerToken}` }
    );
    recordTest('RejectGuard', 'Approving REJECTED voucher returns 400', approveRejectedRes.status === 400 && approveRejectedRes.data.error === 'Lệnh chi không ở trạng thái chờ phê duyệt.');

    const unauthVchId = `vch_adv_unauth_${Date.now()}`;
    await makeRequest(
      'POST',
      '/api/treasury/vouchers',
      { Authorization: `Bearer ${makerToken}` },
      { id: unauthVchId, amount: 120000000, purpose: 'Role unauth check' }
    );
    const makerRejectRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${unauthVchId}/reject`,
      { Authorization: `Bearer ${makerToken}` }
    );
    recordTest('RejectGuard', 'Maker cannot reject voucher (403)', makerRejectRes.status === 403 && makerRejectRes.data.error === 'Chỉ Checker mới có quyền từ chối lệnh chi.');

    const amlRejectRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${unauthVchId}/reject`,
      { Authorization: `Bearer ${amlToken}` }
    );
    recordTest('RejectGuard', 'AML Specialist cannot reject voucher (403)', amlRejectRes.status === 403 && amlRejectRes.data.error === 'Chỉ Checker mới có quyền từ chối lệnh chi.');

    const noAuthRejectRes = await makeRequest(
      'POST',
      `/api/treasury/vouchers/${unauthVchId}/reject`
    );
    recordTest('RejectGuard', 'Unauthenticated reject returns 401', noAuthRejectRes.status === 401 && noAuthRejectRes.data.error === 'Chưa đăng nhập.');

    console.log('\n================================================================');
    console.log(`TOTAL ADVERSARIAL STRESS PROBES: ${results.length}`);
    console.log(`PASSED: ${results.filter((r) => r.passed).length}`);
    console.log(`FAILED: ${results.filter((r) => !r.passed).length}`);
    console.log('ALL ADVERSARIAL STRESS PROBES COMPLETED SUCCESSFULLY — 100% PASS');
    console.log('================================================================\n');
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}

runAdversarialHarness().catch((err) => {
  console.error('\nADVERSARIAL HARNESS FAILURE:', err);
  process.exit(1);
});
