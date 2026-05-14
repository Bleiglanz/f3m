//! [`DenseMatrix<T>`] — the concrete row-major matrix backed by `Vec<T>`.
//!
//! Implements both the inherent constructor surface (`from_row_slice`,
//! `as_slice`, …) and the [`Matrix<T>`] trait. Determinant uses Bareiss
//! integer-preserving elimination; inverse uses the classical adjugate
//! formula.

use super::{Matrix, Scalar};
use std::fmt;
use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

/// Row-major dense matrix backed by a heap-allocated `Vec<T>`.
///
/// Element `(i, j)` is stored at `data[i * ncols + j]`.
#[derive(Clone)]
pub struct DenseMatrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T: fmt::Debug> fmt::Debug for DenseMatrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DenseMatrix({}×{}) {:?}",
            self.rows, self.cols, self.data
        )
    }
}

impl<T: fmt::Display> fmt::Display for DenseMatrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.rows {
            write!(f, "[")?;
            for j in 0..self.cols {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.data[i * self.cols + j])?;
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

impl<T: PartialEq> PartialEq for DenseMatrix<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.cols == other.cols && self.data == other.data
    }
}

impl<T: Eq> Eq for DenseMatrix<T> {}

impl<T> Index<(usize, usize)> for DenseMatrix<T> {
    type Output = T;
    fn index(&self, (r, c): (usize, usize)) -> &T {
        &self.data[r * self.cols + c]
    }
}

impl<T> IndexMut<(usize, usize)> for DenseMatrix<T> {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut T {
        &mut self.data[r * self.cols + c]
    }
}

impl<T> DenseMatrix<T> {
    /// Number of rows. Unbounded over `T` so it works for the `usize`
    /// matrices produced by `c_red` / `kunz_matrix` too.
    #[must_use]
    pub const fn nrows(&self) -> usize {
        self.rows
    }

    /// Number of columns. See [`Self::nrows`].
    #[must_use]
    pub const fn ncols(&self) -> usize {
        self.cols
    }

    /// Returns a reference to the underlying flat row-major data.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Constructs a matrix by taking ownership of a row-major `Vec`.
    /// Intended for sibling submodules in the `matrix` tree that build
    /// matrices entry-by-entry into a `Vec<T>`.
    ///
    /// # Panics
    ///
    /// Panics if `data.len() != rows * cols`.
    #[must_use]
    pub(super) fn from_vec(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(
            data.len(),
            rows * cols,
            "data length must equal rows × cols"
        );
        Self { rows, cols, data }
    }
}

impl<T: Scalar> DenseMatrix<T> {
    /// Constructs a matrix from a row-major flat slice.
    ///
    /// # Panics
    ///
    /// Panics if `data.len() != rows * cols`.
    #[must_use]
    pub fn from_row_slice(rows: usize, cols: usize, data: &[T]) -> Self {
        assert_eq!(
            data.len(),
            rows * cols,
            "data length must equal rows × cols"
        );
        Self {
            rows,
            cols,
            data: data.to_vec(),
        }
    }

    // ── private helpers ───────────────────────────────────────────────────────

    /// Returns the (n−1) × (n−1) submatrix obtained by deleting `del_row` and `del_col`.
    fn minor_matrix(&self, del_row: usize, del_col: usize) -> Self {
        let n = self.rows;
        let mut data = Vec::with_capacity((n - 1) * (n - 1));
        for i in 0..n {
            if i == del_row {
                continue;
            }
            for j in 0..n {
                if j == del_col {
                    continue;
                }
                data.push(self.data[i * n + j]);
            }
        }
        Self {
            rows: n - 1,
            cols: n - 1,
            data,
        }
    }

    /// Cofactor C(i, j) = (−1)^(i+j) · det(minor(i, j)).
    fn cofactor(&self, i: usize, j: usize) -> T {
        let d = self.minor_matrix(i, j).det();
        if (i + j).is_multiple_of(2) { d } else { -d }
    }
}

