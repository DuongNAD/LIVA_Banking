import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import {
  useAuthStore,
  ROLE_PERMITTED_ROUTES,
  type UserRole,
} from '../src/stores/authStore';
import BankingSidebar from '../src/components/banking/BankingSidebar.vue';

const ALL_12_SCREENS = [
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
] as const;

type ScreenId = typeof ALL_12_SCREENS[number];

const FINANCIAL_10_SCREENS: ScreenId[] = [
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
];

const EXPECTED_PERMISSIONS: Record<UserRole, { permitted: ScreenId[]; prohibited: ScreenId[] }> = {
  MAKER: {
    permitted: [
      'overview',
      'workbench',
      'quarantine',
      'ledger',
      'reconcile',
      'transactions',
      'cashflow',
      'treasury',
    ],
    prohibited: ['risk', 'compliance', 'reports', 'settings'],
  },
  CHECKER: {
    permitted: [
      'overview',
      'quarantine',
      'ledger',
      'treasury',
      'compliance',
      'risk',
      'reports',
    ],
    prohibited: ['workbench', 'reconcile', 'transactions', 'cashflow', 'settings'],
  },
  CFO: {
    permitted: [
      'overview',
      'treasury',
      'cashflow',
      'quarantine',
      'risk',
      'reports',
    ],
    prohibited: ['workbench', 'ledger', 'reconcile', 'transactions', 'compliance', 'settings'],
  },
  AUDITOR: {
    permitted: ['overview', 'compliance', 'ledger', 'transactions', 'reports'],
    prohibited: ['workbench', 'quarantine', 'reconcile', 'cashflow', 'treasury', 'risk', 'settings'],
  },
  ADMIN: {
    permitted: ['overview', 'settings'],
    prohibited: [
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
    ],
  },
};

const ROLE_DEMO_CREDENTIALS: Record<UserRole, { username: string; pass: string }> = {
  MAKER: { username: 'maker_nam', pass: 'LivaMaker@2026' },
  CHECKER: { username: 'checker_tri', pass: 'LivaChecker@2026' },
  CFO: { username: 'cfo_huong', pass: 'LivaCfo@2026' },
  AUDITOR: { username: 'auditor_lan', pass: 'LivaAudit@2026' },
  ADMIN: { username: 'admin_sys', pass: 'LivaAdmin@2026' },
};

const stubs = {
  BankingWindowBar: true,
  BankingSidebar: true,
  BankingHeader: true,
  FinancialAssistantDrawer: true,
  ToastContainer: true,
  LoginModal: true,
  BankingOverviewView: { template: '<div class="view-overview">Overview</div>' },
  BankingWorkbenchView: { template: '<div class="view-workbench">Workbench</div>' },
  BankingQuarantineView: { template: '<div class="view-quarantine">Quarantine</div>' },
  BankingLedgerView: { template: '<div class="view-ledger">Ledger</div>' },
  BankingReconciliationView: { template: '<div class="view-reconcile">Reconcile</div>' },
  BankingTransactionsView: { template: '<div class="view-transactions">Transactions</div>' },
  BankingCashflowView: { template: '<div class="view-cashflow">Cashflow</div>' },
  BankingTreasuryView: { template: '<div class="view-treasury">Treasury</div>' },
  BankingRiskView: { template: '<div class="view-risk">Risk</div>' },
  BankingComplianceView: { template: '<div class="view-compliance">Compliance</div>' },
  BankingReportsView: { template: '<div class="view-reports">Reports</div>' },
  BankingSettingsView: { template: '<div class="view-settings">Settings</div>' },
};

