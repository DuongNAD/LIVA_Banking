//! Property-based testing for `liva-money` arithmetic invariants.
//!
//! Validates:
//! 1. Additive commutativity, associativity, and identity across i64.
//! 2. Currency mismatch isolation.
//! 3. Zero penny drift on arbitrary split allocations (positive, negative, and dynamic ratios).
//! 4. Split allocation error handling (empty slices, all-zero ratios).
//! 5. Deterministic Banker's Rounding (Round-Half-to-Even) rational scaling.
//! 6. Scaling identity, zero numerator, and division by zero rejection.
//! 7. Zero overflow boundary enforcement against 128-bit reference integer arithmetic.
//! 8. Zero float drift and exact integer reversibility.

use liva_money::{Currency, Money, MoneyError};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100_000))]

    /// 1. Invariant: Additive commutativity holds across the entire i64 domain.
    /// If an addition overflows, both (a + b) and (b + a) return Err(MoneyError::Overflow).
    /// If it fits, both produce identical amounts.
    #[test]
    fn prop_addition_commutativity(
        a in any::<i64>(),
        b in any::<i64>(),
        curr in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let m_a = Money::from_minor(a, curr);
        let m_b = Money::from_minor(b, curr);

        let sum_ab = m_a.checked_add(m_b);
        let sum_ba = m_b.checked_add(m_a);

        prop_assert_eq!(sum_ab, sum_ba);
    }

    /// 2. Invariant: Additive associativity holds within realistic banking bounds (quadrillion scale).
    #[test]
    fn prop_addition_associativity(
        a in -1_000_000_000_000_000i64..1_000_000_000_000_000i64,
        b in -1_000_000_000_000_000i64..1_000_000_000_000_000i64,
        c in -1_000_000_000_000_000i64..1_000_000_000_000_000i64,
        curr in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let m_a = Money::from_minor(a, curr);
        let m_b = Money::from_minor(b, curr);
        let m_c = Money::from_minor(c, curr);

        let left = m_a.checked_add(m_b).unwrap().checked_add(m_c).unwrap();
        let right = m_a.checked_add(m_b.checked_add(m_c).unwrap()).unwrap();

        prop_assert_eq!(left, right);
    }

    /// 3. Invariant: Identity and inverse elements hold for all amounts.
    /// m + 0 == m, m - 0 == m, m - m == 0.
    #[test]
    fn prop_identity_and_inverse(
        a in any::<i64>(),
        curr in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        let m_a = Money::from_minor(a, curr);
        let zero = Money::zero(curr);

        prop_assert_eq!(m_a.checked_add(zero).unwrap(), m_a);
        prop_assert_eq!(m_a.checked_sub(zero).unwrap(), m_a);
        prop_assert_eq!(m_a.checked_sub(m_a).unwrap(), zero);
    }

    /// 4. Invariant: Operations across mismatched currencies are strictly rejected.
    #[test]
    fn prop_currency_mismatch_isolation(
        a in -1_000_000_000_000i64..1_000_000_000_000i64,
        b in -1_000_000_000_000i64..1_000_000_000_000i64,
        (c1, c2) in prop_oneof![
            Just((Currency::VND, Currency::USD)),
            Just((Currency::VND, Currency::EUR)),
            Just((Currency::USD, Currency::VND)),
            Just((Currency::USD, Currency::EUR)),
            Just((Currency::EUR, Currency::VND)),
            Just((Currency::EUR, Currency::USD)),
        ],
    ) {
        let m1 = Money::from_minor(a, c1);
        let m2 = Money::from_minor(b, c2);

        prop_assert_eq!(
            m1.checked_add(m2),
            Err(MoneyError::CurrencyMismatch {
                expected: c1,
                found: c2
            })
        );
        prop_assert_eq!(
            m1.checked_sub(m2),
            Err(MoneyError::CurrencyMismatch {
                expected: c1,
                found: c2
            })
        );
    }

    /// 5. Invariant: Arbitrary split allocations guarantee ZERO PENNY DRIFT across
    /// any arbitrary non-empty ratios slice (sum > 0) for positive, negative, and zero amounts.
    #[test]
    fn prop_arbitrary_split_allocation_zero_penny_drift(
        amount in -1_000_000_000_000_000i64..1_000_000_000_000_000i64,
        ratios in proptest::collection::vec(0u32..500u32, 1..=12),
        curr in prop_oneof![Just(Currency::VND), Just(Currency::USD), Just(Currency::EUR)],
    ) {
        prop_assume!(ratios.iter().any(|&r| r > 0));

        let m = Money::from_minor(amount, curr);
        let parts = m.allocate(&ratios).unwrap();

        // Invariant 1: Exact length preserved
        prop_assert_eq!(parts.len(), ratios.len());

        // Invariant 2: Zero Penny Drift — sum of parts strictly equals original amount
        let sum: i64 = parts.iter().map(|p| p.amount()).sum();
        prop_assert_eq!(sum, amount);

        // Invariant 3: Currency consistency
        prop_assert!(parts.iter().all(|p| p.currency() == curr));

        // Invariant 4: Zero ratio yields zero allocation
        for (i, &ratio) in ratios.iter().enumerate() {
            if ratio == 0 {
                prop_assert_eq!(parts[i].amount(), 0);
            }
        }

        // Invariant 5: Sign preservation
        if amount >= 0 {
            prop_assert!(parts.iter().all(|p| p.amount() >= 0));
        } else {
            prop_assert!(parts.iter().all(|p| p.amount() <= 0));
        }
    }

    /// 6. Invariant: Split allocation fails cleanly on invalid inputs (empty or all-zero ratios).
    #[test]
    fn prop_split_allocation_error_handling(
        amount in -1_000_000_000i64..1_000_000_000i64,
        zero_count in 1usize..=10,
    ) {
        let m = Money::vnd(amount);

        // Empty ratios
        prop_assert_eq!(m.allocate(&[]), Err(MoneyError::InvalidAllocationRatios));

        // All-zero ratios
        let zeros = vec![0u32; zero_count];
        prop_assert_eq!(m.allocate(&zeros), Err(MoneyError::InvalidAllocationRatios));
    }

    /// 7. Invariant: Banker's Rounding (Round-Half-to-Even) deterministic behavior.
    /// Halfway cases (odd / 2 = integer + 0.5) must strictly round to the unique adjacent EVEN integer.
    #[test]
    fn prop_bankers_rounding_half_to_even(
        n in -50_000_000i64..50_000_000i64,
    ) {
        let odd_val = 2 * n + 1;
        let m = Money::vnd(odd_val);
        let half = m.checked_mul_ratio(1, 2).unwrap();

        // Invariant 1: Result is strictly even
        prop_assert_eq!(half.amount() % 2, 0);

        // Invariant 2: Rounding error distance is strictly 1 minor unit step (half-unit of denominator 2)
        prop_assert_eq!((half.amount() * 2 - odd_val).abs(), 1);
    }

    /// 8. Invariant: Rational scaling identity, division-by-zero rejection, and zero numerator.
    #[test]
    fn prop_ratio_multiplication_invariants(
        a in -1_000_000_000_000_000i64..1_000_000_000_000_000i64,
        factor in 1i64..10_000_000i64,
    ) {
        let m = Money::vnd(a);

        // Identity: a * (k / k) == a
        prop_assert_eq!(m.checked_mul_ratio(factor, factor).unwrap(), m);
        prop_assert_eq!(m.checked_mul_ratio(-factor, -factor).unwrap(), m);

        // Division by zero is rejected
        prop_assert_eq!(m.checked_mul_ratio(factor, 0), Err(MoneyError::DivisionByZero));
        prop_assert_eq!(m.checked_mul_ratio(0, 0), Err(MoneyError::DivisionByZero));

        // Zero numerator yields zero
        prop_assert_eq!(m.checked_mul_ratio(0, factor).unwrap(), Money::zero(Currency::VND));
    }

    /// 9. Invariant: Checked arithmetic overflow detection verified against 128-bit integer reference.
    #[test]
    fn prop_overflow_detection_checked_arithmetic(
        a in any::<i64>(),
        b in any::<i64>(),
    ) {
        // Addition
        let add_128 = (a as i128) + (b as i128);
        if add_128 > i64::MAX as i128 || add_128 < i64::MIN as i128 {
            prop_assert_eq!(Money::vnd(a).checked_add(Money::vnd(b)), Err(MoneyError::Overflow));
        } else {
            prop_assert_eq!(
                Money::vnd(a).checked_add(Money::vnd(b)).unwrap().amount(),
                add_128 as i64
            );
        }

        // Subtraction
        let sub_128 = (a as i128) - (b as i128);
        if sub_128 > i64::MAX as i128 || sub_128 < i64::MIN as i128 {
            prop_assert_eq!(Money::vnd(a).checked_sub(Money::vnd(b)), Err(MoneyError::Overflow));
        } else {
            prop_assert_eq!(
                Money::vnd(a).checked_sub(Money::vnd(b)).unwrap().amount(),
                sub_128 as i64
            );
        }
    }

    /// 10. Invariant: Checked multiplication overflow detection verified against 128-bit reference.
    #[test]
    fn prop_overflow_detection_multiplication(
        a in any::<i64>(),
        factor in any::<i64>(),
    ) {
        let mul_128 = (a as i128) * (factor as i128);
        if mul_128 > i64::MAX as i128 || mul_128 < i64::MIN as i128 {
            prop_assert_eq!(Money::vnd(a).checked_mul(factor), Err(MoneyError::Overflow));
        } else {
            prop_assert_eq!(
                Money::vnd(a).checked_mul(factor).unwrap().amount(),
                mul_128 as i64
            );
        }
    }
}
