import type {
  IPlatformAdapter,
  PlatformCapabilities,
  BankingOverviewResponse,
  ReconciliationMatrixResponse,
  HitlConfirmationPayload,
  HitlResolutionResponse,
  ComplianceStatusResponse,
  StatementIngestResult,
} from './IPlatformAdapter';
import { logger } from '../utils/logger';

export class MockWebAdapter implements IPlatformAdapter {
  readonly platformName = 'web' as const;
  readonly capabilities: PlatformCapabilities = {
    hasNativeFileSystem: false,
    hasNativeDialogs: false,
    hasHardwareKeystore: false,
    hasWindowControls: false,
    hasProcessControl: false,
    supportsStreamingUpload: true,
  };

  private readonly vaultSecretKeys = new Set<string>();

  constructor() {
    if (typeof document !== 'undefined') {
      document.body.classList.add('web-mock-mode');
    }
  }

  async getWindowSize() {
    return {
      width: typeof window !== 'undefined' ? window.innerWidth : 1280,
      height: typeof window !== 'undefined' ? window.innerHeight : 800,
    };
  }

  async toggleGhostMode(enabled: boolean) {
    logger.debug('[MockWebAdapter]', `Toggle Ghost Mode: ${enabled}`);
  }

  async minimizeToTray() {
    logger.debug('[MockWebAdapter]', 'Minimize to tray requested.');
  }

  async quitApp() {
    logger.debug('[MockWebAdapter]', 'Quit app requested. Closing window.');
    if (typeof window !== 'undefined') {
      window.close();
    }
  }

  async minimize() {
    return this.minimizeToTray();
  }

  async maximize() {
    logger.debug('[MockWebAdapter]', 'Maximize requested.');
  }

  async close() {
    return this.quitApp();
  }

  async hasVaultSecret(key: string) {
    return this.vaultSecretKeys.has(key);
  }

  async storeVaultSecret(key: string, value: string) {
    if (!value) throw new Error('vault secret must not be empty');
    this.vaultSecretKeys.add(key);
    logger.debug('[MockWebAdapter]', `Stored mock vault presence: ${key}`);
  }

  async deleteVaultSecret(key: string) {
    this.vaultSecretKeys.delete(key);
  }

  async hasSecret(key: string): Promise<boolean> {
    return this.hasVaultSecret(key);
  }

  async storeSecret(key: string, value: string): Promise<void> {
    return this.storeVaultSecret(key, value);
  }

  async deleteSecret(key: string): Promise<void> {
    return this.deleteVaultSecret(key);
  }

  onGatewayReady(callback: (port: number, token: string | null) => void) {
    logger.info('[MockWebAdapter]', 'Emulating GATEWAY_READY handshake on port 8002');
    setTimeout(() => {
      callback(8002, null);
    }, 1000);
  }