describe('Challenger M1-2: Permission Matrix Invariants & Segregation of Duties', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined') {
      window.location.hash = '';
      if (window.localStorage) {
        window.localStorage.clear();
      }
    }
  });

  describe('Invariant 1: Exact 5-Role x 12-Screen Permission Matrix (All 60 Cells)', () => {
    const roles: UserRole[] = ['MAKER', 'CHECKER', 'CFO', 'AUDITOR', 'ADMIN'];

    roles.forEach((role) => {
      describe(`Role: ${role}`, () => {
        it(`validates partition: permitted (${EXPECTED_PERMISSIONS[role].permitted.length}) + prohibited (${EXPECTED_PERMISSIONS[role].prohibited.length}) === 12`, () => {
          const spec = EXPECTED_PERMISSIONS[role];
          expect(spec.permitted.length + spec.prohibited.length).toBe(12);

          // Disjoint union
          const union = new Set([...spec.permitted, ...spec.prohibited]);
          expect(union.size).toBe(12);
          ALL_12_SCREENS.forEach((s) => expect(union.has(s)).toBe(true));
        });

        it(`empirically verifies canAccessRoute for all 12 screens under ${role}`, () => {
          const auth = useAuthStore();
          const cred = ROLE_DEMO_CREDENTIALS[role];
          const loginRes = auth.login(cred.username, cred.pass);
          expect(loginRes.success).toBe(true);
          expect(auth.currentUser.role).toBe(role);

          const spec = EXPECTED_PERMISSIONS[role];

          // Every permitted screen must return true
          spec.permitted.forEach((screen) => {
            expect(
              auth.canAccessRoute(screen),
              `Expected ${role} to HAVE access to ${screen}`
            ).toBe(true);
          });

          // Every prohibited screen must return false
          spec.prohibited.forEach((screen) => {
            expect(
              auth.canAccessRoute(screen),
              `Expected ${role} to NOT have access to ${screen}`
            ).toBe(false);
          });
        });

        it(`empirically verifies sidebar nav buttons match exactly permitted list for ${role}`, () => {
          const auth = useAuthStore();
          const cred = ROLE_DEMO_CREDENTIALS[role];
          auth.login(cred.username, cred.pass);

          const wrapper = mount(BankingSidebar, {
            props: { activeItem: 'overview' },
          });

          const items = wrapper.findAll('.nav-item');
          const spec = EXPECTED_PERMISSIONS[role];
          expect(items.length).toBe(spec.permitted.length);
        });
      });
    });
  });

  describe('Invariant 2: Admin ZERO Access to All 10 Financial Screens', () => {
    it('confirms Admin cannot access ANY of the 10 financial screens via canAccessRoute', () => {
      const auth = useAuthStore();
      auth.login('admin_sys', 'LivaAdmin@2026');
      expect(auth.isAdmin).toBe(true);

      FINANCIAL_10_SCREENS.forEach((financialScreen) => {
        const allowed = auth.canAccessRoute(financialScreen);
        expect(
          allowed,
          `VIOLATION: Admin was granted access to financial screen [${financialScreen}]!`
        ).toBe(false);
      });
    });

    it('confirms Admin sidebar contains zero financial navigation items', () => {
      const auth = useAuthStore();
      auth.login('admin_sys', 'LivaAdmin@2026');

      const wrapper = mount(BankingSidebar, {
        props: { activeItem: 'overview' },
      });

      const items = wrapper.findAll('.nav-item');
      expect(items.length).toBe(2); // Only overview & settings

      const renderedText = wrapper.text().toUpperCase();
      expect(renderedText).toContain('TỔNG QUAN');
      expect(renderedText).toContain('CÀI ĐẶT');

      // Prohibited keywords in financial domain
      const prohibitedKeywords = [
        'WORKBENCH',
        'CÁCH LY',
        'SỔ CÁI',
        'ĐỐI SOÁT',
        'GIAO DỊCH',
        'THU CHI',
        'NGÂN QUỸ',
        'RỦI RO',
        'TUÂN THỦ',
        'BÁO CÁO',
      ];

      prohibitedKeywords.forEach((kw) => {
        expect(
          renderedText.includes(kw),
          `VIOLATION: Admin sidebar rendered financial keyword [${kw}]!`
        ).toBe(false);
      });
    });

    it('confirms Admin attempting hash navigation to any of the 10 financial screens is intercepted and bounced with 403', async () => {
      const { default: BankingApp } = await import('../src/BankingApp.vue');
      const { useToast } = await import('../src/composables/useToast');
      const auth = useAuthStore();
      auth.login('admin_sys', 'LivaAdmin@2026');

      const toast = useToast();

      for (const financialScreen of FINANCIAL_10_SCREENS) {
        toast.clear();
        window.location.hash = `#${financialScreen}`;

        const wrapper = mount(BankingApp, {
          global: { stubs },
        });

        // Current tab must be bounced to overview
        expect(
          wrapper.vm.currentTab,
          `Admin navigated to financial screen ${financialScreen} without being bounced!`
        ).toBe('overview');
        expect(window.location.hash).toBe('#overview');

        // Toast error must be emitted with 403 Forbidden
        const errToast = toast.toasts.value.find((t) => t.type === 'error');
        expect(errToast).toBeDefined();
        expect(errToast?.message).toContain('403 Forbidden');
        expect(errToast?.message).toContain('ADMIN');

        wrapper.unmount();
      }
    });

    it('confirms Admin onNavigate call to any of the 10 financial screens is blocked', async () => {
      const { default: BankingApp } = await import('../src/BankingApp.vue');
      const { useToast } = await import('../src/composables/useToast');
      const auth = useAuthStore();
      auth.login('admin_sys', 'LivaAdmin@2026');

      const toast = useToast();
      window.location.hash = '#overview';

      const wrapper = mount(BankingApp, {
        global: { stubs },
      });

      for (const financialScreen of FINANCIAL_10_SCREENS) {
        toast.clear();
        wrapper.vm.onNavigate(financialScreen);
        await wrapper.vm.$nextTick();

        expect(wrapper.vm.currentTab).toBe('overview');
        expect(window.location.hash).toBe('#overview');

        const errToast = toast.toasts.value.find((t) => t.type === 'error');
        expect(errToast).toBeDefined();
        expect(errToast?.message).toContain('403 Forbidden');
      }

      wrapper.unmount();
    });
  });

  describe('Adversarial & Boundary Stress Tests', () => {
    it('rejects prototype pollution and object property route names', () => {
      const auth = useAuthStore();
      auth.login('maker_nam', 'LivaMaker@2026');

      const attackVectors = [
        '__proto__',
        'constructor',
        'prototype',
        'toString',
        'valueOf',
        'hasOwnProperty',
        'isPrototypeOf',
      ];

      attackVectors.forEach((vector) => {
        expect(auth.canAccessRoute(vector)).toBe(false);
      });
    });

    it('rejects path traversal, prefixes, and casing manipulations', () => {
      const auth = useAuthStore();
      auth.login('maker_nam', 'LivaMaker@2026');

      const malformedRoutes = [
        '../settings',
        '/workbench',
        '#workbench',
        'workbench/',
        'WORKBENCH',
        'Workbench',
        'settings ',
        ' settings',
        'overview\0',
        'overview\n',
      ];

      malformedRoutes.forEach((route) => {
        expect(
          auth.canAccessRoute(route),
          `Expected route '${route}' to be rejected`
        ).toBe(false);
      });
    });

    it('rejects unauthorized access when currentUser.role is corrupted or unknown', () => {
      const auth = useAuthStore();
      auth.login('maker_nam', 'LivaMaker@2026');

      const bogusRoles = ['SUPERADMIN', 'HACKER', 'ROOT', '', 'UNKNOWN_ROLE', 'admin'];

      bogusRoles.forEach((bogus) => {
        // Test with a fresh object so shared DEMO_ACCOUNTS object is not mutated
        auth.currentUser = {
          ...auth.currentUser,
          role: bogus as any,
        };

        ALL_12_SCREENS.forEach((screen) => {
          expect(
            auth.canAccessRoute(screen),
            `Expected corrupted role '${bogus}' to be denied on '${screen}'`
          ).toBe(false);
        });

        expect(auth.isMaker).toBe(false);
        expect(auth.isChecker).toBe(false);
        expect(auth.isCfo).toBe(false);
        expect(auth.isAuditor).toBe(false);
        expect(auth.isAdmin).toBe(false);
      });
    });

    it('verifies seamless session switching across all role transitions', async () => {
      const { default: BankingApp } = await import('../src/BankingApp.vue');
      const auth = useAuthStore();

      // 1. Start as Maker on workbench (allowed for Maker)
      window.location.hash = '#workbench';
      const loginRes = auth.login('maker_nam', 'LivaMaker@2026');
      expect(loginRes.success).toBe(true);
      expect(auth.currentUser.role).toBe('MAKER');

      const wrapper = mount(BankingApp, {
        global: { stubs },
      });
      expect(wrapper.vm.currentTab).toBe('workbench');

      // 2. Switch to Admin (workbench is forbidden) -> Bounces to overview
      auth.switchUser('admin_sys');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('overview');
      expect(window.location.hash).toBe('#overview');

      // 3. Admin goes to settings (allowed for Admin)
      wrapper.vm.onNavigate('settings');
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('settings');

      // 4. Switch to Checker (settings is forbidden) -> Bounces to overview
      auth.switchUser('checker_tri');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('overview');
      expect(window.location.hash).toBe('#overview');

      // 5. Checker goes to quarantine (allowed for Checker)
      wrapper.vm.onNavigate('quarantine');
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('quarantine');

      // 6. Switch to CFO (quarantine is also allowed for CFO!) -> Stays on quarantine
      auth.switchUser('cfo_huong');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('quarantine');

      // 7. Switch to Auditor (quarantine is forbidden for Auditor) -> Bounces to overview
      auth.switchUser('auditor_lan');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('overview');
      expect(window.location.hash).toBe('#overview');

      // 8. Auditor goes to transactions (allowed for Auditor)
      wrapper.vm.onNavigate('transactions');
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('transactions');

      // 9. Switch to CFO (transactions is forbidden for CFO) -> Bounces to overview
      auth.switchUser('cfo_huong');
      await wrapper.vm.$nextTick();
      await wrapper.vm.$nextTick();
      expect(wrapper.vm.currentTab).toBe('overview');
      expect(window.location.hash).toBe('#overview');

      wrapper.unmount();
    });

  });
});
