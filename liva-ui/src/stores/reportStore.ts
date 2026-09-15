import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '../utils/logger';

export type ReportingPeriod = '2026-Q1' | '2026-Q2' | '2026-Q3' | '2026-Q4' | '2026-FY';
export type ReportTab = 'TRIAL_BALANCE' | 'INCOME_STATEMENT' | 'CASH_FLOW' | 'TAX_RETURNS';

export interface TrialBalanceLineItem {
  accountCode: string;
  accountName: string;
  openingDebit: number;
  openingCredit: number;
  periodDebit: number;
  periodCredit: number;
  closingDebit: number;
  closingCredit: number;
}

export interface IncomeStatementData {
  grossRevenue: number;         // Mã 01
  revenueDeductions: number;    // Mã 02
  netRevenue: number;           // Mã 10 = 01 - 02
  cogs: number;                 // Mã 11
  grossProfit: number;          // Mã 20 = 10 - 11
  financialIncome: number;      // Mã 21
  financialExpense: number;     // Mã 22
  interestExpense: number;      // Mã 23
  sellingExpense: number;       // Mã 25
  adminExpense: number;         // Mã 26
  operatingProfit: number;      // Mã 30 = 20 + 21 - 22 - 25 - 26
  otherIncome: number;          // Mã 31
  otherExpense: number;         // Mã 32
  otherProfit: number;          // Mã 40 = 31 - 32
  profitBeforeTax: number;      // Mã 50 = 30 + 40
  currentCitExpense: number;    // Mã 51 = 50 * 20%
  netProfitAfterTax: number;    // Mã 60 = 50 - 51
}

export interface CashFlowData {
  operatingInflow: number;
  operatingOutflow: number;
  netOperatingFlow: number;     // Mã 20
  investingInflow: number;
  investingOutflow: number;
  netInvestingFlow: number;     // Mã 30
  financingInflow: number;
  financingOutflow: number;
  netFinancingFlow: number;     // Mã 40
  netCashFlow: number;          // Mã 50 = 20 + 30 + 40
  openingCash: number;          // Mã 60
  closingCash: number;          // Mã 70 = 60 + 50
}

export interface VatReturnData {
  deductibleInputTax: number;   // [25]
  taxableSales8pct: number;     // [26]
  outputTax8pct: number;        // [27]
  taxableSales10pct: number;    // [32]
  outputTax10pct: number;       // [33]
  totalOutputTax: number;       // [35]
  netVatPayable: number;        // [40a]
  carriedForwardTax: number;    // [43]
}

export interface CitFinalizationData {
  taxYear: string;
  accountingPbt: number;          // [A1]
  nonDeductibleExpenses: number;  // [B4]
  taxableIncome: number;          // [C4]
  taxRateBps: number;             // [C7] = 2000 (20%)
  totalCitLiability: number;      // [C8]
  provisionalTaxPaid: number;     // [E1]
  remainingTaxPayable: number;    // [G] dương
  overpaidTax: number;            // [G] âm
}

