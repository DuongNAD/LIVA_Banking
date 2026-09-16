//! Integration and property-based tests for `liva-core` continuity invariants and checked arithmetic re-exports.

use liva_core::{
    verify_period_sequence, verify_period_transition, ContinuityError, Currency, Money, MoneyError,
    PeriodBalance,
};
use proptest::prelude::*;

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_successful_3_period_sequence() {
    let p1 = PeriodBalance::new(
        "2026-01",
        "ACC-VCB-001",
        Money::vnd(10_000_000),
        Money::vnd(15_000_000),
    );
    let p2 = PeriodBalance::new(
        "2026-02",
        "ACC-VCB-001",
        Money::vnd(15_000_000),
        Money::vnd(12_000_000),
    );
    let p3 = PeriodBalance::new(
        "2026-03",
        "ACC-VCB-001",
        Money::vnd(12_000_000),
        Money::vnd(25_000_000),
    );

    let periods = vec![p1.clone(), p2.clone(), p3.clone()];

    assert_eq!(verify_period_transition(&p1, &p2), Ok(()));
    assert_eq!(verify_period_transition(&p2, &p3), Ok(()));
    assert_eq!(verify_period_sequence(&periods), Ok(()));
}

#[test]
fn test_single_period_sequence_is_valid() {
    let p1 = PeriodBalance::new(
        "2026-01",
        "ACC-VCB-001",
        Money::vnd(10_000_000),
        Money::vnd(15_000_000),
    );
    assert_eq!(verify_period_sequence(&[p1]), Ok(()));
}

#[test]
fn test_discrepancy_detected_positive_delta() {
    let p1 = PeriodBalance::new(
        "2026-01",
        "ACC-VCB-001",
        Money::vnd(10_000_000),
        Money::vnd(15_000_000),
    );
    let p2 = PeriodBalance::new(
        "2026-02",
        "ACC-VCB-001",
        Money::vnd(15_000_500),
        Money::vnd(20_000_000),
    );

    let expected_err = ContinuityError::Discrepancy {
        account_id: "ACC-VCB-001".to_string(),
        period_id: "2026-02".to_string(),
        expected_opening: Money::vnd(15_000_000),
        actual_opening: Money::vnd(15_000_500),
        delta: 500,
    };

    assert_eq!(verify_period_transition(&p1, &p2), Err(expected_err.clone()));
    assert_eq!(verify_period_sequence(&[p1, p2]), Err(expected_err));
}

#[test]
fn test_discrepancy_detected_negative_delta() {
    let p1 = PeriodBalance::new(
        "2026-01",
        "ACC-VCB-001",
        Money::vnd(10_000_000),
        Money::vnd(15_000_000),
    );
    let p2 = PeriodBalance::new(
        "2026-02",
        "ACC-VCB-001",
        Money::vnd(14_999_999),
        Money::vnd(20_000_000),
    );

    let expected_err = ContinuityError::Discrepancy {
        account_id: "ACC-VCB-001".to_string(),
        period_id: "2026-02".to_string(),
        expected_opening: Money::vnd(15_000_000),
        actual_opening: Money::vnd(14_999_999),
        delta: -1,
    };

    assert_eq!(verify_period_transition(&p1, &p2), Err(expected_err.clone()));
    assert_eq!(verify_period_sequence(&[p1, p2]), Err(expected_err));
}

#[test]
fn test_account_mismatch() {
    let p1 = PeriodBalance::new(
        "2026-01",
        "ACC-001",
        Money::vnd(10_000_000),
        Money::vnd(15_000_000),
    );
    let p2 = PeriodBalance::new(
        "2026-02",
        "ACC-002",
        Money::vnd(15_000_000),
        Money::vnd(20_000_000),
    );

    let expected_err = ContinuityError::AccountMismatch {
        expected: "ACC-001".to_string(),
        actual: "ACC-002".to_string(),
    };

    assert_eq!(verify_period_transition(&p1, &p2), Err(expected_err.clone()));
    assert_eq!(verify_period_sequence(&[p1, p2]), Err(expected_err));
}

#[test]
fn test_currency_mismatch() {
    let p1 = PeriodBalance::new(
        "2026-01",
        "ACC-001",
        Money::vnd(10_000_000),
        Money::vnd(15_000_000),
    );
    let p2 = PeriodBalance::new(
        "2026-02",
        "ACC-001",
        Money::usd_cents(15_000_000),
        Money::usd_cents(20_000_000),
    );

    assert_eq!(
        verify_period_transition(&p1, &p2),
        Err(ContinuityError::CurrencyMismatch)
    );
    assert_eq!(
        verify_period_sequence(&[p1, p2]),
        Err(ContinuityError::CurrencyMismatch)
    );
}

#[test]
fn test_empty_sequence() {
    assert_eq!(
        verify_period_sequence(&[]),
        Err(ContinuityError::EmptySequence)
    );
}

