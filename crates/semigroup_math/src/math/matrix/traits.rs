//! [`Matrix<T>`] — the interface every dense matrix type implements.

use super::Scalar;
use std::fmt;

/// Interface for dense matrices over a scalar type `T`.
pub trait Matrix<T: Scalar>: Sized + Clone + PartialEq + fmt::Debug {
    /// All-zero matrix of shape `rows × cols`.
    fn zero(rows: usize, cols: usize) -> Self;

    /// n × n identity matrix.
    fn identity(n: usize) -> Self;

    /// Number of rows.
    fn nrows(&self) -> usize;

    /// Number of columns.
    fn ncols(&self) -> usize;

    /// Returns element at (`row`, `col`).
    ///
    /// # Panics
    ///
    /// Panics if `row >= nrows()` or `col >= ncols()`.
    fn get(&self, row: usize, col: usize) -> T;

    /// Sets element at (`row`, `col`) to `value`.
    ///
    /// # Panics
    ///
    /// Panics if `row >= nrows()` or `col >= ncols()`.
    fn set(&mut self, row: usize, col: usize, value: T);

    /// Element-wise sum `self + other`.
    ///
    /// # Panics
    ///
    /// Panics if shapes differ.
    #[must_use]
    fn mat_add(&self, other: &Self) -> Self;

    /// Element-wise difference `self - other`.
    ///
    /// # Panics
    ///
    /// Panics if shapes differ.
    #[must_use]
    fn mat_sub(&self, other: &Self) -> Self;

    /// Matrix product `self × other`.
    ///
    /// # Panics
    ///
    /// Panics if `self.ncols() != other.nrows()`.
    #[must_use]
    fn mat_mul(&self, other: &Self) -> Self;

    /// Scalar multiplication `s · self`.
    #[must_use]
    fn scalar_mul(&self, s: T) -> Self;

    /// Transpose of `self`.
    #[must_use]
    fn transpose(&self) -> Self;

    /// Determinant of a square matrix, computed via the Bareiss algorithm.
    ///
    /// # Panics
    ///
    /// Panics if the matrix is not square.
    fn det(&self) -> T;

    /// Inverse of a square matrix, or `None` if singular.
    ///
    /// For integer scalars, `Some` is returned only when `det(self) = ±1`.
    ///
    /// # Panics
    ///
    /// Panics if the matrix is not square.
    fn inverse(&self) -> Option<Self>;
}
