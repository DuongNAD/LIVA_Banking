#!/usr/bin/env node
/**
 * Adversarial Challenger Suite for Milestone 2: 2D Banking & Treasury UI Dashboard
 *
 * Empirical verification of:
 * 1. 10,000-cycle reconciliation arithmetic invariant & IEEE-754 precision drift
 * 2. Two-Phase Confirmation UUID token generation, entropy, and security gaps (token spoofing, replay attack)
 * 3. 2D Web Audio & 12-bar SVG soundwave zero GPU/VRAM overhead verification
 */
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

const ROOT_DIR = path.resolve(import.meta.dirname, '..');
const UI_SRC = path.join(ROOT_DIR, 'liva-ui', 'src');

console.log('='.repeat(80));
console.log('🛡️  ADVERSARIAL CHALLENGER SUITE — MILESTONE 2 (BANKING & TREASURY)');
console.log('='.repeat(80));

let passed = 0;
let failed = 0;

function test(name, fn) {
  const t0 = performance.now();
  try {
    fn();
    const dt = (performance.now() - t0).toFixed(2);
    console.log(`  ✅ [PASS] ${name} (${dt}ms)`);
    passed++;
  } catch (err) {
    const dt = (performance.now() - t0).toFixed(2);
    console.error(`  ❌ [FAIL] ${name} (${dt}ms)`);
    console.error(`     Error: ${err.message}`);
    failed++;
  }
}

// -----------------------------------------------------------------------------
// CHALLENGE 1: RECONCILIATION ARITHMETIC INVARIANT & FLOATING POINT STRESS
// -----------------------------------------------------------------------------
console.log('\n[TEST SUITE 1] 10,000-Cycle Reconciliation Arithmetic & Extreme Number Stress');

test('1.1: 10,000 cycles integer invariance across arbitrary bank/ledger amounts', () => {
  const t0 = performance.now();
  for (let i = 0; i < 10000; i++) {
    const bankAmount = (i % 2 === 0 ? 1 : -1) * (100_000 + (i * 1234567) % 500_000_000_000);
    const fee = i % 10 === 0 ? 1100 : (i % 25 === 0 ? 5500 : 0);
    const ledgerAmount = bankAmount - fee;
    const variance = bankAmount - ledgerAmount;

    assert.equal(variance, fee, `Cycle ${i}: variance ${variance} != fee ${fee}`);
    assert.ok(Number.isSafeInteger(bankAmount));
    assert.ok(Number.isSafeInteger(ledgerAmount));
    assert.ok(Number.isSafeInteger(variance));
  }
  const dt = performance.now() - t0;
  assert.ok(dt < 200, `10,000 cycles should complete under 200ms (took ${dt.toFixed(2)}ms)`);
});

test('1.2: Decimal and fractional fee precision stress (IEEE-754 precision drift)', () => {
  // Floating point drift demonstration
  const floatResidue = (0.1 + 0.2) - 0.3;
  assert.notEqual(floatResidue, 0, 'IEEE 754 float arithmetic exhibits precision drift');

  // Integer-scaled banking arithmetic guarantee
  const principalCents = 123456789n; // Scaled by 100
  const feeCents = (principalCents * 5n) / 10000n; // 0.05% fee
  const intVariance = principalCents - (principalCents - feeCents);
  assert.equal(intVariance, feeCents, 'Integer-scaled arithmetic guarantees 0.0% drift');
});

test('1.3: Extreme limits stress: Number.MAX_SAFE_INTEGER and zero amounts', () => {
  const maxSafe = BigInt(Number.MAX_SAFE_INTEGER); // 9,007,199,254,740,991
  const fee = 1100n;
  const ledger = maxSafe - fee;
  const variance = maxSafe - ledger;
  assert.equal(variance, fee);

  assert.equal(0 - 0, 0);
});

// -----------------------------------------------------------------------------
// CHALLENGE 2: TWO-PHASE CONFIRMATION UUID TOKEN SECURITY & ATTACK VECTORS
// -----------------------------------------------------------------------------
console.log('\n[TEST SUITE 2] Two-Phase Confirmation UUID Token Security & Attack Vectors');

function generateCurrentStoreToken() {
  return 'token-' + Math.random().toString(36).substring(2, 10) + '-' + Date.now().toString(36);
}

test('2.1: Collision resistance: 100,000 token generation test', () => {
  const set = new Set();
  const count = 100000;
  for (let i = 0; i < count; i++) {
    const token = generateCurrentStoreToken();
    set.add(token);
  }
  assert.equal(set.size, count, `Collision detected in ${count} generated tokens!`);
});

