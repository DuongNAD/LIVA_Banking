/**
 * Milestone M1 — Challenger 2 Empirical Adversarial Stress Suite
 * Focus: Frontend Dynamic State, De-seeding & Sandbox Isolation Verification
 *
 * Adversarially challenges:
 * 1. reconciliationStore.matchRate is exactly 0% when un-reconciled (never falls back to 99.8%).
 * 2. FinancialCopilotDrawer.vue does not display static 6.2B / 99.8% in default/fallback state.
 * 3. ComplianceAmlView.vue and amlStore do not leak sandbox smurfing presets into operational alert feeds.
 * 4. Copilot messaging dynamics, SSE synchronization isolation, and store reactivity.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

// Headless environment mocks
const mockStorageMap = new Map<string, string>();
const mockStorage = {
  getItem: (k: string) => (mockStorageMap.has(k) ? mockStorageMap.get(k)! : null),
  setItem: (k: string, v: string) => mockStorageMap.set(k, String(v)),
  removeItem: (k: string) => mockStorageMap.delete(k),
  clear: () => mockStorageMap.clear(),
  key: (i: number) => Array.from(mockStorageMap.keys())[i] ?? null,
  get length() {
    return mockStorageMap.size;
  },
};

(globalThis as any).localStorage = mockStorage;
(globalThis as any).window = {
  location: { hostname: 'localhost' },
  localStorage: mockStorage,
};

import { useReconciliationStore } from '../src/stores/reconciliationStore';
import { useBankingStore } from '../src/stores/bankingStore';
import { useAmlStore, AML_SANDBOX_PRESETS } from '../src/stores/amlStore';
import { useTreasuryStore } from '../src/stores/treasuryStore';
import type { AmlAlert } from '../src/types/aml';

describe('Milestone M1 Challenger 2: Dynamic State & De-seeding Adversarial Challenge', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockStorage.clear();
  });

  // ==========================================================================
  // SECTION 1: ADVERSARIAL STRESS TESTING OF matchRate IN reconciliationStore
  // ==========================================================================
  describe('1. Adversarial Verification of matchRate in reconciliationStore', () => {
    it('returns exactly 0% when store is newly initialized with no statements or summary', () => {
      const store = useReconciliationStore();
      expect(store.reconciliationSummary).toBeNull();
      expect(store.bankTransactions).toHaveLength(0);
      expect(store.ledgerEntries).toHaveLength(0);

      // Invariant: matchRate MUST be strictly 0, never 99.8
      expect(store.matchRate).toBe(0);
      expect(store.matchRate).not.toBe(99.8);
    });

    it('returns exactly 0% when bank transactions exist but no reconciliation summary exists', () => {
      const store = useReconciliationStore();
      store.bankTransactions = [
        {
          id: 'tx-unrec-1',
          date: '2026-08-10',
          txDate: 1786320000,
          txCode: 'FT01',
          debit: 0,
          credit: 100_000_000,
          netAmount: 100_000_000,
          amount: 100_000_000,
          txType: 'CREDIT',
          balance: 100_000_000,
          narration: 'Test tx un-reconciled',
          bankCode: 'CITAD',
        },
      ];
      expect(store.reconciliationSummary).toBeNull();
      expect(store.matchRate).toBe(0);
    });

    it('returns exactly 0% when summary has totalBankTransactions = 0', () => {
      const store = useReconciliationStore();
      store.reconciliationSummary = {
        totalBankTransactions: 0,
        totalLedgerEntries: 0,
        matchedCount: 0,
        tier1Count: 0,
        tier2Count: 0,
        tier3Count: 0,
        hitlQuarantineCount: 0,
        matchRate: 0,
        totalMatchedAmount: 0,
        totalFeeDisentangled: 0,
        matches: [],
        quarantined: [],
        unallocatedBankTransactions: [],
        unallocatedLedgerEntries: [],
      };
      expect(store.matchRate).toBe(0);
    });

    it('returns exactly 0% when summary explicitly has matchRate = 0', () => {
      const store = useReconciliationStore();
      store.applyReconciliationSummary({
        totalBankTransactions: 50,
        matchedCount: 0,
        matchRate: 0,
      });
      expect(store.matchRate).toBe(0);
    });

    it('correctly calculates dynamic matchRate when summary.matchRate is undefined', () => {
      const store = useReconciliationStore();
      store.reconciliationSummary = {
        totalBankTransactions: 200,
        totalLedgerEntries: 200,
        matchedCount: 150,
        tier1Count: 100,
        tier2Count: 40,
        tier3Count: 10,
        hitlQuarantineCount: 50,
        matchRate: undefined as any,
        totalMatchedAmount: 1_000_000_000,
        totalFeeDisentangled: 0,
        matches: [],
        quarantined: [],
        unallocatedBankTransactions: [],
        unallocatedLedgerEntries: [],
      };

      // 150 / 200 * 100 = 75.0%
      expect(store.matchRate).toBe(75);
    });

    it('correctly handles non-integer rounding with toFixed(1)', () => {
      const store = useReconciliationStore();
      store.reconciliationSummary = {
        totalBankTransactions: 3,
        totalLedgerEntries: 3,
        matchedCount: 1,
        tier1Count: 1,
        tier2Count: 0,
        tier3Count: 0,
        hitlQuarantineCount: 2,
        matchRate: undefined as any,
        totalMatchedAmount: 100_000,
        totalFeeDisentangled: 0,
        matches: [],
        quarantined: [],
        unallocatedBankTransactions: [],
        unallocatedLedgerEntries: [],
      };

      // 1 / 3 = 33.3%
      expect(store.matchRate).toBe(33.3);

      store.reconciliationSummary!.matchedCount = 2;
      // 2 / 3 = 66.7%
      expect(store.matchRate).toBe(66.7);
    });

    it('fuzz tests 500 permutations: matchRate equals 99.8 ONLY when mathematically 99.8% or explicitly set', () => {
      const store = useReconciliationStore();

      for (let i = 0; i < 500; i++) {
        const total = Math.floor(Math.random() * 500) + 1; // 1 .. 500
        const matched = Math.floor(Math.random() * (total + 1)); // 0 .. total
        const expected = Number(((matched / total) * 100).toFixed(1));

        store.reconciliationSummary = {
          totalBankTransactions: total,
          totalLedgerEntries: total,
          matchedCount: matched,
          tier1Count: matched,
          tier2Count: 0,
          tier3Count: 0,
          hitlQuarantineCount: total - matched,
          matchRate: undefined as any,
          totalMatchedAmount: 0,
          totalFeeDisentangled: 0,
          matches: [],
          quarantined: [],
          unallocatedBankTransactions: [],
          unallocatedLedgerEntries: [],
        };

        expect(store.matchRate).toBe(expected);

        // If matched / total is not 99.8%, matchRate MUST NEVER be 99.8
        if (expected !== 99.8) {
          expect(store.matchRate).not.toBe(99.8);
        }
      }
    });

    it('scans reconciliationStore.ts source code to ensure 0 occurrences of hardcoded fallback to 99.8', () => {
      const storePath = path.join(rootDir, 'src', 'stores', 'reconciliationStore.ts');
      const content = fs.readFileSync(storePath, 'utf-8');

      // The old bug was `matchRate ?? 99.8`
      expect(content).not.toContain('?? 99.8');
      expect(content).not.toContain('|| 99.8');
      expect(content).not.toContain(': 99.8');
    });
  });

  // ==========================================================================
  // SECTION 2: ADVERSARIAL VERIFICATION OF FinancialCopilotDrawer.vue
  // ==========================================================================
  describe('2. Adversarial Verification of FinancialCopilotDrawer.vue', () => {
    const drawerPath = path.join(rootDir, 'src', 'components', 'copilot', 'FinancialCopilotDrawer.vue');
    const drawerContent = fs.readFileSync(drawerPath, 'utf-8');

    it('verifies withDefaults props in FinancialCopilotDrawer.vue have undefined defaults (no static 6.2B or 99.8)', () => {
      // Must define defaults as undefined
      expect(drawerContent).toContain('liveBalance: undefined');
      expect(drawerContent).toContain('matchRate: undefined');
      expect(drawerContent).toContain('pendingAlertsCount: undefined');

      // Must NOT contain hardcoded static defaults
      expect(drawerContent).not.toContain('liveBalance: 6_200_000_000');
      expect(drawerContent).not.toContain('liveBalance: 6200000000');
      expect(drawerContent).not.toContain('matchRate: 99.8');
      expect(drawerContent).not.toContain('pendingAlertsCount: 3');
    });

    it('verifies effectiveMatchRate logic renders "0.0% (Chưa đối soát)" when un-reconciled', () => {
      // In template:
      // {{ effectiveMatchRate > 0 ? `${effectiveMatchRate.toFixed(1)}%` : '0.0%' }}
      // {{ effectiveMatchRate >= 99.8 ? '(Đạt chuẩn)' : effectiveMatchRate > 0 ? '(Chưa đạt)' : '(Chưa đối soát)' }}
      expect(drawerContent).toContain("effectiveMatchRate > 0 ? `${effectiveMatchRate.toFixed(1)}%` : '0.0%'");
      expect(drawerContent).toContain("effectiveMatchRate >= 99.8 ? '(Đạt chuẩn)' : effectiveMatchRate > 0 ? '(Chưa đạt)' : '(Chưa đối soát)'");

      // Verify the simulated computed function under edge conditions
      const evaluateEffectiveMatchRate = (propMatchRate: number | undefined, storeMatchRate: number) => {
        if (propMatchRate !== undefined) return propMatchRate;
        return storeMatchRate;
      };

      const formatBadge = (rate: number) => {
        const text = rate > 0 ? `${rate.toFixed(1)}%` : '0.0%';
        const label = rate >= 99.8 ? '(Đạt chuẩn)' : rate > 0 ? '(Chưa đạt)' : '(Chưa đối soát)';
        return `${text} ${label}`;
      };

      // Case A: Unmounted/Mounted without props when store is un-reconciled (rate = 0)
      const unrecRate = evaluateEffectiveMatchRate(undefined, 0);
      expect(unrecRate).toBe(0);
      expect(formatBadge(unrecRate)).toBe('0.0% (Chưa đối soát)');
      expect(formatBadge(unrecRate)).not.toContain('99.8%');
      expect(formatBadge(unrecRate)).not.toContain('(Đạt chuẩn)');

      // Case B: Partial match 75.4%
      const partialRate = evaluateEffectiveMatchRate(undefined, 75.4);
      expect(formatBadge(partialRate)).toBe('75.4% (Chưa đạt)');

      // Case C: Fully reconciled 99.8%
      const fullRate = evaluateEffectiveMatchRate(undefined, 99.8);
      expect(formatBadge(fullRate)).toBe('99.8% (Đạt chuẩn)');

      // Case D: Explicit prop override with 0
      const propZero = evaluateEffectiveMatchRate(0, 99.8);
      expect(propZero).toBe(0);
      expect(formatBadge(propZero)).toBe('0.0% (Chưa đối soát)');
    });

    it('verifies effectiveBalance evaluates dynamically without hardcoded 6.2B fallback', () => {
      const evaluateEffectiveBalance = (propBalance: number | undefined, storeBalance: number) => {
        if (propBalance !== undefined) return propBalance;
        return storeBalance;
      };

      const bankingStore = useBankingStore();

      // Default bankingStore initial balance is the sum of the 4 accounts
      expect(bankingStore.totalLiquidVnd).toBe(6_200_000_000);

      // If accounts are updated (e.g. via SSE or rebalancing to 2.5B), effectiveBalance reflects it
      bankingStore.accounts[0].balanceVnd = 500_000_000;
      const updatedSum = bankingStore.totalLiquidVnd; // 3.45B
      expect(evaluateEffectiveBalance(undefined, updatedSum)).toBe(updatedSum);
      expect(evaluateEffectiveBalance(undefined, updatedSum)).not.toBe(6_200_000_000);

      // If propBalance is explicitly 0 VND (e.g. empty account), it MUST evaluate to 0, NOT 6.2B
      expect(evaluateEffectiveBalance(0, updatedSum)).toBe(0);
    });

    it('verifies App.vue passes dynamic live-balance, match-rate, and pending-alerts-count to FinancialCopilotDrawer', () => {
      const appPath = path.join(rootDir, 'src', 'App.vue');
      const appContent = fs.readFileSync(appPath, 'utf-8');

      expect(appContent).toContain(':live-balance="bankingStore.totalLiquidVnd"');
      expect(appContent).toContain(':match-rate="reconStore.matchRate"');
      expect(appContent).toContain(':pending-alerts-count="amlStore.totalAlertsCount"');
    });
  });

  // ==========================================================================
  // SECTION 3: ADVERSARIAL VERIFICATION OF AML SANDBOX ISOLATION
  // ==========================================================================
  describe('3. Adversarial Verification of AML Sandbox Isolation in ComplianceAmlView & amlStore', () => {
    it('verifies amlStore initial state has 0 operational alerts and 0 sandbox alerts', () => {
      const amlStore = useAmlStore();
      expect(amlStore.alerts).toHaveLength(0);
      expect(amlStore.sandboxAlerts).toHaveLength(0);
      expect(amlStore.totalAlertsCount).toBe(0);
      expect(amlStore.criticalAlertsCount).toBe(0);
      expect(amlStore.highAlertsCount).toBe(0);
    });

    it('adversarially triggers evaluation on ALL 4 sandbox presets and verifies ZERO leakage into operational alerts', () => {
      const amlStore = useAmlStore();

      for (const preset of AML_SANDBOX_PRESETS) {
        // Select preset and run sandbox evaluation
        amlStore.selectPreset(preset.id);
        expect(amlStore.sandboxPresetId).toBe(preset.id);
        expect(amlStore.sandboxTransactions).toEqual(preset.transactions);

        // Run evaluation explicitly
        amlStore.runSandboxEvaluation();

        // Invariants:
        // 1. If preset is an anomaly preset, sandboxAlerts MUST have detections
        if (preset.id !== 'NORMAL') {
          expect(
            amlStore.sandboxAlerts.length,
            `Sandbox alerts empty for preset ${preset.id}`
          ).toBeGreaterThan(0);
        }

        // 2. CRITICAL INVARIANT: store.alerts MUST REMAIN COMPLETELY EMPTY!
        expect(
          amlStore.alerts.length,
          `Preset ${preset.id} leaked into store.alerts!`
        ).toBe(0);

        // 3. Operational getters MUST ALL REMAIN ZERO
        expect(amlStore.totalAlertsCount).toBe(0);
        expect(amlStore.criticalAlertsCount).toBe(0);
        expect(amlStore.highAlertsCount).toBe(0);
      }
    });

    it('specifically verifies STRUCTURING smurfing preset generates sandbox alerts but 0 operational alerts', () => {
      const amlStore = useAmlStore();
      amlStore.selectPreset('STRUCTURING');

      expect(amlStore.sandboxTransactions).toHaveLength(3);
      // 390M + 385M + 395M = 1.17B VND smurfing
      expect(amlStore.sandboxAlerts.length).toBeGreaterThan(0);
      const smurfAlert = amlStore.sandboxAlerts.find((a) => a.anomalyType === 'STRUCTURING_SMURFING');
      expect(smurfAlert).toBeDefined();
      expect(smurfAlert?.severity).toBe('CRITICAL');

      // OPERATIONAL ALERTS MUST BE 0!
      expect(amlStore.alerts).toHaveLength(0);
      expect(amlStore.totalAlertsCount).toBe(0);
    });

    it('verifies ComplianceAmlView.vue computed activeAlerts derives strictly from amlStore.alerts', () => {
      const amlViewPath = path.join(rootDir, 'src', 'views', 'ComplianceAmlView.vue');
      const amlViewContent = fs.readFileSync(amlViewPath, 'utf-8');

      // Check computed activeAlerts definition
      expect(amlViewContent).toContain('const activeAlerts = computed<AmlAlert[]>(() => {');
      expect(amlViewContent).toContain('return amlStore.alerts;');

      // Must NOT return sandboxAlerts or concatenate them
      expect(amlViewContent).not.toContain('amlStore.sandboxAlerts');
      expect(amlViewContent).not.toContain('...amlStore.sandboxAlerts');

      // Verify onMounted does NOT call runSandboxEvaluation or inject fake alerts
      expect(amlViewContent).not.toContain('amlStore.runSandboxEvaluation');
      expect(amlViewContent).not.toContain('amlStore.selectPreset');
    });

    it('verifies operational alerts can be ingested via applySseAlert without interference from sandbox presets', () => {
      const amlStore = useAmlStore();

      // Trigger sandbox evaluation first
      amlStore.selectPreset('NIGHT_PASS_THROUGH');
      amlStore.runSandboxEvaluation();
      expect(amlStore.sandboxAlerts.length).toBeGreaterThan(0);
      expect(amlStore.alerts).toHaveLength(0);

      // Now receive genuine operational alert via SSE
      const realAlert: AmlAlert = {
        alertId: 'alt-real-citad-01',
        anomalyType: 'HIGH_VALUE',
        severity: 'HIGH',
        reasoning: 'Giao dịch chuyển tiền liên ngân hàng 500M VND',
        detectedAt: new Date().toISOString(),
        involvedTransactionIds: ['tx-citad-99'],
        totalAmount: 500_000_000,
        statutoryRuleRef: 'Quyết định 11/2023/QĐ-TTg',
        suggestedStrReport: true,
      };

      amlStore.applySseAlert(realAlert);

      // Invariants:
      // 1. Operational alerts now has exactly 1 item
      expect(amlStore.alerts).toHaveLength(1);
      expect(amlStore.totalAlertsCount).toBe(1);
      expect(amlStore.highAlertsCount).toBe(1);
      expect(amlStore.alerts[0].alertId).toBe('alt-real-citad-01');

      // 2. Running sandbox evaluation again does NOT wipe or corrupt operational alerts
      amlStore.selectPreset('STRUCTURING');
      amlStore.runSandboxEvaluation();
      expect(amlStore.alerts).toHaveLength(1);
      expect(amlStore.alerts[0].alertId).toBe('alt-real-citad-01');

      // 3. Deduplication prevents duplicate SSE alerts
      amlStore.applySseAlert(realAlert);
      expect(amlStore.alerts).toHaveLength(1);
    });
  });

  // ==========================================================================
  // SECTION 4: ADVERSARIAL VERIFICATION OF DYNAMIC TELEMETRY PROPAGATION
  // ==========================================================================
  describe('4. Dynamic Telemetry & Sibling Store Integration', () => {
    it('verifies bankingStore burn rate and runway days are computed dynamically from actual outflows', () => {
      const bankingStore = useBankingStore();

      // Default state has syncedDailyBurnVnd: 0
      expect(bankingStore.syncedDailyBurnVnd).toBe(0);

      // Outflows: 514.5M + 45M + 420M + 180M = 1,159,500,000 VND
      // Daily average burn = Math.round(1,159,500,000 / 30) = 38,650,000 VND
      const expectedDailyBurn = Math.round(1_159_500_000 / 30);
      expect(bankingStore.dailyAverageBurnVnd).toBe(expectedDailyBurn);
      expect(bankingStore.dailyAverageBurnVnd).not.toBe(85_000_000); // Old hardcoded static mock

      // Liquidity Runway Days: 6.2B / 38.65M = ~160 days
      const expectedRunway = Math.round(6_200_000_000 / expectedDailyBurn);
      expect(bankingStore.liquidityRunwayDays).toBe(expectedRunway);

      // When syncedDailyBurnVnd is received via SSE, it overrides the fallback
      bankingStore.syncedDailyBurnVnd = 50_000_000;
      expect(bankingStore.dailyAverageBurnVnd).toBe(50_000_000);
      expect(bankingStore.liquidityRunwayDays).toBe(Math.round(6_200_000_000 / 50_000_000));
    });

    it('verifies treasuryStore.processSseEvent routes events to sibling stores reactively', () => {
      const treasuryStore = useTreasuryStore();
      const bankingStore = useBankingStore();
      const reconStore = useReconciliationStore();
      const amlStore = useAmlStore();

      // 1. Test account:updated event
      treasuryStore.processSseEvent('account:updated', {
        channelCode: 'CITAD',
        balanceVnd: 2_500_000_000,
        availableBalanceVnd: 2_500_000_000,
        outflowMonthVnd: 600_000_000,
      });

      const citad = bankingStore.accounts.find((a) => a.channelCode === 'CITAD');
      expect(citad?.balanceVnd).toBe(2_500_000_000);
      expect(citad?.outflowMonthVnd).toBe(600_000_000);

      // 2. Test reconciliation:summary event
      treasuryStore.processSseEvent('reconciliation:summary', {
        matchedCount: 380,
        totalBankTransactions: 400,
        matchRate: 95.0,
      });
      expect(reconStore.matchRate).toBe(95);

      // 3. Test aml:alert event
      treasuryStore.processSseEvent('aml:alert', {
        alertId: 'alt-sse-dynamic-1',
        anomalyType: 'NIGHT_VELOCITY',
        severity: 'HIGH',
      });
      expect(amlStore.alerts).toHaveLength(1);
      expect(amlStore.alerts[0].alertId).toBe('alt-sse-dynamic-1');
    });

    it('verifies copilot query message receives live store values and generates dynamic response', () => {
      const treasuryStore = useTreasuryStore();
      const bankingStore = useBankingStore();
      const reconStore = useReconciliationStore();

      // Set custom state in stores
      bankingStore.accounts[0].balanceVnd = 1_000_000_000;
      bankingStore.accounts[1].balanceVnd = 500_000_000;
      bankingStore.accounts[2].balanceVnd = 500_000_000;
      bankingStore.accounts[3].balanceVnd = 0;
      // Total = 2.0B VND
      expect(bankingStore.totalLiquidVnd).toBe(2_000_000_000);

      // Reconcile some data
      reconStore.applyReconciliationSummary({
        totalBankTransactions: 100,
        matchedCount: 95,
        matchRate: 95.0,
      });
      expect(reconStore.matchRate).toBe(95);

      // Send copilot message for liquidity
      treasuryStore.sendCopilotMessage('Tình hình thanh khoản hiện tại?');
      const lastMsg = treasuryStore.copilotMessages[treasuryStore.copilotMessages.length - 1];
      expect(lastMsg.intent).toBe('LIQUIDITY_RUNWAY');
      expect(lastMsg.text).toContain('2.000.000.000 VND');

      // Send copilot message for reconciliation rate
      treasuryStore.sendCopilotMessage('Tỷ lệ đối soát hiện tại?');
      const reconMsg = treasuryStore.copilotMessages[treasuryStore.copilotMessages.length - 1];
      expect(reconMsg.intent).toBe('RECONCILIATION_RATE');
      expect(reconMsg.text).toContain('95%');
    });
  });
});
