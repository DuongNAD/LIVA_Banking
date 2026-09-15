import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '../utils/logger';

export type RiskCategory = 'HEALTHY' | 'WATCHLIST' | 'DISTRESSED' | 'DEBT_FREE';
export type LiquidityStatus = 'STRONG' | 'ADEQUATE' | 'CRITICAL';
export type DebtGroup = 'GROUP_1' | 'GROUP_2' | 'GROUP_3' | 'GROUP_4' | 'GROUP_5';
export type StressScenarioKey = 'BASE' | 'MODERATE' | 'SEVERE';
export type UnderwritingStatus = 'APPROVED' | 'CONDITIONAL' | 'REJECTED';

export interface FinancialInput {
  ebitda: number;
  capex: number;
  debtServicePrincipal: number;
  debtServiceInterest: number;
  cashAndEquivalents: number;
  marketableSecurities: number;
  accountsReceivable: number;
  inventory: number;
  currentLiabilities: number;
  ebit: number;
  interestExpense: number;
  outstandingLoan: number;
  daysOverdue: number;
  isRestructured: boolean;
}

export interface StressScenarioResult {
  scenario: StressScenarioKey;
  label: string;
  stressedNoi: number;
  stressedDebtService: number;
  stressedDscr: number;
  stressedDscrBps: number;
  category: RiskCategory;
  buffer: number;
  passes: boolean;
}

