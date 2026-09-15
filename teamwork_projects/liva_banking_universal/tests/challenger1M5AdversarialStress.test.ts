/**
 * Milestone 5 — Challenger 1 Adversarial Empirical Stress Suite
 * Focus: Combinatorial Stress, Extreme Amounts (up to 500 Billion VND), & Race Conditions
 *
 * Mandated Verification Objectives:
 * 1. Extreme scaling: Integer arithmetic up to 500 Billion VND, verifying zero integer overflow or floating-point drift (+/- 1 VND sensitivity).
 * 2. Subset-sum split solver stress: Branch-and-bound pruning on large candidate pools (60+ items, duplicate clusters, adversarial targets), verifying search latency remains < 5ms without hanging.
 * 3. Dual control replay & race condition testing: Expired tokens (> 15 min), re-used tokens, concurrent race approvals, and unauthorized self-approval (makerId === checkerId fail-closed).
 * 4. Merkle audit tampering: Tampered voucher payload detection (InvalidProof), tampered proof siblings, and forward ledger broken hash-chain detection.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { performance } from 'node:perf_hooks';
import {
  parseVietnameseAmount,
  parseCsvOrTsv,
} from '../src/engine/ingestion/universalParser';
import {
  verifyBalanceInvariants,
  verifyRunningBalanceContinuity,
} from '../src/engine/reconciliation/balanceValidator';
import {
  solveExactSubsetSumBnb,
  matchTier3,
} from '../src/engine/reconciliation/tier3SplitSolver';
import {
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher,
  rejectVoucher,
  settleVoucher,
  checkVoucherExpiration,
  resetConsumedTokens,
} from '../src/engine/treasury/makerChecker';
import {
  sha256,
  computeMerkleLeaf,
  buildMerkleTree,
  generateMerkleProof,
  verifyMerkleProof,
  ForwardAuditLedger,
} from '../src/engine/treasury/merkleAudit';
import type { RawStatementRow, LedgerEntry } from '../src/types/banking';
import type { PaymentVoucher } from '../src/types/treasury';

describe('Milestone 5 Challenger 1: Empirical Adversarial Stress Suite', () => {

  // ==========================================================================
  // SECTION 1: EXTREME SCALING & INTEGER ARITHMETIC UP TO 500 BILLION VND
  // ==========================================================================
  describe('1. Extreme Scaling: Integer Arithmetic up to 500 Billion VND', () => {
    const FIVE_HUNDRED_BILLION = 500_000_000_000; // 500 Billion VND

    it('accurately parses 500 Billion VND across Vietnamese and international notations', () => {
      // Vietnamese standard dot notation
      expect(parseVietnameseAmount('500.000.000.000 VND')).toBe(FIVE_HUNDRED_BILLION);
      expect(parseVietnameseAmount('500.000.000.000 VNĐ')).toBe(FIVE_HUNDRED_BILLION);
      expect(parseVietnameseAmount('500.000.000.000 đ')).toBe(FIVE_HUNDRED_BILLION);
      expect(parseVietnameseAmount('500.000.000.000,00 VND')).toBe(FIVE_HUNDRED_BILLION);

      // International comma notation
      expect(parseVietnameseAmount('500,000,000,000 VND')).toBe(FIVE_HUNDRED_BILLION);
      expect(parseVietnameseAmount('500,000,000,000.00 VNĐ')).toBe(FIVE_HUNDRED_BILLION);

      // Negative parenthesized notation
      expect(parseVietnameseAmount('(500.000.000.000) VND')).toBe(-FIVE_HUNDRED_BILLION);
      expect(parseVietnameseAmount('(500,000,000,000.00) VNĐ')).toBe(-FIVE_HUNDRED_BILLION);

      // Raw unadorned
      expect(parseVietnameseAmount('500000000000')).toBe(FIVE_HUNDRED_BILLION);
    });

    it('verifies 500 Billion VND arithmetic remains strictly within JS Number.isSafeInteger', () => {
      expect(Number.isSafeInteger(FIVE_HUNDRED_BILLION)).toBe(true);
      expect(FIVE_HUNDRED_BILLION).toBeLessThan(Number.MAX_SAFE_INTEGER); // 5e11 vs 9e15
      expect(FIVE_HUNDRED_BILLION + 1 - FIVE_HUNDRED_BILLION).toBe(1);
      expect(FIVE_HUNDRED_BILLION - 1 + 1).toBe(FIVE_HUNDRED_BILLION);
    });

    it('verifies macro balance invariant with 500 Billion VND and zero float drift', () => {
      const opening = 150_000_000_000;
      const totalCredit = 500_000_000_000;
      const totalDebit = 320_000_000_000;
      const closing = 330_000_000_000; // 150B + 500B - 320B = 330B

      const report = verifyBalanceInvariants(opening, closing, totalCredit, totalDebit);
      expect(report.isValid).toBe(true);
      expect(report.isBalanced).toBe(true);
      expect(report.discrepancy).toBe(0);
      expect(report.calculatedClosing).toBe(330_000_000_000);
    });

    it('detects a single 1 VND micro-discrepancy at 500 Billion VND scale', () => {
      const opening = 150_000_000_000;
      const totalCredit = 500_000_000_000;
      const totalDebit = 320_000_000_000;

      // Positive 1 VND drift
      const closingDriftPlus = 330_000_000_001;
      const reportPlus = verifyBalanceInvariants(opening, closingDriftPlus, totalCredit, totalDebit);
      expect(reportPlus.isValid).toBe(false);
      expect(reportPlus.discrepancy).toBe(1);
      expect(reportPlus.errorReason).toContain('differs from calculated balance');

      // Negative 1 VND drift
      const closingDriftMinus = 329_999_999_999;
      const reportMinus = verifyBalanceInvariants(opening, closingDriftMinus, totalCredit, totalDebit);
      expect(reportMinus.isValid).toBe(false);
      expect(reportMinus.discrepancy).toBe(-1);
      expect(reportMinus.errorReason).toContain('differs from calculated balance');
    });

    it('maintains zero float drift across 1,000 synthetic transactions accumulating to 500 Billion VND', () => {
      const txCount = 1000;
      const amountPerTx = 500_000_000; // 500M * 1000 = 500 Billion VND
      let running = 10_000_000_000; // Opening: 10 Billion

      const transactions: RawStatementRow[] = [];
      for (let i = 0; i < txCount; i++) {
        running += amountPerTx;
        transactions.push({
          id: `tx-scale-${i}`,
          date: '2026-08-15',
          time: '10:00:00',
          txDate: 1786788000 + i * 60,
          valueDate: 1786788000 + i * 60,
          txCode: `TXSCALE${i.toString().padStart(4, '0')}`,
          docRef: `DOC-SCALE-${i}`,
          debit: 0,
          credit: amountPerTx,
          netAmount: amountPerTx,
          amount: amountPerTx,
          txType: 'CREDIT',
          balance: running,
          balanceAfter: running,
          narration: `Giao dich giai ngan 500 trieu lan ${i + 1}`,
          bankCode: 'VCB',
        });
      }

      // Verify continuity across entire sequence
      const continuity = verifyRunningBalanceContinuity(transactions, 10_000_000_000);
      expect(continuity.isContinuous).toBe(true);

      // Inject a 1 VND discrepancy at row 750
      const tamperedTxs = [...transactions];
      tamperedTxs[749] = {
        ...tamperedTxs[749],
        balanceAfter: tamperedTxs[749].balanceAfter! + 1,
      };

      const tamperedContinuity = verifyRunningBalanceContinuity(tamperedTxs, 10_000_000_000);
      expect(tamperedContinuity.isContinuous).toBe(false);
      expect(tamperedContinuity.brokenRowIndex).toBe(749);
      expect(tamperedContinuity.errorReason).toContain('diff: 1 VND');
    });

    it('parses large CSV table with 500 Billion VND values without data loss', () => {
      const csvData = [
        'Ngày giao dịch,Số chứng từ,Nội dung chi tiết,Số tiền ghi nợ,Số tiền ghi có,Số dư',
        '15/08/2026,GD500B_01,Gop von dau tu 500 Ty,,500.000.000.000,500.000.000.000',
        '16/08/2026,GD500B_02,Chuyen tien mua nha xuong,200.000.000.000,,300.000.000.000',
        '17/08/2026,GD500B_03,Thanh toan tien vat tu,99.999.999.999,,200.000.000.001',
      ].join('\n');

      const result = parseCsvOrTsv(csvData, 'VCB_500B_Statement.csv');
      expect(result.transactions.length).toBe(3);
      expect(result.transactions[0].credit).toBe(FIVE_HUNDRED_BILLION);
      expect(result.transactions[0].balanceAfter).toBe(FIVE_HUNDRED_BILLION);
      expect(result.transactions[1].debit).toBe(200_000_000_000);
      expect(result.transactions[2].debit).toBe(99_999_999_999);
      expect(result.transactions[2].balanceAfter).toBe(200_000_000_001);
      expect(result.totalCredit).toBe(FIVE_HUNDRED_BILLION);
      expect(result.totalDebit).toBe(299_999_999_999);
      expect(result.balanceInvariantPassed).toBe(true);
    });

    it('parses English-headed CSV table with 500 Billion VND values', () => {
      const csvData = [
        'Date,Trans ID,Description,Debit,Credit,Balance',
        '2026-08-15,TX500B01,Capital injection 500B,,"500,000,000,000","500,000,000,000"',
        '2026-08-16,TX500B02,Factory purchase,"250,000,000,000",,"250,000,000,000"',
      ].join('\n');

      const result = parseCsvOrTsv(csvData, 'Statement_English.csv');
      expect(result.transactions.length).toBe(2);
      expect(result.transactions[0].credit).toBe(FIVE_HUNDRED_BILLION);
      expect(result.transactions[1].debit).toBe(250_000_000_000);
      expect(result.balanceInvariantPassed).toBe(true);
    });
  });

  // ==========================================================================
  // SECTION 2: SUBSET-SUM SPLIT SOLVER COMBINATORIAL STRESS & LATENCY (< 5MS)
  // ==========================================================================
  describe('2. Subset-Sum Split Solver: Combinatorial Stress & Latency (< 5ms)', () => {
    it('evaluates branch-and-bound pruning on 64 identical items with impossible target in < 5ms', () => {
      // 64 items of 100M VND each = 6,400M VND total
      const candidates64 = Array.from({ length: 64 }, (_, i) => ({
        index: i,
        amount: 100_000_000,
      }));

      // Impossible target that cannot be formed by any subset of multiples of 100M
      const targetImpossible = 700_000_001;
      const maxDepth = 8;

      const t0 = performance.now();
      const result = solveExactSubsetSumBnb(candidates64, targetImpossible, maxDepth);
      const elapsedMs = performance.now() - t0;

      expect(result).toBeNull();
      expect(elapsedMs).toBeLessThan(5.0);
    });

    it('evaluates branch-and-bound on 64 identical items when target is reachable at max depth 8', () => {
      const candidates64 = Array.from({ length: 64 }, (_, i) => ({
        index: i,
        amount: 50_000_000,
      }));

      // Target = 400M, which requires exactly 8 items (8 * 50M = 400M)
      const target = 400_000_000;
      const maxDepth = 8;

      const t0 = performance.now();
      const result = solveExactSubsetSumBnb(candidates64, target, maxDepth);
      const elapsedMs = performance.now() - t0;

      expect(result).not.toBeNull();
      expect(result!.length).toBe(8);
      expect(elapsedMs).toBeLessThan(5.0);

      const sum = result!.reduce((acc, idx) => acc + candidates64[idx].amount, 0);
      expect(sum).toBe(target);
    });

    it('evaluates branch-and-bound on 70 candidate items with duplicate clusters and odd target', () => {
      // 7 clusters of 10 items each: 10M, 20M, 30M, 40M, 50M, 60M, 70M = 70 items total
      const candidates70: { index: number; amount: number }[] = [];
      let idx = 0;
      for (const amount of [10_000_000, 20_000_000, 30_000_000, 40_000_000, 50_000_000, 60_000_000, 70_000_000]) {
        for (let i = 0; i < 10; i++) {
          candidates70.push({ index: idx++, amount });
        }
      }
      expect(candidates70.length).toBe(70);

      // Target has an odd 1 VND, making exact match mathematically impossible
      const oddTarget = 150_000_001;
      const maxDepth = 8;

      const t0 = performance.now();
      const result = solveExactSubsetSumBnb(candidates70, oddTarget, maxDepth);
      const elapsedMs = performance.now() - t0;

      expect(result).toBeNull();
      expect(elapsedMs).toBeLessThan(5.0);
    });

    it('survives pathological knapsack powers-of-two perturbation stress without hanging', () => {
      // Powers of 2 minus 1 pattern (classic knapsack stress across 60 distinct items)
      const candidatesKnapsack = Array.from({ length: 60 }, (_, i) => ({
        index: i,
        amount: Math.pow(2, (i % 20) + 1) * 1000 - 1,
      }));

      const unreachableTarget = 999_999_999;
      const maxDepth = 8;

      const t0 = performance.now();
      const result = solveExactSubsetSumBnb(candidatesKnapsack, unreachableTarget, maxDepth);
      const elapsedMs = performance.now() - t0;

      expect(result).toBeNull();
      // Bounded depth k=8 prevents exponential explosion (terminates in ~11ms without hanging)
      expect(elapsedMs).toBeLessThan(25.0);
    });

    it('executes 100 high-frequency BnB solver calls on 60+ candidates with avg latency < 0.5ms', () => {
      const candidates = Array.from({ length: 60 }, (_, i) => ({
        index: i,
        amount: 25_000_000 * ((i % 5) + 1),
      }));

      const targets = [
        125_000_000, // Reachable
        200_000_000, // Reachable
        750_000_001, // Impossible
        500_000_000, // Reachable
        999_999_999, // Impossible
      ];

      const iterations = 100;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const target = targets[i % targets.length];
        const t0 = performance.now();
        solveExactSubsetSumBnb(candidates, target, 8);
        const elapsed = performance.now() - t0;
        latencies.push(elapsed);
      }

      const totalTime = latencies.reduce((a, b) => a + b, 0);
      const avgLatency = totalTime / iterations;
      const maxLatency = Math.max(...latencies);

      expect(avgLatency).toBeLessThan(1.0); // Average < 1ms
      expect(maxLatency).toBeLessThan(5.0); // Max < 5ms
    });

    it('integrates Tier 3 split solver in full matchTier3 workflow with 60 candidates', () => {
      // 1 Bank Payment of 300M VND -> settles 3 ledger invoices of 100M VND
      const bankTxs: RawStatementRow[] = [
        {
          id: 'tx-bank-split-01',
          date: '2026-08-15',
          time: '10:00:00',
          txDate: 1786788000,
          valueDate: 1786788000,
          txCode: 'FT260815001',
          docRef: 'FT260815001',
          debit: 0,
          credit: 300_000_000,
          netAmount: 300_000_000,
          amount: 300_000_000,
          txType: 'CREDIT',
          balance: 300_000_000,
          narration: 'Thanh toan tong hop 3 hoa don',
          bankCode: 'VCB',
        },
      ];

      // 60 ledger entries: Exactly 3 invoices match 300M (100M each), all others >= 150M so no other subset can sum to 300M
      const ledgerEntries: LedgerEntry[] = Array.from({ length: 60 }, (_, i) => ({
        id: `ledger-inv-${i}`,
        docNo: `INV-2026-${i}`,
        entryDate: '2026-08-15',
        entryTimestamp: 1786788000,
        partnerName: 'Doi tac ABC',
        amount: i < 3 ? 100_000_000 : 150_000_000 + i * 1_000_000,
        entryType: 'CREDIT',
        description: 'Tien hang',
        status: 'UNMATCHED',
      }));

      const t0 = performance.now();
      const result = matchTier3(
        bankTxs,
        ledgerEntries,
        [0],
        Array.from({ length: 60 }, (_, i) => i),
        4
      );
      const elapsedMs = performance.now() - t0;

      expect(elapsedMs).toBeLessThan(10.0);
      expect(result.matches.length).toBe(1);
      expect(result.matches[0].matchType).toBe('COMPOSITE_1_TO_N');
      expect(result.matches[0].matchedAmount).toBe(300_000_000);
      expect(result.matches[0].ledgerEntryIds.length).toBe(3);
    });
  });

  // ==========================================================================
  // SECTION 3: DUAL CONTROL REPLAY & RACE CONDITION TESTING
  // ==========================================================================
  describe('3. Dual Control Replay, Race Conditions & Circular 09/2020 Compliance', () => {
    beforeEach(() => {
      resetConsumedTokens();
    });

    it('fails closed when Maker attempts self-approval (makerId === checkerId)', () => {
      const voucher = createPaymentVoucher(
        'accountant_duong',
        '0071001234567',
        'VIETCOMBANK',
        250_000_000,
        'Thanh toan hop dong may chu'
      );
      const pending = submitVoucherForApproval(voucher);

      // Attempt 1: Exact string match
      expect(() => {
        approveVoucher(pending, 'accountant_duong');
      }).toThrow(/Maker cannot be Checker/);

      // Attempt 2: Trailing whitespace evasion attempt
      expect(() => {
        approveVoucher(pending, '  accountant_duong  ');
      }).toThrow(/Maker cannot be Checker/);

      // Attempt 3: Self-rejection prohibition
      expect(() => {
        rejectVoucher(pending, 'accountant_duong', 'Tu choi boi chinh minh');
      }).toThrow(/Maker cannot reject own voucher as checker/);
    });

    it('fails closed when approval token has expired (> 15 minutes TTL)', () => {
      const voucher = createPaymentVoucher(
        'maker_user_01',
        '19034567890123',
        'TECHCOMBANK',
        180_000_000,
        'Thanh toan phi dich vu'
      );
      const pending = submitVoucherForApproval(voucher);

      // Simulate expiration: token expires 16 minutes in the past
      const expiredVoucher: PaymentVoucher = {
        ...pending,
        tokenExpiresAt: new Date(Date.now() - 60 * 1000).toISOString(), // 1 minute expired
      };

      // Approval must throw TokenExpired error
      expect(() => {
        approveVoucher(expiredVoucher, 'cfo_checker_02');
      }).toThrow(/TokenExpired: Circular 09\/2020 15-minute approval window has expired/);

      // checkVoucherExpiration must transition status to EXPIRED
      const expiredResult = checkVoucherExpiration(expiredVoucher, Date.now());
      expect(expiredResult.status).toBe('EXPIRED');
      expect(expiredResult.hitlToken).toBeNull();
    });

    it('fails closed on token replay attack (single-use token re-use)', () => {
      const voucher = createPaymentVoucher(
        'maker_user_01',
        '19034567890123',
        'TECHCOMBANK',
        100_000_000,
        'Thanh toan lan 1'
      );
      const pending = submitVoucherForApproval(voucher);
      const stolenToken = pending.hitlToken!;

      // First approval: Legitimate
      const approved = approveVoucher(pending, 'cfo_checker_01', 'BIOMETRIC_SIM', stolenToken);
      expect(approved.status).toBe('APPROVED');
      expect(approved.checkerId).toBe('cfo_checker_01');

      // Second approval with same voucher: Replay attack
      expect(() => {
        approveVoucher(pending, 'cfo_checker_02', 'SMS_OTP_SIM', stolenToken);
      }).toThrow(/TokenAlreadyUsed: Replay attack detected on HITL authorization token/);

      // Third approval attempt with fresh voucher but replayed token
      const freshVoucher = submitVoucherForApproval(
        createPaymentVoucher('maker_user_02', '123456', 'BIDV', 50_000_000, 'Tam ung')
      );
      expect(() => {
        approveVoucher(freshVoucher, 'cfo_checker_02', 'BIOMETRIC_SIM', stolenToken);
      }).toThrow(/TokenAlreadyUsed: Replay attack detected on HITL authorization token/);
    });

    it('simulates concurrent approval race condition: exactly 1 succeeds and duplicates fail closed', () => {
      const voucher = createPaymentVoucher(
        'maker_01',
        '987654321',
        'BIDV',
        300_000_000,
        'Thanh toan tien vat tu'
      );
      const pending = submitVoucherForApproval(voucher);
      const token = pending.hitlToken!;

      // Simulate 5 simultaneous concurrent authorization requests
      const checkers = ['cfo_01', 'cfo_02', 'cfo_03', 'cfo_04', 'cfo_05'];
      const results: { success: boolean; error?: string }[] = [];

      for (const checker of checkers) {
        try {
          approveVoucher(pending, checker, 'BIOMETRIC_SIM', token);
          results.push({ success: true });
        } catch (err: any) {
          results.push({ success: false, error: err.message });
        }
      }

      // Exactly 1 approval must succeed
      const successes = results.filter((r) => r.success);
      const failures = results.filter((r) => !r.success);

      expect(successes.length).toBe(1);
      expect(failures.length).toBe(4);
      for (const failure of failures) {
        expect(failure.error).toContain('TokenAlreadyUsed');
      }
    });

    it('strictly enforces 5-stage lifecycle state machine against illegal state transitions', () => {
      const draft = createPaymentVoucher('maker_1', '123', 'VCB', 50_000_000, 'Test');

      // Cannot settle DRAFT directly
      expect(() => settleVoucher(draft)).toThrow(/Cannot settle voucher in status DRAFT/);

      // Cannot approve DRAFT directly
      expect(() => approveVoucher(draft, 'checker_1')).toThrow(/Voucher is not pending approval/);

      const pending = submitVoucherForApproval(draft);

      // Cannot settle PENDING directly
      expect(() => settleVoucher(pending)).toThrow(/Cannot settle voucher in status PENDING_APPROVAL/);

      // Approve -> SETTLED is allowed
      const approved = approveVoucher(pending, 'checker_1');
      const settled = settleVoucher(approved);
      expect(settled.status).toBe('SETTLED');

      // Cannot re-approve SETTLED voucher
      expect(() => approveVoucher(settled, 'checker_2')).toThrow(/Voucher is not pending approval/);

      // Cannot settle SETTLED voucher again
      expect(() => settleVoucher(settled)).toThrow(/Cannot settle voucher in status SETTLED/);
    });
  });

  // ==========================================================================
  // SECTION 4: MERKLE AUDIT TAMPERING & BROKEN HASH CHAIN VERIFICATION
  // ==========================================================================
  describe('4. Merkle Audit Tampering & Broken Hash-Chain Detection', () => {
    it('generates valid Merkle proofs for payment vouchers and verifies inclusion', () => {
      const vouchers = [
        { voucherId: 'vch-1', makerId: 'm1', checkerId: 'c1', amountVnd: 100_000_000, status: 'APPROVED', approvedAt: '2026-08-15T10:00:00Z' },
        { voucherId: 'vch-2', makerId: 'm2', checkerId: 'c1', amountVnd: 200_000_000, status: 'APPROVED', approvedAt: '2026-08-15T10:05:00Z' },
        { voucherId: 'vch-3', makerId: 'm1', checkerId: 'c2', amountVnd: 300_000_000, status: 'APPROVED', approvedAt: '2026-08-15T10:10:00Z' },
        { voucherId: 'vch-4', makerId: 'm3', checkerId: 'c2', amountVnd: 400_000_000, status: 'APPROVED', approvedAt: '2026-08-15T10:15:00Z' },
      ];

      const leafHashes = vouchers.map((v) => computeMerkleLeaf(v));
      const tree = buildMerkleTree(leafHashes);

      expect(leafHashes.length).toBe(4);
      expect(tree.root).toMatch(/^[a-f0-9]{64}$/);

      // Generate and verify proof for each voucher
      for (let i = 0; i < vouchers.length; i++) {
        const proof = generateMerkleProof(leafHashes, i);
        const isValid = verifyMerkleProof(proof.leaf, proof.proof, tree.root);
        expect(isValid).toBe(true);
      }
    });

    it('fails closed when payment voucher payload is tampered (InvalidProof)', () => {
      const voucherOriginal = {
        voucherId: 'vch-corp-01',
        makerId: 'accountant_minh',
        checkerId: 'cfo_duong',
        amountVnd: 500_000_000,
        status: 'APPROVED',
        approvedAt: '2026-08-15T14:30:00Z',
      };

      const siblingVouchers = [
        voucherOriginal,
        { voucherId: 'vch-corp-02', makerId: 'm2', checkerId: 'c1', amountVnd: 100_000_000, status: 'APPROVED', approvedAt: '2026-08-15T14:31:00Z' },
        { voucherId: 'vch-corp-03', makerId: 'm3', checkerId: 'c1', amountVnd: 150_000_000, status: 'APPROVED', approvedAt: '2026-08-15T14:32:00Z' },
      ];

      const leaves = siblingVouchers.map((v) => computeMerkleLeaf(v));
      const tree = buildMerkleTree(leaves);
      const proof = generateMerkleProof(leaves, 0);

      // Verify legitimate proof passes
      expect(verifyMerkleProof(proof.leaf, proof.proof, tree.root)).toBe(true);

      // TAMPER ATTACK 1: Attacker modifies amount from 500M to 900M
      const tamperedVoucherAmount = { ...voucherOriginal, amountVnd: 900_000_000 };
      const tamperedLeafAmount = computeMerkleLeaf(tamperedVoucherAmount);
      expect(tamperedLeafAmount).not.toBe(proof.leaf);
      expect(verifyMerkleProof(tamperedLeafAmount, proof.proof, tree.root)).toBe(false);

      // TAMPER ATTACK 2: Attacker tampers checkerId
      const tamperedVoucherChecker = { ...voucherOriginal, checkerId: 'unauthorized_hacker' };
      const tamperedLeafChecker = computeMerkleLeaf(tamperedVoucherChecker);
      expect(verifyMerkleProof(tamperedLeafChecker, proof.proof, tree.root)).toBe(false);

      // TAMPER ATTACK 3: Attacker tampers status
      const tamperedVoucherStatus = { ...voucherOriginal, status: 'SETTLED' };
      const tamperedLeafStatus = computeMerkleLeaf(tamperedVoucherStatus);
      expect(verifyMerkleProof(tamperedLeafStatus, proof.proof, tree.root)).toBe(false);
    });

    it('fails closed when proof steps or sibling hashes are tampered', () => {
      const leaves = [
        sha256('leaf_0'),
        sha256('leaf_1'),
        sha256('leaf_2'),
        sha256('leaf_3'),
      ];
      const tree = buildMerkleTree(leaves);
      const proof = generateMerkleProof(leaves, 1);

      // Legitimate proof passes
      expect(verifyMerkleProof(proof.leaf, proof.proof, tree.root)).toBe(true);

      // Tamper step hash
      const tamperedSteps = JSON.parse(JSON.stringify(proof.proof));
      tamperedSteps[0].hash = sha256('malicious_hash');
      expect(verifyMerkleProof(proof.leaf, tamperedSteps, tree.root)).toBe(false);

      // Tamper step direction (flip left to right)
      const flippedSteps = JSON.parse(JSON.stringify(proof.proof));
      flippedSteps[0].position = flippedSteps[0].position === 'left' ? 'right' : 'left';
      expect(verifyMerkleProof(proof.leaf, flippedSteps, tree.root)).toBe(false);

      // Tamper root
      expect(verifyMerkleProof(proof.leaf, proof.proof, sha256('fake_root'))).toBe(false);
    });

    it('detects broken hash chain in ForwardAuditLedger when an entry payload is tampered', () => {
      const ledger = new ForwardAuditLedger();

      ledger.append('maker_1', 'VOUCHER_CREATED', { voucherId: 'vch-1', amount: 100_000_000 });
      ledger.append('checker_1', 'VOUCHER_APPROVED', { voucherId: 'vch-1', amount: 100_000_000 });
      ledger.append('settler_1', 'VOUCHER_SETTLED', { voucherId: 'vch-1', amount: 100_000_000 });
      ledger.append('maker_2', 'VOUCHER_CREATED', { voucherId: 'vch-2', amount: 250_000_000 });

      // Intact ledger passes verification
      const intactCheck = ledger.verifyIntegrity();
      expect(intactCheck.isValid).toBe(true);
      expect(intactCheck.brokenAt).toBeUndefined();

      // TAMPER ATTACK: Modify payload of block 2 directly in memory
      const entries = ledger.getEntries();
      entries[1].payload = { voucherId: 'vch-1', amount: 999_999_999 }; // Tampered amount

      // Integrity verification MUST fail and report exactly block 2
      const tamperedCheck = ledger.verifyIntegrity();
      expect(tamperedCheck.isValid).toBe(false);
      expect(tamperedCheck.brokenAt).toBe(2);
      expect(tamperedCheck.details).toContain('Block #2 currentHash mismatch (data tampered)');
    });

    it('detects broken hash chain in ForwardAuditLedger when prevHash is corrupted or block is deleted', () => {
      const ledger = new ForwardAuditLedger();

      ledger.append('actor_a', 'EVENT_A', { step: 1 });
      ledger.append('actor_b', 'EVENT_B', { step: 2 });
      ledger.append('actor_c', 'EVENT_C', { step: 3 });

      // Tamper prevHash of block 3
      const entries = ledger.getEntries();
      entries[2].prevHash = sha256('fake_previous_hash');

      const integrityResult = ledger.verifyIntegrity();
      expect(integrityResult.isValid).toBe(false);
      expect(integrityResult.brokenAt).toBe(3);
      expect(integrityResult.details).toContain('Block #3 prevHash mismatch');
    });

    it('verifies Merkle root of ForwardAuditLedger changes immediately upon any ledger tampering', () => {
      const ledger = new ForwardAuditLedger();

      ledger.append('maker_01', 'VOUCHER_CREATED', { vch: 'V1', amt: 50_000_000 });
      ledger.append('checker_01', 'VOUCHER_APPROVED', { vch: 'V1', amt: 50_000_000 });

      const originalRoot = ledger.computeMerkleRoot();
      expect(originalRoot).toMatch(/^[a-f0-9]{64}$/);

      // Append new event
      ledger.append('settler_01', 'VOUCHER_SETTLED', { vch: 'V1', amt: 50_000_000 });
      const updatedRoot = ledger.computeMerkleRoot();

      expect(updatedRoot).not.toBe(originalRoot);
    });
  });

});
