//! Credit & Risk Scoring Engine.
//!
//! Provides deterministic, zero-floating-point-drift solvency and liquidity scoring:
//! 1. DSCR (Debt Service Coverage Ratio):
//!    $$DSCR = \frac{EBITDA - CAPEX}{Principal + Interest}$$
//!    Handles saturated edge cases: zero debt service (Debt-Free entity) and negative NOI (Distressed).
//! 2. Quick Ratio (Acid-Test Liquidity):
//!    $$QuickRatio = \frac{Cash + MarketableSecurities + Receivables}{CurrentLiabilities}$$
//!    Handles zero current liabilities (Zero Short-Term Obligations).
//! 3. Rolling 30/90-Day Cashflow Deficit Simulation:
//!    Simulates daily cumulative liquidity trajectory, identifies earliest shortfall date,
//!    and calculates required minimum liquidity buffer.
//!
//! All calculations are performed using scaled 64-bit/128-bit integer basis points (1 bps = 0.01% = 0.0001)
//! to completely eliminate IEEE-754 binary floating point precision drift.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DscrRiskCategory {
    Healthy,
    Watchlist,
    Distressed,
    DebtFree,
}

impl DscrRiskCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            DscrRiskCategory::Healthy => "HEALTHY",
            DscrRiskCategory::Watchlist => "WATCHLIST",
            DscrRiskCategory::Distressed => "DISTRESSED",
            DscrRiskCategory::DebtFree => "DEBT_FREE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LiquidityStatus {
    Strong,
    Adequate,
    Critical,
}

