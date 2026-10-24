//! Comprehensive unit tests for Dynamic Fee Unbundling & Expense Routing Engine.
//!
//! Validates:
//! 1. Exact integer decomposition for Vietnamese commercial bank fee schedules.
//! 2. Zero penny drift: Net + Vat == Gross down to 1 VND.
//! 3. 8% and 10% VAT rates, 0% exempt rate, and boundary conditions.
//! 4. Statutory account routing: Circular 200 (6427/1331) vs Circular 133 (6422/1331).

use liva_ledger::{
    route_and_unbundle_fee, unbundle_gross_fee, AccountStandard, LedgerError,
    VAT_EXEMPT_RATE_BPS, VAT_REDUCED_RATE_BPS, VAT_STANDARD_RATE_BPS,
};
use liva_money::Money;

#[test]
fn test_golden_vietnamese_bank_fees() {
    // 1. VCB Digibank maintenance fee: 11,000 VND gross (10% VAT)
    let (net, vat) = unbundle_gross_fee(Money::vnd(11_000), VAT_STANDARD_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(10_000));
    assert_eq!(vat, Money::vnd(1_000));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(11_000));

    // 2. TCB Account management fee: 22,000 VND gross (10% VAT)
    let (net, vat) = unbundle_gross_fee(Money::vnd(22_000), VAT_STANDARD_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(20_000));
    assert_eq!(vat, Money::vnd(2_000));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(22_000));

    // 3. BIDV Security Token renewal fee: 220,000 VND gross (10% VAT)
    let (net, vat) = unbundle_gross_fee(Money::vnd(220_000), VAT_STANDARD_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(200_000));
    assert_eq!(vat, Money::vnd(20_000));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(220_000));

    // 4. VCB Napas 247 interbank wire fee: 3,300 VND gross (10% VAT)
    let (net, vat) = unbundle_gross_fee(Money::vnd(3_300), VAT_STANDARD_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(3_000));
    assert_eq!(vat, Money::vnd(300));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(3_300));

    // 5. CTG Annual card fee: 1,650 VND gross (10% VAT)
    let (net, vat) = unbundle_gross_fee(Money::vnd(1_650), VAT_STANDARD_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(1_500));
    assert_eq!(vat, Money::vnd(150));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(1_650));

    // 6. OCB SMS Banking fee: 8,800 VND gross (10% VAT)
    let (net, vat) = unbundle_gross_fee(Money::vnd(8_800), VAT_STANDARD_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(8_000));
    assert_eq!(vat, Money::vnd(800));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(8_800));

    // 7. SME Reduced VAT rate: 1,080 VND gross (8% VAT = 800 bps)
    let (net, vat) = unbundle_gross_fee(Money::vnd(1_080), VAT_REDUCED_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(1_000));
    assert_eq!(vat, Money::vnd(80));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(1_080));
}

#[test]
fn test_exempt_and_zero_vat() {
    let gross = Money::vnd(50_000);
    let (net, vat) = unbundle_gross_fee(gross, VAT_EXEMPT_RATE_BPS).unwrap();
    assert_eq!(net, gross);
    assert_eq!(vat, Money::vnd(0));
    assert_eq!(net.checked_add(vat).unwrap(), gross);
}

#[test]
fn test_small_amount_boundary() {
    // 1 VND cannot have fractional VAT in integer VND, so VAT = 0 and Net = 1
    let (net, vat) = unbundle_gross_fee(Money::vnd(1), VAT_STANDARD_RATE_BPS).unwrap();
    assert_eq!(net, Money::vnd(1));
    assert_eq!(vat, Money::vnd(0));
    assert_eq!(net.checked_add(vat).unwrap(), Money::vnd(1));
}

#[test]
fn test_invalid_gross_rejected() {
    assert_eq!(
        unbundle_gross_fee(Money::vnd(0), VAT_STANDARD_RATE_BPS),
        Err(LedgerError::NonPositiveAmount)
    );
    assert_eq!(
        unbundle_gross_fee(Money::vnd(-5_000), VAT_STANDARD_RATE_BPS),
        Err(LedgerError::NonPositiveAmount)
    );
}

#[test]
fn test_statutory_routing() {
    // Circular 200: routes to 6427 and 1331
    let res_200 = route_and_unbundle_fee(
        Money::vnd(11_000),
        VAT_STANDARD_RATE_BPS,
        AccountStandard::Circular200_2014,
        None,
        None,
    )
    .unwrap();
    assert_eq!(res_200.expense_account, "6427");
    assert_eq!(res_200.vat_account, "1331");
    assert_eq!(res_200.net, Money::vnd(10_000));
    assert_eq!(res_200.vat, Money::vnd(1_000));
    assert!(res_200.has_vat());

    // Circular 133: routes to 6422 and 1331
    let res_133 = route_and_unbundle_fee(
        Money::vnd(22_000),
        VAT_STANDARD_RATE_BPS,
        AccountStandard::Circular133_2016,
        None,
        None,
    )
    .unwrap();
    assert_eq!(res_133.expense_account, "6422");
    assert_eq!(res_133.vat_account, "1331");
    assert_eq!(res_133.net, Money::vnd(20_000));
    assert_eq!(res_133.vat, Money::vnd(2_000));

    // Custom overrides: routes to custom accounts
    let res_custom = route_and_unbundle_fee(
        Money::vnd(33_000),
        VAT_STANDARD_RATE_BPS,
        AccountStandard::Custom,
        Some("6427.VCB.FEE"),
        Some("1331.TAX.IN"),
    )
    .unwrap();
    assert_eq!(res_custom.expense_account, "6427.VCB.FEE");
    assert_eq!(res_custom.vat_account, "1331.TAX.IN");
    assert_eq!(res_custom.net, Money::vnd(30_000));
    assert_eq!(res_custom.vat, Money::vnd(3_000));
}
