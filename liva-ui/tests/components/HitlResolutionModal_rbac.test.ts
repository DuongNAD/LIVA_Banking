import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import HitlResolutionModal from '../../src/components/banking/HitlResolutionModal.vue';
import { useAuthStore } from '../../src/stores/authStore';
import { useReconciliationStore, type BankTransaction } from '../../src/stores/reconciliationStore';

describe('HitlResolutionModal.vue — RBAC & Circular 09 Dual Control (F05)', () => {
  const mockTx: BankTransaction = {
    id: 'tx-hitl-01',
    txCode: 'VCB2026103001',
    time: '10:15 AM',
    date: '2026-10-30',
    bankCode: 'VCB',
    accountNumber: 'VCB-00710009821',
    memo: 'LECH PHI GIAO DICH CHUYEN KHOAN 12000 VND',
    bankAmount: 12000,
    ledgerAmount: 0,
    variance: 12000,
    status: 'PENDING_HITL',
    statusLabel: 'Chờ duyệt',
    confidenceScore: 0.95,
    tokenUuid: 'tok-single-use-hitl-01',
  };

  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
  });

  it('Maker session has locked readonly identity inputs and Phase 1 proposal action only', async () => {
    const authStore = useAuthStore();
    authStore.login('maker_nam', 'LivaMaker@2026');

    const wrapper = mount(HitlResolutionModal, {
      props: { transaction: mockTx, isOpen: true },
    });

    // Verify inputs have readonly attribute and readonly-input class
    const inputs = wrapper.findAll('.role-input');
    expect(inputs).toHaveLength(2);
    expect(inputs[0].attributes('readonly')).toBeDefined();
    expect(inputs[1].attributes('readonly')).toBeDefined();
    expect(inputs[0].classes()).toContain('readonly-input');
    expect(inputs[1].classes()).toContain('readonly-input');

    // Maker sees Phase 1 proposal action
    const proposeBtn = wrapper.find('.btn-propose');
    expect(proposeBtn.exists()).toBe(true);
    expect(proposeBtn.text()).toContain('Lập Đề Xuất Đối Soát (Phase 1)');

    // Checker actions must not exist
    expect(wrapper.find('.btn-reject').exists()).toBe(false);
    expect(wrapper.find('.btn-approve').exists()).toBe(false);
  });

  it('Checker session has Phase 2 approval & rejection actions, and enforces SoD self-approval block', async () => {
    const authStore = useAuthStore();
    authStore.login('checker_tri', 'LivaChecker@2026');

    const wrapper = mount(HitlResolutionModal, {
      props: { transaction: mockTx, isOpen: true },
    });

    // Checker sees Phase 2 approval & rejection
    const approveBtn = wrapper.find('.btn-approve');
    const rejectBtn = wrapper.find('.btn-reject');
    expect(approveBtn.exists()).toBe(true);
    expect(rejectBtn.exists()).toBe(true);
    expect(approveBtn.text()).toContain('Xác Nhận Phê Duyệt (Ký Số Phase 2)');
    expect(rejectBtn.text()).toContain('Bác Bỏ Đề Xuất (Reject)');

    // Maker proposal button must not exist
    expect(wrapper.find('.btn-propose').exists()).toBe(false);

    // Initial distinct identities: no violation
    expect(wrapper.find('.violation-alert').exists()).toBe(false);
    expect(approveBtn.attributes('disabled')).toBeUndefined();
    expect(rejectBtn.attributes('disabled')).toBeUndefined();

    // Trigger SoD self-approval violation by setting same identity
    const inputs = wrapper.findAll('.role-input');
    await inputs[0].setValue('SAME_USER_IDENTITY');
    await inputs[1].setValue('SAME_USER_IDENTITY');

    // SoD violation banner must appear
    expect(wrapper.find('.violation-alert').exists()).toBe(true);
    expect(wrapper.text()).toContain('CẢNH BÁO VI PHẠM THÔNG TƯ 09/2020/TT-NHNN');

    // Both approve and reject must be disabled
    expect(approveBtn.attributes('disabled')).toBeDefined();
    expect(rejectBtn.attributes('disabled')).toBeDefined();
  });

  it('Auditor session displays read-only banner, notice pill, and no mutation buttons', async () => {
    const authStore = useAuthStore();
    authStore.login('auditor_lan', 'LivaAudit@2026');

    const wrapper = mount(HitlResolutionModal, {
      props: { transaction: mockTx, isOpen: true },
    });

    // Read-only banner and pill
    expect(wrapper.find('.auditor-banner').exists()).toBe(true);
    expect(wrapper.find('.auditor-pill').exists()).toBe(true);
    expect(wrapper.find('.auditor-pill').text()).toBe('[HỒ SƠ KIỂM TOÁN CHỈ ĐỌC]');

    // No mutation buttons (propose, approve, reject)
    expect(wrapper.find('.btn-propose').exists()).toBe(false);
    expect(wrapper.find('.btn-approve').exists()).toBe(false);
    expect(wrapper.find('.btn-reject').exists()).toBe(false);

    // Only close button exists
    const closeBtn = wrapper.find('.btn-secondary');
    expect(closeBtn.exists()).toBe(true);
    expect(closeBtn.text()).toBe('Đóng');
  });
});
