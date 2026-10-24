import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import BankingQuarantineView from '../../src/views/banking/BankingQuarantineView.vue';
import { useAuthStore } from '../../src/stores/authStore';
import { useWorkbenchStore } from '../../src/stores/workbenchStore';

describe('BankingQuarantineView.vue — RBAC & Static Session Officer Badge (F04, F06, F07)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
  });

  it('verifies interactive role switcher box is removed and static session badge displays Maker', async () => {
    const authStore = useAuthStore();
    authStore.login('maker_nam', 'LivaMaker@2026');

    const workbenchStore = useWorkbenchStore();
    workbenchStore.seedBenchmarkData();

    const wrapper = mount(BankingQuarantineView);

    // Old interactive role switcher box must not exist
    expect(wrapper.find('.role-switcher-box').exists()).toBe(false);

    // Static session officer badge must exist and show Maker profile
    const badge = wrapper.find('.session-officer-badge');
    expect(badge.exists()).toBe(true);
    expect(badge.find('.officer-name').text()).toBe('Trịnh Văn Nam');
    expect(badge.find('.officer-role-pill').text()).toContain('Maker');
    expect(badge.find('.readonly-pill').exists()).toBe(false);
  });

  it('displays Checker session badge when logged in as Checker', async () => {
    const authStore = useAuthStore();
    authStore.login('checker_tri', 'LivaChecker@2026');

    const workbenchStore = useWorkbenchStore();
    workbenchStore.seedBenchmarkData();

    const wrapper = mount(BankingQuarantineView);

    const badge = wrapper.find('.session-officer-badge');
    expect(badge.exists()).toBe(true);
    expect(badge.find('.officer-name').text()).toBe('Nguyễn Minh Trí');
    expect(badge.find('.officer-role-pill').text()).toContain('Checker');
    expect(badge.find('.readonly-pill').exists()).toBe(false);
  });

  it('displays [CHỈ ĐỌC] badge and read-only auditor workspace banner for Auditor', async () => {
    const authStore = useAuthStore();
    authStore.login('auditor_lan', 'LivaAudit@2026');

    const workbenchStore = useWorkbenchStore();
    workbenchStore.seedBenchmarkData();

    const wrapper = mount(BankingQuarantineView);

    // Badge must have [CHỈ ĐỌC] pill
    const badge = wrapper.find('.session-officer-badge');
    expect(badge.exists()).toBe(true);
    expect(badge.find('.readonly-pill').exists()).toBe(true);
    expect(badge.find('.readonly-pill').text()).toBe('[CHỈ ĐỌC]');

    // Detail pane must show auditor read-only workspace
    const auditorWorkspace = wrapper.find('.auditor-workspace');
    expect(auditorWorkspace.exists()).toBe(true);
    expect(auditorWorkspace.text()).toContain('[CHỈ ĐỌC / READ-ONLY]');
  });

  it('detects Segregation of Duties violation when Checker is the proposing Maker', async () => {
    const authStore = useAuthStore();
    authStore.login('checker_tri', 'LivaChecker@2026');

    const workbenchStore = useWorkbenchStore();
    workbenchStore.seedBenchmarkData();

    // Set quarantine item q-02 makerId to match checker_tri
    const targetItem = workbenchStore.quarantineItems.find(q => q.status === 'SUBMITTED_TO_CHECKER')!;
    expect(targetItem).toBeDefined();
    targetItem.makerId = 'checker_tri';
    workbenchStore.selectedQuarantineId = targetItem.id;

    const wrapper = mount(BankingQuarantineView);
    await wrapper.vm.$nextTick();

    // Check for SoD violation alert
    const sodAlert = wrapper.find('.sod-violation-alert');
    expect(sodAlert.exists()).toBe(true);
    expect(sodAlert.text()).toContain('CẢNH BÁO VI PHẠM NGUYÊN TẮC 4 MẮT');

    // Confirm action button must be disabled
    const approveBtn = wrapper.find('.btn-approve');
    if (approveBtn.exists()) {
      expect(approveBtn.attributes('disabled')).toBeDefined();
    }
  });
});
