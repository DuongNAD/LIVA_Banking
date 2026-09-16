//! Adversarial Challenge Test Suite for `liva-core`.
//!
//! EMPIRICAL CHALLENGER STRESS HARNESS
//! Tests:
//! 1. Extreme mathematical bounds (i64::MAX, i64::MIN, 0, saturation behavior).
//! 2. Single-point fault injection across arbitrary sequence lengths (up to 1,000 periods).
//! 3. Combined mismatch scenarios (AccountMismatch vs CurrencyMismatch precedence).
//! 4. Banker's rounding comprehensive oracle (positive/negative half-way, even/odd).
//! 5. Ratio multiplication overflow and zero denominator rejection.
//! 6. Scrambled / reverse sequence continuity rejection.
//! 7. String representation formatting under extreme bounds.

use liva_core::{
    verify_period_sequence, verify_period_transition, ContinuityError, Currency, Money, MoneyError,
    PeriodBalance,
};
use proptest::prelude::*;

// ============================================================================
// 1. Extreme Mathematical Boundaries & Saturation
// ============================================================================

#[test]
fn test_boundary_max_to_min_saturation_no_panic() {
    let p1 = PeriodBalance::new(
        "P1",
        "ACC-001",
        Money::vnd(0),
        Money::vnd(i64::MAX),
    );
    let p2 = PeriodBalance::new(
        "P2",
        "ACC-001",
        Money::vnd(i64::MIN),
        Money::vnd(0),
    );

    let res = verify_period_transition(&p1, &p2);
    // i64::MIN.saturating_sub(i64::MAX) should saturate to i64::MIN without panic
    assert_eq!(
        res,
        Err(ContinuityError::Discrepancy {
            account_id: "ACC-001".to_string(),
            period_id: "P2".to_string(),
            expected_opening: Money::vnd(i64::MAX),
            actual_opening: Money::vnd(i64::MIN),
            delta: i64::MIN,
        })
    );
}

#[test]
fn test_boundary_min_to_max_saturation_no_panic() {
    let p1 = PeriodBalance::new(
        "P1",
        "ACC-001",
        Money::vnd(0),
        Money::vnd(i64::MIN),
    );
    let p2 = PeriodBalance::new(
        "P2",
        "ACC-001",
        Money::vnd(i64::MAX),
        Money::vnd(0),
    );

    let res = verify_period_transition(&p1, &p2);
    // i64::MAX.saturating_sub(i64::MIN) should saturate to i64::MAX without panic
    assert_eq!(
        res,
        Err(ContinuityError::Discrepancy {
            account_id: "ACC-001".to_string(),
            period_id: "P2".to_string(),
            expected_opening: Money::vnd(i64::MIN),
            actual_opening: Money::vnd(i64::MAX),
            delta: i64::MAX,
        })
    );
}

#[test]
fn test_boundary_max_to_max_identity() {
    let p1 = PeriodBalance::new("P1", "ACC-001", Money::vnd(0), Money::vnd(i64::MAX));
    let p2 = PeriodBalance::new("P2", "ACC-001", Money::vnd(i64::MAX), Money::vnd(0));
    assert_eq!(verify_period_transition(&p1, &p2), Ok(()));
}

#[test]
fn test_boundary_min_to_min_identity() {
    let p1 = PeriodBalance::new("P1", "ACC-001", Money::vnd(0), Money::vnd(i64::MIN));
    let p2 = PeriodBalance::new("P2", "ACC-001", Money::vnd(i64::MIN), Money::vnd(0));
    assert_eq!(verify_period_transition(&p1, &p2), Ok(()));
}

#[test]
fn test_boundary_zero_to_plus_minus_one() {
    let p1 = PeriodBalance::new("P1", "ACC-001", Money::vnd(0), Money::vnd(0));
    let p2_plus = PeriodBalance::new("P2", "ACC-001", Money::vnd(1), Money::vnd(0));
    let p2_minus = PeriodBalance::new("P2", "ACC-001", Money::vnd(-1), Money::vnd(0));

    assert_eq!(
        verify_period_transition(&p1, &p2_plus),
        Err(ContinuityError::Discrepancy {
            account_id: "ACC-001".to_string(),
            period_id: "P2".to_string(),
            expected_opening: Money::vnd(0),
            actual_opening: Money::vnd(1),
            delta: 1,
        })
    );

    assert_eq!(
        verify_period_transition(&p1, &p2_minus),
        Err(ContinuityError::Discrepancy {
            account_id: "ACC-001".to_string(),
            period_id: "P2".to_string(),
            expected_opening: Money::vnd(0),
            actual_opening: Money::vnd(-1),
            delta: -1,
        })
    );
}

