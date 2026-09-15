//! VAS (Vietnamese Accounting Standards) & Tax Reporting Engine
//! =================================================================
//! Implements statutory reporting under Circular 200/2014/TT-BTC:
//! - F01-DN: Bảng Cân Đối Số Phát Sinh (Trial Balance) with 3 mathematical invariants.
//! - B02-DN: Báo Cáo Kết Quả Hoạt Động Kinh Doanh (Income Statement / P&L).
//! - B03-DN: Báo Cáo Lưu Chuyển Tiền Tệ (Statement of Cash Flows - Direct Method).
//!
//! Implements statutory tax declarations under Circular 80/2021/TT-BTC:
//! - Mẫu 01/GTGT: Tờ Khai Thuế Giá Trị Gia Tăng (Khấu trừ, thuế suất 8% & 10%).
//! - Mẫu 03/TNDN: Tờ Khai Quyết Toán Thuế Thu Nhập Doanh Nghiệp (20%, B4 add-backs).
//!
//! Strictly enforces `#![deny(clippy::float_arithmetic)]` — 100% integer basis points and minor units.

use crate::LedgerError;
use liva_money::{Currency, Money};
use serde::{Deserialize, Serialize};

pub const BPS_DIVISOR: i64 = 10_000;
pub const CIT_STANDARD_RATE_BPS: i64 = 2_000; // 20.00%
pub const VAT_STANDARD_RATE_BPS: i64 = 1_000; // 10.00%
pub const VAT_REDUCED_RATE_BPS: i64 = 800;    // 8.00%

// -----------------------------------------------------------------------------
// 1. F01-DN: Trial Balance (Bảng Cân Đối Tài Khoản)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialBalanceLine {
    pub account_code: String,
    pub account_name: String,
    pub opening_debit: Money,
    pub opening_credit: Money,
    pub period_debit: Money,
    pub period_credit: Money,
    pub closing_debit: Money,
    pub closing_credit: Money,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialBalanceReport {
    pub period: String,
    pub lines: Vec<TrialBalanceLine>,
    pub total_opening_debit: Money,
    pub total_opening_credit: Money,
    pub total_period_debit: Money,
    pub total_period_credit: Money,
    pub total_closing_debit: Money,
    pub total_closing_credit: Money,
    pub is_balanced: bool,
}

/// Builds and mathematically validates a Trial Balance report under Circular 200/2014/TT-BTC.
/// Returns Ok(report) if:
/// 1. Sum(Opening Debit) == Sum(Opening Credit)
/// 2. Sum(Period Debit) == Sum(Period Credit)
/// 3. Sum(Closing Debit) == Sum(Closing Credit)
pub fn build_trial_balance(
    period: &str,
    lines: Vec<TrialBalanceLine>,
    currency: Currency,
) -> Result<TrialBalanceReport, LedgerError> {
    let mut total_opening_debit = Money::from_minor(0, currency);
    let mut total_opening_credit = Money::from_minor(0, currency);
    let mut total_period_debit = Money::from_minor(0, currency);
    let mut total_period_credit = Money::from_minor(0, currency);
    let mut total_closing_debit = Money::from_minor(0, currency);
    let mut total_closing_credit = Money::from_minor(0, currency);

    for line in &lines {
        total_opening_debit = total_opening_debit.checked_add(line.opening_debit)?;
        total_opening_credit = total_opening_credit.checked_add(line.opening_credit)?;
        total_period_debit = total_period_debit.checked_add(line.period_debit)?;
        total_period_credit = total_period_credit.checked_add(line.period_credit)?;
        total_closing_debit = total_closing_debit.checked_add(line.closing_debit)?;
        total_closing_credit = total_closing_credit.checked_add(line.closing_credit)?;
    }

    let is_opening_balanced = total_opening_debit == total_opening_credit;
    let is_period_balanced = total_period_debit == total_period_credit;
    let is_closing_balanced = total_closing_debit == total_closing_credit;

    let is_balanced = is_opening_balanced && is_period_balanced && is_closing_balanced;

    if !is_balanced {
        return Err(LedgerError::InvariantViolation {
            expected: format!(
                "Balanced Trial Balance (OpDr={total_opening_debit}, OpCr={total_opening_credit}, PerDr={total_period_debit}, PerCr={total_period_credit}, ClDr={total_closing_debit}, ClCr={total_closing_credit})"
            ),
            calculated: "Unbalanced Trial Balance".to_string(),
        });
    }

    Ok(TrialBalanceReport {
        period: period.to_string(),
        lines,
        total_opening_debit,
        total_opening_credit,
        total_period_debit,
        total_period_credit,
        total_closing_debit,
        total_closing_credit,
        is_balanced,
    })
}

