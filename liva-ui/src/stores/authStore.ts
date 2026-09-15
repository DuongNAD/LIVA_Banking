import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invokeBackend } from '../utils/ipc';

export type UserRole = 'MAKER' | 'CHECKER' | 'CFO' | 'AUDITOR' | 'ADMIN';

export interface UserProfile {
  id: string;
  username: string;
  fullName: string;
  role: UserRole;
  roleTitle: string;
  department: string;
  description: string;
  avatarInitials: string;
  avatarColor: string;
}

export const DEMO_ACCOUNTS: UserProfile[] = [
  {
    id: 'KT_TRINH_VAN_NAM',
    username: 'maker_nam',
    fullName: 'Trịnh Văn Nam',
    role: 'MAKER',
    roleTitle: 'Kế toán viên (Maker)',
    department: 'Phòng Kế toán Vốn',
    description: 'Tải sao kê, lập đề xuất xử lý lệch, tạo lệnh chi tiền',
    avatarInitials: 'VN',
    avatarColor: '#2563eb',
  },
  {
    id: 'KT_LE_PHUONG_MAI',
    username: 'maker_mai',
    fullName: 'Lê Phương Mai',
    role: 'MAKER',
    roleTitle: 'Kế toán viên (Maker)',
    department: 'Phòng Kế toán Thanh toán',
    description: 'Nhập liệu giao dịch, điều hòa hóa đơn bán hàng',
    avatarInitials: 'PM',
    avatarColor: '#0284c7',
  },
  {
    id: 'KTT_NGUYEN_MINH_TRI',
    username: 'checker_tri',
    fullName: 'Nguyễn Minh Trí',
    role: 'CHECKER',
    roleTitle: 'Kế toán trưởng (Checker)',
    department: 'Ban Giám đốc Tài chính - Kế toán',
    description: 'Phê duyệt ngoại lệ đối soát (4-Eyes HITL), duyệt chi',
    avatarInitials: 'MT',
    avatarColor: '#059669',
  },
  {
    id: 'KTT_DO_LAN_HUONG',
    username: 'checker_huong',
    fullName: 'Đỗ Lan Hương',
    role: 'CHECKER',
    roleTitle: 'Phó phòng Kế toán (Checker)',
    department: 'Ban Kiểm soát Kế toán',
    description: 'Kiểm soát viên độc lập, ký duyệt lệnh chi',
    avatarInitials: 'LH',
    avatarColor: '#0d9488',
  },
  {
    id: 'CFO_TRAN_VIET_HOANG',
    username: 'cfo_hoang',
    fullName: 'Trần Việt Hoàng',
    role: 'CFO',
    roleTitle: 'Giám đốc Tài chính (CFO)',
    department: 'Ban Điều hành C-Suite',
    description: 'Giám sát ngân quỹ 30/90 ngày, quản trị hạn mức thanh khoản',
    avatarInitials: 'VH',
    avatarColor: '#7c3aed',
  },
  {
    id: 'AUDIT_PHAM_HUONG_LAN',
    username: 'auditor_lan',
    fullName: 'Phạm Hương Lan',
    role: 'AUDITOR',
    roleTitle: 'Kiểm toán viên (Auditor)',
    department: 'Ban Kiểm toán & Tuân thủ',
    description: 'Quyền chỉ đọc, kiểm tra Merkle Audit Chain & Nghị định 13',
    avatarInitials: 'HL',
    avatarColor: '#d97706',
  },
  {
    id: 'ADMIN_HE_THONG',
    username: 'admin_sys',
    fullName: 'Quản trị viên An ninh',
    role: 'ADMIN',
    roleTitle: 'Quản trị Hệ thống (Admin)',
    department: 'Trung tâm An toàn Thông tin (SOC)',
    description: 'Cấu hình bảo mật Zero-Egress, AI offline, nhật ký vận hành',
    avatarInitials: 'AD',
    avatarColor: '#dc2626',
  },
];

const STORAGE_KEY = 'liva_banking_active_user';

const DEV_FALLBACK_PASSWORDS: Record<string, string> = {
  maker_nam: 'LivaMaker@2026',
  maker_mai: 'LivaMaker@2026',
  checker_tri: 'LivaChecker@2026',
  checker_huong: 'LivaChecker@2026',
  cfo_hoang: 'LivaCfo@2026',
  auditor_lan: 'LivaAudit@2026',
  admin_sys: 'LivaAdmin@2026',
};

export interface LoginResult {
  success: boolean;
  error?: string;
  user?: UserProfile;
}

export function mapRawUser(raw: any): UserProfile {
  return {
    id: String(raw?.id || ''),
    username: String(raw?.username || ''),
    fullName: String(raw?.full_name || raw?.fullName || ''),
    role: (raw?.role || 'MAKER') as UserRole,
    roleTitle: String(raw?.role_title || raw?.roleTitle || ''),
    department: String(raw?.department || ''),
    description: String(raw?.description || ''),
    avatarInitials: String(raw?.avatar_initials || raw?.avatarInitials || ''),
    avatarColor: String(raw?.avatar_color || raw?.avatarColor || '#2563eb'),
  };
}

