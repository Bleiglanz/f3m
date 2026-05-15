//! Pure-math tests that do not exercise the [`Semigroup`] type:
//! `gcd`, `binom`. Semigroup-bearing tests live in `semigroup_tests.rs`.
//!
//! [`Semigroup`]: semigroup_math::math::Semigroup

use semigroup_math::math::{binom, gcd};

#[test]
fn test_gcd() {
    assert_eq!(gcd(12, 8), 4);
    assert_eq!(gcd(7, 3), 1);
    assert_eq!(gcd(0, 5), 5);
    assert_eq!(gcd(9, 0), 9);
}

#[test]
fn test_binom_basics() {
    // Edge cases: b > a, b = 0, b = a.
    assert_eq!(binom(3, 5), Some(0));
    assert_eq!(binom(0, 0), Some(1));
    assert_eq!(binom(7, 0), Some(1));
    assert_eq!(binom(7, 7), Some(1));

    // Known values.
    assert_eq!(binom(5, 2), Some(10));
    assert_eq!(binom(10, 3), Some(120));
    assert_eq!(binom(20, 10), Some(184_756));

    // Symmetry: C(a, b) = C(a, a − b).
    for a in 0..=20 {
        for b in 0..=a {
            assert_eq!(binom(a, b), binom(a, a - b), "symmetry for ({a}, {b})");
        }
    }

    // Pascal's identity: C(n, k) = C(n−1, k−1) + C(n−1, k) for 1 ≤ k ≤ n−1.
    for n in 2..=20 {
        for k in 1..n {
            let lhs = binom(n, k).expect("no overflow at n ≤ 20");
            let a = binom(n - 1, k - 1).expect("no overflow");
            let b = binom(n - 1, k).expect("no overflow");
            assert_eq!(lhs, a + b, "Pascal at ({n}, {k})");
        }
    }
}

#[test]
fn test_binom_overflow() {
    // On 64-bit hosts: C(62, 31) ≈ 4.65·10^17 still fits in u64 = usize;
    // C(70, 35) ≈ 1.12·10^20 overflows u64. We pick a value safely above
    // u64::MAX so the test is meaningful on both 32- and 64-bit usize.
    assert!(
        binom(200, 100).is_none(),
        "C(200,100) ≈ 9·10^58 must overflow",
    );
    // And a value that fits even on 32-bit usize (C(20,10) = 184_756).
    assert_eq!(binom(20, 10), Some(184_756));
}