// -----------------------------------------------------------------------------
// 2. B02-DN: Income Statement (Báo Cáo Kết Quả Hoạt Động Kinh Doanh)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncomeStatementReport {
    pub period: String,
    pub gross_revenue: Money,          // Mã 01: Doanh thu bán hàng và CCDV
    pub revenue_deductions: Money,     // Mã 02: Các khoản giảm trừ doanh thu
    pub net_revenue: Money,            // Mã 10 = 01 - 02: Doanh thu thuần
    pub cogs: Money,                   // Mã 11: Giá vốn hàng bán
    pub gross_profit: Money,           // Mã 20 = 10 - 11: Lợi nhuận gộp
    pub financial_income: Money,       // Mã 21: Doanh thu HĐ tài chính
    pub financial_expense: Money,      // Mã 22: Chi phí tài chính
    pub interest_expense: Money,       // Mã 23: Chi phí lãi vay (trong 22)
    pub selling_expense: Money,        // Mã 25: Chi phí bán hàng
    pub admin_expense: Money,          // Mã 26: Chi phí quản lý doanh nghiệp
    pub operating_profit: Money,       // Mã 30 = 20 + 21 - 22 - 25 - 26
    pub other_income: Money,           // Mã 31: Thu nhập khác
    pub other_expense: Money,          // Mã 32: Chi phí khác
    pub other_profit: Money,           // Mã 40 = 31 - 32
    pub profit_before_tax: Money,      // Mã 50 = 30 + 40
    pub current_cit_expense: Money,    // Mã 51: Chi phí thuế TNDN hiện hành
    pub net_profit_after_tax: Money,   // Mã 60 = 50 - 51
}

#[allow(clippy::too_many_arguments)]
pub fn compute_income_statement(
    period: &str,
    gross_revenue: Money,
    revenue_deductions: Money,
    cogs: Money,
    financial_income: Money,
    financial_expense: Money,
    interest_expense: Money,
    selling_expense: Money,
    admin_expense: Money,
    other_income: Money,
    other_expense: Money,
    cit_tax_rate_bps: i64,
) -> Result<IncomeStatementReport, LedgerError> {
    let net_revenue = gross_revenue.checked_sub(revenue_deductions)?;
    let gross_profit = net_revenue.checked_sub(cogs)?;

    let operating_add = gross_profit.checked_add(financial_income)?;
    let operating_deduct = financial_expense
        .checked_add(selling_expense)?
        .checked_add(admin_expense)?;
    let operating_profit = operating_add.checked_sub(operating_deduct)?;

    let other_profit = other_income.checked_sub(other_expense)?;
    let profit_before_tax = operating_profit.checked_add(other_profit)?;

    // CIT expense is only applied if PBT > 0
    let current_cit_expense = if profit_before_tax.amount() > 0 {
        let tax_minor = (profit_before_tax.amount() * cit_tax_rate_bps) / BPS_DIVISOR;
        Money::from_minor(tax_minor, profit_before_tax.currency())
    } else {
        Money::from_minor(0, profit_before_tax.currency())
    };

    let net_profit_after_tax = profit_before_tax.checked_sub(current_cit_expense)?;

    Ok(IncomeStatementReport {
        period: period.to_string(),
        gross_revenue,
        revenue_deductions,
        net_revenue,
        cogs,
        gross_profit,
        financial_income,
        financial_expense,
        interest_expense,
        selling_expense,
        admin_expense,
        operating_profit,
        other_income,
        other_expense,
        other_profit,
        profit_before_tax,
        current_cit_expense,
        net_profit_after_tax,
    })
}

// -----------------------------------------------------------------------------
// 3. B03-DN: Cash Flow Statement (Lưu Chuyển Tiền Tệ)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CashFlowStatementReport {
    pub period: String,
    pub operating_inflow: Money,
    pub operating_outflow: Money,
    pub net_operating_flow: Money,     // Mã 20
    pub investing_inflow: Money,
    pub investing_outflow: Money,
    pub net_investing_flow: Money,     // Mã 30
    pub financing_inflow: Money,
    pub financing_outflow: Money,
    pub net_financing_flow: Money,     // Mã 40
    pub net_cash_flow: Money,          // Mã 50 = 20 + 30 + 40
    pub opening_cash: Money,           // Mã 60
    pub closing_cash: Money,           // Mã 70 = 60 + 50
}

#[allow(clippy::too_many_arguments)]
pub fn compute_cashflow_statement(
    period: &str,
    operating_inflow: Money,
    operating_outflow: Money,
    investing_inflow: Money,
    investing_outflow: Money,
    financing_inflow: Money,
    financing_outflow: Money,
    opening_cash: Money,
) -> Result<CashFlowStatementReport, LedgerError> {
    let net_operating_flow = operating_inflow.checked_sub(operating_outflow)?;
    let net_investing_flow = investing_inflow.checked_sub(investing_outflow)?;
    let net_financing_flow = financing_inflow.checked_sub(financing_outflow)?;

    let net_cash_flow = net_operating_flow
        .checked_add(net_investing_flow)?
        .checked_add(net_financing_flow)?;

    let closing_cash = opening_cash.checked_add(net_cash_flow)?;

    Ok(CashFlowStatementReport {
        period: period.to_string(),
        operating_inflow,
        operating_outflow,
        net_operating_flow,
        investing_inflow,
        investing_outflow,
        net_investing_flow,
        financing_inflow,
        financing_outflow,
        net_financing_flow,
        net_cash_flow,
        opening_cash,
        closing_cash,
    })
}

