//! Property-based testing for `liva-money` arithmetic invariants.
//!
//! Validates:
//! 1. Additive commutativity and identity.
//! 2. Zero penny drift on arbitrary split allocations.
//! 3. Deterministic Banker's rounding ratio invariants.

use liva_money::{Currency, Money};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_addition_commutativity(
        a in -1_000_000_000_000i64..1_000_000_000_000i64,
        b in -1_000_000_000_000i64..1_000_000_000_000i64,
    ) {
        let m_a = Money::vnd(a);
        let m_b = Money::vnd(b);

        let sum_ab = m_a.checked_add(m_b).unwrap();
        let sum_ba = m_b.checked_add(m_a).unwrap();

        prop_assert_eq!(sum_ab, sum_ba);
    }

    #[test]
    fn prop_identity_and_inverse(
        a in -1_000_000_000_000i64..1_000_000_000_000i64,
    ) {
        let m_a = Money::vnd(a);
        let zero = Money::zero(Currency::VND);

        // m + 0 == m
        prop_assert_eq!(m_a.checked_add(zero).unwrap(), m_a);
        // m - 0 == m
        prop_assert_eq!(m_a.checked_sub(zero).unwrap(), m_a);
        // m - m == 0
        prop_assert_eq!(m_a.checked_sub(m_a).unwrap(), zero);
    }

    #[test]
    fn prop_identity_ratio_multiplication(
        a in -1_000_000_000_000i64..1_000_000_000_000i64,
        factor in 1i64..1_000_000i64,
    ) {
        let m = Money::vnd(a);
        let scaled = m.checked_mul_ratio(factor, factor).unwrap();
        prop_assert_eq!(scaled, m);
    }

    #[test]
    fn prop_split_allocation_zero_loss(
        amount in 1i64..10_000_000_000i64,
        r1 in 1u32..100u32,
        r2 in 1u32..100u32,
        r3 in 1u32..100u32,
        r4 in 1u32..100u32,
    ) {
        let m = Money::vnd(amount);
        let ratios = [r1, r2, r3, r4];
        let parts = m.allocate(&ratios).unwrap();

        let sum: i64 = parts.iter().map(|p| p.amount()).sum();
        prop_assert_eq!(sum, amount);
    }
}
