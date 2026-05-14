//! [`Scalar`] — the arithmetic bound matrix entries must satisfy.
//!
//! Implementations are provided for `isize`, `i32`, `i64`, `i128`, `f32`, and
//! `f64` via the two macros at the bottom.

use num_traits::{One, Zero};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

/// Arithmetic bound for types that can serve as matrix entries.
///
/// Requires: `Copy`, all four arithmetic operators, negation, additive [`Zero`],
/// multiplicative [`One`], `abs`, and comparison/formatting.
pub trait Scalar:
    Copy
    + PartialEq
    + PartialOrd
    + fmt::Debug
    + fmt::Display
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + Neg<Output = Self>
    + Zero
    + One
{
    /// Returns the absolute value of `self`.
    #[must_use]
    fn abs(self) -> Self;

    /// Returns `true` if `self` is exactly divisible by `d`.
    ///
    /// For integer types this is `self % d == 0`; for floating-point types
    /// division is always considered exact, so this returns `true`.
    #[must_use]
    fn is_divisible_by(self, d: Self) -> bool;
}

macro_rules! impl_scalar_int {
    ($($t:ty),+) => {
        $(impl Scalar for $t {
            fn abs(self) -> Self { <$t>::abs(self) }
            fn is_divisible_by(self, d: Self) -> bool { self % d == 0 }
        })+
    };
}
macro_rules! impl_scalar_float {
    ($($t:ty),+) => {
        $(impl Scalar for $t {
            fn abs(self) -> Self { <$t>::abs(self) }
            // Float division is always treated as exact; callers use approximate equality.
            fn is_divisible_by(self, _d: Self) -> bool { true }
        })+
    };
}
impl_scalar_int!(isize, i32, i64, i128);
impl_scalar_float!(f32, f64);
