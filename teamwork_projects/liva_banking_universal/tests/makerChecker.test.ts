import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import {
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher,
  rejectVoucher,
  settleVoucher,
  checkVoucherExpiration,
  resetConsumedTokens,
} from '../src/engine/treasury/makerChecker';
import { useTreasuryStore } from '../src/stores/treasuryStore';

describe('Feature F17: Maker-Checker Dual Control State Machine (Circular 09/2020/TT-NHNN)', () => {
  beforeEach(() => {
    resetConsumedTokens();
    setActivePinia(createPinia());
  });

  describe('1. Payment Voucher Creation (DRAFT)', () => {
    it('creates valid payment voucher in DRAFT status', () => {
      const vch = createPaymentVoucher(
        'maker_accountant_01',
        '19034567890123',
        'TECHCOMBANK',
        550_000_000,
        'Cung ứng thiết bị Masan',
        'CONG TY MASAN'
      );

      expect(vch.status).toBe('DRAFT');
      expect(vch.makerId).toBe('maker_accountant_01');
      expect(vch.amountVnd).toBe(550_000_000);
      expect(vch.beneficiaryAccount).toBe('19034567890123');
      expect(vch.beneficiaryBank).toBe('TECHCOMBANK');
      expect(vch.beneficiaryName).toBe('CONG TY MASAN');
      expect(vch.voucherId).toMatch(/^vch-/);
      expect(Boolean(vch.createdAt)).toBe(true);
      expect(vch.checkerId).toBeUndefined();
      expect(vch.hitlToken).toBeUndefined();
    });

    it('fails closed when makerId is empty or whitespace', () => {
      expect(() => {
        createPaymentVoucher('', '1903456789', 'TCB', 50_000_000, 'Test');
      }).toThrow(/Maker ID is required/);

      expect(() => {
        createPaymentVoucher('   ', '1903456789', 'TCB', 50_000_000, 'Test');
      }).toThrow(/Maker ID is required/);
    });

    it('fails closed when amount is zero or negative', () => {
      expect(() => {
        createPaymentVoucher('maker_1', '1903456789', 'TCB', 0, 'Test');
      }).toThrow(/Payment amount must be positive/);

      expect(() => {
        createPaymentVoucher('maker_1', '1903456789', 'TCB', -100_000, 'Test');
      }).toThrow(/Payment amount must be positive/);
    });

    it('fails closed when beneficiary account is missing', () => {
      expect(() => {
        createPaymentVoucher('maker_1', '', 'TCB', 10_000_000, 'Test');
      }).toThrow(/Beneficiary account is required/);
    });

    it('rounds floating point amount strictly to integer VND', () => {
      const vch = createPaymentVoucher('maker_1', '1903456789', 'TCB', 10_000_000.75, 'Test');
      expect(vch.amountVnd).toBe(10_000_001);
    });
  });

  describe('2. Voucher Submission & HITL Token Generation', () => {
    it('transitions DRAFT voucher to PENDING_APPROVAL with 15-min TTL token', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 100_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      expect(pending.status).toBe('PENDING_APPROVAL');
      expect(typeof pending.hitlToken).toBe('string');
      expect(pending.hitlToken?.length).toBeGreaterThan(10);
      expect(Boolean(pending.tokenExpiresAt)).toBe(true);
      expect(Boolean(pending.submittedAt)).toBe(true);

      const ttlMs = new Date(pending.tokenExpiresAt!).getTime() - new Date(pending.submittedAt!).getTime();
      expect(ttlMs).toBeCloseTo(15 * 60 * 1000, -3); // ~900,000ms
    });

    it('fails closed when attempting to submit non-DRAFT voucher', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 100_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      expect(() => {
        submitVoucherForApproval(pending);
      }).toThrow(/Cannot submit voucher in PENDING_APPROVAL status/);
    });
  });

  describe('3. Circular 09/2020 Dual-Control Authorization (Checker Approval)', () => {
    it('approves voucher with distinct checker and generates HMAC & Merkle leaf', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      const approved = approveVoucher(pending, 'checker_cfo_01', 'BIOMETRIC_SIM');

      expect(approved.status).toBe('APPROVED');
      expect(approved.checkerId).toBe('checker_cfo_01');
      expect(approved.authMethod).toBe('BIOMETRIC_SIM');
      expect(Boolean(approved.approvedAt)).toBe(true);
      expect(approved.hitlToken).toBeNull();
      expect(approved.signatureHmac).toHaveLength(64);
      expect(approved.merkleLeafHash).toHaveLength(64);
    });

    it('enforces fail-closed self-approval block (makerId === checkerId)', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      expect(() => {
        approveVoucher(pending, 'maker_1');
      }).toThrow(/Circular 09\/2020\/TT-NHNN Violation: Maker cannot be Checker/);
    });

    it('fails closed when checkerId is empty', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      expect(() => {
        approveVoucher(pending, '');
      }).toThrow(/Checker ID is required/);
    });

    it('fails closed when approving draft voucher before submission', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');

      expect(() => {
        approveVoucher(draft, 'checker_1');
      }).toThrow(/Voucher is not pending approval/);
    });

    it('fails closed on token replay attack', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);
      const token = pending.hitlToken!;

      approveVoucher(pending, 'checker_1');

      // Attempt replay with consumed token
      const pending2 = { ...pending, hitlToken: token };
      expect(() => {
        approveVoucher(pending2, 'checker_2');
      }).toThrow(/TokenAlreadyUsed: Replay attack detected/);
    });

    it('fails closed when token has expired (> 15 minutes)', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      // Artificially expire the token
      pending.tokenExpiresAt = new Date(Date.now() - 1000).toISOString();

      expect(() => {
        approveVoucher(pending, 'checker_1');
      }).toThrow(/TokenExpired/);
    });
  });

  describe('4. Checker Rejection', () => {
    it('rejects pending voucher with reason and consumes token', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      const rejected = rejectVoucher(pending, 'checker_1', 'Hóa đơn không đúng mẫu quy định');

      expect(rejected.status).toBe('REJECTED');
      expect(rejected.checkerId).toBe('checker_1');
      expect(rejected.rejectReason).toBe('Hóa đơn không đúng mẫu quy định');
      expect(Boolean(rejected.rejectedAt)).toBe(true);
      expect(rejected.hitlToken).toBeNull();
    });

    it('prevents Maker from rejecting own voucher as checker', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      expect(() => {
        rejectVoucher(pending, 'maker_1');
      }).toThrow(/Circular 09\/2020\/TT-NHNN Violation: Maker cannot reject own voucher as checker/);
    });
  });

  describe('5. Settle & Expiration Management', () => {
    it('settles approved voucher transitioning to SETTLED', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);
      const approved = approveVoucher(pending, 'checker_1');

      const settled = settleVoucher(approved);
      expect(settled.status).toBe('SETTLED');
      expect(Boolean(settled.settledAt)).toBe(true);
    });

    it('fails closed when trying to settle non-APPROVED voucher', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      expect(() => {
        settleVoucher(draft);
      }).toThrow(/Cannot settle voucher in status DRAFT/);
    });

    it('marks pending voucher as EXPIRED when 900 seconds have elapsed', () => {
      const draft = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Test');
      const pending = submitVoucherForApproval(draft);

      const futureTime = Date.now() + 16 * 60 * 1000;
      const expired = checkVoucherExpiration(pending, futureTime);

      expect(expired.status).toBe('EXPIRED');
      expect(expired.hitlToken).toBeNull();
    });
  });

  describe('6. Pinia Treasury Store End-to-End Workflow', () => {
    it('manages full payment lifecycle through Pinia store', () => {
      const store = useTreasuryStore();

      // 1. Create Voucher
      const voucher = store.createVoucher(
        'maker_accountant_01',
        '001100987654',
        'VIETCOMBANK',
        120_000_000,
        'Hợp đồng bảo trì máy chủ DC',
        'CONG TY CMC'
      );
      expect(voucher.status).toBe('DRAFT');

      // 2. Submit Voucher
      const submitted = store.submitVoucher(voucher.voucherId);
      expect(submitted.status).toBe('PENDING_APPROVAL');
      expect(store.pendingVouchers.some((v) => v.voucherId === voucher.voucherId)).toBe(true);

      // 3. Approve Voucher with Independent Checker
      const approved = store.approveVoucher(voucher.voucherId, 'checker_cfo_01', 'BIOMETRIC_SIM');
      expect(approved.status).toBe('APPROVED');
      expect(store.approvedVouchers.some((v) => v.voucherId === voucher.voucherId)).toBe(true);

      // 4. Check Merkle Root includes approved voucher
      expect(store.merkleRoot).toHaveLength(64);

      // 5. Settle Voucher
      const settled = store.settleVoucher(voucher.voucherId);
      expect(settled.status).toBe('SETTLED');
      expect(store.settledVouchers.some((v) => v.voucherId === voucher.voucherId)).toBe(true);

      // 6. Verify forward audit ledger integrity
      const auditResult = store.verifyAuditIntegrity();
      expect(auditResult.isValid).toBe(true);
    });
  });
});