impl LiquidityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LiquidityStatus::Strong => "STRONG",
            LiquidityStatus::Adequate => "ADEQUATE",
            LiquidityStatus::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DscrReport {
    /// Formatted decimal representation of ratio for display/export.
    pub ratio: f64,
    /// Exact basis points (ratio * 10,000) for zero float drift.
    pub ratio_bps: i64,
    pub risk_category: DscrRiskCategory,
    pub buffer_vnd: i64,
    pub noi_vnd: i64,
    pub debt_service_vnd: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickRatioReport {
    /// Formatted decimal representation of ratio for display/export.
    pub ratio: f64,
    /// Exact basis points (ratio * 10,000) for zero float drift.
    pub ratio_bps: i64,
    pub liquidity_status: LiquidityStatus,
    pub liquid_assets_vnd: i64,
    pub current_liabilities_vnd: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCashflowPoint {
    pub day_offset: i32,
    pub net_inflow_vnd: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBalanceProjection {
    pub day_offset: i32,
    pub projected_balance_vnd: i64,
    pub net_inflow_vnd: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashflowForecastReport {
    pub projected_end_balance_vnd: i64,
    pub minimum_balance_vnd: i64,
    pub deficit_date_offset: Option<i32>,
    pub shortfall_warning: bool,
    pub forecast_days: usize,
    pub projections: Vec<DailyBalanceProjection>,
}

pub struct CreditRiskEngine;

impl CreditRiskEngine {
    /// Computes Debt Service Coverage Ratio (DSCR):
    /// $$DSCR = \frac{EBITDA - CAPEX}{Principal + Interest}$$
    ///
    /// Benchmarks:
    /// - DSCR >= 1.30: HEALTHY (robust debt coverage)
    /// - 1.00 <= DSCR < 1.30: WATCHLIST (vulnerable to revenue contraction)
    /// - DSCR < 1.00 or NOI <= 0: DISTRESSED (cannot service debt from operations)
    /// - Principal + Interest == 0: DEBT_FREE (unleveraged entity)
    pub fn calculate_dscr(
        ebitda_vnd: i64,
        capex_vnd: i64,
        debt_service_principal_vnd: i64,
        debt_service_interest_vnd: i64,
    ) -> DscrReport {
        let noi_vnd = ebitda_vnd.saturating_sub(capex_vnd);
        let debt_service_vnd = debt_service_principal_vnd.saturating_add(debt_service_interest_vnd);

        if debt_service_vnd <= 0 {
            return DscrReport {
                ratio: 999.0,
                ratio_bps: 9_990_000,
                risk_category: DscrRiskCategory::DebtFree,
                buffer_vnd: noi_vnd,
                noi_vnd,
                debt_service_vnd: 0,
            };
        }

        // Integer basis point math: ratio_bps = (NOI * 10,000) / debt_service
        let ratio_bps = ((noi_vnd as i128 * 10_000) / (debt_service_vnd as i128)) as i64;
        let ratio = (ratio_bps as f64) / 10_000.0;
        let buffer_vnd = noi_vnd.saturating_sub(debt_service_vnd);

        let risk_category = if noi_vnd <= 0 || ratio_bps < 10_000 {
            DscrRiskCategory::Distressed
        } else if ratio_bps < 13_000 {
            DscrRiskCategory::Watchlist
        } else {
            DscrRiskCategory::Healthy
        };

        DscrReport {
            ratio,
            ratio_bps,
            risk_category,
            buffer_vnd,
            noi_vnd,
            debt_service_vnd,
        }
    }

    /// Computes Quick Ratio (Acid-Test Ratio):
    /// $$Quick Ratio = \frac{Cash + MarketableSecurities + AccountsReceivable}{CurrentLiabilities}$$
    ///
    /// Benchmarks:
    /// - Quick Ratio >= 1.00: STRONG (every 1 VND of short-term debt covered by >= 1 VND liquid assets)
    /// - 0.80 <= Quick Ratio < 1.00: ADEQUATE (acceptable, but tight)
    /// - Quick Ratio < 0.80: CRITICAL (illiquidity risk)
    /// - Current Liabilities == 0: STRONG (zero short-term obligations)
    pub fn calculate_quick_ratio(
        cash_and_equivalents_vnd: i64,
        marketable_securities_vnd: i64,
        accounts_receivable_vnd: i64,
        current_liabilities_vnd: i64,
    ) -> QuickRatioReport {
        let liquid_assets_vnd = cash_and_equivalents_vnd
            .saturating_add(marketable_securities_vnd)
            .saturating_add(accounts_receivable_vnd);

        if current_liabilities_vnd <= 0 {
            return QuickRatioReport {
                ratio: 999.0,
                ratio_bps: 9_990_000,
                liquidity_status: LiquidityStatus::Strong,
                liquid_assets_vnd,
                current_liabilities_vnd: 0,
            };
        }

        let ratio_bps =
            ((liquid_assets_vnd as i128 * 10_000) / (current_liabilities_vnd as i128)) as i64;
        let ratio = (ratio_bps as f64) / 10_000.0;

        let liquidity_status = if ratio_bps >= 10_000 {
            LiquidityStatus::Strong
        } else if ratio_bps >= 8_000 {
            LiquidityStatus::Adequate
        } else {
            LiquidityStatus::Critical
        };

        QuickRatioReport {
            ratio,
            ratio_bps,
            liquidity_status,
            liquid_assets_vnd,
            current_liabilities_vnd,
        }
    }

    /// Simulates rolling daily cash balance over a forecast horizon (30 to 90 days).
    /// Identifies the first date of liquidity deficit below 0 or required minimum reserve.
    pub fn simulate_cashflow(
        starting_cash_vnd: i64,
        scheduled_cashflows: &[DailyCashflowPoint],
        forecast_days: usize,
        minimum_reserve_vnd: i64,
    ) -> CashflowForecastReport {
        let days = forecast_days.clamp(1, 90);
        let mut projections = Vec::with_capacity(days);

        let mut current_balance = starting_cash_vnd;
        let mut min_balance = starting_cash_vnd;
        let mut deficit_date = None;

        for day in 1..=(days as i32) {
            let day_net: i64 = scheduled_cashflows
                .iter()
                .filter(|cf| cf.day_offset == day)
                .map(|cf| cf.net_inflow_vnd)
                .sum();

            current_balance = current_balance.saturating_add(day_net);
            if current_balance < min_balance {
                min_balance = current_balance;
            }

            if deficit_date.is_none() && current_balance < minimum_reserve_vnd {
                deficit_date = Some(day);
            }

            projections.push(DailyBalanceProjection {
                day_offset: day,
                projected_balance_vnd: current_balance,
                net_inflow_vnd: day_net,
            });
        }

        let shortfall_warning = min_balance < minimum_reserve_vnd;

        CashflowForecastReport {
            projected_end_balance_vnd: current_balance,
            minimum_balance_vnd: min_balance,
            deficit_date_offset: deficit_date,
            shortfall_warning,
            forecast_days: days,
            projections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dscr_healthy_scenario() {
        // EBITDA: 1.5B, CAPEX: 200M -> NOI = 1.3B
        // Principal: 600M, Interest: 200M -> Debt Service = 800M
        // DSCR = 1.3B / 800M = 1.625 (>= 1.30 -> HEALTHY)
        let report = CreditRiskEngine::calculate_dscr(
            1_500_000_000,
            200_000_000,
            600_000_000,
            200_000_000,
        );

        assert_eq!(report.risk_category, DscrRiskCategory::Healthy);
        assert_eq!(report.ratio_bps, 16_250);
        assert!((report.ratio - 1.625).abs() < 1e-4);
        assert_eq!(report.buffer_vnd, 500_000_000);
    }

    #[test]
    fn test_dscr_watchlist_scenario() {
        // NOI: 1.0B, Debt Service: 900M -> DSCR = 1.1111 (1.0 <= DSCR < 1.30 -> WATCHLIST)
        let report = CreditRiskEngine::calculate_dscr(
            1_200_000_000,
            200_000_000,
            700_000_000,
            200_000_000,
        );

        assert_eq!(report.risk_category, DscrRiskCategory::Watchlist);
        assert!(report.ratio >= 1.0 && report.ratio < 1.30);
    }

    #[test]
    fn test_dscr_distressed_scenario() {
        // NOI: 600M, Debt Service: 800M -> DSCR = 0.75 (< 1.0 -> DISTRESSED)
        let report = CreditRiskEngine::calculate_dscr(
            800_000_000,
            200_000_000,
            600_000_000,
            200_000_000,
        );

        assert_eq!(report.risk_category, DscrRiskCategory::Distressed);
        assert_eq!(report.ratio_bps, 7_500);
        assert_eq!(report.buffer_vnd, -200_000_000);
    }

    #[test]
    fn test_dscr_debt_free_scenario() {
        // Debt Service = 0
        let report = CreditRiskEngine::calculate_dscr(1_000_000_000, 100_000_000, 0, 0);
        assert_eq!(report.risk_category, DscrRiskCategory::DebtFree);
        assert_eq!(report.buffer_vnd, 900_000_000);
    }

    #[test]
    fn test_quick_ratio_scenarios() {
        // Cash: 400M, Securities: 100M, AR: 500M -> Liquid Assets: 1.0B
        // Liabilities: 800M -> Quick Ratio = 1.25 (>= 1.0 -> STRONG)
        let report1 = CreditRiskEngine::calculate_quick_ratio(
            400_000_000,
            100_000_000,
            500_000_000,
            800_000_000,
        );
        assert_eq!(report1.liquidity_status, LiquidityStatus::Strong);
        assert_eq!(report1.ratio_bps, 12_500);

        // Liabilities: 1.2B -> Quick Ratio = 1.0B / 1.2B = 0.8333 (0.8 <= ratio < 1.0 -> ADEQUATE)
        let report2 = CreditRiskEngine::calculate_quick_ratio(
            400_000_000,
            100_000_000,
            500_000_000,
            1_200_000_000,
        );
        assert_eq!(report2.liquidity_status, LiquidityStatus::Adequate);

        // Liabilities: 2.0B -> Quick Ratio = 0.50 (< 0.80 -> CRITICAL)
        let report3 = CreditRiskEngine::calculate_quick_ratio(
            400_000_000,
            100_000_000,
            500_000_000,
            2_000_000_000,
        );
        assert_eq!(report3.liquidity_status, LiquidityStatus::Critical);
    }

    #[test]
    fn test_cashflow_simulation_with_deficit() {
        let starting_cash = 500_000_000;
        let flows = vec![
            DailyCashflowPoint { day_offset: 2, net_inflow_vnd: 100_000_000 },
            DailyCashflowPoint { day_offset: 5, net_inflow_vnd: -700_000_000 }, // Balance drops to -100M at T+5
            DailyCashflowPoint { day_offset: 10, net_inflow_vnd: 400_000_000 },
        ];

        let forecast = CreditRiskEngine::simulate_cashflow(starting_cash, &flows, 30, 0);

        assert!(forecast.shortfall_warning);
        assert_eq!(forecast.deficit_date_offset, Some(5));
        assert_eq!(forecast.minimum_balance_vnd, -100_000_000);
        assert_eq!(forecast.projected_end_balance_vnd, 300_000_000);
        assert_eq!(forecast.projections.len(), 30);
    }
}
