/**
 * LIVA Banking Universal — Auth Store & Core Banking Officer RBAC Tests (Milestone M1)
 * Adhering to Circular 09/2020/TT-NHNN & Decree 13/2023/ND-CP
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

// Polyfill window and localStorage for Node.js test environment
const store = new Map<string, string>();
const mockStorage = {
  getItem: (k: string) => (store.has(k) ? store.get(k)! : null),
  setItem: (k: string, v: string) => store.set(k, String(v)),
  removeItem: (k: string) => store.delete(k),
  clear: () => store.clear(),
  key: (i: number) => Array.from(store.keys())[i] ?? null,
  get length() {
    return store.size;
  },
};

(globalThis as any).localStorage = mockStorage;
(globalThis as any).window = {
  location: { hostname: 'localhost' },
  localStorage: mockStorage,
};

import { useAuthStore, PRESET_PROFILES } from '../src/stores/authStore';

describe('Auth Store & Core Banking RBAC Role Switcher (Milestone M1)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockStorage.clear();
  });

  it('defines pre-seeded profiles for all 4 commercial bank staff roles', () => {
    expect(PRESET_PROFILES.MAKER).toBeDefined();
    expect(PRESET_PROFILES.CHECKER).toBeDefined();
    expect(PRESET_PROFILES.AML).toBeDefined();
    expect(PRESET_PROFILES.TREASURY).toBeDefined();

    // Verify MAKER
    expect(PRESET_PROFILES.MAKER.id).toBe('usr_maker_01');
    expect(PRESET_PROFILES.MAKER.email).toBe('maker@livabanking.vn');
    expect(PRESET_PROFILES.MAKER.fullName).toBe('Nguyễn Văn Kế Toán');
    expect(PRESET_PROFILES.MAKER.officerId).toBe('OPR-77092');
    expect(PRESET_PROFILES.MAKER.branchCode).toBe('HO-HN-001');

    // Verify CHECKER
    expect(PRESET_PROFILES.CHECKER.id).toBe('usr_checker_01');
    expect(PRESET_PROFILES.CHECKER.email).toBe('checker@livabanking.vn');
    expect(PRESET_PROFILES.CHECKER.fullName).toBe('Trần Thị Giám Đốc');
    expect(PRESET_PROFILES.CHECKER.officerId).toBe('SUP-88214');
    expect(PRESET_PROFILES.CHECKER.branchCode).toBe('HO-HN-001');

    // Verify AML
    expect(PRESET_PROFILES.AML.id).toBe('usr_aml_01');
    expect(PRESET_PROFILES.AML.email).toBe('aml@livabanking.vn');
    expect(PRESET_PROFILES.AML.fullName).toBe('Lê Hoàng Thanh Tra');
    expect(PRESET_PROFILES.AML.officerId).toBe('CMP-99015');
    expect(PRESET_PROFILES.AML.branchCode).toBe('HO-HN-001');

    // Verify TREASURY
    expect(PRESET_PROFILES.TREASURY.id).toBe('usr_treasury_01');
    expect(PRESET_PROFILES.TREASURY.email).toBe('treasury@livabanking.vn');
    expect(PRESET_PROFILES.TREASURY.fullName).toBe('Đặng Đình Bảo');
    expect(PRESET_PROFILES.TREASURY.officerId).toBe('TRZ-55038');
    expect(PRESET_PROFILES.TREASURY.branchCode).toBe('HO-HN-001');
  });

  it('switches role to MAKER and stores liva_auth_token in localStorage', async () => {
    const auth = useAuthStore();
    await auth.switchRole('MAKER');

    expect(auth.isMaker).toBe(true);
    expect(auth.isChecker).toBe(false);
    expect(auth.isAml).toBe(false);
    expect(auth.isTreasury).toBe(false);
    expect(auth.currentOfficerId).toBe('OPR-77092');
    expect(auth.currentBranchCode).toBe('HO-HN-001');
    expect(auth.currentUser?.fullName).toBe('Nguyễn Văn Kế Toán');

    const storedToken = mockStorage.getItem('liva_auth_token');
    expect(storedToken).toBeDefined();
    expect(storedToken).not.toBeNull();
    expect(auth.token).toBe(storedToken);
  });

  it('switches role to CHECKER with proper officer metadata', async () => {
    const auth = useAuthStore();
    await auth.switchRole('CHECKER');

    expect(auth.isChecker).toBe(true);
    expect(auth.isMaker).toBe(false);
    expect(auth.currentOfficerId).toBe('SUP-88214');
    expect(auth.currentBranchCode).toBe('HO-HN-001');
    expect(auth.currentUser?.fullName).toBe('Trần Thị Giám Đốc');
    expect(mockStorage.getItem('liva_auth_token')).toBeDefined();
  });

  it('switches role to AML Specialist and supports auditor backwards-compatibility', async () => {
    const auth = useAuthStore();
    await auth.switchRole('AML');

    expect(auth.isAml).toBe(true);
    expect(auth.isAuditor).toBe(true);
    expect(auth.currentOfficerId).toBe('CMP-99015');
    expect(auth.currentUser?.fullName).toBe('Lê Hoàng Thanh Tra');
    expect(mockStorage.getItem('liva_auth_token')).toBeDefined();
  });

  it('switches role to Treasury Desk with TRZ-55038', async () => {
    const auth = useAuthStore();
    await auth.switchRole('TREASURY');

    expect(auth.isTreasury).toBe(true);
    expect(auth.currentOfficerId).toBe('TRZ-55038');
    expect(auth.currentUser?.fullName).toBe('Đặng Đình Bảo');
    expect(mockStorage.getItem('liva_auth_token')).toBeDefined();
  });

  it('purges token and resets user identity on logout', async () => {
    const auth = useAuthStore();
    await auth.switchRole('MAKER');
    expect(auth.isAuthenticated).toBe(true);

    auth.logout();
    expect(auth.token).toBeNull();
    expect(auth.currentUser).toBeNull();
    expect(auth.isAuthenticated).toBe(false);
    expect(mockStorage.getItem('liva_auth_token')).toBeNull();
  });

  it('authenticates maker_nam successfully with role MAKER', async () => {
    const auth = useAuthStore();
    const success = await auth.login('maker_nam', 'LivaMaker@2026', 'MAKER');

    expect(success).toBe(true);
    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isMaker).toBe(true);
    expect(auth.currentOfficerId).toBe('OPR-77092');
    expect(auth.authError).toBeNull();
  });

  it('rejects maker_nam when attempting to log in as CHECKER (SoD Violation)', async () => {
    const auth = useAuthStore();
    const success = await auth.login('maker_nam', 'LivaMaker@2026', 'CHECKER');

    expect(success).toBe(false);
    expect(auth.isAuthenticated).toBe(false);
    expect(auth.authError).toContain('Tách bạch trách nhiệm (SoD');
  });

  it('authenticates checker_tri successfully with role CHECKER', async () => {
    const auth = useAuthStore();
    const success = await auth.login('checker_tri', 'LivaChecker@2026', 'CHECKER');

    expect(success).toBe(true);
    expect(auth.isAuthenticated).toBe(true);
    expect(auth.isChecker).toBe(true);
    expect(auth.currentOfficerId).toBe('SUP-88214');
    expect(auth.authError).toBeNull();
  });

  it('rejects checker_tri when attempting to log in as MAKER (SoD Violation)', async () => {
    const auth = useAuthStore();
    const success = await auth.login('checker_tri', 'LivaChecker@2026', 'MAKER');

    expect(success).toBe(false);
    expect(auth.isAuthenticated).toBe(false);
    expect(auth.authError).toContain('Tách bạch trách nhiệm (SoD');
  });

  it('rejects incorrect password attempts', async () => {
    const auth = useAuthStore();
    const success = await auth.login('maker_nam', 'WrongPassword123', 'MAKER');

    expect(success).toBe(false);
    expect(auth.isAuthenticated).toBe(false);
    expect(auth.authError).toContain('Mật khẩu truy cập không chính xác');
  });
});
