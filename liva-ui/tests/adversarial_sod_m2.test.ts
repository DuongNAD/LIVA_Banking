import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import HitlResolutionModal from '../src/components/banking/HitlResolutionModal.vue';
import PaymentDisbursementModal from '../src/components/banking/treasury/PaymentDisbursementModal.vue';
import BankingQuarantineView from '../src/views/banking/BankingQuarantineView.vue';
import { useAuthStore } from '../src/stores/authStore';
import { useTreasuryStore, HIGH_LIMIT_THRESHOLD_VND, type PaymentOrder } from '../src/stores/treasuryStore';
import { useReconciliationStore, type BankTransaction } from '../src/stores/reconciliationStore';
import { useWorkbenchStore } from '../src/stores/workbenchStore';

describe('Adversarial SoD & Dual Control Attacks — Milestone 2 Verification', () => {
  const mockTx: BankTransaction = {
    id: 'tx-adv-01',
    txCode: 'VCB2026103099',
    time: '02:30 PM',
    date: '2026-10-30',
    bankCode: 'VCB',
    accountNumber: 'VCB-00710009821',
    memo: 'CHI PHI DU AN ADVERSARIAL CHALLENGE',
    bankAmount: 150000000,
    ledgerAmount: 0,
    variance: 150000000,
    status: 'PENDING_HITL',
    statusLabel: 'Chờ duyệt',
    confidenceScore: 0.88,
    tokenUuid: 'tok-adv-hitl-99',
  };

  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
    vi.restoreAllMocks();
  });

  // =========================================================================
  // 1. Adversarial Self-Approval in HitlResolutionModal
  // =========================================================================
  describe('1. Adversarial Self-Approval in HitlResolutionModal', () => {
    it('blocks self-approval when identities match with case variations and whitespace', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const wrapper = mount(HitlResolutionModal, {
        props: { transaction: mockTx, isOpen: true },
      });

      const inputs = wrapper.findAll('.role-input');
      expect(inputs).toHaveLength(2);

      // Attack Vector 1: Case variations ('maker_nam' vs 'MAKER_NAM')
      await inputs[0].setValue('maker_nam');
      await inputs[1].setValue('MAKER_NAM');

      expect(wrapper.find('.violation-alert').exists()).toBe(true);
      expect(wrapper.find('.btn-approve').attributes('disabled')).toBeDefined();
      expect(wrapper.find('.btn-reject').attributes('disabled')).toBeDefined();

      // Attack Vector 2: Whitespace padding ('  checker_tri  ' vs 'CHECKER_TRI\t')
      await inputs[0].setValue('  checker_tri  ');
      await inputs[1].setValue('CHECKER_TRI\t');

      expect(wrapper.find('.violation-alert').exists()).toBe(true);
      expect(wrapper.find('.btn-approve').attributes('disabled')).toBeDefined();
      expect(wrapper.find('.btn-reject').attributes('disabled')).toBeDefined();
    });

    it('rejects execution in handleConfirm and handleReject if isSelfApprovalViolation is true', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const reconcileStore = useReconciliationStore();
      const confirmSpy = vi.spyOn(reconcileStore, 'confirmHitlResolution');

      const wrapper = mount(HitlResolutionModal, {
        props: { transaction: mockTx, isOpen: true },
      });

      const inputs = wrapper.findAll('.role-input');
      await inputs[0].setValue('compromised_identity');
      await inputs[1].setValue('compromised_identity');

      expect(wrapper.find('.violation-alert').exists()).toBe(true);

      // Attack Vector: Attacker uses browser DevTools to remove disabled attribute and clicks
      const approveBtn = wrapper.find('.btn-approve');
      expect(approveBtn.attributes('disabled')).toBeDefined();
      approveBtn.element.removeAttribute('disabled');
      await approveBtn.trigger('click');
      await wrapper.vm.$nextTick();

      // Reconcile store must NOT be called
      expect(confirmSpy).not.toHaveBeenCalled();
      // Violation error banner must be populated
      expect(wrapper.find('.submit-error-banner').exists()).toBe(true);
      expect(wrapper.find('.submit-error-banner').text()).toContain('Vi phạm Thông tư 09/2020/TT-NHNN');

      // Attempt to invoke reject click by removing disabled attribute
      const rejectBtn = wrapper.find('.btn-reject');
      expect(rejectBtn.attributes('disabled')).toBeDefined();
      rejectBtn.element.removeAttribute('disabled');
      await rejectBtn.trigger('click');
      await wrapper.vm.$nextTick();

      expect(confirmSpy).not.toHaveBeenCalled();
      expect(wrapper.find('.submit-error-banner').text()).toContain('Vi phạm Thông tư 09/2020/TT-NHNN');
    });

    it('enforces Phase 1 proposal only for Maker and hides approval/rejection buttons', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');

      const wrapper = mount(HitlResolutionModal, {
        props: { transaction: mockTx, isOpen: true },
      });

      // Maker can only propose
      expect(wrapper.find('.btn-propose').exists()).toBe(true);
      expect(wrapper.find('.btn-approve').exists()).toBe(false);
      expect(wrapper.find('.btn-reject').exists()).toBe(false);

      // Self-approval check applies to proposal too
      const inputs = wrapper.findAll('.role-input');
      await inputs[0].setValue('SAME_USER');
      await inputs[1].setValue('SAME_USER');

      expect(wrapper.find('.violation-alert').exists()).toBe(true);
      expect(wrapper.find('.btn-propose').attributes('disabled')).toBeDefined();
    });

    it('Auditor and Admin sessions are strictly read-only with no action buttons', async () => {
      const authStore = useAuthStore();

      for (const roleUsername of ['auditor_lan', 'admin_sys']) {
        authStore.switchUser(roleUsername);

        const wrapper = mount(HitlResolutionModal, {
          props: { transaction: mockTx, isOpen: true },
        });

        expect(wrapper.find('.auditor-banner').exists()).toBe(roleUsername === 'auditor_lan');
        expect(wrapper.find('.auditor-pill').exists()).toBe(true);
        expect(wrapper.find('.auditor-pill').text()).toBe('[HỒ SƠ KIỂM TOÁN CHỈ ĐỌC]');

        expect(wrapper.find('.btn-propose').exists()).toBe(false);
        expect(wrapper.find('.btn-approve').exists()).toBe(false);
        expect(wrapper.find('.btn-reject').exists()).toBe(false);
        expect(wrapper.find('.btn-secondary').text()).toBe('Đóng');
      }
    });
    it('dynamically adapts and blocks when session switches while modal is open', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');

      const txWithCheckerMaker: BankTransaction = {
        ...mockTx,
        id: 'tx-sod-switch',
        // Maker was originally checker_tri
        ...({ makerId: 'checker_tri' } as any),
      };

      const wrapper = mount(HitlResolutionModal, {
        props: { transaction: txWithCheckerMaker, isOpen: true },
      });

      // Initially Maker sees propose
      expect(wrapper.find('.btn-propose').exists()).toBe(true);

      // Now session switches to checker_tri (who was the maker of this tx!)
      authStore.switchUser('checker_tri');
      await wrapper.vm.$nextTick();

      // Checker sees approve/reject buttons
      expect(wrapper.find('.btn-approve').exists()).toBe(true);
      // But because makerId == checkerId ('checker_tri'), SoD violation triggers!
      expect(wrapper.find('.violation-alert').exists()).toBe(true);
      expect(wrapper.find('.btn-approve').attributes('disabled')).toBeDefined();
      expect(wrapper.find('.btn-reject').attributes('disabled')).toBeDefined();
    });
  });

  // =========================================================================
  // 2. Adversarial Treasury Disbursement & Role Bypass
  // =========================================================================
  describe('2. Adversarial Treasury Disbursement & Role Bypass', () => {
    it('rejects invalid inputs on order creation (zero amount, negative amount)', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');
      const treasuryStore = useTreasuryStore();

      expect(() => {
        treasuryStore.createPaymentOrder({
          sourceAccountId: 'acc-vcb-01',
          beneficiaryName: 'Target',
          beneficiaryBank: 'VCB',
          beneficiaryAccount: '123',
          amount: 0,
          memo: 'Zero amount attack',
          priority: 'NORMAL',
        });
      }).toThrow('Số tiền thanh toán phải lớn hơn 0 VND');

      expect(() => {
        treasuryStore.createPaymentOrder({
          sourceAccountId: 'acc-vcb-01',
          beneficiaryName: 'Target',
          beneficiaryBank: 'VCB',
          beneficiaryAccount: '123',
          amount: -10000000,
          memo: 'Negative amount attack',
          priority: 'NORMAL',
        });
      }).toThrow('Số tiền thanh toán phải lớn hơn 0 VND');
    });

    it('rejects approval note shorter than 5 characters', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');
      const treasuryStore = useTreasuryStore();

      const validOrder = treasuryStore.paymentOrders.find(
        (o) => o.status === 'SUBMITTED_TO_CHECKER' && o.makerId !== 'checker_tri' && o.amount < HIGH_LIMIT_THRESHOLD_VND
      );
      expect(validOrder).toBeDefined();

      expect(() => {
        treasuryStore.approvePaymentOrder(validOrder!.id, 'ok');
      }).toThrow('Cần nhập ý kiến phê duyệt tối thiểu 5 ký tự');

      expect(() => {
        treasuryStore.approvePaymentOrder(validOrder!.id, '   ');
      }).toThrow('Cần nhập ý kiến phê duyệt tối thiểu 5 ký tự');
    });

    it('blocks self-rejection when Checker is the Maker of the payment order', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');
      const treasuryStore = useTreasuryStore();

      const selfOrder: PaymentOrder = {
        id: 'pay-self-reject',
        paymentGuid: 'guid-self-reject',
        orderNumber: 'UNC-REJECT',
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Nhà cung cấp',
        beneficiaryBank: 'VCB',
        beneficiaryAccount: '0071',
        amount: 30000000,
        memo: 'Thanh toán test self reject',
        priority: 'NORMAL',
        status: 'SUBMITTED_TO_CHECKER',
        makerId: 'checker_tri',
        submittedAt: new Date().toISOString(),
      };
      treasuryStore.paymentOrders.unshift(selfOrder);

      expect(() => {
        treasuryStore.rejectPaymentOrder(selfOrder.id, 'Checker cố tình tự từ chối lệnh của mình');
      }).toThrow(/Vi phạm nguyên tắc 4 mắt \(SoD\): Người từ chối không được trùng người lập lệnh!/);
    });

    it('prevents Maker from approving, rejecting, or disbursing in UI and store', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');

      const treasuryStore = useTreasuryStore();

      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      // UI: Maker cannot review, approve, or disburse
      expect(wrapper.find('.checker-decision-box').exists()).toBe(false);
      expect(wrapper.find('.btn-disburse').exists()).toBe(false);

      const targetOrder = treasuryStore.paymentOrders[0];

      // Store: Maker direct method invocation attacks
      expect(() => {
        treasuryStore.approvePaymentOrder(targetOrder.id, 'Adversarial maker approval');
      }).toThrow(/Chỉ Kế toán trưởng hoặc Giám đốc tài chính/);

      expect(() => {
        treasuryStore.rejectPaymentOrder(targetOrder.id, 'Adversarial maker reject');
      }).toThrow(/Chỉ Kế toán trưởng hoặc Giám đốc tài chính/);

      expect(() => {
        treasuryStore.executePaymentDisbursement(targetOrder.id);
      }).toThrow();

      expect(() => {
        treasuryStore.executeCashSweeping('acc-vcb-01', 'acc-tcb-01', 10000000);
      }).toThrow(/Kế toán viên \(Maker\) không được phép thực hiện điều chuyển vốn trực tiếp/);
    });

    it('detects and blocks Checker self-approval across IDs, usernames, and aliases', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const treasuryStore = useTreasuryStore();

      // Test alias / username variants where maker matches checker
      const testCases = [
        'checker_tri',
        'CHECKER_TRI',
        'usr-cfo-checker-01',
        'USR-CFO-CHECKER-01',
      ];

      for (const testMakerId of testCases) {
        const orderId = `pay-sod-test-${Date.now()}`;
        const order: PaymentOrder = {
          id: orderId,
          paymentGuid: `guid-${orderId}`,
          orderNumber: `UNC-SOD-${orderId.slice(-4)}`,
          sourceAccountId: 'acc-vcb-01',
          beneficiaryName: 'Công ty Đối tác SoD',
          beneficiaryBank: 'TCB',
          beneficiaryAccount: '1903000123',
          amount: 50000000,
          memo: 'Thanh toán đối tác',
          priority: 'NORMAL',
          status: 'SUBMITTED_TO_CHECKER',
          makerId: testMakerId,
          submittedAt: new Date().toISOString(),
        };
        treasuryStore.paymentOrders.unshift(order);
        treasuryStore.selectPaymentOrder(order.id);

        const wrapper = mount(PaymentDisbursementModal, {
          props: { isOpen: true },
        });
        await wrapper.vm.$nextTick();

        // UI: Must show SoD warning and disable decision buttons
        expect(wrapper.find('.sod-warning-box').exists()).toBe(true);
        expect(wrapper.find('.btn-approve').attributes('disabled')).toBeDefined();
        expect(wrapper.find('.btn-reject').attributes('disabled')).toBeDefined();

        // Store: Must throw SoD exception
        expect(() => {
          treasuryStore.approvePaymentOrder(order.id, 'Adversarial self approval');
        }).toThrow(/Vi phạm nguyên tắc 4 mắt \(SoD\)/);

        expect(() => {
          treasuryStore.rejectPaymentOrder(order.id, 'Adversarial self rejection');
        }).toThrow(/Vi phạm nguyên tắc 4 mắt \(SoD\)/);
      }
    });

    it('enforces 500M VND high-limit approval gate strictly: Checker blocked, CFO allowed', async () => {
      const authStore = useAuthStore();
      const treasuryStore = useTreasuryStore();

      const createOrderWithAmount = (amount: number, id: string): PaymentOrder => ({
        id,
        paymentGuid: `guid-${id}`,
        orderNumber: `UNC-${id}`,
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Công ty Máy Móc Siêu Trường',
        beneficiaryBank: 'BIDV',
        beneficiaryAccount: '1201000999',
        amount,
        memo: `Mua thiết bị ${amount} VND`,
        priority: 'NORMAL',
        status: 'SUBMITTED_TO_CHECKER',
        makerId: 'maker_nam',
        submittedAt: new Date().toISOString(),
      });

      // Boundary 1: 499,999,999 VND (< 500M)
      const belowLimit = createOrderWithAmount(499999999, 'pay-below-limit');
      treasuryStore.paymentOrders.unshift(belowLimit);

      // Boundary 2: 500,000,000 VND (Exact threshold)
      const exactLimit = createOrderWithAmount(HIGH_LIMIT_THRESHOLD_VND, 'pay-exact-limit');
      treasuryStore.paymentOrders.unshift(exactLimit);

      // Boundary 3: 1,000,000,000 VND (> 500M)
      const aboveLimit = createOrderWithAmount(1000000000, 'pay-above-limit');
      treasuryStore.paymentOrders.unshift(aboveLimit);

      // TEST AS CHECKER
      authStore.login('checker_tri', 'LivaChecker@2026');

      // 1. Below limit: Checker CAN approve
      expect(() => {
        treasuryStore.approvePaymentOrder(belowLimit.id, 'Hợp lệ dưới hạn mức 500M');
      }).not.toThrow();
      expect(belowLimit.status).toBe('APPROVED');

      // 2. Exact limit (500M): Checker BLOCKED
      expect(() => {
        treasuryStore.approvePaymentOrder(exactLimit.id, 'Cố tình duyệt hạn mức lớn');
      }).toThrow(/Lệnh chi vượt hạn mức 500.000.000 ₫ yêu cầu Giám đốc tài chính \(CFO\) phê duyệt!/);

      // 3. Above limit (1B): Checker BLOCKED
      expect(() => {
        treasuryStore.approvePaymentOrder(aboveLimit.id, 'Cố tình duyệt 1 tỷ');
      }).toThrow(/Lệnh chi vượt hạn mức 500.000.000 ₫ yêu cầu Giám đốc tài chính \(CFO\) phê duyệt!/);

      // Verify UI behavior for Checker on exactLimit
      treasuryStore.selectPaymentOrder(exactLimit.id);
      const checkerWrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });
      await checkerWrapper.vm.$nextTick();

      expect(checkerWrapper.find('.high-limit-warning-box').exists()).toBe(true);
      expect(checkerWrapper.find('.btn-approve').attributes('disabled')).toBeDefined();

      // TEST AS CFO (cfo_huong)
      authStore.login('cfo_huong', 'LivaCfo@2026');

      // CFO CAN approve exact limit (500M)
      expect(() => {
        treasuryStore.approvePaymentOrder(exactLimit.id, 'CFO phê duyệt lệnh 500M hợp lệ');
      }).not.toThrow();
      expect(exactLimit.status).toBe('APPROVED');

      // CFO CAN approve above limit (1B)
      expect(() => {
        treasuryStore.approvePaymentOrder(aboveLimit.id, 'CFO phê duyệt lệnh 1 tỷ hợp lệ');
      }).not.toThrow();
      expect(aboveLimit.status).toBe('APPROVED');

      // Verify UI behavior for CFO on aboveLimit
      treasuryStore.selectPaymentOrder(aboveLimit.id);
      const cfoWrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });
      await cfoWrapper.vm.$nextTick();

      // Since it is now APPROVED, CFO sees disbursement button
      expect(cfoWrapper.find('.btn-disburse').exists()).toBe(true);
    });

    it('strictly locks down Auditor and Admin from all state mutations', async () => {
      const authStore = useAuthStore();
      const treasuryStore = useTreasuryStore();

      // Test Auditor
      authStore.login('auditor_lan', 'LivaAudit@2026');

      expect(() => {
        treasuryStore.createPaymentOrder({
          sourceAccountId: 'acc-vcb-01',
          beneficiaryName: 'Hack',
          beneficiaryBank: 'VCB',
          beneficiaryAccount: '001',
          amount: 1000,
          memo: 'test',
          priority: 'NORMAL',
        });
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      expect(() => {
        treasuryStore.approvePaymentOrder('pay-01', 'Auditor try approve');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      expect(() => {
        treasuryStore.rejectPaymentOrder('pay-01', 'Auditor try reject');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      // pay-02 is an APPROVED order
      expect(() => {
        treasuryStore.executePaymentDisbursement('pay-02');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      expect(() => {
        treasuryStore.executeCashSweeping('acc-vcb-01', 'acc-tcb-01', 50000000);
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      // Test Admin
      authStore.login('admin_sys', 'LivaAdmin@2026');

      expect(() => {
        treasuryStore.approvePaymentOrder('pay-01', 'Admin try approve');
      }).toThrow(/Quản trị viên \(Admin\) không có thẩm quyền/);

      expect(() => {
        treasuryStore.rejectPaymentOrder('pay-01', 'Admin try reject');
      }).toThrow(/Quản trị viên \(Admin\) không có thẩm quyền/);

      expect(() => {
        treasuryStore.executePaymentDisbursement('pay-02');
      }).toThrow(/Quản trị viên \(Admin\) không có thẩm quyền/);

      expect(() => {
        treasuryStore.executeCashSweeping('acc-vcb-01', 'acc-tcb-01', 50000000);
      }).toThrow(/Quản trị viên \(Admin\) không có thẩm quyền/);
    });

    it('blocks disbursement execution on unapproved orders and insufficient balance', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');
      const treasuryStore = useTreasuryStore();

      // Attack: Attempt to disburse unapproved order
      const unapprovedOrder = treasuryStore.paymentOrders.find(
        (o) => o.status === 'SUBMITTED_TO_CHECKER'
      );
      expect(unapprovedOrder).toBeDefined();

      expect(() => {
        treasuryStore.executePaymentDisbursement(unapprovedOrder!.id);
      }).toThrow('Chỉ lệnh chi đã được phê duyệt mới được giải ngân');

      // Attack: Attempt to disburse an approved order exceeding balance
      const poorAcc = treasuryStore.accounts.find((a) => a.id === 'acc-mbb-01');
      expect(poorAcc).toBeDefined();
      poorAcc!.balance = 1000000; // Only 1M VND left

      const hugeOrder: PaymentOrder = {
        id: 'pay-poor-test',
        paymentGuid: 'guid-poor',
        orderNumber: 'UNC-POOR',
        sourceAccountId: 'acc-mbb-01',
        beneficiaryName: 'Nhà cung cấp',
        beneficiaryBank: 'VCB',
        beneficiaryAccount: '007100',
        amount: 50000000, // 50M VND
        memo: 'Thanh toán vượt số dư',
        priority: 'NORMAL',
        status: 'APPROVED',
        makerId: 'maker_nam',
        checkerId: 'checker_tri',
        submittedAt: new Date().toISOString(),
        approvedAt: new Date().toISOString(),
      };
      treasuryStore.paymentOrders.unshift(hugeOrder);

      expect(() => {
        treasuryStore.executePaymentDisbursement(hugeOrder.id);
      }).toThrow(/không đủ để chi/);
    });
  });

  // =========================================================================
  // 3. Complete Absence of Interactive Switchers
  // =========================================================================
  describe('3. Elimination of Interactive Role Switchers (F04)', () => {
    it('BankingQuarantineView has no .role-switcher-box and binds to session officer badge', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');

      const wrapper = mount(BankingQuarantineView);

      // Interactive role switcher must not exist
      expect(wrapper.find('.role-switcher-box').exists()).toBe(false);

      // Session officer badge must exist and display Maker info
      const badge = wrapper.find('.session-officer-badge');
      expect(badge.exists()).toBe(true);
      expect(badge.text()).toContain('Trịnh Văn Nam');
      expect(badge.text()).toContain('Kế toán viên (Maker)');
    });

    it('PaymentDisbursementModal has no .role-toggle-group and binds to session identity badge', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      // Interactive role toggle group must not exist
      expect(wrapper.find('.role-toggle-group').exists()).toBe(false);

      // Session identity badge must exist and display Checker info
      const badge = wrapper.find('.session-identity-badge');
      expect(badge.exists()).toBe(true);
      expect(badge.text()).toContain('CHECKER');
      expect(badge.text()).toContain('Nguyễn Minh Trí');
    });
  });

  // =========================================================================
  // 4. Adversarial Quarantine Workbench SoD & Dual Control (F06)
  // =========================================================================
  describe('4. Adversarial Quarantine Workbench SoD & Dual Control', () => {
    it('prevents Maker from approving or rejecting quarantine items in workbenchStore', () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');
      const workbenchStore = useWorkbenchStore();
      workbenchStore.seedBenchmarkData();

      expect(() => {
        workbenchStore.approveQuarantineItem('q-02', 'Maker tự duyệt hợp đồng');
      }).toThrow(/Chỉ Kế toán trưởng \(Checker\) mới có quyền phê duyệt/);

      expect(() => {
        workbenchStore.rejectQuarantineItem('q-02', 'Maker tự từ chối hợp đồng');
      }).toThrow(/Chỉ Kế toán trưởng \(Checker\) mới có quyền từ chối/);
    });

    it('enforces SoD: blocks Checker from approving or rejecting their own quarantine proposal', () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');
      const workbenchStore = useWorkbenchStore();
      workbenchStore.seedBenchmarkData();
      workbenchStore.switchOperatorRole('CHECKER');

      // Find an item and tamper its makerId to match checker
      const item = workbenchStore.quarantineItems.find((i) => i.id === 'q-02')!;
      item.makerId = workbenchStore.currentOperatorId;

      expect(() => {
        workbenchStore.approveQuarantineItem(item.id, 'Checker cố tình tự phê duyệt đề xuất của chính mình');
      }).toThrow(/Vi phạm nguyên tắc 4 mắt \(SoD\)/);

      expect(() => {
        workbenchStore.rejectQuarantineItem(item.id, 'Checker cố tình tự bác bỏ đề xuất của chính mình');
      }).toThrow(/Vi phạm nguyên tắc 4 mắt \(SoD\)/);
    });

    it('rejects checker approval note shorter than 10 characters', () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');
      const workbenchStore = useWorkbenchStore();
      workbenchStore.seedBenchmarkData();
      workbenchStore.switchOperatorRole('CHECKER');

      const item = workbenchStore.quarantineItems.find((i) => i.id === 'q-02')!;
      item.makerId = 'USR-MAKER-KETOAN-01';

      expect(() => {
        workbenchStore.approveQuarantineItem(item.id, 'Duyệt');
      }).toThrow('Lý do phê duyệt phải có độ dài tối thiểu 10 ký tự');

      expect(() => {
        workbenchStore.rejectQuarantineItem(item.id, 'Bác bỏ');
      }).toThrow('Lý do bác bỏ/từ chối phải có độ dài tối thiểu 10 ký tự');
    });

    it('Auditor is strictly locked down from approving or rejecting quarantine items', () => {
      const authStore = useAuthStore();
      authStore.login('auditor_lan', 'LivaAudit@2026');
      const workbenchStore = useWorkbenchStore();
      workbenchStore.seedBenchmarkData();

      const item = workbenchStore.quarantineItems.find((i) => i.id === 'q-02')!;

      expect(() => {
        workbenchStore.approveQuarantineItem(item.id, 'Auditor duyệt thử nghiệm');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);

      expect(() => {
        workbenchStore.rejectQuarantineItem(item.id, 'Auditor từ chối thử nghiệm');
      }).toThrow(/Kiểm toán viên ở chế độ Chỉ đọc/);
    });
  });
});