#[test]
fn test_checked_arithmetic_reexport() {
    let a = Money::vnd(100_000);
    let b = Money::vnd(250_000);

    let sum = a.checked_add(b).expect("add should succeed");
    assert_eq!(sum, Money::vnd(350_000));

    let diff = a.checked_sub(b).expect("sub should succeed");
    assert_eq!(diff, Money::vnd(-150_000));

    let usd = Money::usd_cents(100);
    assert_eq!(
        a.checked_add(usd),
        Err(MoneyError::CurrencyMismatch {
            expected: Currency::VND,
            found: Currency::USD
        })
    );

    // Banker's Rounding: 2.5 rounds to 2 (even), 3.5 rounds to 4 (even)
    let two_half = Money::vnd(5).checked_mul_ratio(1, 2).unwrap();
    assert_eq!(two_half, Money::vnd(2));

    let three_half = Money::vnd(7).checked_mul_ratio(1, 2).unwrap();
    assert_eq!(three_half, Money::vnd(4));
}

#[test]
fn test_period_balance_serde_roundtrip() {
    let p = PeriodBalance::new(
        "2026-Q1",
        "ACC-999",
        Money::vnd(1_000_000),
        Money::vnd(2_000_000),
    );
    let json = serde_json::to_string(&p).expect("serialization failed");
    let deserialized: PeriodBalance = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(p, deserialized);
}

// ============================================================================
// Property-Based Tests (proptest)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25_000))]

    /// Property: Any dynamically generated continuous period sequence
    /// where Opening(i) == Closing(i-1) must pass verification without discrepancy.
    #[test]
    fn prop_continuous_sequence_always_valid(
        initial_balance in -1_000_000_000_000_000i64..1_000_000_000_000_000i64,
        deltas in prop::collection::vec(-10_000_000_000i64..10_000_000_000i64, 1..10),
        currency in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let account_id = "ACC-PROPTEST-CONTINUOUS";
        let mut periods = Vec::new();
        let mut current_bal = initial_balance;

        for (idx, delta) in deltas.into_iter().enumerate() {
            let next_bal = current_bal.saturating_add(delta);
            periods.push(PeriodBalance::new(
                format!("PERIOD-{idx}"),
                account_id,
                Money::from_minor(current_bal, currency),
                Money::from_minor(next_bal, currency),
            ));
            current_bal = next_bal;
        }

        prop_assert_eq!(verify_period_sequence(&periods), Ok(()));
    }

    /// Property: Any non-zero perturbation Δ between Closing(Period N-1) and
    /// Opening(Period N) must be detected and return the exact delta.
    #[test]
    fn prop_perturbed_transition_fails_with_exact_delta(
        closing_amount in -1_000_000_000_000_000i64..1_000_000_000_000_000i64,
        perturbation in prop_oneof![
            -1_000_000_000i64..-1i64,
            1i64..1_000_000_000i64
        ],
        currency in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let opening_amount = closing_amount.saturating_add(perturbation);
        // Ensure that saturation did not produce a zero difference
        prop_assume!(opening_amount != closing_amount);

        let p1 = PeriodBalance::new(
            "P-1",
            "ACC-TEST",
            Money::from_minor(0, currency),
            Money::from_minor(closing_amount, currency),
        );
        let p2 = PeriodBalance::new(
            "P-2",
            "ACC-TEST",
            Money::from_minor(opening_amount, currency),
            Money::from_minor(0, currency),
        );

        let expected_delta = opening_amount.saturating_sub(closing_amount);

        let result = verify_period_transition(&p1, &p2);
        match result {
            Err(ContinuityError::Discrepancy { delta, expected_opening, actual_opening, .. }) => {
                prop_assert_eq!(delta, expected_delta);
                prop_assert_eq!(expected_opening, Money::from_minor(closing_amount, currency));
                prop_assert_eq!(actual_opening, Money::from_minor(opening_amount, currency));
            }
            other => {
                prop_assert!(false, "Expected Discrepancy error, got {:?}", other);
            }
        }
    }

    /// Property: Account mismatch between adjacent periods is always detected.
    #[test]
    fn prop_account_mismatch_detected(
        acc1 in "[A-Z]{3}-[0-9]{3}",
        acc2 in "[A-Z]{3}-[0-9]{3}",
        bal in any::<i64>(),
    ) {
        prop_assume!(acc1 != acc2);

        let p1 = PeriodBalance::new("P1", acc1.clone(), Money::vnd(bal), Money::vnd(bal));
        let p2 = PeriodBalance::new("P2", acc2.clone(), Money::vnd(bal), Money::vnd(bal));

        let result = verify_period_transition(&p1, &p2);
        prop_assert_eq!(
            result,
            Err(ContinuityError::AccountMismatch {
                expected: acc1,
                actual: acc2,
            })
        );
    }

    /// Property: Currency mismatch between adjacent periods is always detected.
    #[test]
    fn prop_currency_mismatch_detected(
        (curr1, curr2) in prop_oneof![
            Just((Currency::VND, Currency::USD)),
            Just((Currency::VND, Currency::EUR)),
            Just((Currency::USD, Currency::VND)),
            Just((Currency::USD, Currency::EUR)),
            Just((Currency::EUR, Currency::VND)),
            Just((Currency::EUR, Currency::USD)),
        ],
        bal in any::<i64>(),
    ) {
        let p1 = PeriodBalance::new("P1", "ACC-1", Money::from_minor(bal, curr1), Money::from_minor(bal, curr1));
        let p2 = PeriodBalance::new("P2", "ACC-1", Money::from_minor(bal, curr2), Money::from_minor(bal, curr2));

        let result = verify_period_transition(&p1, &p2);
        prop_assert_eq!(result, Err(ContinuityError::CurrencyMismatch));
    }
}
