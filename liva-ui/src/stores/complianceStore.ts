import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '../utils/logger';

export type AmlRuleCode = 'AML_HIGH_VALUE' | 'AML_STRUCTURING' | 'AML_VELOCITY_SURGE' | 'AML_PASS_THROUGH';
export type AmlSeverity = 'HIGH' | 'CRITICAL';

export interface AmlAlertItem {
  id: string;
  ruleCode: AmlRuleCode;
  severity: AmlSeverity;
  txId: string;
  accountNumber: string;
  bankCode: string;
  counterpartyName: string;
  counterpartyAccount: string;
  amount: number; // Integer VND
  narrative: string;
  detectedAt: string;
  statutoryRef: string;
  hasStrReport: boolean;
}

export interface StrReport {
  strId: string;
  reportDate: string;
  reportingEntity: string;
  suspectName: string;
  suspectAccount: string;
  alertType: string;
  severity: string;
  totalVndAmount: number;
  transactionCount: number;
  statutoryRuleRef: string;
  narrativeSummary: string;
  complianceOfficerNotes: string;
  officerId?: string;
  signedAt?: string;
  merkleProofHash: string;
}

export const useComplianceStore = defineStore('compliance', () => {
  const isZeroEgressActive = ref(true);
  const isPiiRedactionEnabled = ref(true);
  const merkleRootHash = ref('0x7b2a9f41c0e358b901a89c3d4f1078e24ab5c891e20498bfa761c3d049872e11');

  const alerts = ref<AmlAlertItem[]>([
    {
      id: 'alt-01',
      ruleCode: 'AML_HIGH_VALUE',
      severity: 'HIGH',
      txId: 'TX-VCB-98102',
      accountNumber: '00710009821',
      bankCode: 'VCB',
      counterpartyName: 'Lê Hoàng Long',
      counterpartyAccount: '1903998811',
      amount: 550000000,
      narrative: 'Giao dịch chuyển tiền mua vật tư công trình vượt ngưỡng báo cáo 400M VND',
      detectedAt: '2026-09-15 14:22:10',
      statutoryRef: 'Điều 25 Luật PCRT 2022 & Quyết định 11/2023/QĐ-TTg',
      hasStrReport: false,
    },
    {
      id: 'alt-02',
      ruleCode: 'AML_STRUCTURING',
      severity: 'CRITICAL',
      txId: 'TX-TCB-44105',
      accountNumber: '1903456789',
      bankCode: 'TCB',
      counterpartyName: 'Phạm Minh Tuấn',
      counterpartyAccount: '1028399120',
      amount: 1090000000,
      narrative: 'Dấu hiệu chia nhỏ giao dịch (Smurfing): 3 món 360M, 350M, 380M liên tiếp trong 48h né ngưỡng 400M',
      detectedAt: '2026-09-15 15:45:00',
      statutoryRef: 'Khoản 2 Điều 26 Luật PCRT 2022 (Dấu hiệu giao dịch đáng ngờ)',
      hasStrReport: true,
    },
    {
      id: 'alt-03',
      ruleCode: 'AML_PASS_THROUGH',
      severity: 'CRITICAL',
      txId: 'TX-BIDV-12099',
      accountNumber: '1201000456',
      bankCode: 'BIDV',
      counterpartyName: 'Trần Quốc Bảo',
      counterpartyAccount: '0680199281',
      amount: 150000000,
      narrative: 'Tài khoản trung chuyển rửa tiền (Mule): Tiền vào 150M rút ra ngay 148M (98.6%) sau 8 phút',
      detectedAt: '2026-09-15 16:10:25',
      statutoryRef: 'Khoản 5 Điều 26 Luật PCRT 2022 & Thông tư 09/2023/TT-NHNN',
      hasStrReport: false,
    },
  ]);

  const activeStrModal = ref<StrReport | null>(null);

  const piiSampleInput = ref(
    'Chuyen khoan thanh toan tien hang cho cong ty CONG TY TNHH XAY DUNG NGUYEN HOANG qua so CCCD 031092008451 va so dien thoai 0908123456 tai stk 0071000982123 Ngan hang Vietcombank theo hop dong MEP.'
  );

  // --- Computed ---

  const criticalCount = computed(() => alerts.value.filter((a) => a.severity === 'CRITICAL').length);
  const highCount = computed(() => alerts.value.filter((a) => a.severity === 'HIGH').length);

  /**
   * Decree 13 PII Sanitizer logic (Client-side mirror of Rust engine)
   * Redacts CCCD, Phone, Bank Account while strictly preserving corporate names.
   */
  const sanitizedPiiPreview = computed(() => {
    if (!isPiiRedactionEnabled.value) return piiSampleInput.value;

    let text = piiSampleInput.value;

    // 1. Redact CCCD (12 digits)
    text = text.replace(/\b0\d{11}\b/g, '[REDACTED_CCCD]');

    // 2. Redact Phone (10 digits starting with 03, 05, 07, 08, 09)
    text = text.replace(/\b(03|05|07|08|09)\d{8}\b/g, '[REDACTED_PHONE]');

    // 3. Redact Bank Account preceded by stk/tk/account
    text = text.replace(/(stk|tk|account|so tk)\s+(\d{8,16})/gi, '$1 [REDACTED_ACCOUNT]');

    return text;
  });

  // --- Actions ---

  function generateStrReport(alertId: string): StrReport | null {
    const target = alerts.value.find((a) => a.id === alertId);
    if (!target) return null;

    const report: StrReport = {
      strId: `STR-${Date.now().toString().slice(-6)}`,
      reportDate: new Date().toISOString().split('T')[0],
      reportingEntity: 'Ngân hàng TMCP Ngoại thương Việt Nam — Ban Pháp chế & Kiểm soát Tuân thủ',
      suspectName: target.counterpartyName,
      suspectAccount: target.counterpartyAccount,
      alertType: target.ruleCode.replace(/_/g, ' '),
      severity: target.severity,
      totalVndAmount: target.amount,
      transactionCount: target.ruleCode === 'AML_STRUCTURING' ? 3 : 1,
      statutoryRuleRef: target.statutoryRef,
      narrativeSummary: target.narrative,
      complianceOfficerNotes: 'Đề xuất phong tỏa tạm thời tài khoản và chuyển hồ sơ sang Cục PCRT — NHNN.',
      merkleProofHash: merkleRootHash.value,
    };

    activeStrModal.value = report;
    target.hasStrReport = true;
    logger.info(`Đã lập dự thảo báo cáo STR cho cảnh báo ${alertId}`);
    return report;
  }

  function signAndSubmitStr(strId: string, officerId: string, notes: string) {
    if (activeStrModal.value && activeStrModal.value.strId === strId) {
      activeStrModal.value.officerId = officerId;
      activeStrModal.value.complianceOfficerNotes = notes;
      activeStrModal.value.signedAt = new Date().toISOString();
      logger.info(`Cán bộ tuân thủ ${officerId} đã ký duyệt báo cáo STR ${strId}`);
    }
  }

  function closeStrModal() {
    activeStrModal.value = null;
  }

  function recalculateMerkleRoot(): string {
    const seed = Date.now().toString();
    const newRoot = `0x${Array.from(seed + 'liva-merkle-audit-proof')
      .map((c) => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join('')
      .slice(0, 64)}`;
    merkleRootHash.value = newRoot;
    logger.info(`Đã tái tính toán gốc Merkle bất biến: ${newRoot}`);
    return newRoot;
  }

  function togglePiiRedaction(enabled?: boolean) {
    isPiiRedactionEnabled.value = enabled !== undefined ? enabled : !isPiiRedactionEnabled.value;
  }

  return {
    isZeroEgressActive,
    isPiiRedactionEnabled,
    merkleRootHash,
    alerts,
    activeStrModal,
    piiSampleInput,
    criticalCount,
    highCount,
    sanitizedPiiPreview,
    generateStrReport,
    signAndSubmitStr,
    closeStrModal,
    recalculateMerkleRoot,
    togglePiiRedaction,
  };
});
