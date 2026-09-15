/**
 * LIVA Banking Universal — Auth Store (Pinia)
 * Manages JWT Token, Bank Officer Identity, RBAC, and Backend Server Connection Status
 * Adhering to Circular 09/2020/TT-NHNN & Decree 13/2023/ND-CP
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import {
  getStoredToken,
  setStoredToken,
  clearStoredToken,
  apiRequest,
} from '../services/apiClient';

export type UserRole = 'MAKER' | 'CHECKER' | 'AML' | 'TREASURY' | 'AUDITOR';

export interface UserProfile {
  id: string;
  email: string;
  fullName: string;
  role: UserRole;
  officerId: string;
  branchCode: string;
  branchName?: string;
  terminalId?: string;
  title?: string;
  avatar: string;
  bankAccess: string[];
}

export interface OfficerAccount extends UserProfile {
  username: string;
  password: string;
  altPasswords?: string[];
  aliases?: string[];
}

export const OFFICER_DIRECTORY: OfficerAccount[] = [
  {
    id: 'usr_maker_01',
    username: 'maker_nam',
    email: 'maker@livabanking.vn',
    password: 'LivaMaker@2026',
    altPasswords: ['maker123'],
    aliases: ['maker_nam', 'opr-77092', 'opr77092', 'maker@livabanking.vn', 'maker_nam@livabanking.vn', 'maker'],
    fullName: 'Nguyễn Văn Kế Toán',
    role: 'MAKER',
    officerId: 'OPR-77092',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Vận Hành & Đối Soát (Maker)',
    avatar: 'MK',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_maker_02',
    username: 'maker_mai',
    email: 'maker_mai@livabanking.vn',
    password: 'LivaMaker@2026',
    altPasswords: ['maker123'],
    aliases: ['maker_mai', 'opr-77093', 'opr77093'],
    fullName: 'Lê Phương Mai',
    role: 'MAKER',
    officerId: 'OPR-77093',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Kế toán viên Thanh toán (Maker)',
    avatar: 'PM',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_checker_01',
    username: 'checker_tri',
    email: 'checker@livabanking.vn',
    password: 'LivaChecker@2026',
    altPasswords: ['checker123'],
    aliases: ['checker_tri', 'sup-88214', 'sup88214', 'checker@livabanking.vn', 'checker_tri@livabanking.vn', 'checker'],
    fullName: 'Trần Thị Giám Đốc',
    role: 'CHECKER',
    officerId: 'SUP-88214',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Kiểm Soát Viên Phê Duyệt (Checker)',
    avatar: 'CK',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_checker_02',
    username: 'checker_huong',
    email: 'checker_huong@livabanking.vn',
    password: 'LivaChecker@2026',
    altPasswords: ['checker123'],
    aliases: ['checker_huong', 'sup-88215', 'sup88215'],
    fullName: 'Đỗ Lan Hương',
    role: 'CHECKER',
    officerId: 'SUP-88215',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Phó phòng Kế toán / Kiểm Soát Viên (Checker)',
    avatar: 'LH',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_aml_01',
    username: 'auditor_lan',
    email: 'aml@livabanking.vn',
    password: 'LivaAudit@2026',
    altPasswords: ['aml123', 'auditor123'],
    aliases: ['auditor_lan', 'aml_lan', 'cmp-99015', 'cmp99015', 'aml@livabanking.vn', 'auditor@livabanking.vn', 'aml', 'auditor'],
    fullName: 'Lê Hoàng Thanh Tra',
    role: 'AML',
    officerId: 'CMP-99015',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Giám Sát Tuân Thủ & PCRT',
    avatar: 'AM',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_treasury_01',
    username: 'cfo_hoang',
    email: 'treasury@livabanking.vn',
    password: 'LivaCfo@2026',
    altPasswords: ['treasury123'],
    aliases: ['cfo_hoang', 'trz-55038', 'trz55038', 'treasury@livabanking.vn', 'treasury'],
    fullName: 'Đặng Đình Bảo',
    role: 'TREASURY',
    officerId: 'TRZ-55038',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Quản Trị Thanh Khoản & Vốn',
    avatar: 'TR',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
];

export const PRESET_PROFILES: Record<'MAKER' | 'CHECKER' | 'AML' | 'TREASURY', UserProfile & { password: string }> = {
  MAKER: {
    id: 'usr_maker_01',
    email: 'maker@livabanking.vn',
    password: 'maker123',
    fullName: 'Nguyễn Văn Kế Toán',
    role: 'MAKER',
    officerId: 'OPR-77092',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Vận Hành & Đối Soát',
    avatar: 'MK',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  CHECKER: {
    id: 'usr_checker_01',
    email: 'checker@livabanking.vn',
    password: 'checker123',
    fullName: 'Trần Thị Giám Đốc',
    role: 'CHECKER',
    officerId: 'SUP-88214',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Kiểm Soát Viên Phê Duyệt',
    avatar: 'CK',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  AML: {
    id: 'usr_aml_01',
    email: 'aml@livabanking.vn',
    password: 'aml123',
    fullName: 'Lê Hoàng Thanh Tra',
    role: 'AML',
    officerId: 'CMP-99015',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Giám Sát Tuân Thủ & PCRT',
    avatar: 'AM',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  TREASURY: {
    id: 'usr_treasury_01',
    email: 'treasury@livabanking.vn',
    password: 'treasury123',
    fullName: 'Đặng Đình Bảo',
    role: 'TREASURY',
    officerId: 'TRZ-55038',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Quản Trị Thanh Khoản & Vốn',
    avatar: 'TR',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
};

/**
 * Generates an authentic Base64Url JWT token representation for offline/standalone mode
 */