impl<T: Scalar> Matrix<T> for DenseMatrix<T> {
    fn zero(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![T::zero(); rows * cols],
        }
    }

    fn identity(n: usize) -> Self {
        let mut m = Self::zero(n, n);
        for i in 0..n {
            m.data[i * n + i] = T::one();
        }
        m
    }

    fn nrows(&self) -> usize {
        self.rows
    }
    fn ncols(&self) -> usize {
        self.cols
    }

    fn get(&self, row: usize, col: usize) -> T {
        assert!(
            row < self.rows && col < self.cols,
            "index ({row},{col}) out of bounds"
        );
        self.data[row * self.cols + col]
    }

    fn set(&mut self, row: usize, col: usize, value: T) {
        assert!(
            row < self.rows && col < self.cols,
            "index ({row},{col}) out of bounds"
        );
        self.data[row * self.cols + col] = value;
    }

    fn mat_add(&self, other: &Self) -> Self {
        assert_eq!(
            (self.rows, self.cols),
            (other.rows, other.cols),
            "shape mismatch in mat_add"
        );
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(&a, &b)| a + b)
                .collect(),
        }
    }

    fn mat_sub(&self, other: &Self) -> Self {
        assert_eq!(
            (self.rows, self.cols),
            (other.rows, other.cols),
            "shape mismatch in mat_sub"
        );
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(&a, &b)| a - b)
                .collect(),
        }
    }

    fn mat_mul(&self, other: &Self) -> Self {
        assert_eq!(
            self.cols, other.rows,
            "shape mismatch in mat_mul: {}×{} · {}×{}",
            self.rows, self.cols, other.rows, other.cols
        );
        let mut out = Self::zero(self.rows, other.cols);
        for i in 0..self.rows {
            for k in 0..self.cols {
                let a = self.data[i * self.cols + k];
                if a == T::zero() {
                    continue;
                }
                for j in 0..other.cols {
                    let idx = i * other.cols + j;
                    out.data[idx] = out.data[idx] + a * other.data[k * other.cols + j];
                }
            }
        }
        out
    }

    fn scalar_mul(&self, s: T) -> Self {
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|&x| x * s).collect(),
        }
    }

    fn transpose(&self) -> Self {
        let mut out = Self::zero(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                out.data[j * self.rows + i] = self.data[i * self.cols + j];
            }
        }
        out
    }

    /// Bareiss integer-preserving elimination.
    ///
    /// At each step k the inner update is:
    /// `m[i][j] = (m[k][k] · m[i][j] − m[i][k] · m[k][j]) / prev`
    /// where `prev` is the (k−1)-th pivot (1 for k = 0).
    /// The Bareiss theorem guarantees this division is always exact.
    fn det(&self) -> T {
        assert_eq!(self.rows, self.cols, "det requires a square matrix");
        let n = self.rows;
        match n {
            0 => return T::one(),
            1 => return self.data[0],
            _ => {}
        }
        let mut m = self.data.clone();
        let mut sign = T::one();
        let mut prev = T::one();

        for k in 0..n - 1 {
            // Partial pivot: find first non-zero element in column k at or below row k.
            let pivot = (k..n).find(|&i| m[i * n + k] != T::zero());
            let Some(pivot) = pivot else {
                return T::zero();
            };
            if pivot != k {
                for j in 0..n {
                    m.swap(k * n + j, pivot * n + j);
                }
                sign = -sign;
            }
            for i in k + 1..n {
                for j in k + 1..n {
                    // This division is exact (Bareiss theorem).
                    m[i * n + j] =
                        (m[k * n + k] * m[i * n + j] - m[i * n + k] * m[k * n + j]) / prev;
                }
                m[i * n + k] = T::zero();
            }
            prev = m[k * n + k];
        }
        sign * m[(n - 1) * n + (n - 1)]
    }

    /// Adjugate formula: A⁻¹ = adj(A) / det(A).
    ///
    /// Returns `None` for singular matrices.  For integer scalars the per-entry
    /// division `cofactor / det` must be exact; this holds if and only if
    /// `det = ±1` (unimodular matrices).
    fn inverse(&self) -> Option<Self> {
        assert_eq!(self.rows, self.cols, "inverse requires a square matrix");
        let n = self.rows;
        let d = self.det();
        if d == T::zero() {
            return None;
        }
        if n == 1 {
            // 1×1: check exact divisibility (for integer types, only d=±1 is exact)
            if !T::one().is_divisible_by(d) {
                return None;
            }
            return Some(Self::from_row_slice(1, 1, &[T::one() / d]));
        }
        // adj[j][i] = cofactor(i, j)  — note the transposition.
        let mut adj = vec![T::zero(); n * n];
        for i in 0..n {
            for j in 0..n {
                adj[j * n + i] = self.cofactor(i, j);
            }
        }
        // Each cofactor must be exactly divisible by det; for integer types this
        // guards against non-unimodular matrices, for floats it is always true.
        if adj.iter().any(|&x| !x.is_divisible_by(d)) {
            return None;
        }
        Some(Self {
            rows: n,
            cols: n,
            data: adj.into_iter().map(|x| x / d).collect(),
        })
    }
}

