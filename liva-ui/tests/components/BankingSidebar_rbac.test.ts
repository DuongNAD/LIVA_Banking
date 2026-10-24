import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import BankingSidebar from '../../src/components/banking/BankingSidebar.vue';
import { useAuthStore } from '../../src/stores/authStore';

describe('BankingSidebar.vue (RBAC Navigation & Officer Status)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
  });

  it('renders exactly 8 nav items for MAKER (maker_nam) and customized quarantine label', async () => {
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const wrapper = mount(BankingSidebar, {
      props: { activeItem: 'overview' },
    });

    const items = wrapper.findAll('.nav-item');
    expect(items).toHaveLength(8);

    const labels = items.map((i) => i.find('.nav-item-label').text());
    expect(labels).toContain('TỔNG QUAN');
    expect(labels).toContain('P42 WORKBENCH');
    expect(labels).toContain('P44 LẬP ĐỀ XUẤT');
    expect(labels).toContain('P50 SỔ CÁI ERP');
    expect(labels).toContain('ĐỐI SOÁT');
    expect(labels).toContain('GIAO DỊCH');
    expect(labels).toContain('THU CHI');
    expect(labels).toContain('NGÂN QUỸ');

    // Forbidden screens must not be rendered
    expect(labels.some((l) => l.includes('TUÂN THỦ'))).toBe(false);
    expect(labels.some((l) => l.includes('RỦI RO'))).toBe(false);
    expect(labels.some((l) => l.includes('BÁO CÁO'))).toBe(false);
    expect(labels.some((l) => l.includes('CÀI ĐẶT'))).toBe(false);

    // Footer officer card
    const footer = wrapper.find('.officer-footer-card');
    expect(footer.exists()).toBe(true);
    expect(footer.find('.officer-name').text()).toBe('Trịnh Văn Nam');
    expect(footer.find('.officer-role-badge').text()).toBe('MAKER');
    expect(footer.find('.officer-sod-badge').text()).toContain('SoD Active');
  });

  it('renders exactly 7 nav items for CHECKER (checker_tri) with 4-Eyes and signature labels', async () => {
    const auth = useAuthStore();
    auth.login('checker_tri', 'LivaChecker@2026');

    const wrapper = mount(BankingSidebar, {
      props: { activeItem: 'overview' },
    });

    const items = wrapper.findAll('.nav-item');
    expect(items).toHaveLength(7);

    const labels = items.map((i) => i.find('.nav-item-label').text());
    expect(labels).toContain('TỔNG QUAN');
    expect(labels).toContain('P44 PHÊ DUYỆT 4-EYES');
    expect(labels).toContain('P50 SỔ CÁI ERP');
    expect(labels).toContain('KÝ DUYỆT LỆNH CHI');
    expect(labels).toContain('P70 RỦI RO');
    expect(labels).toContain('P80 TUÂN THỦ');
    expect(labels).toContain('BÁO CÁO');

    // Forbidden screens must not be rendered
    expect(labels.some((l) => l.includes('P42 WORKBENCH'))).toBe(false);
    expect(labels.some((l) => l.includes('ĐỐI SOÁT'))).toBe(false);
    expect(labels.some((l) => l.includes('GIAO DỊCH'))).toBe(false);
    expect(labels.some((l) => l.includes('THU CHI'))).toBe(false);
    expect(labels.some((l) => l.includes('CÀI ĐẶT'))).toBe(false);

    // Footer officer card
    const footer = wrapper.find('.officer-footer-card');
    expect(footer.find('.officer-name').text()).toBe('Nguyễn Minh Trí');
    expect(footer.find('.officer-role-badge').text()).toBe('CHECKER');
  });

  it('renders exactly 6 nav items for CFO (cfo_huong) with liquidity & treasury oversight', async () => {
    const auth = useAuthStore();
    auth.login('cfo_huong', 'LivaCfo@2026');

    const wrapper = mount(BankingSidebar, {
      props: { activeItem: 'overview' },
    });

    const items = wrapper.findAll('.nav-item');
    expect(items).toHaveLength(6);

    const labels = items.map((i) => i.find('.nav-item-label').text());
    expect(labels).toContain('TỔNG QUAN');
    expect(labels).toContain('P44 CÁCH LY');
    expect(labels).toContain('THU CHI');
    expect(labels).toContain('NGÂN QUỸ');
    expect(labels).toContain('P70 RỦI RO');
    expect(labels).toContain('BÁO CÁO');

    // Forbidden screens must not be rendered
    expect(labels.some((l) => l.includes('P42 WORKBENCH'))).toBe(false);
    expect(labels.some((l) => l.includes('ĐỐI SOÁT'))).toBe(false);
    expect(labels.some((l) => l.includes('SỔ CÁI ERP'))).toBe(false);
    expect(labels.some((l) => l.includes('TUÂN THỦ'))).toBe(false);
    expect(labels.some((l) => l.includes('CÀI ĐẶT'))).toBe(false);

    // Footer officer card
    const footer = wrapper.find('.officer-footer-card');
    expect(footer.find('.officer-name').text()).toBe('Đỗ Lan Hương');
    expect(footer.find('.officer-role-badge').text()).toBe('CFO');
  });

  it('renders exactly 5 nav items for AUDITOR (auditor_lan) and badges [CHỈ ĐỌC]', async () => {
    const auth = useAuthStore();
    auth.login('auditor_lan', 'LivaAudit@2026');

    const wrapper = mount(BankingSidebar, {
      props: { activeItem: 'overview' },
    });

    const items = wrapper.findAll('.nav-item');
    expect(items).toHaveLength(5);

    const labels = items.map((i) => i.find('.nav-item-label').text());
    expect(labels).toContain('TỔNG QUAN');
    expect(labels).toContain('P50 SỔ CÁI ERP [CHỈ ĐỌC]');
    expect(labels).toContain('GIAO DỊCH [CHỈ ĐỌC]');
    expect(labels).toContain('P80 TUÂN THỦ');
    expect(labels).toContain('BÁO CÁO');

    // Forbidden mutation screens must not be rendered
    expect(labels.some((l) => l.includes('WORKBENCH'))).toBe(false);
    expect(labels.some((l) => l.includes('CÁCH LY'))).toBe(false);
    expect(labels.some((l) => l.includes('ĐỐI SOÁT'))).toBe(false);
    expect(labels.some((l) => l.includes('THU CHI'))).toBe(false);
    expect(labels.some((l) => l.includes('NGÂN QUỸ'))).toBe(false);
    expect(labels.some((l) => l.includes('RỦI RO'))).toBe(false);
    expect(labels.some((l) => l.includes('CÀI ĐẶT'))).toBe(false);

    // Footer officer card
    const footer = wrapper.find('.officer-footer-card');
    expect(footer.find('.officer-name').text()).toBe('Phạm Hương Lan');
    expect(footer.find('.officer-role-badge').text()).toBe('AUDITOR');
  });

  it('renders exactly 2 nav items for ADMIN (admin_sys) with Zero financial access', async () => {
    const auth = useAuthStore();
    auth.login('admin_sys', 'LivaAdmin@2026');

    const wrapper = mount(BankingSidebar, {
      props: { activeItem: 'overview' },
    });

    const items = wrapper.findAll('.nav-item');
    expect(items).toHaveLength(2);

    const labels = items.map((i) => i.find('.nav-item-label').text());
    expect(labels).toContain('TỔNG QUAN');
    expect(labels).toContain('CÀI ĐẶT');

    // All financial screens must be hidden
    expect(labels.some((l) => l.includes('WORKBENCH'))).toBe(false);
    expect(labels.some((l) => l.includes('CÁCH LY'))).toBe(false);
    expect(labels.some((l) => l.includes('SỔ CÁI'))).toBe(false);
    expect(labels.some((l) => l.includes('ĐỐI SOÁT'))).toBe(false);
    expect(labels.some((l) => l.includes('GIAO DỊCH'))).toBe(false);
    expect(labels.some((l) => l.includes('THU CHI'))).toBe(false);
    expect(labels.some((l) => l.includes('NGÂN QUỸ'))).toBe(false);
    expect(labels.some((l) => l.includes('RỦI RO'))).toBe(false);
    expect(labels.some((l) => l.includes('TUÂN THỦ'))).toBe(false);
    expect(labels.some((l) => l.includes('BÁO CÁO'))).toBe(false);

    // Footer officer card
    const footer = wrapper.find('.officer-footer-card');
    expect(footer.find('.officer-name').text()).toBe('Quản trị viên An ninh');
    expect(footer.find('.officer-role-badge').text()).toBe('ADMIN');
  });

  it('renders strictly 7 nav items on clean launch with default Checker session without explicit login (Finding 1 fix)', () => {
    const wrapper = mount(BankingSidebar, {
      props: { activeItem: 'overview' },
    });

    const items = wrapper.findAll('.nav-item');
    expect(items).toHaveLength(7);

    const labels = items.map((i) => i.find('.nav-item-label').text());
    expect(labels).toContain('TỔNG QUAN');
    expect(labels).toContain('P44 PHÊ DUYỆT 4-EYES');
    expect(labels).toContain('P50 SỔ CÁI ERP');
    expect(labels).toContain('KÝ DUYỆT LỆNH CHI');
    expect(labels).toContain('P70 RỦI RO');
    expect(labels).toContain('P80 TUÂN THỦ');
    expect(labels).toContain('BÁO CÁO');

    // Forbidden Maker/Admin screens must NEVER be present on clean launch
    expect(labels.some((l) => l.includes('WORKBENCH'))).toBe(false);
    expect(labels.some((l) => l.includes('ĐỐI SOÁT'))).toBe(false);
    expect(labels.some((l) => l.includes('GIAO DỊCH'))).toBe(false);
    expect(labels.some((l) => l.includes('THU CHI'))).toBe(false);
    expect(labels.some((l) => l.includes('CÀI ĐẶT'))).toBe(false);

    // Footer reflects default Checker
    const footer = wrapper.find('.officer-footer-card');
    expect(footer.find('.officer-name').text()).toBe('Nguyễn Minh Trí');
    expect(footer.find('.officer-role-badge').text()).toBe('CHECKER');
    expect(footer.find('.officer-sod-badge').text()).toContain('SoD Active');
  });

  it('emits navigate event when clicking permitted nav button', async () => {
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const wrapper = mount(BankingSidebar, {
      props: { activeItem: 'overview' },
    });

    const workbenchBtn = wrapper.findAll('.nav-item').find((b) => b.text().includes('WORKBENCH'));
    expect(workbenchBtn).toBeDefined();

    await workbenchBtn!.trigger('click');
    expect(wrapper.emitted('navigate')).toBeTruthy();
    expect(wrapper.emitted('navigate')![0]).toEqual(['workbench']);
  });
});

