//! LIVA Banking — `financial_risk.rs`
//!
//! Quantitative financial risk scoring, debt service solvency modeling (DSCR),
//! liquidity ratios (Current, Quick, Cash, ICR), SBV Circular 11/2021/TT-NHNN debt classification,
//! and 3-scenario stress testing.
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`. All ratios represented as basis points (bps, 1 bps = 0.01%).

use crate::LedgerError;
use liva_money::{Currency, Money};
use serde::{Deserialize, Serialize};

/// 100% in Basis Points (10,000 bps = 1.0000)
pub const BPS_ONE: u32 = 10_000;

/// Category of Debt Service Coverage Risk (DSCR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    /// DSCR >= 1.30x (13,000 bps) — Comfortable financial cushion
    Healthy,
    /// 1.00x <= DSCR < 1.30x (10,000 - 12,999 bps) — Susceptible to shocks
    Watchlist,
    /// DSCR < 1.00x or negative operating income — High default risk
    Distressed,
    /// Enterprise carries no debt obligations (Debt Service = 0)
    DebtFree,
}

/// Liquidity status classification for Quick & Current Ratios
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidityStatus {
    /// Ratio >= 1.50x (15,000 bps)
    Strong,
    /// 1.00x <= Ratio < 1.50x (10,000 - 14,999 bps)
    Adequate,
    /// Ratio < 1.00x (< 10,000 bps)
    Critical,
}

/// Debt classification according to State Bank of Vietnam Circular 11/2021/TT-NHNN
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Circular11DebtGroup {
    /// Nhóm 1: Nợ đủ tiêu chuẩn (Standard debt, overdue < 10 days)
    Group1Standard,
    /// Nhóm 2: Nợ cần chú ý (Special mention, overdue 10..90 days)
    Group2SpecialMention,
    /// Nhóm 3: Nợ dưới tiêu chuẩn (Sub-standard, overdue 91..180 days)
    Group3Substandard,
    /// Nhóm 4: Nợ nghi ngờ (Doubtful, overdue 181..360 days)
    Group4Doubtful,
    /// Nhóm 5: Nợ có khả năng mất vốn (Loss, overdue > 360 days)
    Group5Loss,
}

/// Corporate financial statements data input for risk modeling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatementInput {
    pub currency: Currency,
    pub ebitda: Money,
    pub capex: Money,
    pub debt_service_principal: Money,
    pub debt_service_interest: Money,
    pub cash_and_equivalents: Money,
    pub marketable_securities: Money,
    pub accounts_receivable: Money,
    pub inventory: Money,
    pub current_assets: Money,
    pub current_liabilities: Money,
    pub ebit: Money,
    pub interest_expense: Money,
}

/// Comprehensive Solvency and Liquidity Ratios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolvencyRatios {
    /// Debt Service Coverage Ratio in basis points (1.30x = 13,000 bps)
    pub dscr_bps: u32,
    pub dscr_category: RiskCategory,
    /// Net Operating Income - Total Debt Service (Buffer in VND)
    pub dscr_buffer: Money,
    /// Quick Ratio (Acid-Test) in basis points (1.25x = 12,500 bps)
    pub quick_ratio_bps: u32,
    pub quick_ratio_status: LiquidityStatus,
    /// Current Ratio in basis points
    pub current_ratio_bps: u32,
    /// Cash Ratio in basis points
    pub cash_ratio_bps: u32,
    /// Interest Coverage Ratio (ICR) in basis points
    pub icr_bps: u32,
}

/// Result of SBV Circular 11/2021/TT-NHNN debt classification & provisioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtClassificationResult {
    pub group: Circular11DebtGroup,
    pub days_overdue: u32,
    pub is_restructured: bool,
    pub general_provision_rate_bps: u32,
    pub specific_provision_rate_bps: u32,
    pub required_general_provision: Money,
    pub required_specific_provision: Money,
    pub total_provision: Money,
}

/// Stress-test scenario configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StressScenario {
    /// Base Case (Normal Operations)
    BaseCase,
    /// Moderate Stress (-15% Revenue/EBITDA, +150 bps interest rate)
    ModerateStress,
    /// Severe Stress (-30% Revenue/EBITDA, +300 bps interest rate, +10% CAPEX)
    SevereStress,
}

/// Result of stress test evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub scenario: StressScenario,
    pub stressed_noi: Money,
    pub stressed_debt_service: Money,
    pub stressed_dscr_bps: u32,
    pub stressed_category: RiskCategory,
    pub stressed_buffer: Money,
    pub passes_stress_test: bool,
}

/// Automated Credit Underwriting Recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnderwritingDecision {
    Approved,
    Conditional,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderwritingResult {
    pub decision: UnderwritingDecision,
    pub max_recommended_credit_limit: Money,
    pub rationale: String,
}

/// Compute all solvency and liquidity ratios with zero-float integer arithmetic
pub fn compute_solvency_ratios(input: &FinancialStatementInput) -> Result<SolvencyRatios, LedgerError> {
    let zero = Money::zero(input.currency);

    // 1. Net Operating Income (NOI = EBITDA - CAPEX)
    let noi = if input.ebitda.amount() >= input.capex.amount() {
        input.ebitda.checked_sub(input.capex)?
    } else {
        zero
    };

    let total_debt_service = input.debt_service_principal.checked_add(input.debt_service_interest)?;

    // DSCR Calculation
    let (dscr_bps, dscr_category, dscr_buffer) = if total_debt_service.is_zero() {
        let buf = input.ebitda.checked_sub(input.capex)?;
        (0u32, RiskCategory::DebtFree, buf)
    } else if input.ebitda.amount() < input.capex.amount() {
        let diff = input.capex.amount() - input.ebitda.amount();
        let neg_buf = Money::from_minor(-diff - total_debt_service.amount(), input.currency);
        (0u32, RiskCategory::Distressed, neg_buf)
    } else {
        let noi_cents = noi.amount() as u128;
        let debt_cents = total_debt_service.amount() as u128;
        let ratio_bps = match noi_cents.checked_mul(BPS_ONE as u128) {
            Some(prod) => (prod / debt_cents).min(999_999) as u32,
            None => 999_999,
        };

        let cat = if ratio_bps >= 13_000 {
            RiskCategory::Healthy
        } else if ratio_bps >= 10_000 {
            RiskCategory::Watchlist
        } else {
            RiskCategory::Distressed
        };

        let buf = if noi.amount() >= total_debt_service.amount() {
            noi.checked_sub(total_debt_service)?
        } else {
            let shortfall = total_debt_service.amount() - noi.amount();
            Money::from_minor(-shortfall, input.currency)
        };

        (ratio_bps, cat, buf)
    };

    // 2. Liquidity Ratios
    let liab = input.current_liabilities.amount() as u128;

    // Quick Ratio = (Cash + Securities + Receivables) / Current Liabilities
    let liquid_assets = input.cash_and_equivalents
        .checked_add(input.marketable_securities)?
        .checked_add(input.accounts_receivable)?;

    let quick_ratio_bps = if liab == 0 {
        999_999
    } else {
        ((liquid_assets.amount() as u128 * BPS_ONE as u128) / liab).min(999_999) as u32
    };

    let quick_ratio_status = if quick_ratio_bps >= 15_000 {
        LiquidityStatus::Strong
    } else if quick_ratio_bps >= 10_000 {
        LiquidityStatus::Adequate
    } else {
        LiquidityStatus::Critical
    };

    // Current Ratio = Current Assets / Current Liabilities
    let cur_assets = if input.current_assets.is_zero() {
        liquid_assets.checked_add(input.inventory)?
    } else {
        input.current_assets
    };

    let current_ratio_bps = if liab == 0 {
        999_999
    } else {
        ((cur_assets.amount() as u128 * BPS_ONE as u128) / liab).min(999_999) as u32
    };

    // Cash Ratio = (Cash + Securities) / Current Liabilities
    let cash_and_sec = input.cash_and_equivalents.checked_add(input.marketable_securities)?;
    let cash_ratio_bps = if liab == 0 {
        999_999
    } else {
        ((cash_and_sec.amount() as u128 * BPS_ONE as u128) / liab).min(999_999) as u32
    };

    // ICR = EBIT / Interest Expense
    let int_exp = input.interest_expense.amount() as u128;
    let icr_bps = if int_exp == 0 {
        999_999
    } else if input.ebit.amount() <= 0 {
        0
    } else {
        ((input.ebit.amount() as u128 * BPS_ONE as u128) / int_exp).min(999_999) as u32
    };

    Ok(SolvencyRatios {
        dscr_bps,
        dscr_category,
        dscr_buffer,
        quick_ratio_bps,
        quick_ratio_status,
        current_ratio_bps,
        cash_ratio_bps,
        icr_bps,
    })
}

/// Classify loan and calculate required provisioning according to SBV Circular 11/2021/TT-NHNN
pub fn classify_debt_group(
    outstanding_principal: Money,
    days_overdue: u32,
    is_restructured: bool,
) -> Result<DebtClassificationResult, LedgerError> {
    let group = if is_restructured {
        if days_overdue < 10 {
            Circular11DebtGroup::Group2SpecialMention
        } else if days_overdue <= 90 {
            Circular11DebtGroup::Group3Substandard
        } else if days_overdue <= 180 {
            Circular11DebtGroup::Group4Doubtful
        } else {
            Circular11DebtGroup::Group5Loss
        }
    } else if days_overdue < 10 {
        Circular11DebtGroup::Group1Standard
    } else if days_overdue <= 90 {
        Circular11DebtGroup::Group2SpecialMention
    } else if days_overdue <= 180 {
        Circular11DebtGroup::Group3Substandard
    } else if days_overdue <= 360 {
        Circular11DebtGroup::Group4Doubtful
    } else {
        Circular11DebtGroup::Group5Loss
    };

    let general_rate_bps = if group == Circular11DebtGroup::Group5Loss { 0 } else { 75 };
    let specific_rate_bps = match group {
        Circular11DebtGroup::Group1Standard => 0,
        Circular11DebtGroup::Group2SpecialMention => 500,
        Circular11DebtGroup::Group3Substandard => 2_000,
        Circular11DebtGroup::Group4Doubtful => 5_000,
        Circular11DebtGroup::Group5Loss => 10_000,
    };

    let principal_cents = outstanding_principal.amount() as u128;
    let gen_prov_cents = (principal_cents * general_rate_bps as u128) / BPS_ONE as u128;
    let spec_prov_cents = (principal_cents * specific_rate_bps as u128) / BPS_ONE as u128;

    let required_general_provision = Money::from_minor(gen_prov_cents as i64, outstanding_principal.currency());
    let required_specific_provision = Money::from_minor(spec_prov_cents as i64, outstanding_principal.currency());
    let total_provision = required_general_provision.checked_add(required_specific_provision)?;

    Ok(DebtClassificationResult {
        group,
        days_overdue,
        is_restructured,
        general_provision_rate_bps: general_rate_bps,
        specific_provision_rate_bps: specific_rate_bps,
        required_general_provision,
        required_specific_provision,
        total_provision,
    })
}

/// Run stress-testing scenarios against financial parameters
pub fn run_stress_test(
    input: &FinancialStatementInput,
    scenario: StressScenario,
) -> Result<StressTestResult, LedgerError> {
    let (ebitda_factor_bps, capex_factor_bps, interest_increase_bps) = match scenario {
        StressScenario::BaseCase => (10_000, 10_000, 0),
        StressScenario::ModerateStress => (8_500, 10_000, 1_500),
        StressScenario::SevereStress => (7_000, 11_000, 3_000),
    };

    let stressed_ebitda_cents = (input.ebitda.amount() as u128 * ebitda_factor_bps as u128) / BPS_ONE as u128;
    let stressed_capex_cents = (input.capex.amount() as u128 * capex_factor_bps as u128) / BPS_ONE as u128;

    let stressed_ebitda = Money::from_minor(stressed_ebitda_cents as i64, input.currency);
    let stressed_capex = Money::from_minor(stressed_capex_cents as i64, input.currency);

    let stressed_noi = if stressed_ebitda.amount() >= stressed_capex.amount() {
        stressed_ebitda.checked_sub(stressed_capex)?
    } else {
        Money::zero(input.currency)
    };

    let interest_cents = input.debt_service_interest.amount() as u128;
    let added_int = (interest_cents * interest_increase_bps as u128) / BPS_ONE as u128;
    let stressed_interest = Money::from_minor((interest_cents + added_int) as i64, input.currency);

    let stressed_debt_service = input.debt_service_principal.checked_add(stressed_interest)?;

    let (stressed_dscr_bps, stressed_category, stressed_buffer) = if stressed_debt_service.is_zero() {
        (0u32, RiskCategory::DebtFree, stressed_noi)
    } else if stressed_ebitda.amount() < stressed_capex.amount() {
        let diff = stressed_capex.amount() - stressed_ebitda.amount();
        let neg = Money::from_minor(-diff - stressed_debt_service.amount(), input.currency);
        (0u32, RiskCategory::Distressed, neg)
    } else {
        let noi_cents = stressed_noi.amount() as u128;
        let debt_cents = stressed_debt_service.amount() as u128;
        let bps = ((noi_cents * BPS_ONE as u128) / debt_cents).min(999_999) as u32;

        let cat = if bps >= 13_000 {
            RiskCategory::Healthy
        } else if bps >= 10_000 {
            RiskCategory::Watchlist
        } else {
            RiskCategory::Distressed
        };

        let buf = if stressed_noi.amount() >= stressed_debt_service.amount() {
            stressed_noi.checked_sub(stressed_debt_service)?
        } else {
            let shortfall = stressed_debt_service.amount() - stressed_noi.amount();
            Money::from_minor(-shortfall, input.currency)
        };

        (bps, cat, buf)
    };

    let passes = stressed_dscr_bps >= 10_000 && stressed_category != RiskCategory::Distressed;

    Ok(StressTestResult {
        scenario,
        stressed_noi,
        stressed_debt_service,
        stressed_dscr_bps,
        stressed_category,
        stressed_buffer,
        passes_stress_test: passes,
    })
}

/// Calculate Maximum Debt Capacity and generate Credit Underwriting Recommendation
pub fn evaluate_credit_underwriting(
    input: &FinancialStatementInput,
    debt_classification: &DebtClassificationResult,
) -> Result<UnderwritingResult, LedgerError> {
    let ratios = compute_solvency_ratios(input)?;
    let moderate_stress = run_stress_test(input, StressScenario::ModerateStress)?;

    let noi = if input.ebitda.amount() >= input.capex.amount() {
        input.ebitda.checked_sub(input.capex)?
    } else {
        Money::zero(input.currency)
    };

    let max_annual_service = (noi.amount() as u128 * BPS_ONE as u128) / 13_000;
    let max_limit_cents = max_annual_service * 3;
    let max_credit_limit = Money::from_minor(max_limit_cents.min(i64::MAX as u128) as i64, input.currency);

    if debt_classification.group >= Circular11DebtGroup::Group3Substandard || ratios.dscr_category == RiskCategory::Distressed {
        Ok(UnderwritingResult {
            decision: UnderwritingDecision::Rejected,
            max_recommended_credit_limit: Money::zero(input.currency),
            rationale: "Từ chối cấp tín dụng: Doanh nghiệp thuộc nhóm nợ xấu hoặc DSCR dưới 1.0x (nguy cơ mất thanh khoản).".to_string(),
        })
    } else if ratios.dscr_category == RiskCategory::Healthy
        && ratios.quick_ratio_status != LiquidityStatus::Critical
        && debt_classification.group == Circular11DebtGroup::Group1Standard
        && moderate_stress.passes_stress_test
    {
        Ok(UnderwritingResult {
            decision: UnderwritingDecision::Approved,
            max_recommended_credit_limit: max_credit_limit,
            rationale: "Chấp thuận cấp hạn mức tín dụng: DSCR thặng dư cao (>= 1.3x), thanh khoản tốt, nợ nhóm 1 và vượt qua bài kiểm tra áp lực vừa phải.".to_string(),
        })
    } else {
        let reduced_limit = Money::from_minor(((max_limit_cents * 60) / 100) as i64, input.currency);
        Ok(UnderwritingResult {
            decision: UnderwritingDecision::Conditional,
            max_recommended_credit_limit: reduced_limit,
            rationale: "Phê duyệt có điều kiện: DSCR ở ngưỡng cần theo dõi hoặc có rủi ro áp lực thanh khoản. Yêu cầu tỷ lệ tài sản bảo đảm tối thiểu 150%.".to_string(),
        })
    }
}