// -----------------------------------------------------------------------------
// 4. Mẫu 01/GTGT: VAT Return (Tờ Khai Thuế GTGT Khấu Trừ - TT 80/2021)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VatReturnReport {
    pub period: String,
    pub deductible_input_tax: Money,   // Chỉ tiêu [25]
    pub taxable_sales_8pct: Money,     // Chỉ tiêu [26]
    pub output_tax_8pct: Money,        // Chỉ tiêu [27]
    pub taxable_sales_10pct: Money,    // Chỉ tiêu [32]
    pub output_tax_10pct: Money,       // Chỉ tiêu [33]
    pub total_output_tax: Money,       // Chỉ tiêu [35]
    pub net_vat_payable: Money,        // Chỉ tiêu [40a] = max(0, [35] - [25])
    pub carried_forward_tax: Money,    // Chỉ tiêu [43] = max(0, [25] - [35])
}

pub fn generate_vat_return(
    period: &str,
    deductible_input_tax: Money,
    taxable_sales_8pct: Money,
    taxable_sales_10pct: Money,
) -> Result<VatReturnReport, LedgerError> {
    let currency = deductible_input_tax.currency();

    // 8% output tax
    let output_tax_8_minor = (taxable_sales_8pct.amount() * VAT_REDUCED_RATE_BPS) / BPS_DIVISOR;
    let output_tax_8pct = Money::from_minor(output_tax_8_minor, currency);

    // 10% output tax
    let output_tax_10_minor = (taxable_sales_10pct.amount() * VAT_STANDARD_RATE_BPS) / BPS_DIVISOR;
    let output_tax_10pct = Money::from_minor(output_tax_10_minor, currency);

    let total_output_tax = output_tax_8pct.checked_add(output_tax_10pct)?;

    let (net_vat_payable, carried_forward_tax) = if total_output_tax.amount() >= deductible_input_tax.amount() {
        (total_output_tax.checked_sub(deductible_input_tax)?, Money::from_minor(0, currency))
    } else {
        (Money::from_minor(0, currency), deductible_input_tax.checked_sub(total_output_tax)?)
    };

    Ok(VatReturnReport {
        period: period.to_string(),
        deductible_input_tax,
        taxable_sales_8pct,
        output_tax_8pct,
        taxable_sales_10pct,
        output_tax_10pct,
        total_output_tax,
        net_vat_payable,
        carried_forward_tax,
    })
}

// -----------------------------------------------------------------------------
// 5. Mẫu 03/TNDN: CIT Finalization (Quyết Toán Thuế TNDN - TT 80/2021)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitFinalizationReport {
    pub tax_year: String,
    pub accounting_pbt: Money,             // Chỉ tiêu [A1]: Tổng lợi nhuận kế toán trước thuế
    pub non_deductible_expenses: Money,    // Chỉ tiêu [B4]: Các khoản chi không được trừ
    pub taxable_income: Money,             // Chỉ tiêu [C4] = [A1] + [B4]
    pub tax_rate_bps: i64,                 // Chỉ tiêu [C7]: Thuế suất (2,000 = 20%)
    pub total_cit_liability: Money,        // Chỉ tiêu [C8] = max(0, [C4] * 20%)
    pub provisional_tax_paid: Money,       // Chỉ tiêu [E1]: Số thuế TNDN đã tạm nộp 4 quý
    pub remaining_tax_payable: Money,      // Chỉ tiêu [G] nếu phải nộp thêm
    pub overpaid_tax: Money,               // Chỉ tiêu [G] nếu nộp thừa
}

pub fn calculate_cit_finalization(
    tax_year: &str,
    accounting_pbt: Money,
    non_deductible_expenses: Money,
    provisional_tax_paid: Money,
) -> Result<CitFinalizationReport, LedgerError> {
    let currency = accounting_pbt.currency();
    let taxable_income = accounting_pbt.checked_add(non_deductible_expenses)?;

    let total_cit_liability = if taxable_income.amount() > 0 {
        let tax_minor = (taxable_income.amount() * CIT_STANDARD_RATE_BPS) / BPS_DIVISOR;
        Money::from_minor(tax_minor, currency)
    } else {
        Money::from_minor(0, currency)
    };

    let (remaining_tax_payable, overpaid_tax) = if total_cit_liability.amount() >= provisional_tax_paid.amount() {
        (total_cit_liability.checked_sub(provisional_tax_paid)?, Money::from_minor(0, currency))
    } else {
        (Money::from_minor(0, currency), provisional_tax_paid.checked_sub(total_cit_liability)?)
    };

    Ok(CitFinalizationReport {
        tax_year: tax_year.to_string(),
        accounting_pbt,
        non_deductible_expenses,
        taxable_income,
        tax_rate_bps: CIT_STANDARD_RATE_BPS,
        total_cit_liability,
        provisional_tax_paid,
        remaining_tax_payable,
        overpaid_tax,
    })
}
