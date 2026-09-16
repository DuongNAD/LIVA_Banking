//! Adversarial Challenge Test Suite 2 for `liva-core`.
//!
//! EMPIRICAL CHALLENGER 2 INDEPENDENT STRESS HARNESS
//! Tests:
//! 1. Multi-currency scaling (VND scale=0, USD scale=2, EUR scale=2), ISO string parsing, and boundary scaling.
//! 2. Banker's rounding exhaustive oracle, halfway ties, epsilon boundaries, intermediate 128-bit scaling.
//! 3. Zero-penny-drift allocation property test across arbitrary amounts and ratio partitions.
//! 4. Checked arithmetic extreme boundaries (i64::MIN abs, multiplication overflow).
//! 5. Inter-period balance continuity verification with simulated ledger transitions and randomized fault injections.

use liva_core::{
    verify_period_sequence, ContinuityError, Currency, Money, MoneyError, PeriodBalance,
};
use proptest::prelude::*;
use std::str::FromStr;

// ============================================================================
// 1. Multi-Currency Scaling (VND, USD, EUR) & Scale Precision
// ============================================================================

#[test]
fn test_currency_properties_vnd_usd_eur() {
    // VND: 0 decimal places, minor unit scale = 1
    assert_eq!(Currency::VND.decimal_places(), 0);
    assert_eq!(Currency::VND.minor_unit_scale(), 1);
    assert_eq!(Currency::VND.code(), "VND");
    assert_eq!(format!("{}", Currency::VND), "VND");

    // USD: 2 decimal places, minor unit scale = 100
    assert_eq!(Currency::USD.decimal_places(), 2);
    assert_eq!(Currency::USD.minor_unit_scale(), 100);
    assert_eq!(Currency::USD.code(), "USD");
    assert_eq!(format!("{}", Currency::USD), "USD");

    // EUR: 2 decimal places, minor unit scale = 100
    assert_eq!(Currency::EUR.decimal_places(), 2);
    assert_eq!(Currency::EUR.minor_unit_scale(), 100);
    assert_eq!(Currency::EUR.code(), "EUR");
    assert_eq!(format!("{}", Currency::EUR), "EUR");
}

#[test]
fn test_currency_from_str_comprehensive() {
    // VND variants
    assert_eq!(Currency::from_str("VND").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("vnd").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("VNĐ").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("vnđ").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("DONG").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("dong").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("ĐỒNG").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("đồng").unwrap(), Currency::VND);
    assert_eq!(Currency::from_str("  VND  ").unwrap(), Currency::VND);

    // USD variants
    assert_eq!(Currency::from_str("USD").unwrap(), Currency::USD);
    assert_eq!(Currency::from_str("usd").unwrap(), Currency::USD);
    assert_eq!(Currency::from_str("$").unwrap(), Currency::USD);
    assert_eq!(Currency::from_str("  $ ").unwrap(), Currency::USD);

    // EUR variants
    assert_eq!(Currency::from_str("EUR").unwrap(), Currency::EUR);
    assert_eq!(Currency::from_str("eur").unwrap(), Currency::EUR);
    assert_eq!(Currency::from_str("€").unwrap(), Currency::EUR);
    assert_eq!(Currency::from_str("  €  ").unwrap(), Currency::EUR);

    // Invalid codes
    assert!(matches!(
        Currency::from_str("JPY"),
        Err(MoneyError::ParseError(_))
    ));
    assert!(matches!(
        Currency::from_str("GBP"),
        Err(MoneyError::ParseError(_))
    ));
    assert!(matches!(
        Currency::from_str(""),
        Err(MoneyError::ParseError(_))
    ));
    assert!(matches!(
        Currency::from_str("   "),
        Err(MoneyError::ParseError(_))
    ));
    assert!(matches!(
        Currency::from_str("XYZ"),
        Err(MoneyError::ParseError(_))
    ));
}

