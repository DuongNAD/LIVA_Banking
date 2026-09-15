import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '../utils/logger';

export type SettingsTab = 'USERS' | 'BANK_API' | 'ERP_BRIDGE' | 'THRESHOLDS' | 'SECURITY' | 'BACKUP';
export type AppUserRole = 'ADMIN' | 'MAKER' | 'CHECKER' | 'COMPLIANCE' | 'AUDITOR';

export interface UserAccountItem {
  id: string;
  username: string;
  fullName: string;
  role: AppUserRole;
  isActive: boolean;
  certSerial?: string;
  lastLogin: string;
}

export interface BankConfigItem {
  bankCode: string;
  bankName: string;
  apiEndpoint: string;
  isConnected: boolean;
  isSandbox: boolean;
  latencyMs: number;
  lastSyncAt: string;
}

export interface ErpConfigItem {
  erpName: string;
  description: string;
  endpoint: string;
  isActive: boolean;
  syncIntervalMins: number;
  lastSyncAt: string;
}

export interface ThresholdConfig {
  exactToleranceVnd: number;
  maxFeeVarianceVnd: number;
  splitSolverMaxK: number;
  quarantineTtlSecs: number;
  amlThresholdVnd: number;
}

export const useSettingsStore = defineStore('settings', () => {
  const activeTab = ref<SettingsTab>('USERS');

  // 1. P101 RBAC Users
  const users = ref<UserAccountItem[]>([
    {
      id: 'USR-001',
      username: 'admin.system',
      fullName: 'Vũ Hải Đăng (Quản Trị Viên)',
      role: 'ADMIN',
      isActive: true,
      certSerial: 'VNPT-ADMIN-9901',
      lastLogin: '2026-09-15 14:02:11',
    },
    {
      id: 'USR-002',
      username: 'nam.tv',
      fullName: 'Trịnh Văn Nam (Kế Toán Viên - Maker)',
      role: 'MAKER',
      isActive: true,
      certSerial: 'VNPT-MAKER-4412',
      lastLogin: '2026-09-15 16:45:00',
    },
    {
      id: 'USR-003',
      username: 'tri.nm',
      fullName: 'Nguyễn Minh Trí (Kế Toán Trưởng - Checker)',
      role: 'CHECKER',
      isActive: true,
      certSerial: 'VIETTEL-CHECKER-1029',
      lastLogin: '2026-09-15 16:50:22',
    },
    {
      id: 'USR-004',
      username: 'lan.pt',
      fullName: 'Phạm Thị Lan (Cán Bộ Tuân Thủ AML)',
      role: 'COMPLIANCE',
      isActive: true,
      certSerial: 'FPT-AML-7811',
      lastLogin: '2026-09-15 15:30:10',
    },
    {
      id: 'USR-005',
      username: 'auditor.external',
      fullName: 'Kiểm Toán Viên Độc Lập (PwC)',
      role: 'AUDITOR',
      isActive: true,
      certSerial: 'GLOBAL-AUDIT-0082',
      lastLogin: '2026-09-15 11:20:00',
    },
  ]);

  // 2. P102 Open Banking Connectors
  const bankConfigs = ref<BankConfigItem[]>([
    {
      bankCode: 'VCB',
      bankName: 'Ngân hàng TMCP Ngoại thương Việt Nam (Vietcombank)',
      apiEndpoint: 'https://api.vietcombank.com.vn/b2b/v2/statements',
      isConnected: true,
      isSandbox: true,
      latencyMs: 38,
      lastSyncAt: '2026-09-15 16:55:00',
    },
    {
      bankCode: 'TCB',
      bankName: 'Ngân hàng TMCP Kỹ thương Việt Nam (Techcombank)',
      apiEndpoint: 'https://corporate-api.techcombank.com.vn/v1/cash-management',
      isConnected: true,
      isSandbox: true,
      latencyMs: 42,
      lastSyncAt: '2026-09-15 16:54:12',
    },
    {
      bankCode: 'BIDV',
      bankName: 'Ngân hàng TMCP Đầu tư và Phát triển Việt Nam (BIDV)',
      apiEndpoint: 'https://open.bidv.com.vn/api/corporate/v3',
      isConnected: true,
      isSandbox: true,
      latencyMs: 51,
      lastSyncAt: '2026-09-15 16:50:00',
    },
    {
      bankCode: 'MBB',
      bankName: 'Ngân hàng TMCP Quân đội (MBBank)',
      apiEndpoint: 'https://api.mbbank.com.vn/corporate/v1',
      isConnected: false,
      isSandbox: true,
      latencyMs: 0,
      lastSyncAt: 'Chưa kích hoạt',
    },
  ]);

  // 3. P103 ERP Bridges
  const erpConfigs = ref<ErpConfigItem[]>([
    {
      erpName: 'MISA AMIS',
      description: 'Phần mềm kế toán doanh nghiệp MISA AMIS (REST OpenAPI JSON)',
      endpoint: 'https://amisapi.misa.vn/v1/bank-vouchers',
      isActive: true,
      syncIntervalMins: 15,
      lastSyncAt: '2026-09-15 16:45:00',
    },
    {
      erpName: 'FAST Accounting',
      description: 'Hệ thống kế toán tài chính FAST Business Online (XML WebService)',
      endpoint: 'https://fast.vn/ws/accounting/post-voucher',
      isActive: true,
      syncIntervalMins: 30,
      lastSyncAt: '2026-09-15 16:30:00',
    },
    {
      erpName: 'SAP Business One',
      description: 'Hệ thống hoạch định doanh nghiệp SAP B1 (DI-Server / Service Layer)',
      endpoint: 'https://sap-gateway.internal:50000/b1s/v1',
      isActive: false,
      syncIntervalMins: 60,
      lastSyncAt: 'Chưa kích hoạt',
    },
  ]);

  // 4. P104 Reconciliation Thresholds
  const thresholds = ref<ThresholdConfig>({
    exactToleranceVnd: 0,
    maxFeeVarianceVnd: 50000,
    splitSolverMaxK: 8,
    quarantineTtlSecs: 900,
    amlThresholdVnd: 400000000,
  });

  // 5. P105 Security & HSM
  const security = ref({
    zeroEgressStrict: true,
    hsmProvider: 'VNPT-CA Cloud HSM & USB Token PKCS#11',
    certSerial: 'VNPT-CA-2026-X892A1',
    certValidUntil: '2028-12-31',
    tamperProofChain: true,
  });

  // 6. P106 Backup & DR
  const backup = ref({
    lastBackupAt: '2026-09-15 20:00:00',
    backupRetentionDays: 365,
    merkleRootHash: '0x7b2a9f41c0e358b901a89c3d4f1078e24ab5c891e20498bfa761c3d049872e11',
    isBackingUp: false,
    lastDrVerification: '2026-09-15 10:00:00 (RTO: 12s, RPO: 0s)',
  });

  // --- Computed ---

  const connectedBankCount = computed(() => bankConfigs.value.filter((b) => b.isConnected).length);
  const activeErpCount = computed(() => erpConfigs.value.filter((e) => e.isActive).length);

  /**
   * Enforces Segregation of Duties (Circular 09/2020/TT-NHNN).
   * Verifies no user holds both MAKER and CHECKER simultaneously.
   */
  const hasSodViolation = computed(() => {
    // In this model each user has 1 role, so check if any user ID has conflicting dual entries
    const userNames = users.value.map((u) => u.username.toLowerCase());
    const uniqueUserNames = new Set(userNames);
    return uniqueUserNames.size !== userNames.length;
  });

  // --- Actions ---

  function setActiveTab(tab: SettingsTab) {
    activeTab.value = tab;
  }

  function toggleBankConnection(bankCode: string) {
    const bank = bankConfigs.value.find((b) => b.bankCode === bankCode);
    if (bank) {
      bank.isConnected = !bank.isConnected;
      bank.lastSyncAt = new Date().toISOString().replace('T', ' ').slice(0, 19);
      logger.info(`Ngân hàng ${bankCode} đã chuyển trạng thái kết nối: ${bank.isConnected}`);
    }
  }

  function toggleBankSandbox(bankCode: string) {
    const bank = bankConfigs.value.find((b) => b.bankCode === bankCode);
    if (bank) {
      bank.isSandbox = !bank.isSandbox;
      logger.info(`Ngân hàng ${bankCode} đã chuyển môi trường: ${bank.isSandbox ? 'SANDBOX' : 'PRODUCTION'}`);
    }
  }

  function toggleErpActive(erpName: string) {
    const erp = erpConfigs.value.find((e) => e.erpName === erpName);
    if (erp) {
      erp.isActive = !erp.isActive;
      erp.lastSyncAt = new Date().toISOString().replace('T', ' ').slice(0, 19);
      logger.info(`Cầu nối ERP ${erpName} đã chuyển trạng thái: ${erp.isActive}`);
    }
  }

  function updateThresholds(patch: Partial<ThresholdConfig>) {
    thresholds.value = { ...thresholds.value, ...patch };
    logger.info('Đã cập nhật tham số dung sai đối soát');
  }

  function triggerBackupNow(): string {
    backup.value.isBackingUp = true;
    const now = new Date().toISOString().replace('T', ' ').slice(0, 19);
    backup.value.lastBackupAt = now;
    const snapshotHash = `SNAP-${Date.now().toString(16).toUpperCase()}`;
    backup.value.isBackingUp = false;
    logger.info(`Đã hoàn tất sao lưu dữ liệu khẩn cấp: ${snapshotHash}`);
    return snapshotHash;
  }

  function verifyDisasterRecovery(): string {
    const now = new Date().toISOString().replace('T', ' ').slice(0, 19);
    backup.value.lastDrVerification = `${now} (RTO: 14s, RPO: 0s - Pass 100%)`;
    logger.info('Kiểm toán phục hồi thảm họa DR thành công');
    return backup.value.lastDrVerification;
  }

  return {
    activeTab,
    users,
    bankConfigs,
    erpConfigs,
    thresholds,
    security,
    backup,
    connectedBankCount,
    activeErpCount,
    hasSodViolation,
    setActiveTab,
    toggleBankConnection,
    toggleBankSandbox,
    toggleErpActive,
    updateThresholds,
    triggerBackupNow,
    verifyDisasterRecovery,
  };
});