#[test]
fn test_boundary_max_minus_one_discrepancy() {
    let p1 = PeriodBalance::new("P1", "ACC-001", Money::vnd(0), Money::vnd(i64::MAX));
    let p2 = PeriodBalance::new("P2", "ACC-001", Money::vnd(i64::MAX - 1), Money::vnd(0));

    assert_eq!(
        verify_period_transition(&p1, &p2),
        Err(ContinuityError::Discrepancy {
            account_id: "ACC-001".to_string(),
            period_id: "P2".to_string(),
            expected_opening: Money::vnd(i64::MAX),
            actual_opening: Money::vnd(i64::MAX - 1),
            delta: -1,
        })
    );
}

#[test]
fn test_boundary_min_plus_one_discrepancy() {
    let p1 = PeriodBalance::new("P1", "ACC-001", Money::vnd(0), Money::vnd(i64::MIN));
    let p2 = PeriodBalance::new("P2", "ACC-001", Money::vnd(i64::MIN + 1), Money::vnd(0));

    assert_eq!(
        verify_period_transition(&p1, &p2),
        Err(ContinuityError::Discrepancy {
            account_id: "ACC-001".to_string(),
            period_id: "P2".to_string(),
            expected_opening: Money::vnd(i64::MIN),
            actual_opening: Money::vnd(i64::MIN + 1),
            delta: 1,
        })
    );
}

// ============================================================================
// 2. Error Precedence & Multi-Dimension Mismatches
// ============================================================================

#[test]
fn test_precedence_account_mismatch_over_currency_and_delta() {
    // Both account and currency mismatch, plus delta != 0
    let p1 = PeriodBalance::new("P1", "ACC-AAA", Money::vnd(100), Money::vnd(500));
    let p2 = PeriodBalance::new("P2", "ACC-BBB", Money::usd_cents(600), Money::usd_cents(700));

    assert_eq!(
        verify_period_transition(&p1, &p2),
        Err(ContinuityError::AccountMismatch {
            expected: "ACC-AAA".to_string(),
            actual: "ACC-BBB".to_string(),
        })
    );
}

#[test]
fn test_precedence_currency_mismatch_over_delta() {
    // Account matches, but currency mismatches, plus amount differs
    let p1 = PeriodBalance::new("P1", "ACC-SAME", Money::vnd(100), Money::vnd(500));
    let p2 = PeriodBalance::new("P2", "ACC-SAME", Money::usd_cents(500), Money::usd_cents(700));

    assert_eq!(
        verify_period_transition(&p1, &p2),
        Err(ContinuityError::CurrencyMismatch)
    );
}

#[test]
fn test_account_id_subtle_differences() {
    // Whitespace sensitive
    let p1 = PeriodBalance::new("P1", "ACC-01", Money::vnd(100), Money::vnd(100));
    let p2 = PeriodBalance::new("P2", "ACC-01 ", Money::vnd(100), Money::vnd(100));
    assert!(matches!(
        verify_period_transition(&p1, &p2),
        Err(ContinuityError::AccountMismatch { .. })
    ));

    // Case sensitive
    let p3 = PeriodBalance::new("P3", "acc-01", Money::vnd(100), Money::vnd(100));
    assert!(matches!(
        verify_period_transition(&p1, &p3),
        Err(ContinuityError::AccountMismatch { .. })
    ));
}

#[test]
fn test_all_currency_pair_permutations() {
    let currencies = [Currency::VND, Currency::USD, Currency::EUR];
    for (i, &c1) in currencies.iter().enumerate() {
        for (j, &c2) in currencies.iter().enumerate() {
            let p1 = PeriodBalance::new("P1", "ACC", Money::from_minor(100, c1), Money::from_minor(100, c1));
            let p2 = PeriodBalance::new("P2", "ACC", Money::from_minor(100, c2), Money::from_minor(100, c2));

            if i == j {
                assert_eq!(verify_period_transition(&p1, &p2), Ok(()));
            } else {
                assert_eq!(
                    verify_period_transition(&p1, &p2),
                    Err(ContinuityError::CurrencyMismatch)
                );
            }
        }
    }
}

// ============================================================================
// 3. Sequence Length Stress & Fault Injection
// ============================================================================