// ── std::ops ──────────────────────────────────────────────────────────────────

impl<T: Scalar> Add for DenseMatrix<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Matrix::mat_add(&self, &rhs)
    }
}

impl<T: Scalar> Sub for DenseMatrix<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Matrix::mat_sub(&self, &rhs)
    }
}

impl<T: Scalar> Mul for DenseMatrix<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Matrix::mat_mul(&self, &rhs)
    }
}

impl<T: Scalar> Mul<T> for DenseMatrix<T> {
    type Output = Self;
    fn mul(self, s: T) -> Self {
        self.scalar_mul(s)
    }
}

impl<T: Scalar> Neg for DenseMatrix<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|&x| -x).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::matrix::Matrix;

    fn mat_i(rows: usize, cols: usize, data: &[i64]) -> DenseMatrix<i64> {
        DenseMatrix::from_row_slice(rows, cols, data)
    }
    fn mat_f(rows: usize, cols: usize, data: &[f64]) -> DenseMatrix<f64> {
        DenseMatrix::from_row_slice(rows, cols, data)
    }
    // ── construction ─────────────────────────────────────────────────────────

    #[test]
    fn zero_matrix_is_all_zero() {
        let z: DenseMatrix<i64> = DenseMatrix::zero(3, 4);
        assert_eq!(z.nrows(), 3);
        assert_eq!(z.ncols(), 4);
        assert!(z.as_slice().iter().all(|&x| x == 0));
    }

    #[test]
    fn identity_matrix_shape_and_values() {
        let i3: DenseMatrix<i64> = DenseMatrix::identity(3);
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(i3.get(r, c), i64::from(r == c));
            }
        }
    }

    #[test]
    fn from_row_slice_round_trips() {
        let data = [1i64, 2, 3, 4, 5, 6];
        let m = mat_i(2, 3, &data);
        assert_eq!(m.as_slice(), &data);
    }

    // ── det — 1×1 / 2×2 / 3×3 / known cases ─────────────────────────────────

    #[test]
    fn det_1x1() {
        assert_eq!(mat_i(1, 1, &[7]).det(), 7);
        assert_eq!(mat_i(1, 1, &[-3]).det(), -3);
    }

    #[test]
    fn det_2x2() {
        // |1 2|   = 1·4 − 2·3 = -2
        // |3 4|
        assert_eq!(mat_i(2, 2, &[1, 2, 3, 4]).det(), -2);
        // identity
        assert_eq!(DenseMatrix::<i64>::identity(2).det(), 1);
        // zero row → det = 0
        assert_eq!(mat_i(2, 2, &[0, 0, 1, 2]).det(), 0);
    }

    #[test]
    fn det_3x3() {
        // Sarrus: [1,2,3;4,5,6;7,8,10] = 1(50-48) - 2(40-42) + 3(32-35) = 2+4-9 = -3
        assert_eq!(mat_i(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 10]).det(), -3);
        assert_eq!(DenseMatrix::<i64>::identity(3).det(), 1);
        // All-zero → 0
        assert_eq!(DenseMatrix::<i64>::zero(3, 3).det(), 0);
    }

    #[test]
    fn det_4x4_identity() {
        assert_eq!(DenseMatrix::<i64>::identity(4).det(), 1);
    }

    #[test]
    fn det_permutation_matrix_sign() {
        // Swap rows 0 and 1 of 3×3 identity → det = -1
        let m = mat_i(3, 3, &[0, 1, 0, 1, 0, 0, 0, 0, 1]);
        assert_eq!(m.det(), -1);
    }

    #[test]
    fn det_upper_triangular() {
        // det = product of diagonal = 1*2*3*4 = 24
        let m = mat_i(4, 4, &[1, 5, 6, 7, 0, 2, 8, 9, 0, 0, 3, 10, 0, 0, 0, 4]);
        assert_eq!(m.det(), 24);
    }

    #[test]
    fn det_scalar_multiple() {
        // det(kA) = k^n det(A)  — check for 2×2, k=3
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let ka = a.scalar_mul(3);
        assert_eq!(ka.det(), 9 * a.det());
    }

    #[test]
    fn det_transpose_equals_det() {
        let a = mat_i(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 10]);
        assert_eq!(a.det(), a.transpose().det());
    }

    #[test]
    fn det_product_equals_product_of_dets() {
        let a = mat_i(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 10]);
        let b = mat_i(3, 3, &[2, 0, 1, 0, 3, 1, 1, 0, 4]);
        let ab = a.mat_mul(&b);
        assert_eq!(ab.det(), a.det() * b.det());
    }

    // ── inverse ───────────────────────────────────────────────────────────────

    #[test]
    fn inverse_1x1() {
        let m = mat_f(1, 1, &[4.0]);
        let inv = m.inverse().unwrap();
        assert!((inv.get(0, 0) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn inverse_singular_returns_none() {
        assert!(mat_i(2, 2, &[1, 2, 2, 4]).inverse().is_none());
        assert!(
            mat_i(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9])
                .inverse()
                .is_none()
        );
        assert!(DenseMatrix::<i64>::zero(3, 3).inverse().is_none());
    }

    #[test]
    fn inverse_identity_is_identity() {
        let i3 = DenseMatrix::<i64>::identity(3);
        assert_eq!(i3.inverse().unwrap(), i3);
    }

    #[test]
    fn inverse_2x2_integer_unimodular() {
        // det([[2,1],[1,1]]) = 1 — unimodular, inverse is [[1,-1],[-1,2]]
        let a = mat_i(2, 2, &[2, 1, 1, 1]);
        let inv = a.inverse().unwrap();
        assert_eq!(inv, mat_i(2, 2, &[1, -1, -1, 2]));
        // A * A^{-1} = I
        assert_eq!(a.mat_mul(&inv), DenseMatrix::identity(2));
    }

    #[test]
    fn inverse_non_unimodular_integer_returns_none() {
        // det = 2 ≠ ±1
        let a = mat_i(2, 2, &[3, 1, 1, 1]);
        assert!(a.inverse().is_none());
    }

    #[test]
    fn inverse_2x2_float() {
        let a = mat_f(2, 2, &[3.0, 1.0, 1.0, 1.0]);
        let inv = a.inverse().unwrap();
        let product = a.mat_mul(&inv);
        let id = DenseMatrix::<f64>::identity(2);
        for i in 0..2 {
            for j in 0..2 {
                assert!((product.get(i, j) - id.get(i, j)).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn inverse_3x3_float() {
        // A known invertible matrix
        let a = mat_f(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0]);
        let inv = a.inverse().unwrap();
        let product = a.mat_mul(&inv);
        let id = DenseMatrix::<f64>::identity(3);
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (product.get(i, j) - id.get(i, j)).abs() < 1e-10,
                    "A·A⁻¹[{i}][{j}] off: got {}",
                    product.get(i, j)
                );
            }
        }
    }

    // ── arithmetic ────────────────────────────────────────────────────────────

    #[test]
    fn add_zero_is_identity() {
        let a = mat_i(2, 3, &[1, 2, 3, 4, 5, 6]);
        let z = DenseMatrix::zero(2, 3);
        assert_eq!(a.mat_add(&z), a);
        assert_eq!(z.mat_add(&a), a);
    }

    #[test]
    fn add_commutative() {
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let b = mat_i(2, 2, &[5, 6, 7, 8]);
        assert_eq!(a.mat_add(&b), b.mat_add(&a));
    }

    #[test]
    fn sub_self_is_zero() {
        let a = mat_i(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(a.mat_sub(&a), DenseMatrix::zero(3, 3));
    }

    #[test]
    fn mul_by_identity_is_self() {
        let a = mat_i(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let i3 = DenseMatrix::identity(3);
        assert_eq!(a.mat_mul(&i3), a);
        assert_eq!(i3.mat_mul(&a), a);
    }

    #[test]
    fn mul_by_zero_is_zero() {
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let z = DenseMatrix::zero(2, 2);
        assert_eq!(a.mat_mul(&z), z);
        assert_eq!(z.mat_mul(&a), z);
    }

    #[test]
    fn mul_known_2x2() {
        // [1 2] [5 6]   [1*5+2*7  1*6+2*8]   [19 22]
        // [3 4] [7 8] = [3*5+4*7  3*6+4*8] = [43 50]
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let b = mat_i(2, 2, &[5, 6, 7, 8]);
        assert_eq!(a.mat_mul(&b), mat_i(2, 2, &[19, 22, 43, 50]));
    }

    #[test]
    fn scalar_mul_distributes_over_add() {
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let b = mat_i(2, 2, &[5, 6, 7, 8]);
        let k = 3i64;
        let lhs = a.mat_add(&b).scalar_mul(k);
        let rhs = a.scalar_mul(k).mat_add(&b.scalar_mul(k));
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn scalar_mul_zero_scalar_gives_zero() {
        let a = mat_i(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(a.scalar_mul(0), DenseMatrix::zero(3, 3));
    }

    #[test]
    fn neg_double_is_identity() {
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let neg_neg_a = -(-a.clone());
        assert_eq!(neg_neg_a, a);
    }

    #[test]
    fn ops_overloads_agree_with_trait_methods() {
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let b = mat_i(2, 2, &[5, 6, 7, 8]);
        assert_eq!(a.clone() + b.clone(), a.mat_add(&b));
        assert_eq!(a.clone() - b.clone(), a.mat_sub(&b));
        assert_eq!(a.clone() * b.clone(), a.mat_mul(&b));
        assert_eq!(a.clone() * 3i64, a.scalar_mul(3));
    }

    // ── transpose ─────────────────────────────────────────────────────────────

    #[test]
    fn transpose_involution() {
        let a = mat_i(2, 3, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(a.transpose().transpose(), a);
    }

    #[test]
    fn transpose_shape() {
        let a = mat_i(2, 3, &[1, 2, 3, 4, 5, 6]);
        let t = a.transpose();
        assert_eq!((t.nrows(), t.ncols()), (3, 2));
    }

    #[test]
    fn transpose_add_commutes() {
        let a = mat_i(2, 2, &[1, 2, 3, 4]);
        let b = mat_i(2, 2, &[5, 6, 7, 8]);
        assert_eq!(
            (a.clone() + b.clone()).transpose(),
            a.transpose() + b.transpose()
        );
    }

    #[test]
    fn transpose_mul_reverses_order() {
        let a = mat_i(2, 3, &[1, 2, 3, 4, 5, 6]);
        let b = mat_i(3, 2, &[7, 8, 9, 10, 11, 12]);
        assert_eq!(
            (a.clone() * b.clone()).transpose(),
            b.transpose() * a.transpose()
        );
    }

    // ── index operator ────────────────────────────────────────────────────────

    #[test]
    fn index_operator_read_write() {
        let mut m = DenseMatrix::<i64>::zero(3, 3);
        m[(1, 2)] = 42;
        assert_eq!(m[(1, 2)], 42);
        assert_eq!(m.get(1, 2), 42);
    }

    // ── property-based tests (proptest) ───────────────────────────────────────

    proptest::proptest! {
        // Generate 2×2 f64 matrices with entries in [-5, 5] to avoid overflow.
        #[test]
        fn prop_add_commutative(
            a in proptest::collection::vec(-5.0f64..=5.0, 4),
            b in proptest::collection::vec(-5.0f64..=5.0, 4),
        ) {
            let ma = mat_f(2, 2, &a);
            let mb = mat_f(2, 2, &b);
            let ab = ma.mat_add(&mb);
            let ba = mb.mat_add(&ma);
            for k in 0..4 {
                proptest::prop_assert!((ab.data[k] - ba.data[k]).abs() < 1e-12);
            }
        }

        #[test]
        fn prop_add_associative(
            a in proptest::collection::vec(-5.0f64..=5.0, 4),
            b in proptest::collection::vec(-5.0f64..=5.0, 4),
            c in proptest::collection::vec(-5.0f64..=5.0, 4),
        ) {
            let ma = mat_f(2, 2, &a);
            let mb = mat_f(2, 2, &b);
            let mc = mat_f(2, 2, &c);
            let lhs = (ma.clone() + mb.clone()) + mc.clone();
            let rhs = ma + (mb + mc);
            for k in 0..4 {
                proptest::prop_assert!((lhs.data[k] - rhs.data[k]).abs() < 1e-9);
            }
        }

        #[test]
        fn prop_mul_associative_3x3(
            a in proptest::collection::vec(-3.0f64..=3.0, 9),
            b in proptest::collection::vec(-3.0f64..=3.0, 9),
            c in proptest::collection::vec(-3.0f64..=3.0, 9),
        ) {
            let ma = mat_f(3, 3, &a);
            let mb = mat_f(3, 3, &b);
            let mc = mat_f(3, 3, &c);
            let lhs = (ma.clone() * mb.clone()) * mc.clone();
            let rhs = ma * (mb * mc);
            // Higher tolerance due to 3×3 float accumulation.
            for k in 0..9 {
                proptest::prop_assert!((lhs.data[k] - rhs.data[k]).abs() < 1e-6,
                    "lhs[{k}]={} rhs[{k}]={}", lhs.data[k], rhs.data[k]);
            }
        }

        #[test]
        fn prop_distributivity(
            a in proptest::collection::vec(-5.0f64..=5.0, 4),
            b in proptest::collection::vec(-5.0f64..=5.0, 4),
            c in proptest::collection::vec(-5.0f64..=5.0, 4),
        ) {
            let ma = mat_f(2, 2, &a);
            let mb = mat_f(2, 2, &b);
            let mc = mat_f(2, 2, &c);
            let lhs = ma.clone() * (mb.clone() + mc.clone());
            let rhs = ma.clone() * mb + ma * mc;
            for k in 0..4 {
                proptest::prop_assert!((lhs.data[k] - rhs.data[k]).abs() < 1e-9);
            }
        }

        #[test]
        fn prop_det_product_law(
            a in proptest::collection::vec(-3.0f64..=3.0, 9),
            b in proptest::collection::vec(-3.0f64..=3.0, 9),
        ) {
            let ma = mat_f(3, 3, &a);
            let mb = mat_f(3, 3, &b);
            let det_ab = (ma.clone() * mb.clone()).det();
            let det_a_times_det_b = ma.det() * mb.det();
            proptest::prop_assert!((det_ab - det_a_times_det_b).abs() < 1e-6,
                "det(AB)={det_ab} det(A)det(B)={det_a_times_det_b}");
        }

        #[test]
        fn prop_det_transpose(
            a in proptest::collection::vec(-5.0f64..=5.0, 9),
        ) {
            let ma = mat_f(3, 3, &a);
            let d = ma.det();
            let dt = ma.transpose().det();
            proptest::prop_assert!((d - dt).abs() < 1e-10,
                "det(A)={d} det(Aᵀ)={dt}");
        }

        #[test]
        fn prop_transpose_involution(
            a in proptest::collection::vec(-5.0f64..=5.0, 6),
        ) {
            let ma = mat_f(2, 3, &a);
            proptest::prop_assert_eq!(ma.transpose().transpose(), ma);
        }

        #[test]
        fn prop_scalar_mul_distributes_over_add(
            a in proptest::collection::vec(-5.0f64..=5.0, 4),
            b in proptest::collection::vec(-5.0f64..=5.0, 4),
            s in -5.0f64..=5.0,
        ) {
            let ma = mat_f(2, 2, &a);
            let mb = mat_f(2, 2, &b);
            let lhs = (ma.clone() + mb.clone()).scalar_mul(s);
            let rhs = ma.scalar_mul(s) + mb.scalar_mul(s);
            for k in 0..4 {
                proptest::prop_assert!((lhs.data[k] - rhs.data[k]).abs() < 1e-10);
            }
        }

        #[test]
        fn prop_inverse_times_self_is_identity(
            a in proptest::collection::vec(-3.0f64..=3.0, 4),
        ) {
            let ma = mat_f(2, 2, &a);
            if let Some(inv) = ma.inverse() {
                let product = ma.clone() * inv.clone();
                let id = DenseMatrix::<f64>::identity(2);
                for i in 0..2 {
                    for j in 0..2 {
                        proptest::prop_assert!((product.get(i,j) - id.get(i,j)).abs() < 1e-9,
                            "A·A⁻¹[{i},{j}]={} expected {}", product.get(i,j), id.get(i,j));
                    }
                }
                // And A⁻¹ · A = I
                let product2 = inv * ma;
                for i in 0..2 {
                    for j in 0..2 {
                        proptest::prop_assert!((product2.get(i,j) - id.get(i,j)).abs() < 1e-9);
                    }
                }
            }
        }
    }
}
