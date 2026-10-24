import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import { useAuthStore, ROLE_PERMITTED_ROUTES, type UserRole } from '../../src/stores/authStore';
import { useToast } from '../../src/composables/useToast';
import BankingApp from '../../src/BankingApp.vue';

describe('BankingApp.vue — Adversarial Route & Hash Challenger (M1-1)', () => {
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

  describe('Adversarial Route Access: MAKER (maker_nam)', () => {
    const FORBIDDEN_MAKER_ROUTES = ['settings', 'compliance', 'risk', 'reports'];

    it('confirms canAccessRoute returns false for all prohibited Maker routes', () => {
      const auth = useAuthStore();
      auth.login('maker_nam', 'LivaMaker@2026');

      for (const route of FORBIDDEN_MAKER_ROUTES) {
        expect(auth.canAccessRoute(route)).toBe(false);
      }
    });

    for (const route of FORBIDDEN_MAKER_ROUTES) {
      it(`forces redirect to overview and throws 403 on initial hash injection #${route}`, async () => {
        window.location.hash = `#${route}`;

        const auth = useAuthStore();
        auth.login('maker_nam', 'LivaMaker@2026');

        const toast = useToast();
        toast.clear();

        const wrapper = mount(BankingApp, { global: { stubs } });

        expect(wrapper.vm.currentTab).toBe('overview');
        expect(window.location.hash).toBe('#overview');

        const errToast = toast.toasts.value.find((t) => t.type === 'error');
        expect(errToast).toBeDefined();
        expect(errToast?.message).toContain('403 Forbidden');
        expect(errToast?.message).toContain(route.toUpperCase());
      });

      it(`intercepts programmatic onNavigate('${route}') and stays on overview with 403`, async () => {
        window.location.hash = '#overview';

        const auth = useAuthStore();
        auth.login('maker_nam', 'LivaMaker@2026');

        const toast = useToast();
        toast.clear();

        const wrapper = mount(BankingApp, { global: { stubs } });
        expect(wrapper.vm.currentTab).toBe('overview');

        wrapper.vm.onNavigate(route);
        await wrapper.vm.$nextTick();

        expect(wrapper.vm.currentTab).toBe('overview');
        expect(window.location.hash).toBe('#overview');

        const errToast = toast.toasts.value.find((t) => t.type === 'error');
        expect(errToast).toBeDefined();
        expect(errToast?.message).toContain('403 Forbidden');
        expect(errToast?.message).toContain(route.toUpperCase());
      });

      it(`intercepts dynamic hashchange event to #${route} and bounces to overview with 403`, async () => {
        window.location.hash = '#overview';

        const auth = useAuthStore();
        auth.login('maker_nam', 'LivaMaker@2026');

        const toast = useToast();
        toast.clear();

        const wrapper = mount(BankingApp, { global: { stubs } });
        expect(wrapper.vm.currentTab).toBe('overview');

        // Attacker modifies URL hash at runtime
        window.location.hash = `#${route}`;
        window.dispatchEvent(new Event('hashchange'));
        await wrapper.vm.$nextTick();

        expect(wrapper.vm.currentTab).toBe('overview');
        expect(window.location.hash).toBe('#overview');

        const errToast = toast.toasts.value.find((t) => t.type === 'error');
        expect(errToast).toBeDefined();
        expect(errToast?.message).toContain('403 Forbidden');
        expect(errToast?.message).toContain(route.toUpperCase());
      });
    }
  });

  describe('Adversarial Role Switching & Dynamic Guard', () => {
    it('intercepts Admin on #settings switching dynamically to maker_nam and forces navigation away', async () => {
      window.location.hash = '#settings';

      const auth = useAuthStore();
      auth.login('admin_sys', 'LivaAdmin@2026');

      const toast = useToast();
      toast.clear();

      const wrapper = mount(BankingApp, { global: { stubs } });
      expect(wrapper.vm.currentTab).toBe('settings');
      expect(window.location.hash).toBe('#settings');

      // Admin dynamically switches to maker_nam
      auth.switchUser('maker_nam');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();

      // Reactive watcher must intercept and force overview
      expect(wrapper.vm.currentTab).toBe('overview');
      expect(window.location.hash).toBe('#overview');

      const warnToast = toast.toasts.value.find((t) => t.type === 'warning');
      expect(warnToast).toBeDefined();
      expect(warnToast?.message).toContain('Chuyển đổi vai trò sang [MAKER]');
      expect(warnToast?.message).toContain('SETTING');
    });

    it('intercepts Checker on #compliance switching dynamically to maker_nam and forces navigation away', async () => {
      window.location.hash = '#compliance';

      const auth = useAuthStore();
      auth.login('checker_tri', 'LivaChecker@2026');

      const toast = useToast();
      toast.clear();

      const wrapper = mount(BankingApp, { global: { stubs } });
      expect(wrapper.vm.currentTab).toBe('compliance');

      auth.switchUser('maker_nam');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();

      expect(wrapper.vm.currentTab).toBe('overview');
      expect(window.location.hash).toBe('#overview');

      const warnToast = toast.toasts.value.find((t) => t.type === 'warning');
      expect(warnToast).toBeDefined();
      expect(warnToast?.message).toContain('Chuyển đổi vai trò sang [MAKER]');
    });

    it('intercepts CFO on #cashflow switching dynamically to auditor_lan and forces navigation away', async () => {
      window.location.hash = '#cashflow';

      const auth = useAuthStore();
      auth.login('cfo_huong', 'LivaCfo@2026');

      const toast = useToast();
      toast.clear();

      const wrapper = mount(BankingApp, { global: { stubs } });
      expect(wrapper.vm.currentTab).toBe('cashflow');

      auth.switchUser('auditor_lan');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();

      expect(wrapper.vm.currentTab).toBe('overview');
      expect(window.location.hash).toBe('#overview');

      const warnToast = toast.toasts.value.find((t) => t.type === 'warning');
      expect(warnToast).toBeDefined();
      expect(warnToast?.message).toContain('Chuyển đổi vai trò sang [AUDITOR]');
    });

    it('does NOT redirect when user switches to another role that ALSO possesses access to current tab', async () => {
      window.location.hash = '#quarantine';

      const auth = useAuthStore();
      // Maker has access to quarantine
      auth.login('maker_nam', 'LivaMaker@2026');

      const toast = useToast();
      toast.clear();

      const wrapper = mount(BankingApp, { global: { stubs } });
      expect(wrapper.vm.currentTab).toBe('quarantine');

      // Switch to Checker (Checker ALSO has access to quarantine)
      auth.switchUser('checker_tri');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();

      // Should remain on quarantine view without warning
      expect(wrapper.vm.currentTab).toBe('quarantine');
      expect(toast.toasts.value.filter((t) => t.type === 'warning')).toHaveLength(0);
    });
  });

  describe('Invalid, Malformed, and Tampered URL Hashes', () => {
    const MALFORMED_HASHES = [
      '#hack',
      '#../admin',
      '#null',
      '#undefined',
      '#NaN',
      '#%2e%2e%2fadmin',
      '#!@#$%^&*()',
      '#<script>alert(1)</script>',
      '#12345',
      '#../../etc/passwd',
      '#window',
      '#__proto__',
      '#constructor',
      '#toString',
      '#valueOf',
      '#',
      '#/',
      '#//',
    ];

    for (const malformed of MALFORMED_HASHES) {
      it(`falls back safely to overview when given malformed hash: "${malformed}"`, async () => {
        window.location.hash = malformed;

        const auth = useAuthStore();
        auth.login('maker_nam', 'LivaMaker@2026');

        const wrapper = mount(BankingApp, { global: { stubs } });
        expect(wrapper.vm.currentTab).toBe('overview');
      });
    }

    it('resolves tab with leading slash correctly (e.g. #/workbench for Maker)', async () => {
      window.location.hash = '#/workbench';

      const auth = useAuthStore();
      auth.login('maker_nam', 'LivaMaker@2026');

      const wrapper = mount(BankingApp, { global: { stubs } });
      expect(wrapper.vm.currentTab).toBe('workbench');
    });

    it('normalizes uppercase hashes (e.g. #/WORKBENCH for Maker)', async () => {
      window.location.hash = '#/WORKBENCH';

      const auth = useAuthStore();
      auth.login('maker_nam', 'LivaMaker@2026');

      const wrapper = mount(BankingApp, { global: { stubs } });
      expect(wrapper.vm.currentTab).toBe('workbench');
    });
  });

  describe('Exhaustive 5-Role x 12-Route Permission Matrix Verification', () => {
    const ALL_12_ROUTES = [
      'overview',
      'workbench',
      'quarantine',
      'ledger',
      'reconcile',
      'transactions',
      'cashflow',
      'treasury',
      'risk',
      'compliance',
      'reports',
      'settings',
    ];

    const ROLE_ACCOUNTS: Record<UserRole, { username: string; pass: string }> = {
      MAKER: { username: 'maker_nam', pass: 'LivaMaker@2026' },
      CHECKER: { username: 'checker_tri', pass: 'LivaChecker@2026' },
      CFO: { username: 'cfo_huong', pass: 'LivaCfo@2026' },
      AUDITOR: { username: 'auditor_lan', pass: 'LivaAudit@2026' },
      ADMIN: { username: 'admin_sys', pass: 'LivaAdmin@2026' },
    };

    const roles: UserRole[] = ['MAKER', 'CHECKER', 'CFO', 'AUDITOR', 'ADMIN'];

    for (const role of roles) {
      describe(`Exhaustive Route Checks for ${role}`, () => {
        const permitted = ROLE_PERMITTED_ROUTES[role];
        const { username, pass } = ROLE_ACCOUNTS[role];

        for (const route of ALL_12_ROUTES) {
          const isAllowed = permitted.includes(route);

          it(`${role} accessing '${route}' -> Expected: ${isAllowed ? 'ALLOWED' : 'BLOCKED 403'}`, async () => {
            const auth = useAuthStore();
            auth.login(username, pass);

            expect(auth.canAccessRoute(route)).toBe(isAllowed);

            window.location.hash = `#${route}`;
            const toast = useToast();
            toast.clear();

            const wrapper = mount(BankingApp, { global: { stubs } });

            if (isAllowed) {
              expect(wrapper.vm.currentTab).toBe(route);
              const errs = toast.toasts.value.filter((t) => t.type === 'error');
              expect(errs).toHaveLength(0);
            } else {
              expect(wrapper.vm.currentTab).toBe('overview');
              expect(window.location.hash).toBe('#overview');
              const err = toast.toasts.value.find((t) => t.type === 'error');
              expect(err).toBeDefined();
              expect(err?.message).toContain('403 Forbidden');
            }
          });
        }
      });
    }
  });
});
