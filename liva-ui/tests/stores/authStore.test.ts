import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useAuthStore, DEMO_ACCOUNTS } from '../../src/stores/authStore';

describe('authStore (RBAC & Multi-Role Authentication)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
  });

  it('initializes with default user (Nguyễn Minh Trí - CHECKER)', () => {
    const auth = useAuthStore();
    expect(auth.currentUser.username).toBe('checker_tri');
    expect(auth.currentUser.fullName).toBe('Nguyễn Minh Trí');
    expect(auth.currentUser.role).toBe('CHECKER');
    expect(auth.isChecker).toBe(true);
    expect(auth.isMaker).toBe(false);
    expect(auth.isAuthenticated).toBe(true);
  });

  it('contains all 7 demo accounts covering 5 distinct roles', () => {
    const auth = useAuthStore();
    expect(auth.accounts).toHaveLength(7);

    const roles = auth.accounts.map((a) => a.role);
    expect(roles).toContain('MAKER');
    expect(roles).toContain('CHECKER');
    expect(roles).toContain('CFO');
    expect(roles).toContain('AUDITOR');
    expect(roles).toContain('ADMIN');
  });

  it('authenticates valid credentials successfully', () => {
    const auth = useAuthStore();
    const res = auth.login('maker_nam', 'LivaMaker@2026');
    expect(res.success).toBe(true);
    expect(auth.currentUser.username).toBe('maker_nam');
    expect(auth.currentUser.role).toBe('MAKER');
    expect(auth.isMaker).toBe(true);
    expect(auth.isChecker).toBe(false);
    expect(auth.isAuthenticated).toBe(true);
  });

  it('supports case-insensitive username login', () => {
    const auth = useAuthStore();
    const res = auth.login('  CFO_HOANG  ', 'LivaCfo@2026');
    expect(res.success).toBe(true);
    expect(auth.currentUser.username).toBe('cfo_hoang');
    expect(auth.currentUser.role).toBe('CFO');
    expect(auth.isCfo).toBe(true);
  });

  it('rejects unknown usernames with descriptive error', () => {
    const auth = useAuthStore();
    const res = auth.login('unknown_hacker', 'Pass123');
    expect(res.success).toBe(false);
    expect(res.error).toContain('không tồn tại');
  });

  it('rejects incorrect password with descriptive error', () => {
    const auth = useAuthStore();
    const res = auth.login('maker_nam', 'WrongPassword');
    expect(res.success).toBe(false);
    expect(res.error).toContain('Mật khẩu không chính xác');
  });

  it('switches user instantly via switchUser()', () => {
    const auth = useAuthStore();
    const ok = auth.switchUser('auditor_lan');
    expect(ok).toBe(true);
    expect(auth.currentUser.username).toBe('auditor_lan');
    expect(auth.currentUser.role).toBe('AUDITOR');
    expect(auth.isAuditor).toBe(true);
    expect(auth.currentUser.department).toBe('Ban Kiểm toán & Tuân thủ');

    const fail = auth.switchUser('nonexistent');
    expect(fail).toBe(false);
  });

  it('manages logout and modal state securely', () => {
    const auth = useAuthStore();
    expect(auth.isAuthenticated).toBe(true);

    auth.logout();
    expect(auth.isAuthenticated).toBe(false);
    expect(auth.isLoginModalOpen).toBe(true);

    // Cannot close modal when unauthenticated
    auth.closeLoginModal();
    expect(auth.isLoginModalOpen).toBe(true);

    // Re-login closes modal
    const res = auth.login('admin_sys', 'LivaAdmin@2026');
    expect(res.success).toBe(true);
    expect(auth.isLoginModalOpen).toBe(false);
    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isAdmin).toBe(true);
  });

  it('fetches quick accounts via fetchAccounts() without throwing', async () => {
    const auth = useAuthStore();
    const accounts = await auth.fetchAccounts();
    expect(Array.isArray(accounts)).toBe(true);
    expect(accounts.length).toBeGreaterThanOrEqual(7);
    expect(accounts.some((a) => a.username === 'checker_tri')).toBe(true);
  });

  it('authenticates via loginAsync() for valid credentials', async () => {
    const auth = useAuthStore();
    const res = await auth.loginAsync('maker_mai', 'LivaMaker@2026');
    expect(res.success).toBe(true);
    expect(auth.currentUser.username).toBe('maker_mai');
    expect(auth.currentUser.role).toBe('MAKER');
    expect(auth.isMaker).toBe(true);
  });

  it('supports quick login via loginAsync() without password', async () => {
    const auth = useAuthStore();
    const res = await auth.loginAsync('cfo_hoang', undefined, true);
    expect(res.success).toBe(true);
    expect(auth.currentUser.username).toBe('cfo_hoang');
    expect(auth.isCfo).toBe(true);
  });

  it('rejects invalid password via loginAsync()', async () => {
    const auth = useAuthStore();
    const res = await auth.loginAsync('cfo_hoang', 'WrongPass', false);
    expect(res.success).toBe(false);
    expect(res.error).toBeDefined();
  });
});
