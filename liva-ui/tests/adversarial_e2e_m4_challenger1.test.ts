import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { mount } from '@vue/test-utils';
import HitlResolutionModal from '../src/components/banking/HitlResolutionModal.vue';
import PaymentDisbursementModal from '../src/components/banking/treasury/PaymentDisbursementModal.vue';
import { useAuthStore } from '../src/stores/authStore';
import { useTreasuryStore, HIGH_LIMIT_THRESHOLD_VND, type PaymentOrder } from '../src/stores/treasuryStore';
import { useReconciliationStore, type BankTransaction } from '../src/stores/reconciliationStore';

describe('Milestone 4 Challenger 1: End-to-End Adversarial Integration Tests', () => {
  const mockTransaction: BankTransaction = {
    id: 'tx-m4-challenger-01',
    txCode: 'VCB-E2E-2026-99',
    time: '10:15 AM',
    date: '2026-09-16',
    bankCode: 'VCB',
    accountNumber: '00710009821',
    memo: 'THANH TOAN TIEN MAY BOM HOP DONG 88',
    bankAmount: 75000000,
    ledgerAmount: 0,
    variance: 75000000,
    status: 'PENDING_HITL',
    statusLabel: 'Chờ duyệt',
    confidenceScore: 0.88,
    tokenUuid: 'hitl-tok-m4-1234',
  };

  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
    vi.restoreAllMocks();
  });

  // =========================================================================
  // 1. Reconciliation Workflow & Dual Control Challenge
  // =========================================================================
  describe('1. Reconciliation Workflow & Dual Control Challenge', () => {
    it('enforces Maker propose-only and blocks Maker self-approval in UI and Store', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');

      const wrapper = mount(HitlResolutionModal, {
        props: { transaction: mockTransaction, isOpen: true },
      });

      // Maker can only see proposal button
      expect(wrapper.find('.btn-propose').exists()).toBe(true);
      expect(wrapper.find('.btn-approve').exists()).toBe(false);
      expect(wrapper.find('.btn-reject').exists()).toBe(false);

      // Maker attempts self-approval disguised as Checker
      const inputs = wrapper.findAll('.role-input');
      expect(inputs).toHaveLength(2);
      await inputs[0].setValue('maker_nam');
      await inputs[1].setValue('  MAKER_NAM\t');

      expect(wrapper.find('.violation-alert').exists()).toBe(true);
      expect(wrapper.find('.btn-propose').attributes('disabled')).toBeDefined();
    });

    it('allows independent Checker approval and updates state to MATCHED', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const reconcileStore = useReconciliationStore();
      const confirmSpy = vi.spyOn(reconcileStore, 'confirmHitlResolution');

      const wrapper = mount(HitlResolutionModal, {
        props: { transaction: mockTransaction, isOpen: true },
      });

      // Checker sees Approve and Reject buttons
      expect(wrapper.find('.btn-approve').exists()).toBe(true);
      expect(wrapper.find('.btn-reject').exists()).toBe(true);
      expect(wrapper.find('.btn-propose').exists()).toBe(false);

      // Valid dual control identities: Maker = maker_nam, Checker = checker_tri
      const inputs = wrapper.findAll('.role-input');
      await inputs[0].setValue('maker_nam');
      await inputs[1].setValue('checker_tri');

      expect(wrapper.find('.violation-alert').exists()).toBe(false);

      // Trigger legitimate approval
      await wrapper.find('.btn-approve').trigger('click');
      await wrapper.vm.$nextTick();

      expect(confirmSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          tokenUuid: 'hitl-tok-m4-1234',
          checkerId: 'checker_tri',
        })
      );
    });

    it('locks Auditor and Admin into read-only mode with zero mutations', async () => {
      const authStore = useAuthStore();

      for (const username of ['auditor_lan', 'admin_sys']) {
        authStore.switchUser(username);

        const wrapper = mount(HitlResolutionModal, {
          props: { transaction: mockTransaction, isOpen: true },
        });

        expect(wrapper.find('.auditor-pill').exists()).toBe(true);
        expect(wrapper.find('.btn-propose').exists()).toBe(false);
        expect(wrapper.find('.btn-approve').exists()).toBe(false);
        expect(wrapper.find('.btn-reject').exists()).toBe(false);
      }
    });
  });

  // =========================================================================
  // 2. Treasury Disbursement & High-Limit Threshold Escalation
  // =========================================================================
  describe('2. Treasury Disbursement & High-Limit Threshold Escalation', () => {
    it('enforces high-limit escalation threshold (500M VND): Checker blocked, CFO allowed', async () => {
      const treasuryStore = useTreasuryStore();

      // Switch to Maker role explicitly
      treasuryStore.switchOperatorRole('MAKER');

      // Normal order (< 500M VND)
      const normalOrder = treasuryStore.createPaymentOrder({
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Nhà cung cấp vật tư',
        beneficiaryBank: 'VCB',
        beneficiaryAccount: '0071000111',
        amount: 150000000,
        memo: 'Thanh toán vật tư văn phòng',
        priority: 'NORMAL',
        makerNote: 'Đầy đủ hóa đơn',
      });

      await new Promise((r) => setTimeout(r, 10));

      // High-limit order (>= 500M VND, < 5B VND)
      const highLimitOrder = treasuryStore.createPaymentOrder({
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Nhà thầu xây lắp MEP',
        beneficiaryBank: 'BIDV',
        beneficiaryAccount: '1201000222',
        amount: 750000000,
        memo: 'Thanh toán gói thầu xây lắp đợt 2',
        priority: 'URGENT',
        makerNote: 'Biên bản nghiệm thu hoàn thành',
      });

      await new Promise((r) => setTimeout(r, 10));

      // Extreme high-limit order (>= 5B VND)
      const extremeOrder = treasuryStore.createPaymentOrder({
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Tổng thầu EPC',
        beneficiaryBank: 'MBB',
        beneficiaryAccount: '0680100333',
        amount: 6000000000,
        memo: 'Thanh toán hợp đồng EPC xây dựng nhà máy',
        priority: 'URGENT',
        makerNote: 'Phê duyệt HĐQT số 08/NQ-HĐQT',
      });

      // --- CHECKER TEST ---
      treasuryStore.switchOperatorRole('CHECKER');

      // 1. Normal order: Checker CAN approve
      expect(() => {
        treasuryStore.approvePaymentOrder(normalOrder.id, 'Duyệt thanh toán vật tư hợp lệ');
      }).not.toThrow();
      expect(normalOrder.status).toBe('APPROVED');

      // 2. High-limit order (750M VND): Checker BLOCKED
      expect(() => {
        treasuryStore.approvePaymentOrder(highLimitOrder.id, 'Cố tình duyệt hạn mức cao');
      }).toThrow(`Lệnh chi vượt hạn mức 500.000.000 ₫ yêu cầu Giám đốc tài chính (CFO) phê duyệt!`);

      // 3. Extreme high-limit order (6B VND): Checker BLOCKED
      expect(() => {
        treasuryStore.approvePaymentOrder(extremeOrder.id, 'Cố tình duyệt hạn mức siêu lớn');
      }).toThrow(`Lệnh chi vượt hạn mức 500.000.000 ₫ yêu cầu Giám đốc tài chính (CFO) phê duyệt!`);

      // UI Verification for Checker on high-limit order
      treasuryStore.selectPaymentOrder(highLimitOrder.id);
      const checkerModal = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });
      await checkerModal.vm.$nextTick();

      expect(checkerModal.find('.high-limit-warning-box').exists()).toBe(true);
      expect(checkerModal.find('.btn-approve').attributes('disabled')).toBeDefined();

      // --- CFO TEST (cfo_huong) ---
      treasuryStore.switchOperatorRole('CFO');

      // 4. High-limit order (750M VND): CFO CAN approve
      expect(() => {
        treasuryStore.approvePaymentOrder(highLimitOrder.id, 'CFO phê duyệt hồ sơ gói thầu MEP 750M hợp lệ');
      }).not.toThrow();
      expect(highLimitOrder.status).toBe('APPROVED');

      // 5. Extreme high-limit order (6B VND): CFO CAN approve
      expect(() => {
        treasuryStore.approvePaymentOrder(extremeOrder.id, 'CFO phê duyệt hồ sơ EPC 6 Tỷ theo NQ HĐQT');
      }).not.toThrow();
      expect(extremeOrder.status).toBe('APPROVED');

      // UI Verification for CFO on approved order
      treasuryStore.selectPaymentOrder(highLimitOrder.id);
      const cfoModal = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });
      await cfoModal.vm.$nextTick();
      expect(cfoModal.find('.btn-disburse').exists()).toBe(true);
    });

    it('strictly blocks CFO from self-approving orders proposed by CFO', async () => {
      const treasuryStore = useTreasuryStore();

      treasuryStore.switchOperatorRole('CFO');

      // Simulate an order proposed by cfo_huong
      const cfoOrder: PaymentOrder = {
        id: 'pay-cfo-self',
        paymentGuid: 'guid-cfo-self',
        orderNumber: 'UNC-CFO-001',
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Đối tác chiến lược M&A',
        beneficiaryBank: 'VCB',
        beneficiaryAccount: '0071000999',
        amount: 800000000,
        memo: 'Chi phí thẩm định M&A',
        priority: 'NORMAL',
        status: 'SUBMITTED_TO_CHECKER',
        makerId: 'cfo_huong',
        submittedAt: new Date().toISOString(),
      };
      treasuryStore.paymentOrders.unshift(cfoOrder);

      expect(() => {
        treasuryStore.approvePaymentOrder(cfoOrder.id, 'CFO tự duyệt lệnh của mình');
      }).toThrow(/Vi phạm nguyên tắc 4 mắt \(SoD\)/);
    });

    it('strictly prevents Auditor and Admin from approving, rejecting, or disbursing', async () => {
      const treasuryStore = useTreasuryStore();

      // Test Auditor
      treasuryStore.switchOperatorRole('AUDITOR');

      expect(() => {
        treasuryStore.approvePaymentOrder('pay-01', 'Illegal approval');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      expect(() => {
        treasuryStore.rejectPaymentOrder('pay-01', 'Illegal rejection');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      expect(() => {
        treasuryStore.executePaymentDisbursement('pay-02');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      // Test Admin
      treasuryStore.switchOperatorRole('ADMIN');

      expect(() => {
        treasuryStore.approvePaymentOrder('pay-01', 'Illegal approval');
      }).toThrow(/Quản trị viên \(Admin\) không có thẩm quyền/);

      expect(() => {
        treasuryStore.rejectPaymentOrder('pay-01', 'Illegal rejection');
      }).toThrow(/Quản trị viên \(Admin\) không có thẩm quyền/);

      expect(() => {
        treasuryStore.executePaymentDisbursement('pay-02');
      }).toThrow(/Quản trị viên \(Admin\) không có thẩm quyền/);
    });

    it('executes disbursement with zero-float integer math and balance preservation', async () => {
      const treasuryStore = useTreasuryStore();
      treasuryStore.switchOperatorRole('CFO');

      const orderToDisburse = treasuryStore.paymentOrders.find((p) => p.status === 'APPROVED')!;
      const sourceAcc = treasuryStore.accounts.find((a) => a.id === orderToDisburse.sourceAccountId)!;
      const initialBalance = sourceAcc.balance;
      const disbursementAmount = orderToDisburse.amount;

      treasuryStore.executePaymentDisbursement(orderToDisburse.id);

      expect(orderToDisburse.status).toBe('EXECUTED');
      expect(orderToDisburse.executedAt).toBeDefined();
      expect(orderToDisburse.bankRefNumber).toMatch(/^FT\d+/);
      expect(sourceAcc.balance).toBe(initialBalance - disbursementAmount);
      expect(Number.isInteger(sourceAcc.balance)).toBe(true);
    });
  });
});
