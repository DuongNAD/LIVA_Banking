//! Adversarial stress test suite for `liva-money` (Milestone M1).
//!
//! Evaluates extreme boundary conditions:
//! 1. `i64::MIN` edge cases (checked arithmetic, allocate, display formatting).
//! 2. `i64::MAX` boundary allocation with 12 split ratios and zero penny drift.
//! 3. Zero denominator rejection across full integer range.
//! 4. Negative split sums and sign preservation across complex ratio vectors.
//! 5. Zero float drift and Banker's Rounding half-to-even proofs under stress.

use liva_money::{Currency, Money, MoneyError};

#[test]
fn test_i64_min_checked_arithmetic_boundaries() {
    let min_money = Money::from_minor(i64::MIN, Currency::VND);

    // 1. Basic properties
    assert_eq!(min_money.amount(), i64::MIN);
    assert!(min_money.is_negative());
    assert!(!min_money.is_positive());
    assert!(!min_money.is_zero());

    // 2. abs() must return Overflow error, NOT panic
    assert_eq!(min_money.abs(), Err(MoneyError::Overflow));

    // 3. Checked arithmetic bounds
    assert_eq!(min_money.checked_add(Money::vnd(0)).unwrap(), min_money);
    assert_eq!(min_money.checked_sub(Money::vnd(0)).unwrap(), min_money);
    assert_eq!(min_money.checked_add(Money::vnd(-1)), Err(MoneyError::Overflow));
    assert_eq!(min_money.checked_sub(Money::vnd(1)), Err(MoneyError::Overflow));
    assert_eq!(min_money.checked_mul(-1), Err(MoneyError::Overflow));
    assert_eq!(min_money.checked_mul(1).unwrap(), min_money);
    assert_eq!(min_money.checked_mul(0).unwrap(), Money::zero(Currency::VND));

    // 4. checked_mul_ratio on i64::MIN
    assert_eq!(min_money.checked_mul_ratio(1, 1).unwrap(), min_money);
    assert_eq!(min_money.checked_mul_ratio(-1, 1), Err(MoneyError::Overflow));
    assert_eq!(min_money.checked_mul_ratio(1, 0), Err(MoneyError::DivisionByZero));

    // 5. allocate() on i64::MIN must return Overflow error, NOT panic
    assert_eq!(min_money.allocate(&[1, 2, 3]), Err(MoneyError::Overflow));
}

/// EMPIRICAL BUG TEST:
/// Verifies that to_vietnamese_display() does not panic on i64::MIN.
/// Expected: formats as "-9.223.372.036.854.775.808 VND".
/// Actual (Bug): Panics with "attempt to negate with overflow" at `self.amount.abs()`.
#[test]
fn test_i64_min_display_formatting_no_panic() {
    let min_money = Money::from_minor(i64::MIN, Currency::VND);
    let formatted = min_money.to_vietnamese_display();
    assert_eq!(formatted, "-9.223.372.036.854.775.808 VND");
}

#[test]
fn test_i64_max_extreme_allocation_12_ratios() {
    let max_money = Money::from_minor(i64::MAX, Currency::VND);

    // 12 split ratios (increasing)
    let ratios_12 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let parts = max_money.allocate(&ratios_12).expect("allocate on i64::MAX should succeed");

    assert_eq!(parts.len(), 12);
    let sum: i64 = parts.iter().map(|p| p.amount()).sum();
    assert_eq!(sum, i64::MAX, "Zero penny drift failed on i64::MAX with 12 ratios!");
    assert!(parts.iter().all(|p| p.is_positive()), "All parts of i64::MAX allocation must be positive");

    // 12 extreme ratios with zeros and huge weights
    let extreme_ratios = [
        u32::MAX / 4, 1, 0, 100, u32::MAX / 2, 7, 0, 999, 12345, 0, 42, 1
    ];
    let parts_extreme = max_money.allocate(&extreme_ratios).expect("extreme ratio allocation should succeed");
    assert_eq!(parts_extreme.len(), 12);
    let sum_extreme: i64 = parts_extreme.iter().map(|p| p.amount()).sum();
    assert_eq!(sum_extreme, i64::MAX, "Zero penny drift failed on extreme 12 ratios!");

    // Ratio 0 must yield exactly 0
    assert_eq!(parts_extreme[2].amount(), 0);
    assert_eq!(parts_extreme[6].amount(), 0);
    assert_eq!(parts_extreme[9].amount(), 0);
}

#[test]
fn test_zero_denominator_rejection_exhaustive() {
    let test_amounts = [
        0, 1, -1, 100, -100, 1_000_000_007, -1_000_000_007,
        i64::MAX, i64::MIN, i64::MAX - 1, i64::MIN + 1,
    ];
    let numerators = [0, 1, -1, 10, -10, i64::MAX, i64::MIN];

    for &amt in &test_amounts {
        let m = Money::vnd(amt);
        for &num in &numerators {
            assert_eq!(
                m.checked_mul_ratio(num, 0),
                Err(MoneyError::DivisionByZero),
                "checked_mul_ratio({num}, 0) failed to return DivisionByZero for amount {amt}"
            );
        }
    }
}

#[test]
fn test_negative_split_sums_and_sign_preservation() {
    let negative_amounts = [
        -1i64,
        -100i64,
        -1_000_000_007i64,
        -999_999_999_999_999i64,
        -i64::MAX,
        -i64::MAX + 1,
    ];

    let ratios_matrix: Vec<Vec<u32>> = vec![
        vec![1, 1],
        vec![1, 1, 1],
        vec![1, 2, 3],
        vec![3, 5, 7, 11],
        vec![1, 0, 1],
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    ];

    for &amt in &negative_amounts {
        let m = Money::vnd(amt);
        for ratios in &ratios_matrix {
            let parts = m.allocate(ratios).unwrap();

            // 1. Length
            assert_eq!(parts.len(), ratios.len());

            // 2. Zero penny drift: exact sum matches negative amount
            let sum: i64 = parts.iter().map(|p| p.amount()).sum();
            assert_eq!(
                sum, amt,
                "Negative split drift: expected {amt}, got {sum} for ratios {ratios:?}"
            );

            // 3. Strict sign preservation: every part must be <= 0
            for (idx, p) in parts.iter().enumerate() {
                if ratios[idx] == 0 {
                    assert_eq!(p.amount(), 0);
                } else {
                    assert!(
                        p.amount() <= 0,
                        "Negative amount {amt} split yielded positive part {} at ratio {}",
                        p.amount(), ratios[idx]
                    );
                }
            }
        }
    }
}

#[test]
fn test_bankers_rounding_half_to_even_stress_and_drift() {
    // Tests half-to-even behavior across positive and negative halfway points
    for k in -10_000..=10_000i64 {
        // Odd numbers divided by 2 yield exactly k + 0.5
        let odd = 2 * k + 1;
        let m = Money::vnd(odd);
        let rounded = m.checked_mul_ratio(1, 2).unwrap().amount();

        // Round-Half-to-Even requires rounded to be strictly even
        assert_eq!(
            rounded % 2,
            0,
            "Banker's rounding failed: {odd} * 1/2 produced odd result {rounded}"
        );

        // Distance to unrounded value must be 0.5 (scaled by 2: |2 * rounded - odd| == 1)
        assert_eq!(
            (2 * rounded - odd).abs(),
            1,
            "Rounding error exceeded 0.5 unit for {odd}"
        );
    }
}