export const useReportStore = defineStore('report', () => {
  const currentPeriod = ref<ReportingPeriod>('2026-Q3');
  const activeTab = ref<ReportTab>('TRIAL_BALANCE');
  const activeTaxModal = ref<'VAT' | 'CIT' | null>(null);

  // 1. F01-DN Trial Balance Lines
  const trialBalanceLines = ref<TrialBalanceLineItem[]>([
    {
      accountCode: '111',
      accountName: 'Tiền mặt tại quỹ',
      openingDebit: 85000000,
      openingCredit: 0,
      periodDebit: 210000000,
      periodCredit: 195000000,
      closingDebit: 100000000,
      closingCredit: 0,
    },
    {
      accountCode: '1121',
      accountName: 'Tiền gửi ngân hàng Vietcombank',
      openingDebit: 1450000000,
      openingCredit: 0,
      periodDebit: 4200000000,
      periodCredit: 3850000000,
      closingDebit: 1800000000,
      closingCredit: 0,
    },
    {
      accountCode: '1122',
      accountName: 'Tiền gửi ngân hàng Techcombank',
      openingDebit: 620000000,
      openingCredit: 0,
      periodDebit: 1850000000,
      periodCredit: 1670000000,
      closingDebit: 800000000,
      closingCredit: 0,
    },
    {
      accountCode: '131',
      accountName: 'Phải thu của khách hàng',
      openingDebit: 950000000,
      openingCredit: 0,
      periodDebit: 5120000000,
      periodCredit: 4870000000,
      closingDebit: 1200000000,
      closingCredit: 0,
    },
    {
      accountCode: '156',
      accountName: 'Hàng hóa kho trung tâm',
      openingDebit: 1800000000,
      openingCredit: 0,
      periodDebit: 3200000000,
      periodCredit: 2900000000,
      closingDebit: 2100000000,
      closingCredit: 0,
    },
    {
      accountCode: '211',
      accountName: 'Tài sản cố định hữu hình',
      openingDebit: 4500000000,
      openingCredit: 0,
      periodDebit: 0,
      periodCredit: 0,
      closingDebit: 4500000000,
      closingCredit: 0,
    },
    {
      accountCode: '331',
      accountName: 'Phải trả cho người bán',
      openingDebit: 0,
      openingCredit: 1200000000,
      periodDebit: 3400000000,
      periodCredit: 3600000000,
      closingDebit: 0,
      closingCredit: 1400000000,
    },
    {
      accountCode: '3331',
      accountName: 'Thuế GTGT phải nộp',
      openingDebit: 0,
      openingCredit: 120000000,
      periodDebit: 280000000,
      periodCredit: 310000000,
      closingDebit: 0,
      closingCredit: 150000000,
    },
    {
      accountCode: '341',
      accountName: 'Vay và nợ thuê tài chính VCB',
      openingDebit: 0,
      openingCredit: 2500000000,
      periodDebit: 500000000,
      periodCredit: 0,
      closingDebit: 0,
      closingCredit: 2000000000,
    },
    {
      accountCode: '411',
      accountName: 'Vốn đầu tư của chủ sở hữu',
      openingDebit: 0,
      openingCredit: 5585000000,
      periodDebit: 0,
      periodCredit: 0,
      closingDebit: 0,
      closingCredit: 5585000000,
    },
    {
      accountCode: '421',
      accountName: 'Lợi nhuận sau thuế chưa phân phối',
      openingDebit: 0,
      openingCredit: 0,
      periodDebit: 0,
      periodCredit: 1365000000,
      closingDebit: 0,
      closingCredit: 1365000000,
    },
    {
      accountCode: '511',
      accountName: 'Doanh thu bán hàng và CCDV',
      openingDebit: 0,
      openingCredit: 0,
      periodDebit: 5120000000,
      periodCredit: 5120000000,
      closingDebit: 0,
      closingCredit: 0,
    },
    {
      accountCode: '632',
      accountName: 'Giá vốn hàng bán',
      openingDebit: 0,
      openingCredit: 0,
      periodDebit: 2900000000,
      periodCredit: 2900000000,
      closingDebit: 0,
      closingCredit: 0,
    },
    {
      accountCode: '641',
      accountName: 'Chi phí bán hàng',
      openingDebit: 0,
      openingCredit: 0,
      periodDebit: 320000000,
      periodCredit: 320000000,
      closingDebit: 0,
      closingCredit: 0,
    },
    {
      accountCode: '642',
      accountName: 'Chi phí quản lý doanh nghiệp',
      openingDebit: 0,
      openingCredit: 0,
      periodDebit: 410000000,
      periodCredit: 410000000,
      closingDebit: 0,
      closingCredit: 0,
    },
    {
      accountCode: '911',
      accountName: 'Xác định kết quả kinh doanh',
      openingDebit: 0,
      openingCredit: 0,
      periodDebit: 5120000000,
      periodCredit: 5120000000,
      closingDebit: 0,
      closingCredit: 0,
    },
  ]);

  // 2. B02-DN Income Statement Data
  const incomeStatement = ref<IncomeStatementData>({
    grossRevenue: 5120000000,
    revenueDeductions: 0,
    netRevenue: 5120000000,
    cogs: 2900000000,
    grossProfit: 2220000000,
    financialIncome: 35000000,
    financialExpense: 95000000,
    interestExpense: 75000000,
    sellingExpense: 320000000,
    adminExpense: 410000000,
    operatingProfit: 1430000000,
    otherIncome: 15000000,
    otherExpense: 5000000,
    otherProfit: 10000000,
    profitBeforeTax: 1440000000,
    currentCitExpense: 288000000,
    netProfitAfterTax: 1152000000,
  });

  // 3. B03-DN Cash Flow Data
  const cashFlow = ref<CashFlowData>({
    operatingInflow: 4850000000,
    operatingOutflow: 3420000000,
    netOperatingFlow: 1430000000,
    investingInflow: 0,
    investingOutflow: 350000000,
    netInvestingFlow: -350000000,
    financingInflow: 0,
    financingOutflow: 500000000,
    netFinancingFlow: -500000000,
    netCashFlow: 580000000,
    openingCash: 2155000000,
    closingCash: 2735000000,
  });

  // 4. Mẫu 01/GTGT VAT Return Data
  const vatReturn = ref<VatReturnData>({
    deductibleInputTax: 210000000,
    taxableSales8pct: 1200000000,
    outputTax8pct: 96000000,
    taxableSales10pct: 3920000000,
    outputTax10pct: 392000000,
    totalOutputTax: 488000000,
    netVatPayable: 278000000,
    carriedForwardTax: 0,
  });

  // 5. Mẫu 03/TNDN CIT Finalization Data
  const citFinalization = ref<CitFinalizationData>({
    taxYear: '2026',
    accountingPbt: 1440000000,
    nonDeductibleExpenses: 60000000, // Chi phí không hợp lệ B4
    taxableIncome: 1500000000,
    taxRateBps: 2000, // 20%
    totalCitLiability: 300000000,
    provisionalTaxPaid: 240000000,
    remainingTaxPayable: 60000000,
    overpaidTax: 0,
  });

  // --- Computed Invariants & Metrics ---

  const totalOpeningDebit = computed(() =>
    trialBalanceLines.value.reduce((sum, item) => sum + item.openingDebit, 0)
  );

  const totalOpeningCredit = computed(() =>
    trialBalanceLines.value.reduce((sum, item) => sum + item.openingCredit, 0)
  );

  const totalPeriodDebit = computed(() =>
    trialBalanceLines.value.reduce((sum, item) => sum + item.periodDebit, 0)
  );

  const totalPeriodCredit = computed(() =>
    trialBalanceLines.value.reduce((sum, item) => sum + item.periodCredit, 0)
  );

  const totalClosingDebit = computed(() =>
    trialBalanceLines.value.reduce((sum, item) => sum + item.closingDebit, 0)
  );

  const totalClosingCredit = computed(() =>
    trialBalanceLines.value.reduce((sum, item) => sum + item.closingCredit, 0)
  );

  const isTrialBalanceBalanced = computed(() => {
    const openingMatch = totalOpeningDebit.value === totalOpeningCredit.value;
    const periodMatch = totalPeriodDebit.value === totalPeriodCredit.value;
    const closingMatch = totalClosingDebit.value === totalClosingCredit.value;
    return openingMatch && periodMatch && closingMatch;
  });

  const grossProfitMarginBps = computed(() => {
    if (incomeStatement.value.netRevenue === 0) return 0;
    return Math.round((incomeStatement.value.grossProfit * 10000) / incomeStatement.value.netRevenue);
  });

  const netProfitMarginBps = computed(() => {
    if (incomeStatement.value.netRevenue === 0) return 0;
    return Math.round((incomeStatement.value.netProfitAfterTax * 10000) / incomeStatement.value.netRevenue);
  });

  // --- Actions ---

  function setPeriod(period: ReportingPeriod) {
    currentPeriod.value = period;
    logger.info(`Đã chuyển kỳ báo cáo sang: ${period}`);
  }

  function setActiveTab(tab: ReportTab) {
    activeTab.value = tab;
  }

  function openTaxModal(type: 'VAT' | 'CIT') {
    activeTaxModal.value = type;
  }

  function closeTaxModal() {
    activeTaxModal.value = null;
  }

  function exportReportXml(reportType: string): string {
    const timestamp = new Date().toISOString();
    logger.info(`Đang kết xuất báo cáo chuẩn XML: ${reportType}`);

    if (reportType === 'VAT_01_GTGT') {
      return `<?xml version="1.0" encoding="UTF-8"?>
<HSoThueDTu xmlns="http://kekhaithue.gdt.gov.vn/TKhaiThue">
  <HSoKhaiThue>
    <TTinChung>
      <maTKhai>01/GTGT</maTKhai>
      <tenTKhai>Tờ khai thuế giá trị gia tăng</tenTKhai>
      <kyKKhaiThue>${currentPeriod.value}</kyKKhaiThue>
      <ngayLapTKhai>${timestamp.split('T')[0]}</ngayLapTKhai>
    </TTinChung>
    <CTieuTKhaiChinh>
      <ct25>${vatReturn.value.deductibleInputTax}</ct25>
      <ct26>${vatReturn.value.taxableSales8pct}</ct26>
      <ct27>${vatReturn.value.outputTax8pct}</ct27>
      <ct32>${vatReturn.value.taxableSales10pct}</ct32>
      <ct33>${vatReturn.value.outputTax10pct}</ct33>
      <ct35>${vatReturn.value.totalOutputTax}</ct35>
      <ct40a>${vatReturn.value.netVatPayable}</ct40a>
      <ct43>${vatReturn.value.carriedForwardTax}</ct43>
    </CTieuTKhaiChinh>
  </HSoKhaiThue>
</HSoThueDTu>`;
    }

    if (reportType === 'CIT_03_TNDN') {
      return `<?xml version="1.0" encoding="UTF-8"?>
<HSoThueDTu xmlns="http://kekhaithue.gdt.gov.vn/TKhaiThue">
  <HSoKhaiThue>
    <TTinChung>
      <maTKhai>03/TNDN</maTKhai>
      <tenTKhai>Tờ khai quyết toán thuế thu nhập doanh nghiệp</tenTKhai>
      <namKeKhai>${citFinalization.value.taxYear}</namKeKhai>
      <ngayLapTKhai>${timestamp.split('T')[0]}</ngayLapTKhai>
    </TTinChung>
    <CTieuTKhaiChinh>
      <ctA1>${citFinalization.value.accountingPbt}</ctA1>
      <ctB4>${citFinalization.value.nonDeductibleExpenses}</ctB4>
      <ctC4>${citFinalization.value.taxableIncome}</ctC4>
      <ctC7>${citFinalization.value.taxRateBps / 100}%</ctC7>
      <ctC8>${citFinalization.value.totalCitLiability}</ctC8>
      <ctE1>${citFinalization.value.provisionalTaxPaid}</ctE1>
      <ctG>${citFinalization.value.remainingTaxPayable}</ctG>
    </CTieuTKhaiChinh>
  </HSoKhaiThue>
</HSoThueDTu>`;
    }

    return `<report type="${reportType}" period="${currentPeriod.value}" generatedAt="${timestamp}"/>`;
  }

  return {
    currentPeriod,
    activeTab,
    activeTaxModal,
    trialBalanceLines,
    incomeStatement,
    cashFlow,
    vatReturn,
    citFinalization,
    totalOpeningDebit,
    totalOpeningCredit,
    totalPeriodDebit,
    totalPeriodCredit,
    totalClosingDebit,
    totalClosingCredit,
    isTrialBalanceBalanced,
    grossProfitMarginBps,
    netProfitMarginBps,
    setPeriod,
    setActiveTab,
    openTaxModal,
    closeTaxModal,
    exportReportXml,
  };
});