  async invokeBackend<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T> {
    logger.debug('[MockWebAdapter]', `Invoked command: ${command}`, args);
    const targetCmd = (command === 'native_ipc_call' && args?.command) ? String(args.command) : command;
    const payload = (command === 'native_ipc_call' && args?.payload) ? (args.payload as Record<string, unknown>) : args;

    if (targetCmd === 'auth_get_quick_accounts' || targetCmd === 'auth:get_quick_accounts') {
      const mockAccounts = [
        {
          id: 'KT_TRINH_VAN_NAM',
          username: 'maker_nam',
          full_name: 'Trịnh Văn Nam',
          role: 'MAKER',
          role_title: 'Kế toán viên (Maker)',
          department: 'Phòng Kế toán Vốn',
          description: 'Tải sao kê, lập đề xuất xử lý lệch, tạo lệnh chi tiền',
          avatar_initials: 'VN',
          avatar_color: '#2563eb',
          status: 'active',
        },
        {
          id: 'KT_LE_PHUONG_MAI',
          username: 'maker_mai',
          full_name: 'Lê Phương Mai',
          role: 'MAKER',
          role_title: 'Kế toán viên (Maker)',
          department: 'Phòng Kế toán Thanh toán',
          description: 'Nhập liệu giao dịch, điều hòa hóa đơn bán hàng',
          avatar_initials: 'PM',
          avatar_color: '#0284c7',
          status: 'active',
        },
        {
          id: 'KTT_NGUYEN_MINH_TRI',
          username: 'checker_tri',
          full_name: 'Nguyễn Minh Trí',
          role: 'CHECKER',
          role_title: 'Kế toán trưởng (Checker)',
          department: 'Ban Giám đốc Tài chính - Kế toán',
          description: 'Phê duyệt ngoại lệ đối soát (4-Eyes HITL), duyệt chi',
          avatar_initials: 'MT',
          avatar_color: '#059669',
          status: 'active',
        },
        {
          id: 'KTT_DO_LAN_HUONG',
          username: 'checker_huong',
          full_name: 'Đỗ Lan Hương',
          role: 'CHECKER',
          role_title: 'Phó phòng Kế toán (Checker)',
          department: 'Ban Kiểm soát Kế toán',
          description: 'Kiểm soát viên độc lập, ký duyệt lệnh chi',
          avatar_initials: 'LH',
          avatar_color: '#0d9488',
          status: 'active',
        },
        {
          id: 'CFO_TRAN_VIET_HOANG',
          username: 'cfo_hoang',
          full_name: 'Trần Việt Hoàng',
          role: 'CFO',
          role_title: 'Giám đốc Tài chính (CFO)',
          department: 'Ban Điều hành C-Suite',
          description: 'Giám sát ngân quỹ 30/90 ngày, quản trị hạn mức thanh khoản',
          avatar_initials: 'VH',
          avatar_color: '#7c3aed',
          status: 'active',
        },
        {
          id: 'AUDIT_PHAM_HUONG_LAN',
          username: 'auditor_lan',
          full_name: 'Phạm Hương Lan',
          role: 'AUDITOR',
          role_title: 'Kiểm toán viên (Auditor)',
          department: 'Ban Kiểm toán & Tuân thủ',
          description: 'Quyền chỉ đọc, kiểm tra Merkle Audit Chain & Nghị định 13',
          avatar_initials: 'HL',
          avatar_color: '#d97706',
          status: 'active',
        },
        {
          id: 'ADMIN_HE_THONG',
          username: 'admin_sys',
          full_name: 'Quản trị viên An ninh',
          role: 'ADMIN',
          role_title: 'Quản trị Hệ thống (Admin)',
          department: 'Trung tâm An toàn Thông tin (SOC)',
          description: 'Cấu hình bảo mật Zero-Egress, AI offline, nhật ký vận hành',
          avatar_initials: 'AD',
          avatar_color: '#dc2626',
          status: 'active',
        },
      ];
      return {
        success: true,
        accounts: mockAccounts,
      } as unknown as T;
    }

    if (targetCmd === 'auth_login' || targetCmd === 'auth:login') {
      const username = String(payload?.username || '').trim().toLowerCase();
      const password = payload?.password as string | undefined;
      const quickLogin = Boolean(payload?.quick_login || payload?.quickLogin);

      const mockUsers: Record<string, any> = {
        maker_nam: { id: 'KT_TRINH_VAN_NAM', full_name: 'Trịnh Văn Nam', role: 'MAKER', role_title: 'Kế toán viên (Maker)', department: 'Phòng Kế toán Vốn', description: 'Tải sao kê, lập đề xuất xử lý lệch, tạo lệnh chi tiền', avatar_initials: 'VN', avatar_color: '#2563eb', pass: 'LivaMaker@2026' },
        maker_mai: { id: 'KT_LE_PHUONG_MAI', full_name: 'Lê Phương Mai', role: 'MAKER', role_title: 'Kế toán viên (Maker)', department: 'Phòng Kế toán Thanh toán', description: 'Nhập liệu giao dịch, điều hòa hóa đơn bán hàng', avatar_initials: 'PM', avatar_color: '#0284c7', pass: 'LivaMaker@2026' },
        checker_tri: { id: 'KTT_NGUYEN_MINH_TRI', full_name: 'Nguyễn Minh Trí', role: 'CHECKER', role_title: 'Kế toán trưởng (Checker)', department: 'Ban Giám đốc Tài chính - Kế toán', description: 'Phê duyệt ngoại lệ đối soát (4-Eyes HITL), duyệt chi', avatar_initials: 'MT', avatar_color: '#059669', pass: 'LivaChecker@2026' },
        checker_huong: { id: 'KTT_DO_LAN_HUONG', full_name: 'Đỗ Lan Hương', role: 'CHECKER', role_title: 'Phó phòng Kế toán (Checker)', department: 'Ban Kiểm soát Kế toán', description: 'Kiểm soát viên độc lập, ký duyệt lệnh chi', avatar_initials: 'LH', avatar_color: '#0d9488', pass: 'LivaChecker@2026' },
        cfo_hoang: { id: 'CFO_TRAN_VIET_HOANG', full_name: 'Trần Việt Hoàng', role: 'CFO', role_title: 'Giám đốc Tài chính (CFO)', department: 'Ban Điều hành C-Suite', description: 'Giám sát ngân quỹ 30/90 ngày, quản trị hạn mức thanh khoản', avatar_initials: 'VH', avatar_color: '#7c3aed', pass: 'LivaCfo@2026' },
        auditor_lan: { id: 'AUDIT_PHAM_HUONG_LAN', full_name: 'Phạm Hương Lan', role: 'AUDITOR', role_title: 'Kiểm toán viên (Auditor)', department: 'Ban Kiểm toán & Tuân thủ', description: 'Quyền chỉ đọc, kiểm tra Merkle Audit Chain & Nghị định 13', avatar_initials: 'HL', avatar_color: '#d97706', pass: 'LivaAudit@2026' },
        admin_sys: { id: 'ADMIN_HE_THONG', full_name: 'Quản trị viên An ninh', role: 'ADMIN', role_title: 'Quản trị Hệ thống (Admin)', department: 'Trung tâm An toàn Thông tin (SOC)', description: 'Cấu hình bảo mật Zero-Egress, AI offline, nhật ký vận hành', avatar_initials: 'AD', avatar_color: '#dc2626', pass: 'LivaAdmin@2026' },
      };

      const found = mockUsers[username];
      if (!found) {
        return { success: false, error: 'Tên đăng nhập không tồn tại trong hệ thống.' } as unknown as T;
      }

      if (!quickLogin && password && found.pass !== password) {
        return { success: false, error: 'Mật khẩu không chính xác.' } as unknown as T;
      }

      return {
        success: true,
        user: {
          id: found.id,
          username,
          full_name: found.full_name,
          role: found.role,
          role_title: found.role_title,
          department: found.department,
          description: found.description,
          avatar_initials: found.avatar_initials,
          avatar_color: found.avatar_color,
          status: 'active',
        },
      } as unknown as T;
    }

    return null as unknown as T;
  }