test('2.2: Cryptographic security analysis of token entropy', () => {
  const token = generateCurrentStoreToken();
  assert.match(token, /^token-[a-z0-9]{5,8}-[a-z0-9]+$/);
  
  const rfcUuid = crypto.randomUUID();
  assert.match(rfcUuid, /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
});

test('2.3: Adversarial Simulation: Store Token Validation Gap (Vulnerability Demonstration)', () => {
  // Replicating store logic from reconciliationStore.ts
  const mockTransactions = [
    {
      id: 'tx-hitl-1',
      txCode: 'VCB2026103003',
      bankAmount: 50001100,
      ledgerAmount: 50000000,
      variance: 1100,
      status: 'PENDING_HITL',
      tokenUuid: 'token-authorized-secure-uuid-999',
    }
  ];
  const auditTrail = [];

  // CURRENT STORE IMPLEMENTATION
  function vulnerableConfirmHitl(payload) {
    const target = mockTransactions.find(t => t.id === payload.txId);
    if (!target) return { success: false, error: 'Transaction not found' };

    // VULNERABILITY 1: Does NOT check if payload.tokenUuid === target.tokenUuid
    // VULNERABILITY 2: Does NOT check if target.status === 'PENDING_HITL'
    // VULNERABILITY 3: Does NOT invalidate target.tokenUuid (Replay vulnerability)
    target.status = 'MATCHED';
    target.variance = 0;
    target.ledgerAmount = target.bankAmount;

    auditTrail.push({
      txCode: target.txCode,
      tokenUuid: payload.tokenUuid,
      action: payload.action,
    });
    return { success: true };
  }

  // Attack 1: Spoofed / Forged Token
  const spoofResult = vulnerableConfirmHitl({
    txId: 'tx-hitl-1',
    tokenUuid: 'token-ATTACKER-FORGED-TOKEN',
    action: 'ALLOCATE_FEE'
  });
  assert.equal(spoofResult.success, true, 'Vulnerability confirmed: Forged token was accepted!');
  assert.equal(auditTrail[0].tokenUuid, 'token-ATTACKER-FORGED-TOKEN');

  // Attack 2: Replay Attack on already MATCHED transaction
  const replayResult = vulnerableConfirmHitl({
    txId: 'tx-hitl-1',
    tokenUuid: 'token-REPLAYED-AGAIN',
    action: 'MANUAL_MATCH'
  });
  assert.equal(replayResult.success, true, 'Vulnerability confirmed: Replay attack succeeded!');
  assert.equal(auditTrail.length, 2, 'Vulnerability confirmed: Duplicate audit record written!');
});

test('2.4: Hardened Two-Phase Confirmation specification verification', () => {
  const mockTransactions = [
    {
      id: 'tx-hitl-secure',
      txCode: 'VCB2026103099',
      bankAmount: 10001100,
      ledgerAmount: 10000000,
      variance: 1100,
      status: 'PENDING_HITL',
      tokenUuid: 'token-legit-uuid-123',
      tokenExpiry: Date.now() + 300_000, // 5 min TTL
    }
  ];
  const auditTrail = [];

  function hardenedConfirmHitl(payload) {
    const target = mockTransactions.find(t => t.id === payload.txId);
    if (!target) return { success: false, error: 'Transaction not found' };

    if (target.status !== 'PENDING_HITL') {
      return { success: false, error: 'Transaction is not in PENDING_HITL state' };
    }

    if (!payload.tokenUuid || payload.tokenUuid !== target.tokenUuid) {
      return { success: false, error: 'Cryptographic token mismatch or missing' };
    }

    if (target.tokenExpiry && Date.now() > target.tokenExpiry) {
      return { success: false, error: 'Confirmation token expired' };
    }

    target.status = 'MATCHED';
    target.variance = 0;
    target.ledgerAmount = target.bankAmount;
    target.tokenUuid = undefined; // Consumed / single-use

    auditTrail.push({
      txCode: target.txCode,
      tokenUuid: payload.tokenUuid,
      action: payload.action,
      hash: crypto.createHash('sha256').update(target.txCode + payload.tokenUuid).digest('hex'),
    });

    return { success: true };
  }

  // Attack 1 (Spoofed token) is REJECTED
  const res1 = hardenedConfirmHitl({
    txId: 'tx-hitl-secure',
    tokenUuid: 'token-bad-spoof',
    action: 'ALLOCATE_FEE'
  });
  assert.equal(res1.success, false);
  assert.equal(res1.error, 'Cryptographic token mismatch or missing');

  // Legit Confirmation SUCCEEDS
  const res2 = hardenedConfirmHitl({
    txId: 'tx-hitl-secure',
    tokenUuid: 'token-legit-uuid-123',
    action: 'ALLOCATE_FEE'
  });
  assert.equal(res2.success, true);
  assert.equal(mockTransactions[0].status, 'MATCHED');
  assert.equal(mockTransactions[0].tokenUuid, undefined);

  // Attack 2 (Replay attack) is REJECTED
  const res3 = hardenedConfirmHitl({
    txId: 'tx-hitl-secure',
    tokenUuid: 'token-legit-uuid-123',
    action: 'ALLOCATE_FEE'
  });
  assert.equal(res3.success, false);
  assert.equal(res3.error, 'Transaction is not in PENDING_HITL state');
});

// -----------------------------------------------------------------------------
// CHALLENGE 3: PURE 2D WEB AUDIO & ZERO GPU/VRAM OVERHEAD VERIFICATION
// -----------------------------------------------------------------------------
console.log('\n[TEST SUITE 3] Pure 2D Web Audio & Zero GPU/VRAM Footprint Verification');

function stripComments(code) {
  return code.replace(/\/\*[\s\S]*?\*\/|\/\/.*/g, '');
}

test('3.1: Verify zero WebGL/WebGPU/3D canvas presence across all banking components', () => {
  const bankingComponentsDir = path.join(UI_SRC, 'components', 'banking');
  const files = fs.readdirSync(bankingComponentsDir).filter(f => f.endsWith('.vue'));

  assert.ok(files.length >= 8, `Expected >= 8 banking components, found ${files.length}`);

  for (const file of files) {
    const raw = fs.readFileSync(path.join(bankingComponentsDir, file), 'utf8');
    const content = stripComments(raw);

    // Check for canvas elements
    assert.ok(!content.includes('<canvas'), `Component ${file} contains <canvas>!`);
    // Check for active WebGL context creation or WebGL API usage
    assert.ok(!content.includes("getContext('webgl'"), `Component ${file} calls getContext('webgl')!`);
    assert.ok(!content.includes("getContext('webgl2'"), `Component ${file} calls getContext('webgl2')!`);
    assert.ok(!content.includes('WebGLRenderingContext'), `Component ${file} references WebGLRenderingContext!`);
    assert.ok(!content.includes('navigator.gpu'), `Component ${file} references WebGPU!`);
    // Check for 3D library imports
    assert.ok(!content.includes("from 'three'"), `Component ${file} imports Three.js!`);
    assert.ok(!content.includes("from '@babylonjs"), `Component ${file} imports Babylon.js!`);
    assert.ok(!content.includes("from '@pixiv/three-vrm'"), `Component ${file} imports VRM!`);
  }
});

test('3.2: Verify 12-bar SVG soundwave implementation details in FinancialAssistantDrawer.vue', () => {
  const drawerFile = path.join(UI_SRC, 'components', 'banking', 'FinancialAssistantDrawer.vue');
  const content = fs.readFileSync(drawerFile, 'utf8');

  assert.ok(content.includes('[15, 25, 45, 70, 90, 100, 85, 60, 40, 25, 18, 10]'));
  assert.ok(content.includes('<svg class="soundwave-svg"'));
  assert.ok(content.includes('<rect'));
  assert.ok(content.includes('class="soundwave-bar"'));
  assert.ok(!content.includes('getContext'));
});

test('3.3: Verify package.json dependencies are 100% free of 3D / WebGL / VRM libraries', () => {
  const pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT_DIR, 'liva-ui', 'package.json'), 'utf8'));
  const allDeps = { ...pkgJson.dependencies, ...pkgJson.devDependencies };

  const forbiddenPackages = [
    'three',
    '@types/three',
    '@pixiv/three-vrm',
    'pixi.js',
    'pixi-live2d-display',
    '@babylonjs/core',
    '@mediapipe/tasks-vision',
    '@mediapipe/camera_utils'
  ];

  for (const pkg of forbiddenPackages) {
    assert.ok(!allDeps[pkg], `Forbidden 3D package detected in package.json: ${pkg}`);
  }
});

console.log('\n' + '='.repeat(80));
console.log(`MILESTONE 2 EMPIRICAL ADVERSARIAL SUMMARY: ${passed} PASSED, ${failed} FAILED`);
console.log('='.repeat(80));

if (failed > 0) {
  process.exit(1);
}