#[test]
fn test_from_major_scaling_and_overflow() {
    // VND: scale = 1
    let vnd_100 = Money::from_major(100, Currency::VND).unwrap();
    assert_eq!(vnd_100.amount(), 100);
    assert_eq!(vnd_100.currency(), Currency::VND);

    // VND: i64::MAX does not overflow since scale is 1
    let vnd_max = Money::from_major(i64::MAX, Currency::VND).unwrap();
    assert_eq!(vnd_max.amount(), i64::MAX);

    let vnd_min = Money::from_major(i64::MIN, Currency::VND).unwrap();
    assert_eq!(vnd_min.amount(), i64::MIN);

    // USD: scale = 100
    let usd_100 = Money::from_major(100, Currency::USD).unwrap();
    assert_eq!(usd_100.amount(), 10_000); // 100 USD = 10,000 cents
    assert_eq!(usd_100.currency(), Currency::USD);

    let usd_neg_50 = Money::from_major(-50, Currency::USD).unwrap();
    assert_eq!(usd_neg_50.amount(), -5_000);

    // USD: overflow when major * 100 > i64::MAX
    let usd_overflow = Money::from_major(i64::MAX / 100 + 1, Currency::USD);
    assert_eq!(usd_overflow, Err(MoneyError::Overflow));

    let usd_underflow = Money::from_major(i64::MIN / 100 - 1, Currency::USD);
    assert_eq!(usd_underflow, Err(MoneyError::Overflow));

    // EUR: scale = 100
    let eur_250 = Money::from_major(250, Currency::EUR).unwrap();
    assert_eq!(eur_250.amount(), 25_000); // 250 EUR = 25,000 cents
    assert_eq!(eur_250.currency(), Currency::EUR);

    let eur_overflow = Money::from_major(i64::MAX / 100 + 1, Currency::EUR);
    assert_eq!(eur_overflow, Err(MoneyError::Overflow));
}

#[test]
fn test_cross_currency_arithmetic_denial() {
    let vnd = Money::vnd(1_000_000);
    let usd = Money::usd_cents(10_000);
    let eur = Money::from_minor(10_000, Currency::EUR);

    // Additions
    assert_eq!(
        vnd.checked_add(usd),
        Err(MoneyError::CurrencyMismatch {
            expected: Currency::VND,
            found: Currency::USD,
        })
    );
    assert_eq!(
        usd.checked_add(eur),
        Err(MoneyError::CurrencyMismatch {
            expected: Currency::USD,
            found: Currency::EUR,
        })
    );
    assert_eq!(
        eur.checked_add(vnd),
        Err(MoneyError::CurrencyMismatch {
            expected: Currency::EUR,
            found: Currency::VND,
        })
    );

    // Subtractions
    assert_eq!(
        vnd.checked_sub(eur),
        Err(MoneyError::CurrencyMismatch {
            expected: Currency::VND,
            found: Currency::EUR,
        })
    );
}

// ============================================================================
// 2. Banker's Rounding & High-Precision Rational Scaling
// ============================================================================

#[test]
fn test_bankers_rounding_halfway_table_positive_and_negative() {
    // Test half-to-even for values from -10.5 to +10.5
    // amount = 2 * k + 1, ratio = 1/2 -> represents k + 0.5
    let cases: Vec<(i64, i64)> = vec![
        (-21, -10), // -10.5 -> -10 (even)
        (-19, -10), // -9.5  -> -10 (even)
        (-17, -8),  // -8.5  -> -8 (even)
        (-15, -8),  // -7.5  -> -8 (even)
        (-13, -6),  // -6.5  -> -6 (even)
        (-11, -6),  // -5.5  -> -6 (even)
        (-9, -4),   // -4.5  -> -4 (even)
        (-7, -4),   // -3.5  -> -4 (even)
        (-5, -2),   // -2.5  -> -2 (even)
        (-3, -2),   // -1.5  -> -2 (even)
        (-1, 0),    // -0.5  -> 0 (even)
        (1, 0),     //  0.5  -> 0 (even)
        (3, 2),     //  1.5  -> 2 (even)
        (5, 2),     //  2.5  -> 2 (even)
        (7, 4),     //  3.5  -> 4 (even)
        (9, 4),     //  4.5  -> 4 (even)
        (11, 6),    //  5.5  -> 6 (even)
        (13, 6),    //  6.5  -> 6 (even)
        (15, 8),    //  7.5  -> 8 (even)
        (17, 8),    //  8.5  -> 8 (even)
        (19, 10),   //  9.5  -> 10 (even)
        (21, 10),   //  10.5 -> 10 (even)
    ];

    for (amount, expected) in cases {
        let m = Money::vnd(amount);
        let rounded = m.checked_mul_ratio(1, 2).unwrap();
        assert_eq!(
            rounded.amount(),
            expected,
            "Failed Banker's rounding for {} * 1/2: expected {}, got {}",
            amount,
            expected,
            rounded.amount()
        );
    }
}