export const useAuthStore = defineStore('auth', () => {
  const accounts = ref<UserProfile[]>([...DEMO_ACCOUNTS]);

  // Try restore from localStorage or fallback to checker_tri
  const initialUser = (() => {
    if (typeof window !== 'undefined' && window.localStorage) {
      try {
        const savedUsername = window.localStorage.getItem(STORAGE_KEY);
        if (savedUsername) {
          const match = DEMO_ACCOUNTS.find((u) => u.username === savedUsername);
          if (match) return match;
        }
      } catch {
        // Ignore localStorage access errors
      }
    }
    return DEMO_ACCOUNTS[2]; // Default: Nguyễn Minh Trí (Checker)
  })();

  const currentUser = ref<UserProfile>(initialUser);
  const isAuthenticated = ref<boolean>(true);
  const isLoginModalOpen = ref<boolean>(false);
  const isSyncing = ref<boolean>(false);

  const isMaker = computed(() => currentUser.value.role === 'MAKER');
  const isChecker = computed(() => currentUser.value.role === 'CHECKER');
  const isCfo = computed(() => currentUser.value.role === 'CFO');
  const isAuditor = computed(() => currentUser.value.role === 'AUDITOR');
  const isAdmin = computed(() => currentUser.value.role === 'ADMIN');

  function saveSession(user: UserProfile) {
    currentUser.value = user;
    isAuthenticated.value = true;
    if (typeof window !== 'undefined' && window.localStorage) {
      try {
        window.localStorage.setItem(STORAGE_KEY, user.username);
      } catch {
        // Ignore
      }
    }
  }

  async function fetchAccounts(): Promise<UserProfile[]> {
    isSyncing.value = true;
    try {
      let res: any = null;
      try {
        res = await invokeBackend<any>('auth_get_quick_accounts');
      } catch {
        try {
          res = await invokeBackend<any>('native_ipc_call', {
            command: 'auth:get_quick_accounts',
            payload: {},
          });
        } catch {
          res = null;
        }
      }

      const rawList = Array.isArray(res)
        ? res
        : (res && Array.isArray(res.accounts))
          ? res.accounts
          : [];

      if (rawList.length > 0) {
        const mapped = rawList.map(mapRawUser);
        accounts.value = mapped;

        // If current user is in accounts, update it
        const currentMatch = mapped.find(
          (u: UserProfile) => u.username.toLowerCase() === currentUser.value.username.toLowerCase()
        );
        if (currentMatch) {
          currentUser.value = currentMatch;
        }
        return mapped;
      }
    } catch {
      // Keep accounts unchanged on error
    } finally {
      isSyncing.value = false;
    }
    return accounts.value;
  }

  function login(username: string, password?: string, quickLogin?: boolean): LoginResult {
    const trimmedUser = username.trim().toLowerCase();
    const found = accounts.value.find((u) => u.username.toLowerCase() === trimmedUser);

    if (!found) {
      return { success: false, error: 'Tên đăng nhập không tồn tại trong hệ thống.' };
    }

    if (!quickLogin) {
      if (!password) {
        return { success: false, error: 'Vui lòng nhập mật khẩu.' };
      }
      const expectedPass = DEV_FALLBACK_PASSWORDS[trimmedUser];
      if (!expectedPass || expectedPass !== password) {
        return { success: false, error: 'Mật khẩu không chính xác.' };
      }
    }

    saveSession(found);
    isLoginModalOpen.value = false;
    return { success: true, user: found };
  }

  async function loginAsync(
    username: string,
    password?: string,
    quickLogin?: boolean
  ): Promise<LoginResult> {
    const trimmedUser = username.trim().toLowerCase();
    try {
      let backendRes: any = null;
      try {
        backendRes = await invokeBackend<any>('auth_login', {
          username: trimmedUser,
          password: password || null,
          quick_login: Boolean(quickLogin),
        });
      } catch {
        try {
          backendRes = await invokeBackend<any>('native_ipc_call', {
            command: 'auth:login',
            payload: {
              username: trimmedUser,
              password: password || null,
              quick_login: Boolean(quickLogin),
            },
          });
        } catch {
          backendRes = null;
        }
      }

      if (backendRes && typeof backendRes === 'object') {
        if (backendRes.success && backendRes.user) {
          const mapped = mapRawUser(backendRes.user);
          saveSession(mapped);
          isLoginModalOpen.value = false;
          return { success: true, user: mapped };
        } else if (backendRes.success === false) {
          return { success: false, error: backendRes.error || 'Đăng nhập thất bại.' };
        }
      }
    } catch {
      // Fallback to local login
    }
    return login(username, password, quickLogin);
  }

  function switchUser(username: string): boolean {
    const trimmed = username.trim().toLowerCase();
    const found = accounts.value.find((u) => u.username.toLowerCase() === trimmed);
    if (!found) return false;
    saveSession(found);
    return true;
  }

  function logout() {
    isAuthenticated.value = false;
    isLoginModalOpen.value = true;
  }

  function openLoginModal() {
    isLoginModalOpen.value = true;
  }

  function closeLoginModal() {
    // Only allow closing if currently authenticated
    if (isAuthenticated.value) {
      isLoginModalOpen.value = false;
    }
  }

  return {
    accounts,
    currentUser,
    isAuthenticated,
    isLoginModalOpen,
    isSyncing,
    isMaker,
    isChecker,
    isCfo,
    isAuditor,
    isAdmin,
    fetchAccounts,
    login,
    loginAsync,
    switchUser,
    logout,
    openLoginModal,
    closeLoginModal,
  };
});
