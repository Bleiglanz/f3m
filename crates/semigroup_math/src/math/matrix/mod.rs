//! Dense matrix algebra over any [`Scalar`] type.
//!
//! ## Design
//!
//! - [`Scalar`] — bound that scalar types must satisfy (arithmetic, zero, one, abs).
//! - [`Matrix<T>`] — trait defining the full matrix interface.
//! - [`DenseMatrix<T>`] — concrete row-major implementation backed by `Vec<T>`.
//!
//! [`Scalar`] is implemented for `i32`, `i64`, `i128`, `f32`, and `f64`.
//!
//! ## Algorithms
//!
//! **Determinant** — Bareiss integer-preserving elimination (O(n³)).
//! Every intermediate division is exact, so `i32`/`i64`/`i128` matrices produce
//! correct integer determinants with no floating-point error.
//!
//! **Inverse** — classical adjugate formula: A⁻¹ = adj(A) / det(A).
//! Each entry of adj(A) is a cofactor computed via the Bareiss determinant of the
//! corresponding (n−1)×(n−1) minor.  For integer scalars the per-entry division
//! is exact only when the matrix is unimodular (det = ±1); for all other integer
//! matrices `inverse` returns `None`.  For floating-point scalars the division is
//! always performed and the result is numerically close to the true inverse.
//!
//! ## Layout
//!
//! - [`scalar`] — the `Scalar` trait + impls for `iN` / `fN`.
//! - [`traits`] — the `Matrix<T>` trait.
//! - [`dense`] — `DenseMatrix<T>` struct + impls.
//! - [`semigroup_matrices`] — the semigroup-flavoured constructors (U(m),
//!   V(m), L(m), `c_red`, `u_pair_relations`, `zd_vector`, D(m), …).

pub mod dense;
pub mod scalar;
pub mod semigroup_matrices;
pub mod traits;

pub use dense::DenseMatrix;
pub use scalar::Scalar;
pub use semigroup_matrices::{
    c_red, d_matrix, kunz_matrix, l_matrix, to_i64, u_matrix, u_pair_relations, u_times_c_red,
    v_matrix, zd_vector,
};
pub use traits::Matrix;
