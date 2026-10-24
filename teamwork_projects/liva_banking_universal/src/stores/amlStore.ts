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
import {
  localAiClient,
  estimateTokens,
  evaluatePromptBudget,
  type AiConnectionStatus,
  type AiTelemetryMetrics,
  type PromptBudgetStatus,
} from '../services/LocalAiClient';
import sqlDatabaseSeed from '../data/sqlDatabaseSeed.json';

export interface LlmRiskEvaluation {
  riskScore: number;
  riskLevel: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW';
  findings: string;
  recommendedAction: string;
}

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
  {
    id: 'SQL_DATABASE',
    name: '5. Cơ Sở Dữ Liệu SQL (160 Giao Dịch Đa Ngân Hàng)',
    description: 'Tập mẫu 40 giao dịch thực tế từ SQLite (VCB, TCB, BIDV, CTG, MBB) kích hoạt đa dạng các quy tắc AML/STR theo QĐ 11/2023/QĐ-TTg.',
    transactions: ((sqlDatabaseSeed as any).bankStatements || []).slice(0, 40).map((s: any) => {
      const [d, t] = (s.booking_date || '2026-08-15 09:00:00').split(' ');
      return {
        id: s.id,
        date: d || '2026-08-15',
        time: t || '09:00:00',
        amount: s.credit_amount > 0 ? s.credit_amount : s.debit_amount,
        txType: s.credit_amount > 0 ? 'CREDIT' : 'DEBIT',
        counterparty: s.partner_name || 'Đối tác liên ngân hàng',
        counterpartyAccount: s.counterpart_account || '001100234567',
        narration: s.description || '',
      };
    }),
  },
];

function extractStructuredJson(text: string): any {
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    const match = text.match(/\{[\s\S]*\}/);
    if (match) {
      try {
        return JSON.parse(match[0]);
      } catch {
        return null;
      }
    }
    return null;
  }
}

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

      // Local AI Sandbox Evaluation & Latency Telemetry (Milestone M2)
      evaluationMode: 'LOCAL_RULE_FALLBACK' as 'LOCAL_RULE_FALLBACK' | 'LOCAL_AI_QWEN' | 'LOCAL_AI_EVALUATING',
      aiConnectionStatus: 'offline' as AiConnectionStatus,
      llmEvaluationResult: null as LlmRiskEvaluation | null,
      lastTelemetry: null as AiTelemetryMetrics | null,
      promptBudgetStatus: 'OPTIMAL' as PromptBudgetStatus,
      estimatedPromptTokens: 0 as number,
    };
  },

  getters: {
    criticalAlertsCount: (state) => state.alerts.filter((a) => a.severity === 'CRITICAL').length,
    highAlertsCount: (state) => state.alerts.filter((a) => a.severity === 'HIGH').length,
    totalAlertsCount: (state) => state.alerts.length,
    currentPreset: (state) =>
      AML_SANDBOX_PRESETS.find((p) => p.id === state.sandboxPresetId) || AML_SANDBOX_PRESETS[0],
    isAiConnected: (state) => state.aiConnectionStatus === 'connected',
    amlTelemetry: (state) => state.lastTelemetry,
    amlBudgetStatus: (state) => state.promptBudgetStatus,
    amlPromptTokens: (state) => state.estimatedPromptTokens,
  },

  actions: {
    async checkAiHealth(): Promise<AiConnectionStatus> {
      const isHealthy = await localAiClient.checkHealth();
      this.aiConnectionStatus = isHealthy ? 'connected' : 'offline';
      return this.aiConnectionStatus;
    },

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

    async runSandboxEvaluation() {
      this.isEvaluating = true;
      // 1. Immediately evaluate via deterministic statutory rule engine (Decree 13 / Circular 09 / Decision 11)
      this.sandboxAlerts = runFullAmlSurveillance(this.sandboxTransactions, this.config);
      this.evaluationMode = 'LOCAL_RULE_FALLBACK';

      // 2. Check prompt token budget of transaction payload (Milestone M2)
      const payloadJson = JSON.stringify(this.sandboxTransactions, null, 2);
      const systemPromptContent = `${this.activePromptText}\n\nYÊU CẦU ĐẦU RA: Bạn là Chuyên gia Giám sát AML Tuân thủ Ngân hàng (Qwen 14B). Hãy đánh giá rủi ro cho danh sách giao dịch dưới đây và trả về KẾT QUẢ DUY NHẤT Ở ĐỊNH DẠNG JSON với cấu trúc:
{
  "riskScore": number (0-100),
  "riskLevel": "CRITICAL" | "HIGH" | "MEDIUM" | "LOW",
  "findings": "Tóm tắt các phát hiện vi phạm hoặc dấu hiệu đáng ngờ...",
  "recommendedAction": "Đề xuất xử lý tuân thủ (ví dụ: Lập Form STR theo Thông tư 09/2023/TT-NHNN hoặc cho phép xử lý bình thường)..."
}`;
      const userPromptContent = `DANH SÁCH GIAO DỊCH KỊCH BẢN ĐỐI KHÁNG:\n${payloadJson}`;
      this.estimatedPromptTokens = estimateTokens(`${systemPromptContent}\n${userPromptContent}`);
      this.promptBudgetStatus = evaluatePromptBudget(this.estimatedPromptTokens);

      // 3. If Local AI (Qwen 14B) is available, run prompt-driven evaluation
      try {
        const isOnline = localAiClient.getStatus() === 'connected' || (await localAiClient.checkHealth());
        this.aiConnectionStatus = isOnline ? 'connected' : 'offline';

        if (isOnline) {
          this.evaluationMode = 'LOCAL_AI_EVALUATING';
          const messages = [
            {
              role: 'system' as const,
              content: systemPromptContent,
            },
            {
              role: 'user' as const,
              content: userPromptContent,
            },
          ];

          const responseText = await localAiClient.chatCompletion(messages, { temperature: 0.1 });
          this.lastTelemetry = localAiClient.getLastTelemetry();

          const parsed = extractStructuredJson(responseText);

          if (parsed && (parsed.riskScore !== undefined || parsed.findings || parsed.riskLevel)) {
            const rawLevel = String(parsed.riskLevel || 'HIGH').toUpperCase();
            const validLevel: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW' =
              rawLevel === 'CRITICAL' || rawLevel === 'HIGH' || rawLevel === 'MEDIUM' || rawLevel === 'LOW'
                ? rawLevel
                : 'HIGH';

            this.llmEvaluationResult = {
              riskScore: Number(parsed.riskScore ?? (this.sandboxAlerts.length > 0 ? 85 : 15)),
              riskLevel: validLevel,
              findings: String(parsed.findings || responseText),
              recommendedAction: String(parsed.recommendedAction || 'Kiểm soát viên rà soát và lập hồ sơ STR.'),
            };
            this.evaluationMode = 'LOCAL_AI_QWEN';
          } else {
            this.evaluationMode = 'LOCAL_RULE_FALLBACK';
          }
        }
      } catch {
        this.aiConnectionStatus = 'fallback';
        this.evaluationMode = 'LOCAL_RULE_FALLBACK';
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

    applySseAlert(alert: AmlAlert) {
      if (!alert || !alert.alertId) return;
      const exists = this.alerts.some((a) => a.alertId === alert.alertId);
      if (!exists) {
        this.alerts.unshift(alert);
      }
    },
  },
});
