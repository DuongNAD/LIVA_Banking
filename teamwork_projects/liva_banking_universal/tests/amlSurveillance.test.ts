import { describe, it, expect } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import {
  detectAmlHighValue,
  detectAmlStructuring,
  detectAmlNightVelocity,
  detectAmlRapidPassThrough,
  detectAmlWatchlistKeywords,
  runFullAmlSurveillance,
  inspectSystemPrompt,
} from '../src/engine/intelligence/amlSurveillance';
import { generateFormStr, formatOfficialStrDocument } from '../src/engine/intelligence/strGenerator';
import { useAmlStore } from '../src/stores/amlStore';
import type { AmlAlert } from '../src/types/aml';

describe('AML & STR Surveillance Engine (Features F11 - F16)', () => {
  describe('High-Value Transaction Detector (F12)', () => {
    it('should flag single transaction >= 400M VND pursuant to Decision 11/2023', () => {
      const txs = [{ id: 'tx-1', amount: 550_000_000 }];
      const alerts = detectAmlHighValue(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('HIGH_VALUE');
      expect(alerts[0].severity).toBe('HIGH');
      expect(alerts[0].statutoryRuleRef).toContain('Quyết định 11/2023/QĐ-TTg');
    });

    it('should escalate to CRITICAL severity for transactions >= 1 Billion VND', () => {
      const txs = [{ id: 'tx-large', amount: 1_200_000_000 }];
      const alerts = detectAmlHighValue(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].severity).toBe('CRITICAL');
    });

    it('should not flag transaction strictly below 400M VND', () => {
      const txs = [{ id: 'tx-sub', amount: 399_999_999 }];
      const alerts = detectAmlHighValue(txs);
      expect(alerts.length).toBe(0);
    });

    it('should trigger alert on exact 400,000,000 VND boundary', () => {
      const txs = [{ id: 'tx-exact', amount: 400_000_000 }];
      const alerts = detectAmlHighValue(txs);
      expect(alerts.length).toBe(1);
    });

    it('should ignore 0 VND or negative amounts', () => {
      const alerts = detectAmlHighValue([{ id: 'tx-0', amount: 0 }]);
      expect(alerts.length).toBe(0);
    });

    it('should handle large arrays of high-value transactions', () => {
      const txs = Array.from({ length: 5 }, (_, i) => ({ id: `tx-${i}`, amount: 500_000_000 }));
      const alerts = detectAmlHighValue(txs);
      expect(alerts.length).toBe(5);
    });
  });

  describe('Structuring / Smurfing Detector (F11)', () => {
    it('should flag >= 3 sub-400M transfers within 24h summing >= 400M VND', () => {
      const txs = [
        { id: 'tx-1', date: '2026-08-10', amount: 390_000_000 },
        { id: 'tx-2', date: '2026-08-10', amount: 385_000_000 },
        { id: 'tx-3', date: '2026-08-10', amount: 395_000_000 },
      ];
      const alerts = detectAmlStructuring(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('STRUCTURING_SMURFING');
      expect(alerts[0].severity).toBe('CRITICAL');
      expect(alerts[0].totalAmount).toBe(1_170_000_000);
      expect(alerts[0].statutoryRuleRef).toContain('Thông tư 09/2023/TT-NHNN');
    });

    it('should not flag structuring if transaction count < 3', () => {
      const txs = [
        { id: 'tx-1', date: '2026-08-10', amount: 300_000_000 },
        { id: 'tx-2', date: '2026-08-10', amount: 300_000_000 },
      ];
      const alerts = detectAmlStructuring(txs);
      expect(alerts.length).toBe(0);
    });

    it('should not flag structuring if cumulative sum < 400M VND', () => {
      const txs = [
        { id: 'tx-1', date: '2026-08-10', amount: 100_000_000 },
        { id: 'tx-2', date: '2026-08-10', amount: 100_000_000 },
        { id: 'tx-3', date: '2026-08-10', amount: 100_000_000 },
      ];
      const alerts = detectAmlStructuring(txs);
      expect(alerts.length).toBe(0);
    });

    it('should exclude transactions >= 400M from sub-400M smurfing pool', () => {
      const txs = [
        { id: 'tx-1', date: '2026-08-10', amount: 400_000_000 }, // excluded
        { id: 'tx-2', date: '2026-08-10', amount: 200_000_000 },
        { id: 'tx-3', date: '2026-08-10', amount: 200_000_000 },
      ];
      const alerts = detectAmlStructuring(txs);
      expect(alerts.length).toBe(0);
    });

    it('should not aggregate transactions across distinct dates into single 24h alert', () => {
      const txs = [
        { id: 'tx-1', date: '2026-08-01', amount: 150_000_000 },
        { id: 'tx-2', date: '2026-08-02', amount: 150_000_000 },
        { id: 'tx-3', date: '2026-08-03', amount: 150_000_000 },
      ];
      const alerts = detectAmlStructuring(txs);
      expect(alerts.length).toBe(0);
    });
  });

  describe('Night-Time Velocity Anomaly Detector (F14)', () => {
    it('should detect transactions between 23:00 and 05:00 with amount >= 50M VND', () => {
      const txs = [{ id: 'tx-night', time: '02:15:20', amount: 420_000_000 }];
      const alerts = detectAmlNightVelocity(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('NIGHT_VELOCITY');
      expect(alerts[0].severity).toBe('HIGH');
    });

    it('should ignore daytime transaction at 08:30', () => {
      const txs = [{ id: 'tx-day', time: '08:30:00', amount: 500_000_000 }];
      const alerts = detectAmlNightVelocity(txs);
      expect(alerts.length).toBe(0);
    });

    it('should ignore night transaction under 50M VND', () => {
      const txs = [{ id: 'tx-small', time: '01:00:00', amount: 20_000_000 }];
      const alerts = detectAmlNightVelocity(txs);
      expect(alerts.length).toBe(0);
    });

    it('should trigger at 23:00:00 boundary and not at 22:59:59', () => {
      const triggered = detectAmlNightVelocity([{ id: 'tx-1', time: '23:00:00', amount: 50_000_000 }]);
      expect(triggered.length).toBe(1);

      const notTriggered = detectAmlNightVelocity([{ id: 'tx-2', time: '22:59:59', amount: 50_000_000 }]);
      expect(notTriggered.length).toBe(0);
    });

    it('should trigger at 04:59:59 boundary and not at 05:00:00', () => {
      const triggered = detectAmlNightVelocity([{ id: 'tx-1', time: '04:59:59', amount: 50_000_000 }]);
      expect(triggered.length).toBe(1);

      const notTriggered = detectAmlNightVelocity([{ id: 'tx-2', time: '05:00:00', amount: 50_000_000 }]);
      expect(notTriggered.length).toBe(0);
    });
  });

  describe('Rapid Pass-Through / Churn Mule Detector (F13)', () => {
    it('should detect inflow >= 100M followed by drain >= 90%', () => {
      const txs = [
        { id: 'in-1', txType: 'CREDIT', amount: 420_000_000, time: '02:15:20' },
        { id: 'out-1', txType: 'DEBIT', amount: 419_500_000, time: '02:22:45' }, // 99.88% drain
      ];
      const alerts = detectAmlRapidPassThrough(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('RAPID_PASS_THROUGH');
      expect(alerts[0].severity).toBe('CRITICAL');
      expect(alerts[0].involvedTransactionIds).toEqual(['in-1', 'out-1']);
    });

    it('should not flag pass-through if drain ratio < 90%', () => {
      const txs = [
        { id: 'in-1', txType: 'CREDIT', amount: 500_000_000 },
        { id: 'out-1', txType: 'DEBIT', amount: 200_000_000 }, // 40% drain
      ];
      const alerts = detectAmlRapidPassThrough(txs);
      expect(alerts.length).toBe(0);
    });

    it('should not flag pass-through if initial inflow < 100M VND', () => {
      const txs = [
        { id: 'in-1', txType: 'CREDIT', amount: 99_999_999 },
        { id: 'out-1', txType: 'DEBIT', amount: 95_000_000 },
      ];
      const alerts = detectAmlRapidPassThrough(txs);
      expect(alerts.length).toBe(0);
    });

    it('should trigger on exact 90.0% drain ratio', () => {
      const txs = [
        { id: 'in', txType: 'CREDIT', amount: 100_000_000 },
        { id: 'out', txType: 'DEBIT', amount: 90_000_000 },
      ];
      const alerts = detectAmlRapidPassThrough(txs);
      expect(alerts.length).toBe(1);
    });

    it('should ignore inflow followed by credit (not debit)', () => {
      const txs = [
        { id: 'in1', txType: 'CREDIT', amount: 200_000_000 },
        { id: 'in2', txType: 'CREDIT', amount: 200_000_000 },
      ];
      const alerts = detectAmlRapidPassThrough(txs);
      expect(alerts.length).toBe(0);
    });
  });

  describe('Watchlist Keywords Detector', () => {
    it('should flag sensitive blacklisted keywords in transaction narration', () => {
      const txs = [
        { id: 'tx-gambling', amount: 15_000_000, narration: 'Chuyen tien nap diem bet88 vip' },
        { id: 'tx-crypto', amount: 25_000_000, narration: 'Mua 1000 usdt p2p binance' },
      ];
      const alerts = detectAmlWatchlistKeywords(txs);
      expect(alerts.length).toBe(2);
      expect(alerts[0].anomalyType).toBe('WATCHLIST_HIT');
      expect(alerts[0].severity).toBe('CRITICAL');
    });
  });

  describe('Full Surveillance Orchestration & Form STR Generation (F15)', () => {
    it('should run full surveillance suite and detect combined anomalies', () => {
      const combinedTxs = [
        { id: 'bidv-1', txType: 'CREDIT', date: '2026-08-13', time: '02:15:20', amount: 420_000_000, counterparty: 'VU TRONG PHUONG', counterpartyAccount: '12010001234567' },
        { id: 'bidv-2', txType: 'DEBIT', date: '2026-08-13', time: '02:22:45', amount: 419_500_000, counterparty: 'CONG TY TRUNG GIAN X' },
      ];

      const allAlerts = runFullAmlSurveillance(combinedTxs);
      expect(allAlerts.length).toBeGreaterThanOrEqual(3); // High-value + Night + Rapid Pass-Through

      // Generate Form STR
      const rapidAlert = allAlerts.find((a) => a.anomalyType === 'RAPID_PASS_THROUGH');
      expect(rapidAlert).toBeDefined();

      const formStr = generateFormStr(rapidAlert!, combinedTxs, 'Chuyển tiền ra ví điện tử trong đêm');
      expect(formStr.reportingEntity).toBe('LIVA SOLUTIONS CO., LTD');
      expect(formStr.alertType).toBe('RAPID_PASS_THROUGH');
      expect(formStr.suspectName).toBe('VU TRONG PHUONG');
      expect(formStr.suspectAccount).toBe('12010001234567');
      expect(formStr.totalVndAmount).toBe(420_000_000);
      expect(formStr.complianceOfficerNotes).toContain('Chuyển tiền ra ví điện tử');
      expect(formStr.formTemplate).toContain('Phụ lục II Thông tư 09/2023/TT-NHNN');

      // Verify official text document rendering
      const doc = formatOfficialStrDocument(formStr);
      expect(doc).toContain('BÁO CÁO GIAO DỊCH ĐÁNG NGỜ (FORM STR)');
      expect(doc).toContain('CỤC PHÒNG, CHỐNG RỬA TIỀN');
      expect(doc).toContain('420.000.000 VND');
    });
  });

  describe('Explainable System Prompt Inspector (F16)', () => {
    it('should generate active system prompt citing statutory circulars', () => {
      const inspector = inspectSystemPrompt();
      expect(inspector.systemPrompt).toContain('Thông tư 09/2023/TT-NHNN');
      expect(inspector.systemPrompt).toContain('Quyết định 11/2023/QĐ-TTg');
      expect(inspector.parameters.highValueThreshold).toBe(400_000_000);
    });

    it('should support dynamic tuning of threshold parameters in prompt', () => {
      const inspector = inspectSystemPrompt({ highValueThreshold: 300_000_000 });
      expect(inspector.parameters.highValueThreshold).toBe(300_000_000);
      expect(inspector.systemPrompt).toContain('300.000.000 VND');
    });

    it('should execute sandbox evaluation against candidate transactions', () => {
      const inspector = inspectSystemPrompt({ highValueThreshold: 200_000_000 });
      const sandboxTxs = [{ id: 'sb-1', amount: 250_000_000 }];
      const alerts = inspector.evalSandbox(sandboxTxs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].totalAmount).toBe(250_000_000);
    });
  });

  describe('Milestone M2: Local AI & Sandbox Risk Evaluation (AML Store Integration)', () => {
    it('initializes amlStore with default offline status and LOCAL_RULE_FALLBACK mode', () => {
      setActivePinia(createPinia());
      const amlStore = useAmlStore();

      expect(amlStore.aiConnectionStatus).toBe('offline');
      expect(amlStore.evaluationMode).toBe('LOCAL_RULE_FALLBACK');
      expect(amlStore.llmEvaluationResult).toBeNull();
      expect(amlStore.isAiConnected).toBe(false);
    });

    it('performs synchronous rule evaluation and safe fallback in runSandboxEvaluation', async () => {
      setActivePinia(createPinia());
      const amlStore = useAmlStore();
      amlStore.selectPreset('STRUCTURING');

      await amlStore.runSandboxEvaluation();

      expect(amlStore.sandboxAlerts.length).toBeGreaterThan(0);
      expect(amlStore.sandboxAlerts[0].anomalyType).toBe('STRUCTURING_SMURFING');
      expect(amlStore.evaluationMode).toBe('LOCAL_RULE_FALLBACK');
      // Operational alerts must remain isolated
      expect(amlStore.alerts).toHaveLength(0);
    });

    it('checks AI health safely returning offline when no local model daemon is listening', async () => {
      setActivePinia(createPinia());
      const amlStore = useAmlStore();

      const status = await amlStore.checkAiHealth();
      expect(status).toBe('offline');
      expect(amlStore.aiConnectionStatus).toBe('offline');
    });
  });

  describe('Form STR Statutory Modal State & Actions (F15 / amlStore)', () => {
    it('opens Form STR modal, generates structured form, and formats official document text', () => {
      setActivePinia(createPinia());
      const amlStore = useAmlStore();

      const candidateAlert: AmlAlert = {
        alertId: 'aml-test-str-1',
        anomalyType: 'HIGH_VALUE',
        severity: 'HIGH',
        involvedTransactionIds: ['tx-high-1'],
        totalAmount: 420_000_000,
        detectedAt: '2026-09-16T08:00:00Z',
        reasoning: 'Giao dịch giá trị lớn 420.000.000 VND',
        statutoryRuleRef: 'Quyết định 11/2023/QĐ-TTg',
        suggestedStrReport: true,
      };

      const candidateTxs = [
        {
          id: 'tx-high-1',
          amount: 420_000_000,
          counterparty: 'VU TRONG PHUONG',
          counterpartyAccount: '12010001234567',
        },
      ];

      expect(amlStore.isStrModalOpen).toBe(false);
      expect(amlStore.selectedAlert).toBeNull();
      expect(amlStore.generatedStr).toBeNull();
      expect(amlStore.officialStrDocumentText).toBe('');

      // Invoke openStrModal without throwing ReferenceError
      expect(() => {
        amlStore.openStrModal(candidateAlert, candidateTxs, 'Đề xuất phong tỏa tài khoản thụ hưởng');
      }).not.toThrow();

      expect(amlStore.isStrModalOpen).toBe(true);
      expect(amlStore.selectedAlert).toEqual(candidateAlert);
      expect(amlStore.generatedStr).not.toBeNull();
      expect(amlStore.generatedStr?.alertType).toBe('HIGH_VALUE');
      expect(amlStore.generatedStr?.suspectName).toBe('VU TRONG PHUONG');
      expect(amlStore.generatedStr?.suspectAccount).toBe('12010001234567');
      expect(amlStore.generatedStr?.totalVndAmount).toBe(420_000_000);
      expect(amlStore.generatedStr?.complianceOfficerNotes).toBe('Đề xuất phong tỏa tài khoản thụ hưởng');

      // Verify official formatted text document
      expect(amlStore.officialStrDocumentText).toContain('BÁO CÁO GIAO DỊCH ĐÁNG NGỜ (FORM STR)');
      expect(amlStore.officialStrDocumentText).toContain('Phụ lục II Thông tư số 09/2023/TT-NHNN');
      expect(amlStore.officialStrDocumentText).toContain('420.000.000 VND');
      expect(amlStore.officialStrDocumentText).toContain('Đề xuất phong tỏa tài khoản thụ hưởng');
    });

    it('updates compliance officer notes and re-renders official document text dynamically', () => {
      setActivePinia(createPinia());
      const amlStore = useAmlStore();

      const candidateAlert: AmlAlert = {
        alertId: 'aml-test-str-2',
        anomalyType: 'STRUCTURING_SMURFING',
        severity: 'CRITICAL',
        involvedTransactionIds: ['tx-smurf-1'],
        totalAmount: 1_170_000_000,
        detectedAt: '2026-09-16T08:15:00Z',
        reasoning: 'Giao dịch chia nhỏ cấu trúc 3 lần trong ngày',
        statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN',
        suggestedStrReport: true,
      };

      amlStore.openStrModal(candidateAlert, [], 'Ghi chú ban đầu');
      expect(amlStore.generatedStr?.complianceOfficerNotes).toBe('Ghi chú ban đầu');
      expect(amlStore.officialStrDocumentText).toContain('Ghi chú ban đầu');

      // Update notes
      const updatedNotes = 'Đã chuyển hồ sơ sang Cục PCRT NHNN và Bộ Công an (A05)';
      amlStore.updateComplianceNotes(updatedNotes);

      expect(amlStore.generatedStr?.complianceOfficerNotes).toBe(updatedNotes);
      expect(amlStore.officialStrDocumentText).toContain(updatedNotes);
      expect(amlStore.officialStrDocumentText).not.toContain('Ghi chú ban đầu');
    });

    it('safely closes Form STR modal and resets all modal state variables', () => {
      setActivePinia(createPinia());
      const amlStore = useAmlStore();

      const candidateAlert: AmlAlert = {
        alertId: 'aml-test-str-3',
        anomalyType: 'WATCHLIST_HIT',
        severity: 'CRITICAL',
        involvedTransactionIds: ['tx-watch-1'],
        totalAmount: 50_000_000,
        detectedAt: '2026-09-16T08:30:00Z',
        reasoning: 'Khớp danh sách đen cờ bạc',
        statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN',
        suggestedStrReport: true,
      };

      amlStore.openStrModal(candidateAlert, []);
      expect(amlStore.isStrModalOpen).toBe(true);
      expect(amlStore.selectedAlert).not.toBeNull();
      expect(amlStore.generatedStr).not.toBeNull();
      expect(amlStore.officialStrDocumentText.length).toBeGreaterThan(0);

      // Close modal
      amlStore.closeStrModal();
      expect(amlStore.isStrModalOpen).toBe(false);
      expect(amlStore.selectedAlert).toBeNull();
      expect(amlStore.generatedStr).toBeNull();
      expect(amlStore.officialStrDocumentText).toBe('');
    });
  });
});