export const useRiskStore = defineStore('risk', () => {
  const applicantName = ref('Công ty Cổ phần Xây Lắp Điện Miền Nam');
  const taxCode = ref('0314892180');
  const creditFacilityType = ref('Hạn mức vốn lưu động bổ sung');

  const financialInput = ref<FinancialInput>({
    ebitda: 1850000000, // 1.85 Tỷ VND
    capex: 250000000, // 250 Tr VND
    debtServicePrincipal: 650000000, // 650 Tr VND
    debtServiceInterest: 200000000, // 200 Tr VND
    cashAndEquivalents: 500000000, // 500 Tr VND
    marketableSecurities: 150000000, // 150 Tr VND
    accountsReceivable: 750000000, // 750 Tr VND
    inventory: 450000000, // 450 Tr VND
    currentLiabilities: 900000000, // 900 Tr VND
    ebit: 1600000000, // 1.6 Tỷ VND
    interestExpense: 200000000, // 200 Tr VND
    outstandingLoan: 1500000000, // 1.5 Tỷ VND
    daysOverdue: 0,
    isRestructured: false,
  });

  const activeStressScenario = ref<StressScenarioKey>('BASE');

  // --- Computed Ratios (Zero-float integer basis points math) ---

  const netOperatingIncome = computed(() => {
    return Math.max(0, financialInput.value.ebitda - financialInput.value.capex);
  });

  const totalDebtService = computed(() => {
    return financialInput.value.debtServicePrincipal + financialInput.value.debtServiceInterest;
  });

  const dscrData = computed(() => {
    const debt = totalDebtService.value;
    const noi = netOperatingIncome.value;

    if (debt === 0) {
      return {
        ratio: 0,
        ratioBps: 0,
        category: 'DEBT_FREE' as RiskCategory,
        buffer: noi,
      };
    }

    if (financialInput.value.ebitda < financialInput.value.capex) {
      return {
        ratio: 0,
        ratioBps: 0,
        category: 'DISTRESSED' as RiskCategory,
        buffer: -(financialInput.value.capex - financialInput.value.ebitda + debt),
      };
    }

    const bps = Math.min(999999, Math.floor((noi * 10000) / debt));
    const ratio = Math.round(bps / 100) / 100;

    let category: RiskCategory = 'DISTRESSED';
    if (bps >= 13000) category = 'HEALTHY';
    else if (bps >= 10000) category = 'WATCHLIST';

    const buffer = noi - debt;

    return { ratio, ratioBps: bps, category, buffer };
  });

  const liquidAssets = computed(() => {
    return (
      financialInput.value.cashAndEquivalents +
      financialInput.value.marketableSecurities +
      financialInput.value.accountsReceivable
    );
  });

  const quickRatioData = computed(() => {
    const liab = financialInput.value.currentLiabilities;
    if (liab === 0) return { ratio: 99.9, ratioBps: 999999, status: 'STRONG' as LiquidityStatus };

    const bps = Math.min(999999, Math.floor((liquidAssets.value * 10000) / liab));
    const ratio = Math.round(bps / 100) / 100;
    const status: LiquidityStatus = bps >= 15000 ? 'STRONG' : bps >= 10000 ? 'ADEQUATE' : 'CRITICAL';

    return { ratio, ratioBps: bps, status };
  });

  const currentRatioData = computed(() => {
    const liab = financialInput.value.currentLiabilities;
    const totalCurrentAssets = liquidAssets.value + financialInput.value.inventory;
    if (liab === 0) return { ratio: 99.9, ratioBps: 999999, status: 'STRONG' as LiquidityStatus };

    const bps = Math.min(999999, Math.floor((totalCurrentAssets * 10000) / liab));
    const ratio = Math.round(bps / 100) / 100;
    const status: LiquidityStatus = bps >= 15000 ? 'STRONG' : bps >= 10000 ? 'ADEQUATE' : 'CRITICAL';

    return { ratio, ratioBps: bps, status };
  });

  const cashRatioData = computed(() => {
    const liab = financialInput.value.currentLiabilities;
    const cashTotal = financialInput.value.cashAndEquivalents + financialInput.value.marketableSecurities;
    if (liab === 0) return { ratio: 99.9, ratioBps: 999999, status: 'STRONG' as LiquidityStatus };

    const bps = Math.min(999999, Math.floor((cashTotal * 10000) / liab));
    const ratio = Math.round(bps / 100) / 100;
    const status: LiquidityStatus = bps >= 5000 ? 'STRONG' : bps >= 2000 ? 'ADEQUATE' : 'CRITICAL';

    return { ratio, ratioBps: bps, status };
  });

  const icrData = computed(() => {
    const intExp = financialInput.value.interestExpense;
    if (intExp === 0) return { ratio: 99.9, ratioBps: 999999, status: 'STRONG' as LiquidityStatus };

    const bps = Math.min(999999, Math.floor((financialInput.value.ebit * 10000) / intExp));
    const ratio = Math.round(bps / 100) / 100;
    const status: LiquidityStatus = bps >= 30000 ? 'STRONG' : bps >= 15000 ? 'ADEQUATE' : 'CRITICAL';

    return { ratio, ratioBps: bps, status };
  });

  // --- Circular 11/2021/TT-NHNN Debt Classification ---

  const debtClassification = computed(() => {
    const overdue = financialInput.value.daysOverdue;
    const restructured = financialInput.value.isRestructured;

    let group: DebtGroup = 'GROUP_1';
    let groupName = 'Nhóm 1 — Nợ đủ tiêu chuẩn';
    let generalRateBps = 75; // 0.75%
    let specificRateBps = 0;

    if (restructured) {
      if (overdue < 10) {
        group = 'GROUP_2';
        groupName = 'Nhóm 2 — Nợ cần chú ý (Cơ cấu lại lần đầu)';
        specificRateBps = 500;
      } else if (overdue <= 90) {
        group = 'GROUP_3';
        groupName = 'Nhóm 3 — Nợ dưới tiêu chuẩn (Cơ cấu lại quá hạn)';
        specificRateBps = 2000;
      } else if (overdue <= 180) {
        group = 'GROUP_4';
        groupName = 'Nhóm 4 — Nợ nghi ngờ (Cơ cấu lại)';
        specificRateBps = 5000;
      } else {
        group = 'GROUP_5';
        groupName = 'Nhóm 5 — Nợ có khả năng mất vốn';
        generalRateBps = 0;
        specificRateBps = 10000;
      }
    } else if (overdue < 10) {
      group = 'GROUP_1';
      groupName = 'Nhóm 1 — Nợ đủ tiêu chuẩn';
      specificRateBps = 0;
    } else if (overdue <= 90) {
      group = 'GROUP_2';
      groupName = 'Nhóm 2 — Nợ cần chú ý';
      specificRateBps = 500;
    } else if (overdue <= 180) {
      group = 'GROUP_3';
      groupName = 'Nhóm 3 — Nợ dưới tiêu chuẩn';
      specificRateBps = 2000;
    } else if (overdue <= 360) {
      group = 'GROUP_4';
      groupName = 'Nhóm 4 — Nợ nghi ngờ';
      specificRateBps = 5000;
    } else {
      group = 'GROUP_5';
      groupName = 'Nhóm 5 — Nợ có khả năng mất vốn';
      generalRateBps = 0;
      specificRateBps = 10000;
    }

    const loan = financialInput.value.outstandingLoan;
    const requiredGeneral = Math.floor((loan * generalRateBps) / 10000);
    const requiredSpecific = Math.floor((loan * specificRateBps) / 10000);
    const totalProvision = requiredGeneral + requiredSpecific;

    return {
      group,
      groupName,
      daysOverdue: overdue,
      isRestructured: restructured,
      generalRateBps,
      specificRateBps,
      requiredGeneral,
      requiredSpecific,
      totalProvision,
    };
  });

  // --- Stress Testing Scenarios ---

  const stressScenarios = computed<Record<StressScenarioKey, StressScenarioResult>>(() => {
    const calcScenario = (
      key: StressScenarioKey,
      label: string,
      ebitdaFactor: number,
      capexFactor: number,
      interestFactor: number
    ): StressScenarioResult => {
      const sEbitda = Math.floor(financialInput.value.ebitda * ebitdaFactor);
      const sCapex = Math.floor(financialInput.value.capex * capexFactor);
      const sNoi = Math.max(0, sEbitda - sCapex);

      const sInt = Math.floor(financialInput.value.debtServiceInterest * interestFactor);
      const sDebt = financialInput.value.debtServicePrincipal + sInt;

      if (sDebt === 0) {
        return {
          scenario: key,
          label,
          stressedNoi: sNoi,
          stressedDebtService: 0,
          stressedDscr: 0,
          stressedDscrBps: 0,
          category: 'DEBT_FREE',
          buffer: sNoi,
          passes: true,
        };
      }

      if (sEbitda < sCapex) {
        return {
          scenario: key,
          label,
          stressedNoi: 0,
          stressedDebtService: sDebt,
          stressedDscr: 0,
          stressedDscrBps: 0,
          category: 'DISTRESSED',
          buffer: -(sCapex - sEbitda + sDebt),
          passes: false,
        };
      }

      const bps = Math.min(999999, Math.floor((sNoi * 10000) / sDebt));
      const ratio = Math.round(bps / 100) / 100;
      let category: RiskCategory = 'DISTRESSED';
      if (bps >= 13000) category = 'HEALTHY';
      else if (bps >= 10000) category = 'WATCHLIST';

      const buffer = sNoi - sDebt;
      const passes = bps >= 10000 && category !== 'DISTRESSED';

      return {
        scenario: key,
        label,
        stressedNoi: sNoi,
        stressedDebtService: sDebt,
        stressedDscr: ratio,
        stressedDscrBps: bps,
        category,
        buffer,
        passes,
      };
    };

    return {
      BASE: calcScenario('BASE', 'Kịch bản Cơ sở (Bình thường)', 1.0, 1.0, 1.0),
      MODERATE: calcScenario('MODERATE', 'Căng thẳng Vừa phải (-15% EBITDA, +15% Lãi suất)', 0.85, 1.0, 1.15),
      SEVERE: calcScenario('SEVERE', 'Căng thẳng Nghiêm trọng (-30% EBITDA, +30% Lãi suất, +10% CAPEX)', 0.7, 1.1, 1.3),
    };
  });

  // --- Credit Underwriting Decision ---

  const underwritingRecommendation = computed(() => {
    const noi = netOperatingIncome.value;
    const maxAnnualService = Math.floor((noi * 10000) / 13000);
    const maxLimit = maxAnnualService * 3;

    const dscr = dscrData.value;
    const quick = quickRatioData.value;
    const debt = debtClassification.value;
    const moderate = stressScenarios.value.MODERATE;

    let decision: UnderwritingStatus = 'REJECTED';
    let recommendedLimit = 0;
    let rationale = '';

    if (
      debt.group === 'GROUP_3' ||
      debt.group === 'GROUP_4' ||
      debt.group === 'GROUP_5' ||
      dscr.category === 'DISTRESSED'
    ) {
      decision = 'REJECTED';
      recommendedLimit = 0;
      rationale =
        'Từ chối cấp hạn mức: Doanh nghiệp thuộc nhóm nợ dưới tiêu chuẩn / xấu hoặc DSCR dưới 1.0x (nguy cơ mất khả năng thanh toán cao).';
    } else if (
      dscr.category === 'HEALTHY' &&
      quick.status !== 'CRITICAL' &&
      debt.group === 'GROUP_1' &&
      moderate.passes
    ) {
      decision = 'APPROVED';
      recommendedLimit = maxLimit;
      rationale =
        'Chấp thuận cấp hạn mức tín dụng: DSCR thặng dư mạnh (>= 1.3x), thanh khoản tốt, nợ nhóm 1 và vượt qua bài kiểm tra áp lực vừa phải.';
    } else {
      decision = 'CONDITIONAL';
      recommendedLimit = Math.floor((maxLimit * 60) / 100);
      rationale =
        'Phê duyệt có điều kiện: Cắt giảm hạn mức 40%, yêu cầu tỷ lệ bảo đảm bằng bất động sản hoặc tiền gửi tối thiểu 150%.';
    }

    return {
      decision,
      recommendedLimit,
      rationale,
      targetDscr: 1.3,
    };
  });

  // --- Actions ---

  function updateFinancialInput(params: Partial<FinancialInput>) {
    Object.assign(financialInput.value, params);
    logger.info('Đã cập nhật chỉ tiêu tài chính phục vụ thẩm định rủi ro');
  }

  function setStressScenario(scenario: StressScenarioKey) {
    activeStressScenario.value = scenario;
    logger.info(`Chuyển kịch bản stress test sang: ${scenario}`);
  }

  function setDaysOverdue(days: number, restructured = false) {
    financialInput.value.daysOverdue = days;
    financialInput.value.isRestructured = restructured;
    logger.info(`Cập nhật số ngày quá hạn: ${days} ngày, cơ cấu: ${restructured}`);
  }

  return {
    applicantName,
    taxCode,
    creditFacilityType,
    financialInput,
    activeStressScenario,
    netOperatingIncome,
    totalDebtService,
    dscrData,
    liquidAssets,
    quickRatioData,
    currentRatioData,
    cashRatioData,
    icrData,
    debtClassification,
    stressScenarios,
    underwritingRecommendation,
    updateFinancialInput,
    setStressScenario,
    setDaysOverdue,
  };
});