#[test]
fn test_bankers_rounding_epsilon_around_half() {
    // Base scale = 100
    // 49/100 -> 0.49 -> rounds to 0
    assert_eq!(Money::vnd(49).checked_mul_ratio(1, 100).unwrap().amount(), 0);
    // 50/100 -> 0.50 -> rounds to 0 (even)
    assert_eq!(Money::vnd(50).checked_mul_ratio(1, 100).unwrap().amount(), 0);
    // 51/100 -> 0.51 -> rounds to 1
    assert_eq!(Money::vnd(51).checked_mul_ratio(1, 100).unwrap().amount(), 1);

    // 149/100 -> 1.49 -> rounds to 1
    assert_eq!(Money::vnd(149).checked_mul_ratio(1, 100).unwrap().amount(), 1);
    // 150/100 -> 1.50 -> rounds to 2 (even)
    assert_eq!(Money::vnd(150).checked_mul_ratio(1, 100).unwrap().amount(), 2);
    // 151/100 -> 1.51 -> rounds to 2
    assert_eq!(Money::vnd(151).checked_mul_ratio(1, 100).unwrap().amount(), 2);

    // 249/100 -> 2.49 -> rounds to 2
    assert_eq!(Money::vnd(249).checked_mul_ratio(1, 100).unwrap().amount(), 2);
    // 250/100 -> 2.50 -> rounds to 2 (even)
    assert_eq!(Money::vnd(250).checked_mul_ratio(1, 100).unwrap().amount(), 2);
    // 251/100 -> 2.51 -> rounds to 3
    assert_eq!(Money::vnd(251).checked_mul_ratio(1, 100).unwrap().amount(), 3);

    // Negative epsilons
    // -49/100 -> -0.49 -> rounds to 0
    assert_eq!(Money::vnd(-49).checked_mul_ratio(1, 100).unwrap().amount(), 0);
    // -50/100 -> -0.50 -> rounds to 0 (even)
    assert_eq!(Money::vnd(-50).checked_mul_ratio(1, 100).unwrap().amount(), 0);
    // -51/100 -> -0.51 -> rounds to -1
    assert_eq!(Money::vnd(-51).checked_mul_ratio(1, 100).unwrap().amount(), -1);

    // -149/100 -> -1.49 -> rounds to -1
    assert_eq!(Money::vnd(-149).checked_mul_ratio(1, 100).unwrap().amount(), -1);
    // -150/100 -> -1.50 -> rounds to -2 (even)
    assert_eq!(Money::vnd(-150).checked_mul_ratio(1, 100).unwrap().amount(), -2);
    // -151/100 -> -1.51 -> rounds to -2
    assert_eq!(Money::vnd(-151).checked_mul_ratio(1, 100).unwrap().amount(), -2);
}

#[test]
fn test_intermediate_128_bit_scaling_precision() {
    // Large monetary amount: 6 * 10^18 (6 quintillion)
    // In 64-bit: 6e18 * 2 = 12e18 > i64::MAX (9.22e18), would overflow!
    // But in 128-bit: 12e18 / 3 = 4 * 10^18 < i64::MAX, succeeds cleanly!
    let large_val = 6_000_000_000_000_000_000i64;
    let m = Money::vnd(large_val);
    let scaled = m.checked_mul_ratio(2, 3).expect("should succeed via 128-bit math");
    assert_eq!(scaled.amount(), 4_000_000_000_000_000_000i64);

    // Symmetrically negative
    let m_neg = Money::vnd(-large_val);
    let scaled_neg = m_neg.checked_mul_ratio(2, 3).expect("should succeed via 128-bit math");
    assert_eq!(scaled_neg.amount(), -4_000_000_000_000_000_000i64);

    // Signs permutation on fraction
    assert_eq!(m.checked_mul_ratio(-2, 3).unwrap().amount(), -4_000_000_000_000_000_000i64);
    assert_eq!(m.checked_mul_ratio(2, -3).unwrap().amount(), -4_000_000_000_000_000_000i64);
    assert_eq!(m.checked_mul_ratio(-2, -3).unwrap().amount(), 4_000_000_000_000_000_000i64);
}

#[test]
fn test_checked_arithmetic_overflow_boundaries() {
    // abs(i64::MIN) overflows i64
    let min_money = Money::from_minor(i64::MIN, Currency::VND);
    assert_eq!(min_money.abs(), Err(MoneyError::Overflow));

    // checked_mul(i64::MIN, -1) overflows i64
    assert_eq!(min_money.checked_mul(-1), Err(MoneyError::Overflow));

    // checked_mul(i64::MAX, 2) overflows
    let max_money = Money::from_minor(i64::MAX, Currency::VND);
    assert_eq!(max_money.checked_mul(2), Err(MoneyError::Overflow));

    // checked_add(i64::MAX, 1) overflows
    assert_eq!(max_money.checked_add(Money::vnd(1)), Err(MoneyError::Overflow));

    // checked_sub(i64::MIN, 1) overflows
    assert_eq!(min_money.checked_sub(Money::vnd(1)), Err(MoneyError::Overflow));
}

