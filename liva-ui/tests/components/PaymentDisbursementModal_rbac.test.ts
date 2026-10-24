import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import PaymentDisbursementModal from '../../src/components/banking/treasury/PaymentDisbursementModal.vue';
import { useAuthStore } from '../../src/stores/authStore';
import { useTreasuryStore, HIGH_LIMIT_THRESHOLD_VND } from '../../src/stores/treasuryStore';

describe('PaymentDisbursementModal.vue — RBAC & Circular 09 Dual Control', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
  });

  it('renders session badge and allows Maker to propose new orders only', async () => {
    const authStore = useAuthStore();
    authStore.login('maker_nam', 'LivaMaker@2026');

    const treasuryStore = useTreasuryStore();


    const wrapper = mount(PaymentDisbursementModal, {
      props: { isOpen: true },
    });

    // Session identity badge check
    const badge = wrapper.find('.session-identity-badge');
    expect(badge.exists()).toBe(true);
    expect(badge.text()).toContain('MAKER');
    expect(badge.text()).toContain('Trịnh Văn Nam');

    // Maker has + Tạo Lệnh Chi button
    const newOrderBtn = wrapper.find('.btn-new-order');
    expect(newOrderBtn.exists()).toBe(true);

    // Open create form
    await newOrderBtn.trigger('click');
    expect(wrapper.find('.create-form-pane').exists()).toBe(true);
    expect(wrapper.text()).toContain('Lập Lệnh Chi Tiền Điện Tử Mới (Maker)');

    // When an order is selected, Maker cannot approve or disburse
    const orderCard = wrapper.findAll('.order-card')[0];
    await orderCard.trigger('click');
    expect(wrapper.find('.checker-decision-box').exists()).toBe(false);
    expect(wrapper.find('.btn-disburse').exists()).toBe(false);
  });

  it('standard Checker can review orders < 500M but is blocked on high-limit (>= 500M) orders', async () => {
    const authStore = useAuthStore();
    authStore.login('checker_tri', 'LivaChecker@2026');

    const treasuryStore = useTreasuryStore();
    // Add a high-limit order (>= 500M)
    const highOrder = {
      id: 'pay-high-01',
      paymentGuid: 'guid-pay-high-01',
      orderNumber: 'UNC-VCB-HIGH-01',
      sourceAccountId: 'acc-vcb-01',
      beneficiaryName: 'Công ty Cổ phần Thiết Bị Siêu Trường',
      beneficiaryBank: 'BIDV',
      beneficiaryAccount: '12010008888',
      amount: 600000000,
      memo: 'Thanh toán máy móc',
      priority: 'NORMAL' as const,
      status: 'SUBMITTED_TO_CHECKER' as const,
      makerId: 'maker_nam',
      makerNote: 'Hợp đồng mua bán máy',
      submittedAt: new Date().toISOString(),
    };
    treasuryStore.paymentOrders.push(highOrder);

    const standardOrder = treasuryStore.paymentOrders.find(
      o => o.amount < HIGH_LIMIT_THRESHOLD_VND && o.status === 'SUBMITTED_TO_CHECKER'
    );
    expect(standardOrder).toBeDefined();

    const wrapper = mount(PaymentDisbursementModal, {
      props: { isOpen: true },
    });

    // Checker cannot see + Tạo Lệnh Chi
    expect(wrapper.find('.btn-new-order').exists()).toBe(false);

    // Select standard order (< 500M)
    treasuryStore.selectPaymentOrder(standardOrder!.id);
    await wrapper.vm.$nextTick();

    const decisionBox = wrapper.find('.checker-decision-box');
    expect(decisionBox.exists()).toBe(true);
    const approveBtn = wrapper.find('.btn-approve');
    expect(approveBtn.attributes('disabled')).toBeUndefined();

    // Select high-limit order (>= 500M)
    treasuryStore.selectPaymentOrder(highOrder.id);
    await wrapper.vm.$nextTick();

    expect(wrapper.find('.high-limit-warning-box').exists()).toBe(true);
    expect(wrapper.text()).toContain('Vượt hạn mức phê duyệt của Kế toán trưởng');
    const highApproveBtn = wrapper.find('.btn-approve');
    expect(highApproveBtn.attributes('disabled')).toBeDefined();
  });

  it('CFO can approve high-limit orders (>= 500M) and disburse approved orders', async () => {
    const authStore = useAuthStore();
    authStore.login('cfo_hoang', 'LivaCfo@2026');

    const treasuryStore = useTreasuryStore();
    const highOrder = {
      id: 'pay-high-cfo',
      paymentGuid: 'guid-pay-high-cfo',
      orderNumber: 'UNC-VCB-HIGH-CFO',
      sourceAccountId: 'acc-vcb-01',
      beneficiaryName: 'Công ty Cổ phần Thiết Bị Siêu Trường',
      beneficiaryBank: 'BIDV',
      beneficiaryAccount: '12010008888',
      amount: 750000000,
      memo: 'Thanh toán máy móc hạng nặng',
      priority: 'NORMAL' as const,
      status: 'SUBMITTED_TO_CHECKER' as const,
      makerId: 'maker_nam',
      makerNote: 'Hợp đồng mua bán máy CNC',
      submittedAt: new Date().toISOString(),
    };
    treasuryStore.paymentOrders.push(highOrder);

    const wrapper = mount(PaymentDisbursementModal, {
      props: { isOpen: true },
    });

    treasuryStore.selectPaymentOrder(highOrder.id);
    await wrapper.vm.$nextTick();

    // CFO should be able to approve high limit order
    const approveBtn = wrapper.find('.btn-approve');
    expect(approveBtn.attributes('disabled')).toBeUndefined();

    // Select approved order and check disbursement capability
    const approvedOrder = treasuryStore.paymentOrders.find(o => o.status === 'APPROVED');
    expect(approvedOrder).toBeDefined();

    treasuryStore.selectPaymentOrder(approvedOrder!.id);
    await wrapper.vm.$nextTick();

    const disburseBtn = wrapper.find('.btn-disburse');
    expect(disburseBtn.exists()).toBe(true);
  });

  it('enforces Segregation of Duties (SoD): blocks self-approval when Checker is the Maker', async () => {
    const authStore = useAuthStore();
    authStore.login('checker_tri', 'LivaChecker@2026');

    const treasuryStore = useTreasuryStore();

    // Set makerId of first order to match logged in user (checker_tri)
    const targetOrder = treasuryStore.paymentOrders[0];
    targetOrder.makerId = 'checker_tri';
    targetOrder.status = 'SUBMITTED_TO_CHECKER';

    const wrapper = mount(PaymentDisbursementModal, {
      props: { isOpen: true },
    });

    treasuryStore.selectPaymentOrder(targetOrder.id);
    await wrapper.vm.$nextTick();

    // SoD warning box must appear
    expect(wrapper.find('.sod-warning-box').exists()).toBe(true);
    expect(wrapper.text()).toContain('Vi phạm nguyên tắc 4 mắt');

    // Both approve and reject buttons must be disabled
    expect(wrapper.find('.btn-approve').attributes('disabled')).toBeDefined();
    expect(wrapper.find('.btn-reject').attributes('disabled')).toBeDefined();
  });

  it('Auditor views modal in pure read-only mode without mutation controls', async () => {
    const authStore = useAuthStore();
    authStore.login('auditor_lan', 'LivaAudit@2026');

    const treasuryStore = useTreasuryStore();

    const wrapper = mount(PaymentDisbursementModal, {
      props: { isOpen: true },
    });

    // Session badge contains [CHỈ ĐỌC]
    expect(wrapper.find('.session-readonly-tag').text()).toBe('[CHỈ ĐỌC]');

    // Auditor notice box is visible
    expect(wrapper.find('.auditor-notice-box').exists()).toBe(true);
    expect(wrapper.text()).toContain('Hồ sơ chi quỹ kiểm toán — không thể tạo mới, phê duyệt hoặc giải ngân');

    // No create button
    expect(wrapper.find('.btn-new-order').exists()).toBe(false);

    // Select an order
    treasuryStore.selectPaymentOrder(treasuryStore.paymentOrders[0].id);
    await wrapper.vm.$nextTick();

    // No decision box or disbursement button
    expect(wrapper.find('.checker-decision-box').exists()).toBe(false);
    expect(wrapper.find('.btn-disburse').exists()).toBe(false);
  });
});
