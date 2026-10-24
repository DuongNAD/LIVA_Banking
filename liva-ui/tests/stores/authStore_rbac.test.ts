import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import {
  useAuthStore,
  ROLE_PERMITTED_ROUTES,
  CFO_HUONG_ACCOUNT,
  type UserRole,
} from '../../src/stores/authStore';

describe('authStore RBAC & Segregation of Duties (M1)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
  });

  describe('5 Distinct Roles & Permissions Matrix', () => {
    it('defines ROLE_PERMITTED_ROUTES for all 5 roles matching architectural specification', () => {
      const expectedRoles: UserRole[] = ['MAKER', 'CHECKER', 'CFO', 'AUDITOR', 'ADMIN'];
      expect(Object.keys(ROLE_PERMITTED_ROUTES).sort()).toEqual(expectedRoles.sort());

      expect(ROLE_PERMITTED_ROUTES.MAKER).toEqual([
        'overview',
        'workbench',
        'quarantine',
        'ledger',
        'reconcile',
        'transactions',
        'cashflow',
        'treasury',
      ]);

      expect(ROLE_PERMITTED_ROUTES.CHECKER).toEqual([
        'overview',
        'quarantine',
        'ledger',
        'treasury',
        'compliance',
        'risk',
        'reports',
      ]);

      expect(ROLE_PERMITTED_ROUTES.CFO).toEqual([
        'overview',
        'treasury',
        'cashflow',
        'quarantine',
        'risk',
        'reports',
      ]);

      expect(ROLE_PERMITTED_ROUTES.AUDITOR).toEqual([
        'overview',
        'compliance',
        'ledger',
        'transactions',
        'reports',
      ]);

      expect(ROLE_PERMITTED_ROUTES.ADMIN).toEqual(['overview', 'settings']);
    });
  });

  describe('cfo_huong Demo Account Authentication', () => {
    it('authenticates cfo_huong with LivaCfo@2026 and sets CFO profile', () => {
      const auth = useAuthStore();
      const res = auth.login('cfo_huong', 'LivaCfo@2026');

      expect(res.success).toBe(true);
      expect(res.user?.username).toBe('cfo_huong');
      expect(res.user?.id).toBe('CFO_DO_LAN_HUONG');
      expect(res.user?.fullName).toBe('Đỗ Lan Hương');
      expect(res.user?.role).toBe('CFO');
      expect(res.user?.roleTitle).toBe('Giám đốc Tài chính (CFO)');
      expect(res.user?.department).toBe('Ban Điều hành C-Suite');

      expect(auth.currentUser.username).toBe('cfo_huong');
      expect(auth.currentUser.role).toBe('CFO');
      expect(auth.isCfo).toBe(true);
      expect(auth.isMaker).toBe(false);
      expect(auth.isChecker).toBe(false);
      expect(auth.isAuditor).toBe(false);
      expect(auth.isAdmin).toBe(false);
      expect(auth.isReadOnly).toBe(false);
    });

    it('rejects cfo_huong with invalid password', () => {
      const auth = useAuthStore();
      const res = auth.login('cfo_huong', 'WrongCfoPass');
      expect(res.success).toBe(false);
      expect(res.error).toContain('Mật khẩu không chính xác');
    });

    it('allows switching to cfo_huong via switchUser', () => {
      const auth = useAuthStore();
      const switched = auth.switchUser('cfo_huong');
      expect(switched).toBe(true);
      expect(auth.currentUser.username).toBe('cfo_huong');
      expect(auth.currentUser.role).toBe('CFO');
      expect(auth.isCfo).toBe(true);
    });

    it('authenticates cfo_huong via loginAsync', async () => {
      const auth = useAuthStore();
      const res = await auth.loginAsync('cfo_huong', 'LivaCfo@2026');
      expect(res.success).toBe(true);
      expect(auth.currentUser.role).toBe('CFO');
      expect(auth.currentUser.fullName).toBe('Đỗ Lan Hương');
    });

    it('exports CFO_HUONG_ACCOUNT constant matching specification', () => {
      expect(CFO_HUONG_ACCOUNT.id).toBe('CFO_DO_LAN_HUONG');
      expect(CFO_HUONG_ACCOUNT.fullName).toBe('Đỗ Lan Hương');
      expect(CFO_HUONG_ACCOUNT.role).toBe('CFO');
      expect(CFO_HUONG_ACCOUNT.department).toBe('Ban Điều hành C-Suite');
    });
  });

  describe('Route Access Control (canAccessRoute & permittedRoutes)', () => {
    it('verifies MAKER role permissions (maker_nam)', () => {
      const auth = useAuthStore();
      auth.login('maker_nam', 'LivaMaker@2026');

      expect(auth.permittedRoutes).toEqual(ROLE_PERMITTED_ROUTES.MAKER);
      expect(auth.canAccessRoute('overview')).toBe(true);
      expect(auth.canAccessRoute('workbench')).toBe(true);
      expect(auth.canAccessRoute('quarantine')).toBe(true);
      expect(auth.canAccessRoute('ledger')).toBe(true);
      expect(auth.canAccessRoute('reconcile')).toBe(true);
      expect(auth.canAccessRoute('transactions')).toBe(true);
      expect(auth.canAccessRoute('cashflow')).toBe(true);
      expect(auth.canAccessRoute('treasury')).toBe(true);

      // Prohibited routes for Maker
      expect(auth.canAccessRoute('compliance')).toBe(false);
      expect(auth.canAccessRoute('risk')).toBe(false);
      expect(auth.canAccessRoute('reports')).toBe(false);
      expect(auth.canAccessRoute('settings')).toBe(false);
      expect(auth.isReadOnly).toBe(false);
    });

    it('verifies CHECKER role permissions (checker_tri)', () => {
      const auth = useAuthStore();
      auth.login('checker_tri', 'LivaChecker@2026');

      expect(auth.permittedRoutes).toEqual(ROLE_PERMITTED_ROUTES.CHECKER);
      expect(auth.canAccessRoute('overview')).toBe(true);
      expect(auth.canAccessRoute('quarantine')).toBe(true);
      expect(auth.canAccessRoute('ledger')).toBe(true);
      expect(auth.canAccessRoute('treasury')).toBe(true);
      expect(auth.canAccessRoute('compliance')).toBe(true);
      expect(auth.canAccessRoute('risk')).toBe(true);
      expect(auth.canAccessRoute('reports')).toBe(true);

      // Prohibited routes for Checker
      expect(auth.canAccessRoute('workbench')).toBe(false);
      expect(auth.canAccessRoute('reconcile')).toBe(false);
      expect(auth.canAccessRoute('transactions')).toBe(false);
      expect(auth.canAccessRoute('cashflow')).toBe(false);
      expect(auth.canAccessRoute('settings')).toBe(false);
      expect(auth.isReadOnly).toBe(false);
    });

    it('verifies CFO role permissions (cfo_huong / cfo_hoang)', () => {
      const auth = useAuthStore();
      auth.login('cfo_huong', 'LivaCfo@2026');

      expect(auth.permittedRoutes).toEqual(ROLE_PERMITTED_ROUTES.CFO);
      expect(auth.canAccessRoute('overview')).toBe(true);
      expect(auth.canAccessRoute('treasury')).toBe(true);
      expect(auth.canAccessRoute('cashflow')).toBe(true);
      expect(auth.canAccessRoute('quarantine')).toBe(true);
      expect(auth.canAccessRoute('risk')).toBe(true);
      expect(auth.canAccessRoute('reports')).toBe(true);

      // Prohibited routes for CFO
      expect(auth.canAccessRoute('workbench')).toBe(false);
      expect(auth.canAccessRoute('reconcile')).toBe(false);
      expect(auth.canAccessRoute('ledger')).toBe(false);
      expect(auth.canAccessRoute('transactions')).toBe(false);
      expect(auth.canAccessRoute('compliance')).toBe(false);
      expect(auth.canAccessRoute('settings')).toBe(false);
      expect(auth.isReadOnly).toBe(false);
    });

    it('verifies AUDITOR role permissions and isReadOnly flag (auditor_lan)', () => {
      const auth = useAuthStore();
      auth.login('auditor_lan', 'LivaAudit@2026');

      expect(auth.permittedRoutes).toEqual(ROLE_PERMITTED_ROUTES.AUDITOR);
      expect(auth.canAccessRoute('overview')).toBe(true);
      expect(auth.canAccessRoute('compliance')).toBe(true);
      expect(auth.canAccessRoute('ledger')).toBe(true);
      expect(auth.canAccessRoute('transactions')).toBe(true);
      expect(auth.canAccessRoute('reports')).toBe(true);

      // Prohibited routes for Auditor
      expect(auth.canAccessRoute('workbench')).toBe(false);
      expect(auth.canAccessRoute('quarantine')).toBe(false);
      expect(auth.canAccessRoute('reconcile')).toBe(false);
      expect(auth.canAccessRoute('cashflow')).toBe(false);
      expect(auth.canAccessRoute('treasury')).toBe(false);
      expect(auth.canAccessRoute('risk')).toBe(false);
      expect(auth.canAccessRoute('settings')).toBe(false);

      // Read-Only mode active for Auditor
      expect(auth.isReadOnly).toBe(true);
    });

    it('verifies ADMIN role permissions (admin_sys)', () => {
      const auth = useAuthStore();
      auth.login('admin_sys', 'LivaAdmin@2026');

      expect(auth.permittedRoutes).toEqual(ROLE_PERMITTED_ROUTES.ADMIN);
      expect(auth.canAccessRoute('overview')).toBe(true);
      expect(auth.canAccessRoute('settings')).toBe(true);

      // Prohibited financial routes for Admin (Zero financial mutation privilege)
      expect(auth.canAccessRoute('workbench')).toBe(false);
      expect(auth.canAccessRoute('quarantine')).toBe(false);
      expect(auth.canAccessRoute('ledger')).toBe(false);
      expect(auth.canAccessRoute('reconcile')).toBe(false);
      expect(auth.canAccessRoute('transactions')).toBe(false);
      expect(auth.canAccessRoute('cashflow')).toBe(false);
      expect(auth.canAccessRoute('treasury')).toBe(false);
      expect(auth.canAccessRoute('compliance')).toBe(false);
      expect(auth.canAccessRoute('risk')).toBe(false);
      expect(auth.canAccessRoute('reports')).toBe(false);
      expect(auth.isReadOnly).toBe(false);
    });

    it('handles empty and invalid route IDs gracefully in canAccessRoute', () => {
      const auth = useAuthStore();
      expect(auth.canAccessRoute('')).toBe(false);
      expect(auth.canAccessRoute('nonexistent_route_xyz')).toBe(false);
      // @ts-expect-error - testing invalid parameter types
      expect(auth.canAccessRoute(null)).toBe(false);
      // @ts-expect-error - testing invalid parameter types
      expect(auth.canAccessRoute(undefined)).toBe(false);
    });
  });
});