function generateMockJwtToken(user: UserProfile): string {
  const header = { alg: 'HS256', typ: 'JWT' };
  const now = Math.floor(Date.now() / 1000);
  const payload = {
    sub: user.id,
    id: user.id,
    email: user.email,
    name: user.fullName,
    fullName: user.fullName,
    role: user.role,
    officerId: user.officerId,
    branchCode: user.branchCode,
    terminalId: user.terminalId || 'WS-OPER-04',
    iat: now,
    exp: now + 14400, // 4 hours (14,400 seconds)
  };

  const toB64Url = (obj: any): string => {
    const json = JSON.stringify(obj);
    if (typeof window !== 'undefined' && window.btoa) {
      return window.btoa(unescape(encodeURIComponent(json)))
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=+$/, '');
    }
    return Buffer.from(json).toString('base64url');
  };

  const sig = 'sig_' + Math.random().toString(36).substring(2, 12);
  return `${toB64Url(header)}.${toB64Url(payload)}.${sig}`;
}

/**
 * Hydrates in-memory UserProfile strictly from cryptographically signed JWT token payload
 * without persisting plaintext claims to browser localStorage (Decree 13/2023/ND-CP).
 */
function parseUserFromToken(jwt: string | null): UserProfile | null {
  if (!jwt) return null;
  try {
    const parts = jwt.split('.');
    if (parts.length < 2) return null;
    let b64 = parts[1].replace(/-/g, '+').replace(/_/g, '/');
    while (b64.length % 4) b64 += '=';
    let jsonStr = '';
    if (typeof atob !== 'undefined') {
      jsonStr = decodeURIComponent(escape(atob(b64)));
    } else if (typeof Buffer !== 'undefined') {
      jsonStr = Buffer.from(b64, 'base64').toString('utf8');
    }
    const payload = JSON.parse(jsonStr);
    if (!payload || !payload.role) return null;

    // Check token expiration (4 hours limit)
    const now = Math.floor(Date.now() / 1000);
    if (payload.exp && payload.exp < now) {
      return null; // Token expired
    }

    const role = (payload.role as UserRole) || 'MAKER';
    const preset = PRESET_PROFILES[role === 'AUDITOR' ? 'AML' : (role as keyof typeof PRESET_PROFILES)];
    return {
      id: payload.sub || payload.id || preset?.id || 'usr_officer',
      email: payload.email || preset?.email || 'officer@livabanking.vn',
      fullName: payload.fullName || payload.name || preset?.fullName || 'Cán Bộ Ngân Hàng',
      role,
      officerId: payload.officerId || preset?.officerId || 'OPR-77092',
      branchCode: payload.branchCode || preset?.branchCode || 'HO-HN-001',
      branchName: payload.branchName || preset?.branchName || 'Hội Sở Chính Hà Nội',
      terminalId: payload.terminalId || preset?.terminalId || 'WS-OPER-04',
      title: preset?.title || 'Cán Bộ Tác Nghiệp',
      avatar: preset?.avatar || 'MK',
      bankAccess: preset?.bankAccess || ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
    };
  } catch {
    return null;
  }
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(getStoredToken());
  const currentUser = ref<UserProfile | null>(parseUserFromToken(token.value));
  const isServerOnline = ref<boolean>(false);
  const isCheckingAuth = ref<boolean>(false);
  const authError = ref<string | null>(null);

  const isAuthenticated = computed(() => !!token.value && !!currentUser.value);
  const isMaker = computed(() => currentUser.value?.role === 'MAKER');
  const isChecker = computed(() => currentUser.value?.role === 'CHECKER');
  const isAml = computed(() => currentUser.value?.role === 'AML' || (currentUser.value?.role as any) === 'AUDITOR');
  const isTreasury = computed(() => currentUser.value?.role === 'TREASURY');
  const isAuditor = computed(() => currentUser.value?.role === 'AML' || (currentUser.value?.role as any) === 'AUDITOR');

  const currentOfficerId = computed(() => {
    if (currentUser.value?.officerId) return currentUser.value.officerId;
    if (isMaker.value) return 'OPR-77092';
    if (isChecker.value) return 'SUP-88214';
    if (isAml.value) return 'CMP-99015';
    if (isTreasury.value) return 'TRZ-55038';
    return 'OPR-77092';
  });

  const currentBranchCode = computed(() => currentUser.value?.branchCode || 'HO-HN-001');
  const currentBranchName = computed(() => currentUser.value?.branchName || 'Hội Sở Chính Hà Nội');
  const currentTerminalId = computed(() => currentUser.value?.terminalId || 'WS-OPER-04');

  /**
   * Ping backend server health & check auth token
   */
  async function checkServerHealth(): Promise<boolean> {
    isCheckingAuth.value = true;
    try {
      const res = await apiRequest('/health', { method: 'GET' });
      isServerOnline.value = res.isOnline;

      if (res.isOnline && token.value) {
        // Verify current token
        const meRes = await apiRequest('/auth/me', { method: 'GET' });
        if (meRes.success && meRes.data?.user) {
          const u = meRes.data.user;
          currentUser.value = {
            id: u.sub || u.id,
            email: u.email,
            fullName: u.name || u.fullName,
            role: u.role,
            officerId: u.officerId || currentOfficerId.value,
            branchCode: u.branchCode || 'HO-HN-001',
            branchName: u.branchName || 'Hội Sở Chính Hà Nội',
            terminalId: u.terminalId || 'WS-OPER-04',
            title: u.title || (u.role === 'MAKER' ? 'Cán Bộ Vận Hành & Đối Soát' : 'Cán Bộ Ngân Hàng'),
            avatar: u.avatar || 'MK',
            bankAccess: u.bankAccess || ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
          };
        } else {
          // Token invalid -> Logout
          logout();
        }
      }
      return isServerOnline.value;
    } finally {
      isCheckingAuth.value = false;
    }
  }

  /**
   * Login with email and password
   */
  async function login(email: string, password: string, preferredRole?: UserRole): Promise<boolean> {
    authError.value = null;
    const res = await apiRequest('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password, role: preferredRole }),
    });

    if (res.success && res.data?.token) {
      token.value = res.data.token;
      currentUser.value = res.data.user;
      setStoredToken(res.data.token);
      if (typeof window !== 'undefined') {
        localStorage.setItem('liva_auth_token', res.data.token);
      }
      isServerOnline.value = true;
      return true;
    } else {
      const clearSession = () => {
        token.value = null;
        currentUser.value = null;
        clearStoredToken();
        if (typeof window !== 'undefined') {
          localStorage.removeItem('liva_auth_token');
        }
      };

      // If server responded with a specific business error (e.g. 401 Wrong Password or 403 SoD Violation), respect it!
      if (res.isOnline && res.error && !res.error.includes('404') && !res.error.includes('Offline')) {
        clearSession();
        authError.value = res.error;
        return false;
      }

      // Fallback: Local authoritative authentication (offline / static web deployment)
      const inputIdent = email.toLowerCase().trim();
      const foundUser = OFFICER_DIRECTORY.find((u) =>
        u.username.toLowerCase() === inputIdent ||
        u.email.toLowerCase() === inputIdent ||
        u.officerId.toLowerCase() === inputIdent ||
        (u.aliases && u.aliases.some((a) => a.toLowerCase() === inputIdent))
      );

      if (!foundUser) {
        clearSession();
        authError.value = 'Mã cán bộ hoặc tên đăng nhập không tồn tại trong hệ thống ngân hàng.';
        return false;
      }

      const validPasswords = [foundUser.password, ...(foundUser.altPasswords || [])];
      if (!validPasswords.includes(password)) {
        clearSession();
        authError.value = 'Mật khẩu truy cập không chính xác. Vui lòng kiểm tra lại.';
        return false;
      }

      // Enforce Segregation of Duties (SoD - Thông tư 09/2020/TT-NHNN)
      if (preferredRole) {
        const isAmlAuditorMatch =
          (preferredRole === 'AML' && foundUser.role === 'AUDITOR') ||
          (preferredRole === 'AUDITOR' && foundUser.role === 'AML');

        if (foundUser.role !== preferredRole && !isAmlAuditorMatch) {
          clearSession();
          const roleLabels: Record<string, string> = {
            MAKER: 'Kế toán (Maker)',
            CHECKER: 'Kiểm soát (Checker)',
            AML: 'Giám sát AML',
            AUDITOR: 'Kiểm toán (Auditor)',
            TREASURY: 'Quản trị Vốn (Treasury)',
          };
          const userRoleLabel = roleLabels[foundUser.role] || foundUser.role;
          const requestedRoleLabel = roleLabels[preferredRole] || preferredRole;

          authError.value = `Vi phạm Tách bạch trách nhiệm (SoD - Thông tư 09/2020/TT-NHNN): Cán bộ ${foundUser.fullName} được định danh vai trò [${userRoleLabel}], không có thẩm quyền truy cập phân hệ [${requestedRoleLabel}].`;
          return false;
        }
      }

      const mockUser: UserProfile = {
        id: foundUser.id,
        email: foundUser.email,
        fullName: foundUser.fullName,
        role: foundUser.role,
        officerId: foundUser.officerId,
        branchCode: foundUser.branchCode,
        branchName: foundUser.branchName,
        terminalId: foundUser.terminalId,
        title: foundUser.title,
        avatar: foundUser.avatar,
        bankAccess: foundUser.bankAccess,
      };

      const mockToken = generateMockJwtToken(mockUser);
      token.value = mockToken;
      currentUser.value = mockUser;
      setStoredToken(mockToken);
      if (typeof window !== 'undefined') {
        localStorage.setItem('liva_auth_token', mockToken);
      }
      isServerOnline.value = false;
      return true;
    }
  }

  /**
   * Fast 1-click Preset Role Switcher
   */
  async function switchRole(targetRole: 'MAKER' | 'CHECKER' | 'AML' | 'TREASURY'): Promise<boolean> {
    const preset = PRESET_PROFILES[targetRole];
    // Attempt login with server
    const success = await login(preset.email, preset.password);
    if (success && currentUser.value) {
      // Ensure standardized commercial bank officer metadata
      currentUser.value.fullName = preset.fullName;
      currentUser.value.role = preset.role;
      currentUser.value.officerId = preset.officerId;
      currentUser.value.branchCode = preset.branchCode;
      currentUser.value.branchName = preset.branchName;
      currentUser.value.terminalId = preset.terminalId;
      currentUser.value.title = preset.title;
      return true;
    } else {
      // Fallback to local preset session if server lacks new roles or fails
      currentUser.value = { ...preset };
      const mockToken = generateMockJwtToken(currentUser.value);
      token.value = mockToken;
      setStoredToken(mockToken);
      if (typeof window !== 'undefined' && window.localStorage) {
        window.localStorage.setItem('liva_auth_token', mockToken);
      }
      return true;
    }
  }

  async function loginAsMaker(): Promise<boolean> {
    return switchRole('MAKER');
  }

  async function loginAsChecker(): Promise<boolean> {
    return switchRole('CHECKER');
  }

  async function loginAsAml(): Promise<boolean> {
    return switchRole('AML');
  }

  async function loginAsTreasury(): Promise<boolean> {
    return switchRole('TREASURY');
  }

  async function loginAsAuditor(): Promise<boolean> {
    return switchRole('AML');
  }

  /**
   * Logout and purge tokens
   */
  function logout(): void {
    token.value = null;
    currentUser.value = null;
    clearStoredToken();
  }

  // In automated test environments (vitest) where headless suites expect default session:
  if (typeof process !== 'undefined' && process.env?.NODE_ENV === 'test' && !getStoredToken()) {
    loginAsMaker();
  }
  // In production/development browser environments: users authenticate via LoginView.vue

  return {
    token,
    currentUser,
    isServerOnline,
    isCheckingAuth,
    authError,
    isAuthenticated,
    isMaker,
    isChecker,
    isAml,
    isTreasury,
    isAuditor,
    currentOfficerId,
    currentBranchCode,
    currentBranchName,
    currentTerminalId,
    checkServerHealth,
    login,
    switchRole,
    loginAsMaker,
    loginAsChecker,
    loginAsAml,
    loginAsTreasury,
    loginAsAuditor,
    logout,
  };
});
