import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import { useAuthStore } from '../src/stores/authStore';
import { useTreasuryStore, HIGH_LIMIT_THRESHOLD_VND } from '../src/stores/treasuryStore';
import { useLedgerStore } from '../src/stores/ledgerStore';
import { useReconciliationStore } from '../src/stores/reconciliationStore';

import JournalVoucherTable from '../src/components/banking/ledger/JournalVoucherTable.vue';
import CashConcentrationGrid from '../src/components/banking/treasury/CashConcentrationGrid.vue';
import PaymentDisbursementModal from '../src/components/banking/treasury/PaymentDisbursementModal.vue';
import BankingHeader from '../src/components/banking/BankingHeader.vue';
import TransactionLedger from '../src/components/banking/TransactionLedger.vue';

describe('Milestone 2 Challenger: Role Workspaces & Action Invariants', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined') {
      window.location.hash = '';
      if (window.localStorage) {
        window.localStorage.clear();
      }
    }
  });

  // =========================================================================
  // 1. MAKER WORKSPACE INVARIANTS
  // =========================================================================
  describe('Dimension 1: Maker Workspace Invariants (Propose-Only Boundary)', () => {
    it('Maker cannot post to ERP in JournalVoucherTable (.btn-batch-post and .btn-mini-post completely absent)', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');
      expect(authStore.isMaker).toBe(true);
      expect(authStore.isChecker).toBe(false);
      expect(authStore.isCfo).toBe(false);

      const ledgerStore = useLedgerStore();
      // Ensure there are vouchers in pending/draft status
      expect(ledgerStore.filteredVouchers.length).toBeGreaterThan(0);
      const pendingVouchers = ledgerStore.filteredVouchers.filter(v => v.status !== 'POSTED_TO_ERP');
      expect(pendingVouchers.length).toBeGreaterThan(0);

      const wrapper = mount(JournalVoucherTable);

      // Invariant: Batch Post button (.btn-batch-post) MUST NOT exist for Maker
      const batchPostBtn = wrapper.find('.btn-batch-post');
      expect(batchPostBtn.exists(), 'VIOLATION: Maker can see .btn-batch-post in JournalVoucherTable!').toBe(false);

      // Invariant: Single Post button (.btn-mini-post) MUST NOT exist for Maker
      const miniPostBtns = wrapper.findAll('.btn-mini-post');
      expect(miniPostBtns.length, 'VIOLATION: Maker can see .btn-mini-post buttons in JournalVoucherTable!').toBe(0);

      // Verification of table rendering integrity
      expect(wrapper.find('.voucher-table').exists()).toBe(true);
      expect(wrapper.findAll('.voucher-row').length).toBeGreaterThan(0);
    });

    it('Maker cannot sweep funds in CashConcentrationGrid (.btn-sweep-action completely absent)', async () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');
      expect(authStore.isMaker).toBe(true);

      const treasuryStore = useTreasuryStore();
      // Confirm there are sweeping recommendations that would otherwise trigger action buttons
      expect(treasuryStore.sweepingRecommendations.length).toBeGreaterThan(0);

      const wrapper = mount(CashConcentrationGrid);

      // Invariant: Sweeping action button (.btn-sweep-action) MUST NOT exist for Maker
      const sweepActionBtns = wrapper.findAll('.btn-sweep-action');
      expect(sweepActionBtns.length, 'VIOLATION: Maker can see .btn-sweep-action in CashConcentrationGrid!').toBe(0);

      // Alert box should still inform of the condition
      expect(wrapper.find('.sweeping-alert-box').exists()).toBe(true);
    });

    it('Adversarial Store Attack: Maker attempting to execute cash sweeping directly throws RBAC Error', () => {
      const authStore = useAuthStore();
      authStore.login('maker_nam', 'LivaMaker@2026');
      const treasuryStore = useTreasuryStore();

      expect(() => {
        treasuryStore.executeCashSweeping('acc-tcb-01', 'acc-vcb-01', 50000000);
      }).toThrowError(/Kế toán viên \(Maker\) không được phép thực hiện điều chuyển vốn trực tiếp/);
    });
  });

  // =========================================================================
  // 2. CHECKER WORKSPACE INVARIANTS
  // =========================================================================
  describe('Dimension 2: Checker Workspace Invariants (Approval-Only Boundary & SoD)', () => {
    it('Checker cannot create payment orders in PaymentDisbursementModal (.btn-new-order completely absent)', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');
      expect(authStore.isChecker).toBe(true);
      expect(authStore.isMaker).toBe(false);

      const treasuryStore = useTreasuryStore();
      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      // Invariant: Order creation button (.btn-new-order) MUST NOT exist for Checker
      const newOrderBtn = wrapper.find('.btn-new-order');
      expect(newOrderBtn.exists(), 'VIOLATION: Checker can see .btn-new-order in PaymentDisbursementModal!').toBe(false);

      // Invariant: Create form pane must not be rendered
      expect(wrapper.find('.create-form-pane').exists()).toBe(false);
    });

    it('Adversarial Store Attack: Checker attempting to create payment order directly throws RBAC Error', () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');
      const treasuryStore = useTreasuryStore();

      expect(() => {
        treasuryStore.createPaymentOrder({
          sourceAccountId: 'acc-vcb-01',
          beneficiaryName: 'Hacker Beneficiary',
          beneficiaryBank: 'VCB',
          beneficiaryAccount: '9999999999',
          amount: 50000000,
          memo: 'Exploit attempt',
          priority: 'NORMAL',
        });
      }).toThrowError(/Chỉ người có vai trò Thủ quỹ \/ Thanh toán \(Maker\) mới được lập lệnh chi tiền/);
    });

    it('Checker approval decision enabled for standard orders (< 500M VND) created by Maker', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const treasuryStore = useTreasuryStore();
      const standardOrder = treasuryStore.paymentOrders.find(
        (o) => o.amount < HIGH_LIMIT_THRESHOLD_VND && o.status === 'SUBMITTED_TO_CHECKER' && o.makerId !== 'checker_tri'
      );
      expect(standardOrder).toBeDefined();

      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      treasuryStore.selectPaymentOrder(standardOrder!.id);
      await wrapper.vm.$nextTick();

      const decisionBox = wrapper.find('.checker-decision-box');
      expect(decisionBox.exists()).toBe(true);

      const approveBtn = wrapper.find('.btn-approve');
      expect(approveBtn.exists()).toBe(true);
      expect(approveBtn.attributes('disabled'), 'Standard order (<500M) should be approvable by Checker').toBeUndefined();

      const rejectBtn = wrapper.find('.btn-reject');
      expect(rejectBtn.exists()).toBe(true);
      expect(rejectBtn.attributes('disabled'), 'Standard order (<500M) should be rejectable by Checker').toBeUndefined();
    });

    it('Circular 09/2020 SoD Invariant: Checker approval & rejection BLOCKED on self-created orders', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const treasuryStore = useTreasuryStore();
      // Plant a self-order where makerId matches checker_tri
      const selfOrder = {
        id: 'pay-self-01',
        paymentGuid: 'guid-pay-self-01',
        orderNumber: 'UNC-SELF-001',
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Self-Dealing Vendor Co',
        beneficiaryBank: 'VCB',
        beneficiaryAccount: '00710009999',
        amount: 80000000,
        memo: 'Self dealing test order',
        priority: 'NORMAL' as const,
        status: 'SUBMITTED_TO_CHECKER' as const,
        makerId: 'checker_tri',
        makerNote: 'Attempted self-approval',
        submittedAt: new Date().toISOString(),
      };
      treasuryStore.paymentOrders.unshift(selfOrder);

      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      treasuryStore.selectPaymentOrder(selfOrder.id);
      await wrapper.vm.$nextTick();

      // Invariant: SoD warning box MUST be displayed
      const sodWarning = wrapper.find('.sod-warning-box');
      expect(sodWarning.exists(), 'VIOLATION: SoD warning box missing on self-order!').toBe(true);
      expect(sodWarning.text()).toContain('Vi phạm nguyên tắc 4 mắt');

      // Invariant: Approval button MUST be disabled
      const approveBtn = wrapper.find('.btn-approve');
      expect(approveBtn.attributes('disabled'), 'VIOLATION: Self-approval button is not disabled!').toBeDefined();

      // Invariant: Rejection button MUST be disabled
      const rejectBtn = wrapper.find('.btn-reject');
      expect(rejectBtn.attributes('disabled'), 'VIOLATION: Self-rejection button is not disabled!').toBeDefined();

      // Adversarial Store Attack: Direct store call must throw SoD error
      expect(() => {
        treasuryStore.approvePaymentOrder(selfOrder.id, 'Self approve attempt');
      }).toThrowError(/Vi phạm nguyên tắc 4 mắt \(SoD\)/);

      expect(() => {
        treasuryStore.rejectPaymentOrder(selfOrder.id, 'Self reject attempt');
      }).toThrowError(/Vi phạm nguyên tắc 4 mắt \(SoD\)/);
    });

    it('Checker blocked on high-limit (>= 500M VND) orders; CFO escalation permitted', async () => {
      const authStore = useAuthStore();
      authStore.login('checker_tri', 'LivaChecker@2026');

      const treasuryStore = useTreasuryStore();
      const highLimitOrder = {
        id: 'pay-high-limit-01',
        paymentGuid: 'guid-pay-high-01',
        orderNumber: 'UNC-HIGH-001',
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Major Infrastructure Corp',
        beneficiaryBank: 'BIDV',
        beneficiaryAccount: '12010009999',
        amount: 850000000, // 850M VND >= 500M VND
        memo: 'Thanh toán trạm biến áp',
        priority: 'NORMAL' as const,
        status: 'SUBMITTED_TO_CHECKER' as const,
        makerId: 'maker_nam',
        makerNote: 'Hợp đồng dự án 850M',
        submittedAt: new Date().toISOString(),
      };
      treasuryStore.paymentOrders.unshift(highLimitOrder);

      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      treasuryStore.selectPaymentOrder(highLimitOrder.id);
      await wrapper.vm.$nextTick();

      // Checker: High limit warning box appears and approve button is disabled
      expect(wrapper.find('.high-limit-warning-box').exists()).toBe(true);
      expect(wrapper.find('.btn-approve').attributes('disabled')).toBeDefined();

      // Store-level check
      expect(() => {
        treasuryStore.approvePaymentOrder(highLimitOrder.id, 'Checker approving high limit');
      }).toThrowError(/yêu cầu Giám đốc tài chính \(CFO\) phê duyệt/);

      // ESCALATION TEST: Now switch to CFO
      authStore.login('cfo_huong', 'LivaCfo@2026');
      await wrapper.vm.$nextTick();

      // CFO can approve
      expect(wrapper.find('.high-limit-warning-box').exists()).toBe(false);
      expect(wrapper.find('.btn-approve').attributes('disabled')).toBeUndefined();
    });
  });

  // =========================================================================
  // 3. AUDITOR WORKSPACE INVARIANTS
  // =========================================================================
  describe('Dimension 3: Auditor Workspace Invariants (Pure Read-Only Boundary)', () => {
    it('Auditor renders [CHỈ ĐỌC] badge in BankingHeader.vue', async () => {
      const authStore = useAuthStore();
      authStore.login('auditor_lan', 'LivaAudit@2026');
      expect(authStore.isAuditor).toBe(true);
      expect(authStore.isReadOnly).toBe(true);

      const wrapper = mount(BankingHeader);

      const readonlyPill = wrapper.find('.readonly-pill');
      expect(readonlyPill.exists(), 'VIOLATION: Auditor header missing [CHỈ ĐỌC] badge!').toBe(true);
      expect(readonlyPill.text()).toBe('[CHỈ ĐỌC]');

      // Verify that non-auditor roles DO NOT show [CHỈ ĐỌC]
      authStore.login('maker_nam', 'LivaMaker@2026');
      await wrapper.vm.$nextTick();
      expect(wrapper.find('.readonly-pill').exists()).toBe(false);

      authStore.login('checker_tri', 'LivaChecker@2026');
      await wrapper.vm.$nextTick();
      expect(wrapper.find('.readonly-pill').exists()).toBe(false);

      authStore.login('cfo_huong', 'LivaCfo@2026');
      await wrapper.vm.$nextTick();
      expect(wrapper.find('.readonly-pill').exists()).toBe(false);
    });

    it('Auditor in TransactionLedger.vue: file upload dropzone, paste button, reset button, and reconcile button are absent', async () => {
      const authStore = useAuthStore();
      authStore.login('auditor_lan', 'LivaAudit@2026');

      const reconcileStore = useReconciliationStore();
      reconcileStore.seedBenchmarkData();

      const wrapper = mount(TransactionLedger);

      // Invariant: .statement-dropzone MUST NOT exist for Auditor
      expect(wrapper.find('.statement-dropzone').exists(), 'VIOLATION: Auditor can see statement dropzone!').toBe(false);

      // Invariant: .paste-btn MUST NOT exist for Auditor
      expect(wrapper.find('.paste-btn').exists(), 'VIOLATION: Auditor can see paste button!').toBe(false);

      // Invariant: .reset-btn MUST NOT exist for Auditor
      expect(wrapper.find('.reset-btn').exists(), 'VIOLATION: Auditor can see reset button!').toBe(false);

      // Invariant: .run-reconcile-btn MUST NOT exist for Auditor
      expect(wrapper.find('.run-reconcile-btn').exists(), 'VIOLATION: Auditor can see run reconcile button!').toBe(false);

      // But Auditor CAN see the transactions table
      expect(wrapper.find('.transaction-table').exists()).toBe(true);
      expect(wrapper.findAll('.tx-row').length).toBeGreaterThan(0);
    });

    it('Auditor in JournalVoucherTable.vue: all posting buttons (.btn-batch-post, .btn-mini-post) are completely absent', async () => {
      const authStore = useAuthStore();
      authStore.login('auditor_lan', 'LivaAudit@2026');

      const wrapper = mount(JournalVoucherTable);

      expect(wrapper.find('.btn-batch-post').exists(), 'VIOLATION: Auditor can see batch post button!').toBe(false);
      expect(wrapper.findAll('.btn-mini-post').length, 'VIOLATION: Auditor can see single post button!').toBe(0);
    });

    it('Auditor in CashConcentrationGrid.vue: sweep button is completely absent', async () => {
      const authStore = useAuthStore();
      authStore.login('auditor_lan', 'LivaAudit@2026');

      const wrapper = mount(CashConcentrationGrid);

      expect(wrapper.findAll('.btn-sweep-action').length, 'VIOLATION: Auditor can see sweep action button!').toBe(0);
    });

    it('Auditor in PaymentDisbursementModal.vue: displays read-only banner, hides create/approve/disburse buttons', async () => {
      const authStore = useAuthStore();
      authStore.login('auditor_lan', 'LivaAudit@2026');

      const treasuryStore = useTreasuryStore();
      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      // Session tag has [CHỈ ĐỌC]
      expect(wrapper.find('.session-readonly-tag').text()).toBe('[CHỈ ĐỌC]');

      // Auditor notice box is displayed
      expect(wrapper.find('.auditor-notice-box').exists()).toBe(true);
      expect(wrapper.text()).toContain('Hồ sơ chi quỹ kiểm toán — không thể tạo mới, phê duyệt hoặc giải ngân');

      // Creation button absent
      expect(wrapper.find('.btn-new-order').exists()).toBe(false);

      // Select an order
      treasuryStore.selectPaymentOrder(treasuryStore.paymentOrders[0].id);
      await wrapper.vm.$nextTick();

      // Decision box and disbursement button absent
      expect(wrapper.find('.checker-decision-box').exists()).toBe(false);
      expect(wrapper.find('.btn-disburse').exists()).toBe(false);
    });

    it('Adversarial Store Attack: Auditor attempting financial mutations is blocked across stores', () => {
      const authStore = useAuthStore();
      authStore.login('auditor_lan', 'LivaAudit@2026');
      const treasuryStore = useTreasuryStore();

      expect(() => {
        treasuryStore.createPaymentOrder({
          sourceAccountId: 'acc-vcb-01',
          beneficiaryName: 'Vendor',
          beneficiaryBank: 'VCB',
          beneficiaryAccount: '123',
          amount: 1000000,
          memo: 'test',
          priority: 'NORMAL',
        });
      }).toThrowError(/Kiểm toán viên ở chế độ Chỉ đọc, không được lập lệnh chi/);

      expect(() => {
        treasuryStore.approvePaymentOrder('pay-01', 'audit approve');
      }).toThrowError(/Kiểm toán viên ở chế độ Chỉ đọc, không có quyền duyệt lệnh chi/);

      expect(() => {
        treasuryStore.rejectPaymentOrder('pay-01', 'audit reject');
      }).toThrowError(/Kiểm toán viên ở chế độ Chỉ đọc, không có quyền từ chối lệnh chi/);

      expect(() => {
        treasuryStore.executePaymentDisbursement('pay-02');
      }).toThrowError(/Kiểm toán viên ở chế độ Chỉ đọc, không được phép phát lệnh giải ngân/);

      expect(() => {
        treasuryStore.executeCashSweeping('acc-tcb-01', 'acc-vcb-01', 10000000);
      }).toThrowError(/Kiểm toán viên ở chế độ Chỉ đọc, không được phép thực hiện quét vốn/);
    });
  });

  // =========================================================================
  // 4. ADMIN WORKSPACE INVARIANTS
  // =========================================================================
  describe('Dimension 4: Admin Workspace Invariants (Zero Financial Execution)', () => {
    it('Admin has no access to financial transaction mutations in TransactionLedger.vue', async () => {
      const authStore = useAuthStore();
      authStore.login('admin_sys', 'LivaAdmin@2026');
      expect(authStore.isAdmin).toBe(true);

      const reconcileStore = useReconciliationStore();
      reconcileStore.seedBenchmarkData();

      const wrapper = mount(TransactionLedger);

      // Invariant: Ingestion dropzone absent for Admin
      expect(wrapper.find('.statement-dropzone').exists(), 'VIOLATION: Admin can see statement dropzone!').toBe(false);

      // Invariant: Paste button absent for Admin
      expect(wrapper.find('.paste-btn').exists(), 'VIOLATION: Admin can see paste button!').toBe(false);

      // Invariant: Reset button absent for Admin
      expect(wrapper.find('.reset-btn').exists(), 'VIOLATION: Admin can see reset button!').toBe(false);

      // Invariant: Reconcile run button absent for Admin
      expect(wrapper.find('.run-reconcile-btn').exists(), 'VIOLATION: Admin can see reconcile button!').toBe(false);
    });

    it('Admin has no financial posting or sweeping actions in JournalVoucherTable and CashConcentrationGrid', async () => {
      const authStore = useAuthStore();
      authStore.login('admin_sys', 'LivaAdmin@2026');

      const wrapperJournal = mount(JournalVoucherTable);
      expect(wrapperJournal.find('.btn-batch-post').exists(), 'VIOLATION: Admin can see .btn-batch-post!').toBe(false);
      expect(wrapperJournal.findAll('.btn-mini-post').length, 'VIOLATION: Admin can see .btn-mini-post!').toBe(0);

      const wrapperGrid = mount(CashConcentrationGrid);
      expect(wrapperGrid.findAll('.btn-sweep-action').length, 'VIOLATION: Admin can see .btn-sweep-action!').toBe(0);
    });

    it('Admin has no access to payment order creation, approval, or execution in PaymentDisbursementModal', async () => {
      const authStore = useAuthStore();
      authStore.login('admin_sys', 'LivaAdmin@2026');

      const treasuryStore = useTreasuryStore();
      const wrapper = mount(PaymentDisbursementModal, {
        props: { isOpen: true },
      });

      expect(wrapper.find('.btn-new-order').exists(), 'VIOLATION: Admin can see .btn-new-order!').toBe(false);

      treasuryStore.selectPaymentOrder(treasuryStore.paymentOrders[0].id);
      await wrapper.vm.$nextTick();

      expect(wrapper.find('.checker-decision-box').exists(), 'VIOLATION: Admin can see checker-decision-box!').toBe(false);
      expect(wrapper.find('.btn-disburse').exists(), 'VIOLATION: Admin can see .btn-disburse!').toBe(false);
    });

    it('Adversarial Store Attack: Admin direct mutation attempts are rejected at the store level', () => {
      const authStore = useAuthStore();
      authStore.login('admin_sys', 'LivaAdmin@2026');
      const treasuryStore = useTreasuryStore();

      expect(() => {
        treasuryStore.approvePaymentOrder('pay-01', 'Admin bypass attempt');
      }).toThrowError(/Quản trị viên \(Admin\) không có thẩm quyền duyệt lệnh tài chính/);

      expect(() => {
        treasuryStore.rejectPaymentOrder('pay-01', 'Admin reject attempt');
      }).toThrowError(/Quản trị viên \(Admin\) không có thẩm quyền thao tác lệnh tài chính/);

      expect(() => {
        treasuryStore.executePaymentDisbursement('pay-02');
      }).toThrowError(/Quản trị viên \(Admin\) không có thẩm quyền giải ngân tài chính/);

      expect(() => {
        treasuryStore.executeCashSweeping('acc-tcb-01', 'acc-vcb-01', 50000000);
      }).toThrowError(/Quản trị viên \(Admin\) không có thẩm quyền điều chuyển vốn/);
    });
  });

  // =========================================================================
  // 5. FIVE-ROLE COMPARATIVE MATRIX & REACTIVE SESSION INTEGRITY
  // =========================================================================
  describe('Dimension 5: Complete 5-Role Comparative Action Matrix Verification', () => {
    it('verifies exact ERP posting action permissions across all 5 roles in JournalVoucherTable', () => {
      const authStore = useAuthStore();
      const roles = [
        { role: 'MAKER', user: 'maker_nam', pass: 'LivaMaker@2026', canPost: false },
        { role: 'CHECKER', user: 'checker_tri', pass: 'LivaChecker@2026', canPost: true },
        { role: 'CFO', user: 'cfo_huong', pass: 'LivaCfo@2026', canPost: true },
        { role: 'AUDITOR', user: 'auditor_lan', pass: 'LivaAudit@2026', canPost: false },
        { role: 'ADMIN', user: 'admin_sys', pass: 'LivaAdmin@2026', canPost: false },
      ];

      for (const r of roles) {
        authStore.login(r.user, r.pass);
        const wrapper = mount(JournalVoucherTable);

        const hasBatch = wrapper.find('.btn-batch-post').exists();
        const hasMini = wrapper.findAll('.btn-mini-post').length > 0;

        expect(
          hasBatch,
          `Role ${r.role}: Expected canPost=${r.canPost} but got hasBatch=${hasBatch}`
        ).toBe(r.canPost);

        expect(
          hasMini,
          `Role ${r.role}: Expected canPost=${r.canPost} but got hasMini=${hasMini}`
        ).toBe(r.canPost);
      }
    });

    it('verifies exact Cash Sweeping action permissions across all 5 roles in CashConcentrationGrid', () => {
      const authStore = useAuthStore();
      const roles = [
        { role: 'MAKER', user: 'maker_nam', pass: 'LivaMaker@2026', canSweep: false },
        { role: 'CHECKER', user: 'checker_tri', pass: 'LivaChecker@2026', canSweep: true },
        { role: 'CFO', user: 'cfo_huong', pass: 'LivaCfo@2026', canSweep: true },
        { role: 'AUDITOR', user: 'auditor_lan', pass: 'LivaAudit@2026', canSweep: false },
        { role: 'ADMIN', user: 'admin_sys', pass: 'LivaAdmin@2026', canSweep: false },
      ];

      for (const r of roles) {
        authStore.login(r.user, r.pass);
        const wrapper = mount(CashConcentrationGrid);

        const hasSweep = wrapper.findAll('.btn-sweep-action').length > 0;
        expect(
          hasSweep,
          `Role ${r.role}: Expected canSweep=${r.canSweep} but got hasSweep=${hasSweep}`
        ).toBe(r.canSweep);
      }
    });

    it('verifies exact Transaction Ingestion permissions across all 5 roles in TransactionLedger', () => {
      const authStore = useAuthStore();
      const reconcileStore = useReconciliationStore();
      reconcileStore.seedBenchmarkData();

      const roles = [
        { role: 'MAKER', user: 'maker_nam', pass: 'LivaMaker@2026', canIngest: true },
        { role: 'CHECKER', user: 'checker_tri', pass: 'LivaChecker@2026', canIngest: true },
        { role: 'CFO', user: 'cfo_huong', pass: 'LivaCfo@2026', canIngest: true },
        { role: 'AUDITOR', user: 'auditor_lan', pass: 'LivaAudit@2026', canIngest: false },
        { role: 'ADMIN', user: 'admin_sys', pass: 'LivaAdmin@2026', canIngest: false },
      ];

      for (const r of roles) {
        authStore.login(r.user, r.pass);
        const wrapper = mount(TransactionLedger);

        const hasDropzone = wrapper.find('.statement-dropzone').exists();
        const hasPaste = wrapper.find('.paste-btn').exists();
        const hasReset = wrapper.find('.reset-btn').exists();
        const hasReconcile = wrapper.find('.run-reconcile-btn').exists();

        expect(
          hasDropzone,
          `Role ${r.role}: Expected canIngest=${r.canIngest} but got hasDropzone=${hasDropzone}`
        ).toBe(r.canIngest);

        expect(
          hasPaste,
          `Role ${r.role}: Expected canIngest=${r.canIngest} but got hasPaste=${hasPaste}`
        ).toBe(r.canIngest);

        expect(
          hasReset,
          `Role ${r.role}: Expected canIngest=${r.canIngest} but got hasReset=${hasReset}`
        ).toBe(r.canIngest);

        expect(
          hasReconcile,
          `Role ${r.role}: Expected canIngest=${r.canIngest} but got hasReconcile=${hasReconcile}`
        ).toBe(r.canIngest);
      }
    });

    it('verifies immediate reactive UI boundary update upon live session switch', async () => {
      const authStore = useAuthStore();
      // Start as Checker: can post ERP
      authStore.login('checker_tri', 'LivaChecker@2026');

      const wrapperJournal = mount(JournalVoucherTable);
      expect(wrapperJournal.find('.btn-batch-post').exists()).toBe(true);

      // Live switch to Maker: batch post button MUST instantly disappear
      authStore.switchUser('maker_nam');
      await wrapperJournal.vm.$nextTick();
      expect(wrapperJournal.find('.btn-batch-post').exists()).toBe(false);

      // Live switch to Auditor: stays absent
      authStore.switchUser('auditor_lan');
      await wrapperJournal.vm.$nextTick();
      expect(wrapperJournal.find('.btn-batch-post').exists()).toBe(false);

      // Live switch to CFO: reappears
      authStore.switchUser('cfo_huong');
      await wrapperJournal.vm.$nextTick();
      expect(wrapperJournal.find('.btn-batch-post').exists()).toBe(true);
    });
  });
});
