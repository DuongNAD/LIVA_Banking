/**
 * Milestone 4 Web Harness Scaffolding & Integration Test Suite (F21)
 * Tests bankingStore, reconciliationStore, master App shell, and standalone view components.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

import { useBankingStore } from '../src/stores/bankingStore';
import { useReconciliationStore } from '../src/stores/reconciliationStore';
import { useAmlStore } from '../src/stores/amlStore';
import { useTreasuryStore } from '../src/stores/treasuryStore';

import App from '../src/App.vue';
import BankingDashboardView from '../src/views/BankingDashboardView.vue';
import ReconciliationWorkbenchView from '../src/views/ReconciliationWorkbenchView.vue';
import ComplianceAmlView from '../src/views/ComplianceAmlView.vue';
import TreasuryPaymentView from '../src/views/TreasuryPaymentView.vue';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

describe('Milestone 4: Web Harness Scaffolding & Zero-Backend Integration (F21)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('1. Banking Store & Interbank Settlement Channel Architecture', () => {
    it('initializes interbank clearing channels with correct initial balances', () => {
      const store = useBankingStore();
      expect(store.accounts.length).toBe(4);
      expect(store.selectedBankPersona).toBe('ALL');

      const citad = store.accounts.find((a) => a.channelCode === 'CITAD' || a.bankCode === 'CITAD');
      const napas = store.accounts.find((a) => a.channelCode === 'NAPAS' || a.bankCode === 'NAPAS');
      const bilateral = store.accounts.find((a) => a.channelCode === 'BILATERAL' || a.bankCode === 'BILATERAL');
      const swift = store.accounts.find((a) => a.channelCode === 'SWIFT' || a.bankCode === 'SWIFT');

      expect(citad).toBeDefined();
      expect(napas).toBeDefined();
      expect(bilateral).toBeDefined();
      expect(swift).toBeDefined();
      expect(store.totalLiquidVnd).toBeGreaterThan(5_000_000_000);
    });

    it('supports dedicated channel persona switching per operational directive', () => {
      const store = useBankingStore();

      store.setBankPersona('CITAD');
      expect(store.selectedBankPersona).toBe('CITAD');
      expect(store.displayedAccounts.length).toBe(1);
      expect(store.displayedAccounts[0].bankCode).toBe('CITAD');
      expect(store.currentPersonaAccount?.bankCode).toBe('CITAD');

      store.setBankPersona('NAPAS');
      expect(store.displayedAccounts.length).toBe(1);
      expect(store.displayedAccounts[0].bankCode).toBe('NAPAS');

      store.setBankPersona('ALL');
      expect(store.displayedAccounts.length).toBe(4);
      expect(store.currentPersonaAccount).toBeNull();
    });

    it('computes 30-day liquidity runway accurately', () => {
      const store = useBankingStore();
      const runway = store.liquidityRunwayDays;
      expect(runway).toBeGreaterThanOrEqual(30);
    });
  });

  describe('2. Reconciliation Store & Universal Ingestion Integration', () => {
    it('loads sample Vietcombank (.xlsx) dataset and computes balance invariant', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('VCB');

      expect(store.activeBankCode).toBe('VCB');
      expect(store.bankTransactions.length).toBe(8);
      expect(store.statementParseResult?.bankCode).toBe('VCB');
      expect(store.balanceInvariantReport?.isBalanced).toBe(true);
      expect(store.reconciliationSummary).not.toBeNull();
      expect(store.reconciliationSummary?.matchedCount).toBeGreaterThan(0);
    });

    it('loads sample CITAD interbank dataset and verifies channel mapping', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('CITAD');

      expect(store.activeBankCode).toBe('CITAD');
      expect(store.bankTransactions.length).toBe(8);
      expect(store.balanceInvariantReport?.isBalanced).toBe(true);
      expect(store.reconciliationSummary).not.toBeNull();
      expect(store.reconciliationSummary?.matchedCount).toBeGreaterThan(0);
    });

    it('loads sample Techcombank (.csv) dataset and executes 3-tier matching', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('TCB');

      expect(store.activeBankCode).toBe('TCB');
      expect(store.bankTransactions.length).toBe(5);
      expect(store.statementParseResult?.bankCode).toBe('TCB');
      expect(store.balanceInvariantReport?.isBalanced).toBe(true);
    });

    it('loads sample NAPAS 24/7 interbank dataset and verifies channel mapping', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('NAPAS');

      expect(store.activeBankCode).toBe('NAPAS');
      expect(store.bankTransactions.length).toBe(5);
      expect(store.balanceInvariantReport?.isBalanced).toBe(true);
    });

    it('loads sample BIDV (.csv) dataset with night anomaly and rapid pass-through', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('BIDV');

      expect(store.activeBankCode).toBe('BIDV');
      expect(store.bankTransactions.length).toBe(4);
      expect(store.statementParseResult?.bankCode).toBe('BIDV');
      expect(store.balanceInvariantReport?.isBalanced).toBe(true);
    });

    it('loads sample BILATERAL interbank dataset and verifies channel mapping', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('BILATERAL');

      expect(store.activeBankCode).toBe('BILATERAL');
      expect(store.bankTransactions.length).toBe(4);
      expect(store.balanceInvariantReport?.isBalanced).toBe(true);
    });

    it('loads merged multi-bank dataset (ALL) with verified macro balance invariant', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('ALL');

      expect(store.activeBankCode).toBe('ALL');
      expect(store.bankTransactions.length).toBe(17);
      expect(store.reconciliationSummary?.totalBankTransactions).toBe(17);
      expect(store.reconciliationSummary?.tier1Count).toBeGreaterThanOrEqual(1);
      expect(store.reconciliationSummary?.tier2Count).toBeGreaterThanOrEqual(4);
      expect(store.isBalanced).toBe(true);
      expect(store.discrepancyAmount).toBe(0);
    });

    it('parses pasted Google Sheets text accurately', () => {
      const store = useReconciliationStore();
      store.loadSampleDataset('SHEETS');

      expect(store.bankTransactions.length).toBe(8);
      expect(store.statementParseResult).not.toBeNull();
      expect(store.bankTransactions[0].amount).toBe(145000000);
    });
  });

  describe('3. Cross-Store Reactive Integration', () => {
    it('synchronizes AML scan with loaded bank transactions', () => {
      const reconStore = useReconciliationStore();
      const amlStore = useAmlStore();

      reconStore.loadSampleDataset('ALL');
      const alerts = amlStore.scanTransactions(reconStore.bankTransactions);

      expect(alerts.length).toBeGreaterThan(0);
      const highVal = alerts.find((a) => a.anomalyType === 'HIGH_VALUE');
      expect(highVal).toBeDefined();
    });

    it('synchronizes Treasury vouchers with Merkle root computation', () => {
      const treasuryStore = useTreasuryStore();
      const root = treasuryStore.merkleRoot;

      expect(root).toBeDefined();
      expect(root.length).toBe(64);
      expect(treasuryStore.vouchers.length).toBeGreaterThanOrEqual(3);
    });
  });

  describe('4. Standalone View Component Definitions', () => {
    it('exports valid Vue components for all 4 views and master shell', () => {
      expect(App).toBeDefined();
      expect(typeof App).toBe('object');

      expect(BankingDashboardView).toBeDefined();
      expect(typeof BankingDashboardView).toBe('object');

      expect(ReconciliationWorkbenchView).toBeDefined();
      expect(typeof ReconciliationWorkbenchView).toBe('object');

      expect(ComplianceAmlView).toBeDefined();
      expect(typeof ComplianceAmlView).toBe('object');

      expect(TreasuryPaymentView).toBeDefined();
      expect(typeof TreasuryPaymentView).toBe('object');
    });
  });

  describe('5. Deployment & Vercel Static Hosting Readiness', () => {
    it('confirms vercel.json exists at root with wildcard SPA rewrite rule', () => {
      const vercelPath = path.join(rootDir, 'vercel.json');
      expect(fs.existsSync(vercelPath)).toBe(true);

      const vercelConfig = JSON.parse(fs.readFileSync(vercelPath, 'utf-8'));
      expect(Array.isArray(vercelConfig.rewrites)).toBe(true);
      expect(vercelConfig.rewrites.some((r: any) => r.destination === '/index.html')).toBe(true);
      expect(vercelConfig.framework).toBe('vite');
    });

    it('verifies index.html mounts Vue application via #app and /src/main.ts', () => {
      const indexPath = path.join(rootDir, 'index.html');
      expect(fs.existsSync(indexPath)).toBe(true);

      const html = fs.readFileSync(indexPath, 'utf-8');
      expect(html).toContain('id="app"');
      expect(html).toContain('src="/src/main.ts"');
    });
  });
});