#[test]
fn test_large_sequence_1000_periods_continuous() {
    let mut periods = Vec::with_capacity(1000);
    let mut bal = 1_000_000i64;

    for i in 0..1000 {
        let next_bal = bal + (i % 100);
        periods.push(PeriodBalance::new(
            format!("P-{i}"),
            "ACC-CORP-99",
            Money::vnd(bal),
            Money::vnd(next_bal),
        ));
        bal = next_bal;
    }

    assert_eq!(verify_period_sequence(&periods), Ok(()));
}

#[test]
fn test_fault_injection_at_head_middle_tail() {
    let make_base_sequence = || {
        let mut periods = Vec::with_capacity(50);
        let mut bal = 10_000i64;
        for i in 0..50 {
            let next_bal = bal + 100;
            periods.push(PeriodBalance::new(
                format!("P-{i}"),
                "ACC-FAULT",
                Money::vnd(bal),
                Money::vnd(next_bal),
            ));
            bal = next_bal;
        }
        periods
    };

    // 1. Fault at transition 0 -> 1 (head)
    let mut seq_head = make_base_sequence();
    seq_head[1].opening_balance = Money::vnd(seq_head[1].opening_balance.amount() + 1);
    match verify_period_sequence(&seq_head) {
        Err(ContinuityError::Discrepancy { period_id, delta, .. }) => {
            assert_eq!(period_id, "P-1");
            assert_eq!(delta, 1);
        }
        other => panic!("Expected Discrepancy at head, got {other:?}"),
    }

    // 2. Fault at transition 25 -> 26 (middle)
    let mut seq_mid = make_base_sequence();
    seq_mid[26].opening_balance = Money::vnd(seq_mid[26].opening_balance.amount() - 77);
    match verify_period_sequence(&seq_mid) {
        Err(ContinuityError::Discrepancy { period_id, delta, .. }) => {
            assert_eq!(period_id, "P-26");
            assert_eq!(delta, -77);
        }
        other => panic!("Expected Discrepancy at middle, got {other:?}"),
    }

    // 3. Fault at transition 48 -> 49 (tail)
    let mut seq_tail = make_base_sequence();
    seq_tail[49].opening_balance = Money::vnd(seq_tail[49].opening_balance.amount() + 999);
    match verify_period_sequence(&seq_tail) {
        Err(ContinuityError::Discrepancy { period_id, delta, .. }) => {
            assert_eq!(period_id, "P-49");
            assert_eq!(delta, 999);
        }
        other => panic!("Expected Discrepancy at tail, got {other:?}"),
    }
}

#[test]
fn test_scrambled_or_reversed_period_sequence_fails() {
    let p1 = PeriodBalance::new("P1", "ACC-01", Money::vnd(100), Money::vnd(200));
    let p2 = PeriodBalance::new("P2", "ACC-01", Money::vnd(200), Money::vnd(300));
    let p3 = PeriodBalance::new("P3", "ACC-01", Money::vnd(300), Money::vnd(400));

    // Normal order is valid
    assert_eq!(verify_period_sequence(&[p1.clone(), p2.clone(), p3.clone()]), Ok(()));

    // Reversed order fails: p3.closing (400) != p2.opening (200)
    assert!(matches!(
        verify_period_sequence(&[p3, p2, p1]),
        Err(ContinuityError::Discrepancy { delta: -200, .. })
    ));
}

// ============================================================================
// 4. Banker's Rounding Oracles & Ratio Arithmetic Stress
// ============================================================================

