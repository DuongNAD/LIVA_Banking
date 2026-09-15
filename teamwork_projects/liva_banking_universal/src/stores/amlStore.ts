/**
 * Pinia Store for AML Compliance & System Prompt Inspector
 * Manages AML configuration, surveillance alerts, prompt tuning, and Form STR modal state.
 */

import { defineStore } from 'pinia';
import {
  DEFAULT_AML_CONFIG,
  type AmlAlert,
  type AmlScreeningConfig,
  type FormStrData,
} from '../types/aml';
import {
  runFullAmlSurveillance,
  inspectSystemPrompt,
} from '../engine/intelligence/amlSurveillance';
import { generateFormStr, formatOfficialStrDocument } from '../engine/intelligence/strGenerator';

export interface SandboxPreset {
  id: string;
  name: string;
  description: string;
  transactions: any[];
}

export const AML_SANDBOX_PRESETS: SandboxPreset[] = [
  {
    id: 'STRUCTURING',
    name: '1. Structuring / Smurfing (3x390M)',
    description: '3 giao dịch dưới 400M trong 24h tổng cộng 1.170.000.000 VND né ngưỡng báo cáo.',
    transactions: [
      { id: 'smurf-1', date: '2026-08-10', time: '11:00:00', amount: 390_000_000, txType: 'CREDIT', counterparty: 'NGUYEN HOANG PHUC', counterpartyAccount: '001100234567', narration: 'Chuyen tien thanh toan dot 1' },
      { id: 'smurf-2', date: '2026-08-10', time: '11:45:00', amount: 385_000_000, txType: 'CREDIT', counterparty: 'TRAN THI BICH NGOC', counterpartyAccount: '001100234568', narration: 'Chuyen tien thanh toan dot 2' },
      { id: 'smurf-3', date: '2026-08-10', time: '14:10:00', amount: 395_000_000, txType: 'CREDIT', counterparty: 'LE VAN THANG', counterpartyAccount: '001100234569', narration: 'Chuyen tien thanh toan dot 3' },
    ],
  },
  {
    id: 'HIGH_VALUE',
    name: '2. High-Value Transaction (Ngưỡng 420M QĐ 11/2023)',
    description: 'Giao dịch đơn lẻ 420M VND đạt ngưỡng bắt buộc báo cáo theo Quyết định 11/2023/QĐ-TTg.',
    transactions: [
      { id: 'bidv-high-1', date: '2026-08-12', time: '14:30:00', amount: 420_000_000, txType: 'CREDIT', counterparty: 'VU TRONG PHUONG', counterpartyAccount: '12010001234567', narration: 'Thanh toan tien vat tu cong trinh' },
    ],
  },
  {
    id: 'NIGHT_PASS_THROUGH',
    name: '3. Night Pass-Through Churn (420M in -> 419.5M out)',
    description: 'Giao dịch lúc 02:15 sáng, nhận 420M và rút sạch 99.88% trong vòng 7 phút ra ví điện tử.',
    transactions: [
      { id: 'night-in', date: '2026-08-13', time: '02:15:20', amount: 420_000_000, txType: 'CREDIT', counterparty: 'NGUYEN VAN A', counterpartyAccount: '19034567890123', narration: 'Nhan tien chuyen khoan nhanh Napas' },
      { id: 'night-out', date: '2026-08-13', time: '02:22:45', amount: 419_500_000, txType: 'DEBIT', counterparty: 'CONG TY TNHH TRUNG GIAN THANH TOAN X', counterpartyAccount: '007100987654', narration: 'Rut tien ra vi dien tu X' },
    ],
  },
  {
    id: 'NORMAL',
    name: '4. Normal Corporate Invoice Payment',
    description: 'Giao dịch doanh nghiệp hợp lệ trong giờ hành chính, dưới ngưỡng báo cáo, có mã hóa đơn.',
    transactions: [
      { id: 'norm-1', date: '2026-08-14', time: '10:15:00', amount: 145_000_000, txType: 'CREDIT', counterparty: 'CONG TY AN PHAT', counterpartyAccount: '002100888999', narration: 'Thanh toan tien may bien ap HD-88' },
    ],
  },
];

export const useAmlStore = defineStore('aml', {
  state: () => {
    const initialConfig = { ...DEFAULT_AML_CONFIG };
    const promptInspection = inspectSystemPrompt(initialConfig);

    return {
      config: initialConfig as AmlScreeningConfig,
      activePromptText: promptInspection.systemPrompt,
      alerts: [] as AmlAlert[],
      selectedAlert: null as AmlAlert | null,
      generatedStr: null as FormStrData | null,
      officialStrDocumentText: '' as string,
      isStrModalOpen: false as boolean,
      sandboxPresetId: 'STRUCTURING' as string,
      sandboxTransactions: AML_SANDBOX_PRESETS[0].transactions as any[],
      sandboxAlerts: [] as AmlAlert[],
      isEvaluating: false as boolean,
    };
  },

  getters: {
    criticalAlertsCount: (state) => state.alerts.filter((a) => a.severity === 'CRITICAL').length,
    highAlertsCount: (state) => state.alerts.filter((a) => a.severity === 'HIGH').length,
    totalAlertsCount: (state) => state.alerts.length,
    currentPreset: (state) =>
      AML_SANDBOX_PRESETS.find((p) => p.id === state.sandboxPresetId) || AML_SANDBOX_PRESETS[0],
  },

  actions: {
    updateConfig(newConfig: Partial<AmlScreeningConfig>) {
      this.config = { ...this.config, ...newConfig };
      const inspection = inspectSystemPrompt(this.config);
      this.activePromptText = inspection.systemPrompt;
    },

    updatePromptText(customText: string) {
      this.activePromptText = customText;
    },

    resetToSbvDefaults() {
      this.config = { ...DEFAULT_AML_CONFIG };
      const inspection = inspectSystemPrompt(this.config);
      this.activePromptText = inspection.systemPrompt;
    },

    scanTransactions(transactions: any[]) {
      this.alerts = runFullAmlSurveillance(transactions, this.config);
      return this.alerts;
    },

    selectPreset(presetId: string) {
      this.sandboxPresetId = presetId;
      const preset = AML_SANDBOX_PRESETS.find((p) => p.id === presetId);
      if (preset) {
        this.sandboxTransactions = preset.transactions;
        this.runSandboxEvaluation();
      }
    },

    runSandboxEvaluation() {
      this.isEvaluating = true;
      try {
        this.sandboxAlerts = runFullAmlSurveillance(this.sandboxTransactions, this.config);
      } finally {
        this.isEvaluating = false;
      }
    },

    openStrModal(alert: AmlAlert, transactions: any[] = [], notes: string = '') {
      this.selectedAlert = alert;
      this.generatedStr = generateFormStr(alert, transactions, notes);
      this.officialStrDocumentText = formatOfficialStrDocument(this.generatedStr);
      this.isStrModalOpen = true;
    },

    closeStrModal() {
      this.isStrModalOpen = false;
      this.selectedAlert = null;
      this.generatedStr = null;
      this.officialStrDocumentText = '';
    },

    updateComplianceNotes(notes: string) {
      if (this.generatedStr && this.selectedAlert) {
        this.generatedStr.complianceOfficerNotes = notes;
        this.officialStrDocumentText = formatOfficialStrDocument(this.generatedStr);
      }
    },
  },
});
