import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useRiskStore } from '../../src/stores/riskStore';

describe('riskStore (P70–P75 Rủi Ro Tài Chính & Thẩm Định Tín Dụng)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('P71 DSCR & Solvency Ratios', () => {
    it('computes initial DSCR, buffer, and category correctly', () => {
      const store = useRiskStore();
      // EBITDA: 1.85B, CAPEX: 250M => NOI = 1.6B
      expect(store.netOperatingIncome).toBe(1600000000);

      // Principal: 650M, Interest: 200M => Total Debt Service = 850M
      expect(store.totalDebtService).toBe(850000000);

      // DSCR = (1.6B * 10,000) / 850M = 18,823 bps (~1.88x)
      expect(store.dscrData.ratioBps).toBe(18823);
      expect(store.dscrData.ratio).toBe(1.88);
      expect(store.dscrData.category).toBe('HEALTHY');
      expect(store.dscrData.buffer).toBe(750000000); // 1.6B - 850M = +750M VND
    });

    it('handles edge case: DEBT_FREE when debt obligations are zero', () => {
      const store = useRiskStore();
      store.updateFinancialInput({
        debtServicePrincipal: 0,
        debtServiceInterest: 0,
      });

      expect(store.totalDebtService).toBe(0);
      expect(store.dscrData.category).toBe('DEBT_FREE');
      expect(store.dscrData.ratio).toBe(0);
      expect(store.dscrData.buffer).toBe(store.netOperatingIncome);
    });

    it('handles edge case: DISTRESSED when operating income is negative (EBITDA < CAPEX)', () => {
      const store = useRiskStore();
      store.updateFinancialInput({
        ebitda: 100000000,
        capex: 300000000, // CAPEX exceeds EBITDA
      });

      expect(store.dscrData.category).toBe('DISTRESSED');
      expect(store.dscrData.ratio).toBe(0);
      expect(store.dscrData.buffer).toBeLessThan(0);
    });

    it('computes Quick, Current, Cash, and ICR liquidity ratios', () => {
      const store = useRiskStore();
      // Liquid Assets = 500M cash + 150M sec + 750M rec = 1.4B
      expect(store.liquidAssets).toBe(1400000000);
      // Quick Ratio = 1.4B / 900M = 1.5555x -> 1.56x (15,555 bps) -> STRONG
      expect(store.quickRatioData.ratio).toBe(1.56);
      expect(store.quickRatioData.status).toBe('STRONG');

      // Current Ratio = (1.4B + 450M inv) / 900M = 2.05x -> STRONG
      expect(store.currentRatioData.ratio).toBe(2.06);
      expect(store.currentRatioData.status).toBe('STRONG');

      // Cash Ratio = (500M + 150M) / 900M = 0.72x -> STRONG
      expect(store.cashRatioData.ratio).toBe(0.72);
      expect(store.cashRatioData.status).toBe('STRONG');

      // ICR = 1.6B / 200M = 8.0x -> STRONG
      expect(store.icrData.ratio).toBe(8.0);
      expect(store.icrData.status).toBe('STRONG');
    });
  });

  describe('P72 Circular 11/2021/TT-NHNN Debt Classification', () => {
    it('classifies standard debt (Group 1) with 0.75% general and 0% specific provision', () => {
      const store = useRiskStore();
      store.setDaysOverdue(0);

      const debt = store.debtClassification;
      expect(debt.group).toBe('GROUP_1');
      expect(debt.generalRateBps).toBe(75);
      expect(debt.specificRateBps).toBe(0);
      // Loan 1.5B * 0.75% = 11,250,000 VND
      expect(debt.requiredGeneral).toBe(11250000);
      expect(debt.requiredSpecific).toBe(0);
      expect(debt.totalProvision).toBe(11250000);
    });

    it('classifies special mention debt (Group 2) with 5% specific provision', () => {
      const store = useRiskStore();
      store.setDaysOverdue(45);

      const debt = store.debtClassification;
      expect(debt.group).toBe('GROUP_2');
      expect(debt.specificRateBps).toBe(500); // 5%
      // 1.5B * 5% = 75,000,000 VND
      expect(debt.requiredSpecific).toBe(75000000);
      expect(debt.totalProvision).toBe(11250000 + 75000000);
    });

    it('classifies sub-standard debt (Group 3) with 20% specific provision', () => {
      const store = useRiskStore();
      store.setDaysOverdue(120);

      const debt = store.debtClassification;
      expect(debt.group).toBe('GROUP_3');
      expect(debt.specificRateBps).toBe(2000); // 20%
      expect(debt.requiredSpecific).toBe(300000000); // 300M VND
    });

    it('classifies loss debt (Group 5) with 100% specific provision and 0% general provision', () => {
      const store = useRiskStore();
      store.setDaysOverdue(400);

      const debt = store.debtClassification;
      expect(debt.group).toBe('GROUP_5');
      expect(debt.specificRateBps).toBe(10000); // 100%
      expect(debt.generalRateBps).toBe(0);
      expect(debt.requiredSpecific).toBe(1500000000); // 1.5B VND
      expect(debt.requiredGeneral).toBe(0);
    });

    it('promotes restructured loans to higher risk group under Circular 11', () => {
      const store = useRiskStore();
      store.setDaysOverdue(5, true); // Restructured even with < 10 days overdue
      expect(store.debtClassification.group).toBe('GROUP_2');
      expect(store.debtClassification.specificRateBps).toBe(500);
    });
  });

  describe('P74 Stress Testing & P75 Credit Underwriting Decision', () => {
    it('evaluates Base, Moderate, and Severe stress test scenarios', () => {
      const store = useRiskStore();
      const scenarios = store.stressScenarios;

      // Base Case passes
      expect(scenarios.BASE.passes).toBe(true);
      expect(scenarios.BASE.stressedDscr).toBe(1.88);

      // Moderate stress (-15% EBITDA, +15% interest)
      expect(scenarios.MODERATE.stressedNoi).toBeLessThan(scenarios.BASE.stressedNoi);
      expect(scenarios.MODERATE.stressedDebtService).toBeGreaterThan(scenarios.BASE.stressedDebtService);
      expect(scenarios.MODERATE.passes).toBe(true);

      // Severe stress (-30% EBITDA, +30% interest, +10% CAPEX)
      expect(scenarios.SEVERE.stressedDscr).toBeLessThan(scenarios.MODERATE.stressedDscr);
    });

    it('recommends APPROVED decision for healthy borrower', () => {
      const store = useRiskStore();
      const rec = store.underwritingRecommendation;

      expect(rec.decision).toBe('APPROVED');
      expect(rec.recommendedLimit).toBeGreaterThan(0);
      expect(rec.rationale).toContain('Chấp thuận');
    });

    it('recommends REJECTED decision when debt group is Group 3 or worse', () => {
      const store = useRiskStore();
      store.setDaysOverdue(100); // Group 3

      const rec = store.underwritingRecommendation;
      expect(rec.decision).toBe('REJECTED');
      expect(rec.recommendedLimit).toBe(0);
      expect(rec.rationale).toContain('Từ chối');
    });

    it('recommends CONDITIONAL decision with 40% haircut when metrics are borderline', () => {
      const store = useRiskStore();
      // Increase debt service so DSCR drops into WATCHLIST (1.0 - 1.3)
      store.updateFinancialInput({
        debtServicePrincipal: 1100000000, // Total debt service = 1.3B, DSCR = 1.6B / 1.3B = 1.23x
      });

      const rec = store.underwritingRecommendation;
      expect(rec.decision).toBe('CONDITIONAL');
      expect(rec.rationale).toContain('Phê duyệt có điều kiện');
    });
  });
});
