import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useReportStore } from '../../src/stores/reportStore';

describe('reportStore Pinia Unit Tests (P90–P95)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('Trial Balance Invariants (Mẫu F01-DN Circular 200)', () => {
    it('verifies opening, period, and closing balance invariants match exactly', () => {
      const store = useReportStore();

      expect(store.totalOpeningDebit).toBe(store.totalOpeningCredit);
      expect(store.totalPeriodDebit).toBe(store.totalPeriodCredit);
      expect(store.totalClosingDebit).toBe(store.totalClosingCredit);
      expect(store.isTrialBalanceBalanced).toBe(true);
    });

    it('detects trial balance discrepancies when debit and credit mismatch', () => {
      const store = useReportStore();

      // Introduce a 1 VND discrepancy to test strict zero drift
      store.trialBalanceLines[0].closingDebit += 1;
      expect(store.isTrialBalanceBalanced).toBe(false);
    });
  });

  describe('Income Statement Metrics (Mẫu B02-DN Waterfall)', () => {
    it('calculates net revenue, gross profit, and profit after tax accurately', () => {
      const store = useReportStore();
      const pnl = store.incomeStatement;

      expect(pnl.netRevenue).toBe(pnl.grossRevenue - pnl.revenueDeductions);
      expect(pnl.grossProfit).toBe(pnl.netRevenue - pnl.cogs);
      expect(pnl.profitBeforeTax).toBe(1440000000);
      expect(pnl.currentCitExpense).toBe(288000000); // 20% of 1.44B
      expect(pnl.netProfitAfterTax).toBe(1152000000);
    });

    it('computes gross profit and net profit margin in basis points', () => {
      const store = useReportStore();
      expect(store.grossProfitMarginBps).toBe(4336); // 43.36%
      expect(store.netProfitMarginBps).toBe(2250);   // 22.50%
    });
  });

  describe('Cash Flow Invariants (Mẫu B03-DN Direct Method)', () => {
    it('verifies that net cash flow matches sum of 3 activity cashflows', () => {
      const store = useReportStore();
      const cf = store.cashFlow;

      const sumNetFlows = cf.netOperatingFlow + cf.netInvestingFlow + cf.netFinancingFlow;
      expect(cf.netCashFlow).toBe(sumNetFlows);
      expect(cf.closingCash).toBe(cf.openingCash + cf.netCashFlow);
    });
  });

  describe('Statutory Tax Declarations (Circular 80/2021/TT-BTC)', () => {
    it('verifies VAT return indicators [25], [35], and net payable [40a]', () => {
      const store = useReportStore();
      const vat = store.vatReturn;

      expect(vat.totalOutputTax).toBe(vat.outputTax8pct + vat.outputTax10pct);
      expect(vat.netVatPayable).toBe(vat.totalOutputTax - vat.deductibleInputTax);
      expect(vat.netVatPayable).toBe(278000000);
      expect(vat.carriedForwardTax).toBe(0);
    });

    it('verifies CIT finalization with B4 non-deductible add-backs and remaining tax liability', () => {
      const store = useReportStore();
      const cit = store.citFinalization;

      expect(cit.taxableIncome).toBe(cit.accountingPbt + cit.nonDeductibleExpenses);
      expect(cit.totalCitLiability).toBe((cit.taxableIncome * cit.taxRateBps) / 10000);
      expect(cit.remainingTaxPayable).toBe(cit.totalCitLiability - cit.provisionalTaxPaid);
      expect(cit.remainingTaxPayable).toBe(60000000);
    });

    it('exports structured XML payloads conforming to General Department of Taxation (GDT / HTKK)', () => {
      const store = useReportStore();

      const vatXml = store.exportReportXml('VAT_01_GTGT');
      expect(vatXml).toContain('<maTKhai>01/GTGT</maTKhai>');
      expect(vatXml).toContain('<ct40a>278000000</ct40a>');

      const citXml = store.exportReportXml('CIT_03_TNDN');
      expect(citXml).toContain('<maTKhai>03/TNDN</maTKhai>');
      expect(citXml).toContain('<ctC4>1500000000</ctC4>');
      expect(citXml).toContain('<ctG>60000000</ctG>');
    });
  });

  describe('Period Selection & Modal State', () => {
    it('switches reporting periods and controls tax modal state', () => {
      const store = useReportStore();

      store.setPeriod('2026-Q4');
      expect(store.currentPeriod).toBe('2026-Q4');

      store.openTaxModal('VAT');
      expect(store.activeTaxModal).toBe('VAT');

      store.openTaxModal('CIT');
      expect(store.activeTaxModal).toBe('CIT');

      store.closeTaxModal();
      expect(store.activeTaxModal).toBeNull();
    });
  });
});