  async getOverview(): Promise<BankingOverviewResponse> {
    return {
      vcb_balance: 1450230000,
      tcb_balance: 890400000,
      bidv_balance: 512000000,
      discrepancy_count: 3,
      matched_count: 1492,
      total_count: 1495,
      matched_ratio: 99.8,
      automatic_count: 1485,
      hitl_count: 7,
    };
  }

  async runReconciliation(): Promise<Record<string, unknown>> {
    return { success: true, processed: 1495, matched: 1492, discrepancies: 3 };
  }

  async getReconciliationMatrix(_filter?: string): Promise<ReconciliationMatrixResponse> {
    return { items: [], total_count: 0, unmatched_count: 0 };
  }

  async resolveHitl(payload: HitlConfirmationPayload): Promise<HitlResolutionResponse> {
    return {
      success: true,
      auditRecord: {
        timestamp: new Date().toISOString(),
        txCode: payload.txId,
        tokenUuid: payload.tokenUuid,
        action: payload.action,
        makerId: payload.makerId || 'maker_mock',
        checkerId: payload.checkerId || 'checker_mock',
        hash: `mock_audit_hash_${Date.now()}`,
      },
    };
  }

  async getComplianceStatus(): Promise<ComplianceStatusResponse> {
    return {
      decree_13_compliant: true,
      circular_09_compliant: true,
      zero_egress_verified: true,
      merkle_root_hash: '0xmock_root_hash',
      pii_scrubbed_count: 0,
    };
  }

  async ingestStatement(
    _fileInput: File | { name: string; size: number; path?: string; file?: File; content?: string | ArrayBuffer },
    onProgress?: (progressPercent: number, stage: string) => void
  ): Promise<StatementIngestResult> {
    onProgress?.(30, 'SCANNING');
    onProgress?.(60, 'EXTRACTING');
    onProgress?.(100, 'COMPLETED');
    return {
      success: true,
      total_transactions: 42,
      opening_balance: 1000000000,
      closing_balance: 1250000000,
    };
  }

  async subscribeEvents(
    _onEvent: (event: { event: string; payload: unknown }) => void
  ): Promise<() => void> {
    return () => {};
  }
}