#[test]
fn test_comprehensive_bankers_rounding_oracle() {
    // Exact half-way cases (denominator = 2)
    // 0.5 -> 0 (even)
    assert_eq!(Money::vnd(1).checked_mul_ratio(1, 2).unwrap(), Money::vnd(0));
    // 1.5 -> 2 (even)
    assert_eq!(Money::vnd(3).checked_mul_ratio(1, 2).unwrap(), Money::vnd(2));
    // 2.5 -> 2 (even)
    assert_eq!(Money::vnd(5).checked_mul_ratio(1, 2).unwrap(), Money::vnd(2));
    // 3.5 -> 4 (even)
    assert_eq!(Money::vnd(7).checked_mul_ratio(1, 2).unwrap(), Money::vnd(4));
    // 4.5 -> 4 (even)
    assert_eq!(Money::vnd(9).checked_mul_ratio(1, 2).unwrap(), Money::vnd(4));
    // 5.5 -> 6 (even)
    assert_eq!(Money::vnd(11).checked_mul_ratio(1, 2).unwrap(), Money::vnd(6));

    // Negative exact half-way cases
    // -0.5 -> 0 (even)
    assert_eq!(Money::vnd(-1).checked_mul_ratio(1, 2).unwrap(), Money::vnd(0));
    // -1.5 -> -2 (even)
    assert_eq!(Money::vnd(-3).checked_mul_ratio(1, 2).unwrap(), Money::vnd(-2));
    // -2.5 -> -2 (even)
    assert_eq!(Money::vnd(-5).checked_mul_ratio(1, 2).unwrap(), Money::vnd(-2));
    // -3.5 -> -4 (even)
    assert_eq!(Money::vnd(-7).checked_mul_ratio(1, 2).unwrap(), Money::vnd(-4));
    // -4.5 -> -4 (even)
    assert_eq!(Money::vnd(-9).checked_mul_ratio(1, 2).unwrap(), Money::vnd(-4));

    // Non-halfway rounding
    // 1.4 -> 1 (denominator = 10, num = 14)
    assert_eq!(Money::vnd(14).checked_mul_ratio(1, 10).unwrap(), Money::vnd(1));
    // 1.6 -> 2
    assert_eq!(Money::vnd(16).checked_mul_ratio(1, 10).unwrap(), Money::vnd(2));
    // -1.4 -> -1
    assert_eq!(Money::vnd(-14).checked_mul_ratio(1, 10).unwrap(), Money::vnd(-1));
    // -1.6 -> -2
    assert_eq!(Money::vnd(-16).checked_mul_ratio(1, 10).unwrap(), Money::vnd(-2));
}

#[test]
fn test_ratio_multiplication_edge_cases() {
    // Division by zero
    assert_eq!(
        Money::vnd(100).checked_mul_ratio(1, 0),
        Err(MoneyError::DivisionByZero)
    );

    // Negative denominator
    assert_eq!(
        Money::vnd(5).checked_mul_ratio(1, -2).unwrap(),
        Money::vnd(-2) // -2.5 rounds to -2
    );

    // Identity with extreme values
    assert_eq!(
        Money::vnd(i64::MAX).checked_mul_ratio(1, 1).unwrap(),
        Money::vnd(i64::MAX)
    );
    assert_eq!(
        Money::vnd(i64::MIN).checked_mul_ratio(1, 1).unwrap(),
        Money::vnd(i64::MIN)
    );

    // Overflow when multiplied by > 1
    assert_eq!(
        Money::vnd(i64::MAX).checked_mul_ratio(2, 1),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        Money::vnd(i64::MIN).checked_mul_ratio(2, 1),
        Err(MoneyError::Overflow)
    );
}

#[test]
fn test_error_display_formatting_extreme_values() {
    let err = ContinuityError::Discrepancy {
        account_id: "ACC-EXTREME".to_string(),
        period_id: "P-EXTREME".to_string(),
        expected_opening: Money::vnd(i64::MAX),
        actual_opening: Money::vnd(i64::MIN),
        delta: i64::MIN,
    };
    let formatted = err.to_string();
    assert!(formatted.contains("ACC-EXTREME"));
    assert!(formatted.contains("P-EXTREME"));
    assert!(formatted.contains(&i64::MIN.to_string()));
}

// ============================================================================
// 5. Property-Based Stress: Any Discrepancy Caught
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_discrepancy_always_detected_and_delta_never_zero(
        closing in any::<i64>(),
        opening in any::<i64>(),
        currency in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let p1 = PeriodBalance::new("P1", "ACC-PROP", Money::from_minor(0, currency), Money::from_minor(closing, currency));
        let p2 = PeriodBalance::new("P2", "ACC-PROP", Money::from_minor(opening, currency), Money::from_minor(0, currency));

        let res = verify_period_transition(&p1, &p2);
        if closing == opening {
            prop_assert_eq!(res, Ok(()));
        } else {
            match res {
                Err(ContinuityError::Discrepancy { delta, expected_opening, actual_opening, .. }) => {
                    // Delta must NEVER be zero when closing != opening
                    prop_assert_ne!(delta, 0i64);
                    prop_assert_eq!(expected_opening, Money::from_minor(closing, currency));
                    prop_assert_eq!(actual_opening, Money::from_minor(opening, currency));
                }
                other => prop_assert!(false, "Unexpected result {:?}", other),
            }
        }
    }
}