#[test]
fn test_allocation_zero_drift_edge_cases() {
    // Single ratio
    let m = Money::vnd(1_234_567);
    let alloc_single = m.allocate(&[1]).unwrap();
    assert_eq!(alloc_single.len(), 1);
    assert_eq!(alloc_single[0].amount(), 1_234_567);

    // Allocation with ratio containing 0
    let alloc_zero_share = m.allocate(&[1, 0, 1]).unwrap();
    assert_eq!(alloc_zero_share.len(), 3);
    assert_eq!(alloc_zero_share[1].amount(), 0);
    assert_eq!(alloc_zero_share[0].amount() + alloc_zero_share[2].amount(), 1_234_567);

    // Error on empty ratios
    assert_eq!(m.allocate(&[]), Err(MoneyError::InvalidAllocationRatios));

    // Error on sum of ratios == 0
    assert_eq!(m.allocate(&[0, 0, 0]), Err(MoneyError::InvalidAllocationRatios));

    // Negative amount allocation
    let m_neg = Money::vnd(-100);
    let parts_neg = m_neg.allocate(&[1, 1, 1]).unwrap();
    let sum_neg: i64 = parts_neg.iter().map(|p| p.amount()).sum();
    assert_eq!(sum_neg, -100);
}

// ============================================================================
// 3. Inter-Period Continuity Invariant Stress
// ============================================================================

#[test]
fn test_accounting_period_flow_with_ledger_entries() {
    // Simulate real accounting ledger flow across 5 quarters:
    // Closing(Q_i) = Opening(Q_i) + Credits - Debits
    // Opening(Q_{i+1}) = Closing(Q_i)
    let initial_balance = Money::vnd(100_000_000);
    let quarters_activity = [
        (Money::vnd(50_000_000), Money::vnd(20_000_000)), // Q1: +30m -> 130m
        (Money::vnd(10_000_000), Money::vnd(40_000_000)), // Q2: -30m -> 100m
        (Money::vnd(80_000_000), Money::vnd(15_000_000)), // Q3: +65m -> 165m
        (Money::vnd(5_000_000), Money::vnd(70_000_000)),  // Q4: -65m -> 100m
        (Money::vnd(20_000_000), Money::vnd(20_000_000)), // Q5: +0m  -> 100m
    ];

    let mut periods = Vec::new();
    let mut current_open = initial_balance;

    for (idx, (credit, debit)) in quarters_activity.iter().enumerate() {
        let closing = current_open.checked_add(*credit).unwrap().checked_sub(*debit).unwrap();
        periods.push(PeriodBalance::new(
            format!("2026-Q{}", idx + 1),
            "ACC-CORP-VAS",
            current_open,
            closing,
        ));
        current_open = closing;
    }

    assert_eq!(verify_period_sequence(&periods), Ok(()));
}

#[test]
fn test_multi_period_sequence_discrepancy_at_exact_indices() {
    let make_10_periods = || {
        let mut periods = Vec::with_capacity(10);
        let mut bal = Money::usd_cents(100_000);
        for i in 0..10 {
            let next_bal = bal.checked_add(Money::usd_cents(5_000)).unwrap();
            periods.push(PeriodBalance::new(
                format!("2026-M{:02}", i + 1),
                "ACC-USD-01",
                bal,
                next_bal,
            ));
            bal = next_bal;
        }
        periods
    };

    // Test fault at period 1 (transition 0 -> 1)
    let mut seq1 = make_10_periods();
    seq1[1].opening_balance = seq1[1].opening_balance.checked_add(Money::usd_cents(100)).unwrap();
    match verify_period_sequence(&seq1) {
        Err(ContinuityError::Discrepancy { period_id, delta, .. }) => {
            assert_eq!(period_id, "2026-M02");
            assert_eq!(delta, 100);
        }
        other => panic!("Expected discrepancy at M02, got {other:?}"),
    }

    // Test fault at period 5 (transition 4 -> 5)
    let mut seq5 = make_10_periods();
    seq5[5].opening_balance = seq5[5].opening_balance.checked_sub(Money::usd_cents(250)).unwrap();
    match verify_period_sequence(&seq5) {
        Err(ContinuityError::Discrepancy { period_id, delta, .. }) => {
            assert_eq!(period_id, "2026-M06");
            assert_eq!(delta, -250);
        }
        other => panic!("Expected discrepancy at M06, got {other:?}"),
    }

    // Test fault at last period (transition 8 -> 9)
    let mut seq9 = make_10_periods();
    seq9[9].opening_balance = seq9[9].opening_balance.checked_add(Money::usd_cents(1)).unwrap();
    match verify_period_sequence(&seq9) {
        Err(ContinuityError::Discrepancy { period_id, delta, .. }) => {
            assert_eq!(period_id, "2026-M10");
            assert_eq!(delta, 1);
        }
        other => panic!("Expected discrepancy at M10, got {other:?}"),
    }
}