describe('BankingApp.vue (Hash Route Guarding & RBAC Segregation of Duties)', () => {
  const stubs = {
    BankingWindowBar: true,
    BankingSidebar: true,
    BankingHeader: true,
    FinancialAssistantDrawer: true,
    ToastContainer: true,
    LoginModal: true,
    BankingOverviewView: { template: '<div class="view-overview">Overview View</div>' },
    BankingWorkbenchView: { template: '<div class="view-workbench">Workbench View</div>' },
    BankingQuarantineView: { template: '<div class="view-quarantine">Quarantine View</div>' },
    BankingLedgerView: { template: '<div class="view-ledger">Ledger View</div>' },
    BankingReconciliationView: { template: '<div class="view-reconcile">Reconciliation View</div>' },
    BankingTransactionsView: { template: '<div class="view-transactions">Transactions View</div>' },
    BankingCashflowView: { template: '<div class="view-cashflow">Cashflow View</div>' },
    BankingTreasuryView: { template: '<div class="view-treasury">Treasury View</div>' },
    BankingRiskView: { template: '<div class="view-risk">Risk View</div>' },
    BankingComplianceView: { template: '<div class="view-compliance">Compliance View</div>' },
    BankingReportsView: { template: '<div class="view-reports">Reports View</div>' },
    BankingSettingsView: { template: '<div class="view-settings">Settings View</div>' },
  };

  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined') {
      window.location.hash = '';
      if (window.localStorage) {
        window.localStorage.clear();
      }
    }
  });

  it('mounts to overview by default when hash is empty', async () => {
    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    expect(wrapper.vm.currentTab).toBe('overview');
  });

  it('blocks unauthorized initial hash (#settings for Maker) and bounces to overview with 403 toast', async () => {
    window.location.hash = '#settings';

    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const { useToast } = await import('../../src/composables/useToast');
    const toast = useToast();
    toast.clear();

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    // Should redirect to overview
    expect(wrapper.vm.currentTab).toBe('overview');
    expect(window.location.hash).toBe('#overview');

    // 403 Forbidden Toast emitted
    expect(toast.toasts.value.length).toBeGreaterThanOrEqual(1);
    const errToast = toast.toasts.value.find((t) => t.type === 'error');
    expect(errToast).toBeDefined();
    expect(errToast?.message).toContain('403 Forbidden');
    expect(errToast?.message).toContain('SETTINGS');
  });

  it('permits authorized initial hash (#workbench for Maker)', async () => {
    window.location.hash = '#workbench';

    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    expect(wrapper.vm.currentTab).toBe('workbench');
  });

  it('blocks unauthorized onNavigate (#settings for Maker) and emits 403 toast', async () => {
    window.location.hash = '#overview';

    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const { useToast } = await import('../../src/composables/useToast');
    const toast = useToast();
    toast.clear();

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    wrapper.vm.onNavigate('settings');
    await wrapper.vm.$nextTick();

    expect(wrapper.vm.currentTab).toBe('overview');
    expect(window.location.hash).toBe('#overview');

    const errToast = toast.toasts.value.find((t) => t.type === 'error');
    expect(errToast).toBeDefined();
    expect(errToast?.message).toContain('403 Forbidden');
  });

  it('intercepts runtime hashchange to unauthorized route and bounces to overview', async () => {
    window.location.hash = '#overview';

    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const { useToast } = await import('../../src/composables/useToast');
    const toast = useToast();
    toast.clear();

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    expect(wrapper.vm.currentTab).toBe('overview');

    // Simulate user typing #risk in URL bar
    window.location.hash = '#risk';
    window.dispatchEvent(new Event('hashchange'));
    await wrapper.vm.$nextTick();

    // Must bounce back to overview
    expect(wrapper.vm.currentTab).toBe('overview');
    expect(window.location.hash).toBe('#overview');

    // Toast error logged
    const errToast = toast.toasts.value.find((t) => t.type === 'error');
    expect(errToast).toBeDefined();
    expect(errToast?.message).toContain('403 Forbidden');
  });

  it('reactively bounces to overview when active user switches role while on newly prohibited tab', async () => {
    window.location.hash = '#workbench';

    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const { useToast } = await import('../../src/composables/useToast');
    const toast = useToast();
    toast.clear();

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    expect(wrapper.vm.currentTab).toBe('workbench');

    // Switch user to Auditor (auditor_lan has no access to workbench)
    auth.switchUser('auditor_lan');
    await wrapper.vm.$nextTick();
    await wrapper.vm.$nextTick();

    // App watcher should automatically redirect to overview
    expect(wrapper.vm.currentTab).toBe('overview');
    expect(window.location.hash).toBe('#overview');

    // Warning toast displayed
    const warnToast = toast.toasts.value.find((t) => t.type === 'warning');
    expect(warnToast).toBeDefined();
    expect(warnToast?.message).toContain('Chuyển đổi vai trò sang [AUDITOR]');
  });

  it('normalizes window.location.hash to #overview when initialized with non-existent hash (Finding 3 fix)', async () => {
    window.location.hash = '#unknown_tab_xyz';

    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    expect(wrapper.vm.currentTab).toBe('overview');
    expect(window.location.hash).toBe('#overview');
  });

  it('normalizes window.location.hash to #overview when hashchange fires with non-existent route', async () => {
    window.location.hash = '#overview';

    const { default: BankingApp } = await import('../../src/BankingApp.vue');
    const auth = useAuthStore();
    auth.login('maker_nam', 'LivaMaker@2026');

    const wrapper = mount(BankingApp, {
      global: { stubs },
    });

    expect(wrapper.vm.currentTab).toBe('overview');

    window.location.hash = '#random_invalid_route';
    window.dispatchEvent(new Event('hashchange'));
    await wrapper.vm.$nextTick();

    expect(wrapper.vm.currentTab).toBe('overview');
    expect(window.location.hash).toBe('#overview');
  });
});