#[test]
fn test_account_and_currency_mismatches_mid_sequence() {
    let make_5_periods = || {
        vec![
            PeriodBalance::new("P1", "ACC-1", Money::vnd(100), Money::vnd(200)),
            PeriodBalance::new("P2", "ACC-1", Money::vnd(200), Money::vnd(300)),
            PeriodBalance::new("P3", "ACC-1", Money::vnd(300), Money::vnd(400)),
            PeriodBalance::new("P4", "ACC-1", Money::vnd(400), Money::vnd(500)),
            PeriodBalance::new("P5", "ACC-1", Money::vnd(500), Money::vnd(600)),
        ]
    };

    // Account mismatch midway at P3 -> P4
    let mut seq_acc = make_5_periods();
    seq_acc[3].account_id = "ACC-HIJACK".to_string();
    match verify_period_sequence(&seq_acc) {
        Err(ContinuityError::AccountMismatch { expected, actual }) => {
            assert_eq!(expected, "ACC-1");
            assert_eq!(actual, "ACC-HIJACK");
        }
        other => panic!("Expected AccountMismatch, got {other:?}"),
    }

    // Currency mismatch midway at P2 -> P3
    let mut seq_curr = make_5_periods();
    seq_curr[2].opening_balance = Money::usd_cents(300);
    seq_curr[2].closing_balance = Money::usd_cents(400);
    assert_eq!(
        verify_period_sequence(&seq_curr),
        Err(ContinuityError::CurrencyMismatch)
    );
}

// ============================================================================
// 4. Property-Based Stress: Zero Penny Drift & Sequence Invariant Oracles
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Property: `Money::allocate` preserves total sum with zero penny drift across all amounts and partitions.
    #[test]
    fn prop_allocation_zero_penny_drift(
        amount in -1_000_000_000_000i64..1_000_000_000_000i64,
        ratios in prop::collection::vec(1u32..50u32, 1..8),
        currency in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let money = Money::from_minor(amount, currency);
        let allocated = money.allocate(&ratios).expect("allocation must succeed for non-zero ratios");

        prop_assert_eq!(allocated.len(), ratios.len());

        let sum: i64 = allocated.iter().map(|p| p.amount()).sum();
        prop_assert_eq!(sum, amount, "Zero penny drift violated! Expected {}, got sum {}", amount, sum);

        for part in allocated {
            prop_assert_eq!(part.currency(), currency);
        }
    }

    /// Property: Arbitrary sequence with injected perturbation is always caught.
    #[test]
    fn prop_multi_period_chain_with_injected_discrepancy(
        period_count in 2usize..15usize,
        fault_idx in 1usize..15usize,
        perturbation in prop_oneof![-1_000_000i64..-1i64, 1i64..1_000_000i64],
        currency in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let actual_fault_idx = fault_idx % period_count;
        let inject_fault = actual_fault_idx > 0;

        let mut periods = Vec::with_capacity(period_count);
        let mut bal = 50_000_000i64;

        for i in 0..period_count {
            let next_bal = bal.saturating_add(10_000);
            periods.push(PeriodBalance::new(
                format!("PERIOD-{i}"),
                "ACC-PROP-CHAIN",
                Money::from_minor(bal, currency),
                Money::from_minor(next_bal, currency),
            ));
            bal = next_bal;
        }

        if inject_fault {
            let prev_close = periods[actual_fault_idx - 1].closing_balance.amount();
            let bad_open = prev_close.saturating_add(perturbation);
            prop_assume!(bad_open != prev_close);

            periods[actual_fault_idx].opening_balance = Money::from_minor(bad_open, currency);

            let res = verify_period_sequence(&periods);
            match res {
                Err(ContinuityError::Discrepancy { period_id, delta, .. }) => {
                    prop_assert_eq!(period_id, format!("PERIOD-{actual_fault_idx}"));
                    prop_assert_eq!(delta, bad_open.saturating_sub(prev_close));
                }
                other => prop_assert!(false, "Expected Discrepancy, got {:?}", other),
            }
        } else {
            prop_assert_eq!(verify_period_sequence(&periods), Ok(()));
        }
    }
}
