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

use num_traits::{One, Zero};
use std::fmt;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Rem, Sub};

// ── Scalar bound ──────────────────────────────────────────────────────────────

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

// ── Matrix trait ──────────────────────────────────────────────────────────────

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

// ── DenseMatrix ───────────────────────────────────────────────────────────────

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

// ── DenseMatrix public constructors / helpers ─────────────────────────────────

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

    /// Returns a reference to the underlying flat row-major data.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.data
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

// ── Matrix<T> impl ────────────────────────────────────────────────────────────

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

// ── U(m) matrix ───────────────────────────────────────────────────────────────

/// Constructs the (m−1) × (m−1) matrix U(m) with `i64` entries.
///
/// Internal indices `(a, b)` correspond to semigroup indices `(a+1, b+1)`,
/// so the i = 0 row and column are dropped. Entries follow:
///
/// ```text
/// U(m)[a][b] = a + 1 − m   if b < a
///            = a + 1        if b ≥ a
/// ```
///
/// Example for m = 5 (4 × 4 matrix; rows/cols labelled 1..4 externally):
/// ```text
/// a=0 (k=1): [  1,  1,  1,  1]
/// a=1 (k=2): [ -3,  2,  2,  2]
/// a=2 (k=3): [ -2, -2,  3,  3]
/// a=3 (k=4): [ -1, -1, -1,  4]
/// ```
///
/// `det(U(m)) = m^(m−2)` — unchanged from the m × m unit-bordered form,
/// since the border contributes a factor of 1.
///
/// Uses `i64` rather than `isize` so that the Bareiss determinant does
/// not overflow on wasm32 targets (where `isize` is only 32 bits).
///
/// # Panics
///
/// Panics if `m < 2`.
#[must_use]
pub fn u_matrix(m: usize) -> DenseMatrix<i64> {
    assert!(m >= 2, "u_matrix requires m ≥ 2");
    let n = m - 1;
    // ALLOW: semigroup multiplicity m is always small; wrapping is impossible.
    #[allow(clippy::cast_possible_wrap)]
    let mi = m as i64;
    let mut data = vec![0i64; n * n];
    for a in 0..n {
        #[allow(clippy::cast_possible_wrap)]
        let ki = (a + 1) as i64;
        for b in 0..n {
            data[a * n + b] = if b < a { ki - mi } else { ki };
        }
    }
    DenseMatrix {
        rows: n,
        cols: n,
        data,
    }
}

// ── L(m) matrix ───────────────────────────────────────────────────────────────

/// Constructs the (m−1) × (m−1) strictly lower-triangular matrix L(m) of 1s.
///
/// `L(m)[i][j] = 1` if `i > j`, else `0`. The main diagonal and everything
/// above it are zero; everything strictly below is one.
///
/// Used together with [`u_matrix`] in the block identity
///
/// ```text
/// L(m) + U(m) = ⎡ U(m−1)   (1, 2, …, m−2)ᵀ ⎤
///               ⎣ 0  …  0           m−1     ⎦
/// ```
///
/// (verified in `tests::l_plus_u_matches_block_form`). Applying both sides
/// to the first row `c₁` of `C_red` yields the recursion
///
/// ```text
/// (L(m)+U(m))·c₁ = ⎛ U(m−1)·c₁′ + c_{1,m−1} · (1, 2, …, m−2)ᵀ ⎞
///                  ⎝            (m−1) · c_{1,m−1}              ⎠
/// ```
///
/// where `c₁′` is `c₁` with its last entry dropped.
///
/// # Panics
///
/// Panics if `m < 2`.
#[must_use]
pub fn l_matrix(m: usize) -> DenseMatrix<i64> {
    assert!(m >= 2, "l_matrix requires m ≥ 2");
    let n = m - 1;
    let mut data = vec![0i64; n * n];
    for i in 0..n {
        for j in 0..i {
            data[i * n + j] = 1;
        }
    }
    DenseMatrix {
        rows: n,
        cols: n,
        data,
    }
}

// ── V(m) matrix ───────────────────────────────────────────────────────────────

/// Constructs the (m−1) × (m−1) matrix V(m), the integer left inverse of U(m)
/// scaled by m: V(m)·U(m) = m·I.
///
/// V(m) encodes the relation `w₁ + wᵢ = w_{1+i} + c_{1,i}·m` between Apéry
/// elements, expressed in the `C_red` coordinates. Rows (1-indexed):
///
/// ```text
/// row 1:           V[1][1] = 2,   V[1][2] = -1
/// row i (1 < i < m−1): V[i][1] = 1, V[i][i] = 1, V[i][i+1] = -1
/// row m−1:         V[m-1][1] = 1, V[m-1][m-1] = 1
/// ```
///
/// All other entries are zero. For m = 2 the single row is both first and
/// last; the first-row rule applies, giving V(2) = (2).
///
/// Examples:
/// ```text
/// V(2) = [[ 2]]
/// V(3) = [[ 2, -1],
///         [ 1,  1]]
/// V(4) = [[ 2, -1,  0],
///         [ 1,  1, -1],
///         [ 1,  0,  1]]
/// ```
///
/// # Panics
///
/// Panics if `m < 2`.
#[must_use]
pub fn v_matrix(m: usize) -> DenseMatrix<i64> {
    assert!(m >= 2, "v_matrix requires m ≥ 2");
    let n = m - 1;
    let mut data = vec![0i64; n * n];
    if n == 1 {
        data[0] = 2;
        return DenseMatrix {
            rows: n,
            cols: n,
            data,
        };
    }
    data[0] = 2;
    data[1] = -1;
    for a in 1..n - 1 {
        data[a * n] = 1;
        data[a * n + a] = 1;
        data[a * n + a + 1] = -1;
    }
    data[(n - 1) * n] = 1;
    data[(n - 1) * n + (n - 1)] = 1;
    DenseMatrix {
        rows: n,
        cols: n,
        data,
    }
}

// ── Kunz matrix ───────────────────────────────────────────────────────────────

/// Constructs the m × m Kunz coefficient matrix for semigroup `s`.
///
/// Entry `(i, j)` equals `c(i,j) = (w_i + w_j − w_{(i+j) mod m}) / m`
/// where `w_k` is the `k`-th Apéry set element.  The matrix is symmetric
/// and has non-negative integer entries.
#[must_use]
pub fn kunz_matrix(s: &super::Semigroup) -> DenseMatrix<usize> {
    let m = s.m;
    let mut data = vec![0usize; m * m];
    for i in 0..m {
        for j in 0..m {
            data[i * m + j] = s.kunz(i, j);
        }
    }
    DenseMatrix {
        rows: m,
        cols: m,
        data,
    }
}

/// Constructs the (m−1) × (m−1) reduced Kunz matrix `C_red`, i.e. the
/// Kunz matrix with the i = 0 row and column dropped. Internal index
/// `(a, b)` corresponds to semigroup index `(a+1, b+1)`.
///
/// The dropped row and column of the full Kunz matrix are identically
/// zero (since `w_0 = 0`), so `C_red` retains all the information.
///
/// # Panics
///
/// Panics if `s.m < 2`.
#[must_use]
pub fn c_red(s: &super::Semigroup) -> DenseMatrix<usize> {
    let m = s.m;
    assert!(m >= 2, "c_red requires m ≥ 2");
    let n = m - 1;
    let mut data = vec![0usize; n * n];
    for a in 0..n {
        for b in 0..n {
            data[a * n + b] = s.kunz(a + 1, b + 1);
        }
    }
    DenseMatrix {
        rows: n,
        cols: n,
        data,
    }
}

// ── U(m)·C_red product (structure-aware, O(m²)) ──────────────────────────────

/// Computes the product `U(m) · C_red` exploiting the block structure of
/// U(m) to avoid a general O(m³) matrix multiply.
///
/// Both factors are (m−1) × (m−1). Internal index `a` corresponds to
/// semigroup index `a + 1`. Each row `a` of `U(m)` has entries `a+1−m`
/// for `b < a` and `a+1` for `b ≥ a`, giving:
///
/// ```text
/// (U·C_red)[a][b] = (a+1) · S[b] − m · P(a, b)
/// ```
///
/// where `S[b] = Σ_{c=0}^{m−2} C_red[c][b]` (column sum) and
/// `P(a, b) = Σ_{c=0}^{a−1} C_red[c][b]` (prefix sum of rows 0..a).
///
/// All arithmetic uses `i64` to accommodate the signed intermediate values;
/// overflow is not possible for semigroups with practical multiplicity.
///
/// # Panics
///
/// Panics if `c_red` is not square.
#[must_use]
pub fn u_times_c_red(c_red: &DenseMatrix<usize>) -> DenseMatrix<i64> {
    let n = c_red.rows;
    assert_eq!(c_red.cols, n, "u_times_c_red expects a square matrix");
    let m = n + 1;
    let mut col_sum = vec![0i64; n];
    for c in 0..n {
        for b in 0..n {
            // ALLOW: Kunz entries are small non-negative integers.
            #[allow(clippy::cast_possible_wrap)]
            let v = c_red[(c, b)] as i64;
            col_sum[b] += v;
        }
    }
    #[allow(clippy::cast_possible_wrap)]
    let mi = m as i64;
    let mut data = vec![0i64; n * n];
    let mut prefix = vec![0i64; n];
    for a in 0..n {
        #[allow(clippy::cast_possible_wrap)]
        let ki = (a + 1) as i64;
        for b in 0..n {
            data[a * n + b] = ki * col_sum[b] - mi * prefix[b];
        }
        for b in 0..n {
            #[allow(clippy::cast_possible_wrap)]
            let v = c_red[(a, b)] as i64;
            prefix[b] += v;
        }
    }
    DenseMatrix {
        rows: n,
        cols: n,
        data,
    }
}

// ── U(m) pair-relations matrix ────────────────────────────────────────────────

/// Constructs the `m(m−1)/2 × (m−1)` pair-relations matrix.
///
/// For each pair `(i, j)` with `1 ≤ i ≤ j ≤ m−1` (in lexicographic order),
/// the row is
///
/// ```text
/// (U(m)[i] + U(m)[j] − U(m)[(i+j) mod m]) / m
/// ```
///
/// where `U(m)[k]` for `k ∈ {1, …, m−1}` is the `(k−1)`-th row of
/// [`u_matrix`], and `U(m)[0]` is the zero row (the dropped i = 0 row, whose
/// non-trivial part vanishes after dropping column 0 too). The construction
/// is symmetric in `i, j` so pairs with `i > j` are skipped as duplicates.
///
/// The unscaled bracket is always divisible by `m`: every entry of `U(m)[a]`
/// is congruent to `a+1` (mod m), so each unscaled entry is congruent to
/// `i + j − (i+j) mod m`, which is `0` or `m`.
///
/// Multiplied by the first column of `C_red` (see [`c_red`]), this matrix
/// yields the vector of Kunz coefficients `c(i, j)` in the same lex order,
/// because `U[i] · C_red[:,0] = w_i` and `w_i + w_j − w_{(i+j) mod m} = m·c(i,j)`.
///
/// # Panics
///
/// Panics if `m < 2`. In debug builds, panics if any unscaled entry is not
/// divisible by `m` (which would contradict the construction).
#[must_use]
pub fn u_pair_relations(m: usize) -> DenseMatrix<i64> {
    assert!(m >= 2, "u_pair_relations requires m ≥ 2");
    let n = m - 1;
    let u_mat = u_matrix(m);
    #[allow(clippy::cast_possible_wrap)]
    let mi = m as i64;
    let total_rows = n * (n + 1) / 2;
    let mut data = vec![0i64; total_rows * n];
    let mut row_idx = 0;
    for a in 0..n {
        for b in a..n {
            let sum = (a + 1) + (b + 1);
            let k = if sum >= m { sum - m } else { sum };
            for col in 0..n {
                let mut v = u_mat[(a, col)] + u_mat[(b, col)];
                if k != 0 {
                    v -= u_mat[(k - 1, col)];
                }
                debug_assert_eq!(
                    v % mi,
                    0,
                    "u_pair_relations entry not divisible by m at ({a},{b},{col}): {v}"
                );
                data[row_idx * n + col] = v / mi;
            }
            row_idx += 1;
        }
    }
    DenseMatrix {
        rows: total_rows,
        cols: n,
        data,
    }
}

// ── Zero-diagonal row vector ──────────────────────────────────────────────────

/// Computes the zero-diagonal row vector zd(m) as a 1×(m−1) matrix.
///
/// The entry at column `b` (0-indexed) is:
///
/// ```text
/// zd(m)[b] = (1/m) · Σ_{j=1}^{m−1} (U_j[b] + U_{m−j}[b])
///           = 2b − m + 3
/// ```
///
/// The integer formula `2b − m + 3` is exact because each column sum of
/// U(m) equals `m·(2b − m + 3) / 2`, and the factor of 2 from the
/// symmetric pairing cancels the denominator.
///
/// **Key property:** `zd(m) · c₁ = f + m + r`, where `c₁` is the first
/// column of `C_red`, `f` is the Frobenius number, `m` the multiplicity, and
/// `r` the number of reflected gaps.
///
/// # Panics
///
/// Panics if `m < 2`.
#[must_use]
pub fn zd_vector(m: usize) -> DenseMatrix<i64> {
    assert!(m >= 2, "zd_vector requires m ≥ 2");
    let n = m - 1;
    #[allow(clippy::cast_possible_wrap)]
    let data: Vec<i64> = (0..n)
        .map(|b| {
            let bi = b as i64;
            let mi = m as i64;
            2 * bi - mi + 3
        })
        .collect();
    DenseMatrix {
        rows: 1,
        cols: n,
        data,
    }
}

// ── Diagonal matrix D(m) ─────────────────────────────────────────────────────

/// Computes the (m−1)×(m−1) anti-diagonal coefficient matrix D(m).
///
/// Row `a` (0-indexed, corresponding to semigroup index `i = a + 1`) is:
///
/// ```text
/// D(m)[a][b] = (1/m) · Σ_{j=1}^{m−1} (U_j[b] + U_{(i−j) mod m}[b] − U_i[b])
///            = zd(m)[b] − U(m)[a][b]
/// ```
///
/// The compact form follows because the sum over `j` of `U_{(i−j) mod m}[b]`
/// equals the full column sum of U(m) minus `U_i[b]`, and `(1/m)` times
/// twice the column sum equals `zd(m)[b]`.
///
/// **Key property:** `D(m) · c₁ = (d₁, …, d_{m−1})ᵀ` where
/// `dᵢ = Σ_{j=1}^{m−1} c(j, (i−j) mod m)` is the sum of Kunz
/// anti-diagonal entries at position `i`.  Combined with the Apéry
/// elements, `wᵢ + dᵢ = f + m + r` for all `i ∈ {1, …, m−1}`.
///
/// # Panics
///
/// Panics if `m < 2`.
#[must_use]
pub fn d_matrix(m: usize) -> DenseMatrix<i64> {
    assert!(m >= 2, "d_matrix requires m ≥ 2");
    let n = m - 1;
    let zd = zd_vector(m);
    let u = u_matrix(m);
    let mut data = vec![0i64; n * n];
    for a in 0..n {
        for b in 0..n {
            data[a * n + b] = zd[(0, b)] - u[(a, b)];
        }
    }
    DenseMatrix {
        rows: n,
        cols: n,
        data,
    }
}

// ── usize → i64 conversion ────────────────────────────────────────────────────

/// Converts a `DenseMatrix<usize>` to `DenseMatrix<i64>` so that signed
/// operations (determinant, inverse) can be applied.
#[must_use]
pub fn to_i64(mat: &DenseMatrix<usize>) -> DenseMatrix<i64> {
    DenseMatrix {
        rows: mat.rows,
        cols: mat.cols,
        #[allow(clippy::cast_possible_wrap)]
        data: mat.data.iter().map(|&x| x as i64).collect(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::compute;

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

    // ── U(m) ──────────────────────────────────────────────────────────────────

    #[test]
    fn u_matrix_m2_is_1x1() {
        // m=2 → (m−1)×(m−1) = 1×1 = [[1]]
        let u = u_matrix(2);
        assert_eq!(u.rows, 1);
        assert_eq!(u.cols, 1);
        assert_eq!(u[(0, 0)], 1);
    }

    #[test]
    fn u_matrix_m5_is_4x4() {
        let u = u_matrix(5);
        let expected: &[&[i64]] = &[
            &[1, 1, 1, 1],
            &[-3, 2, 2, 2],
            &[-2, -2, 3, 3],
            &[-1, -1, -1, 4],
        ];
        assert_eq!(u.rows, 4);
        assert_eq!(u.cols, 4);
        for (a, row) in expected.iter().enumerate() {
            for (b, &v) in row.iter().enumerate() {
                assert_eq!(u[(a, b)], v, "U(5)[{a}][{b}]");
            }
        }
    }

    #[test]
    fn u_matrix_last_row_pattern() {
        // Last row a = m−2: cols 0..m−3 = −1; last col = m−1.
        for m in 3..=10 {
            let u = u_matrix(m);
            let n = m - 1;
            for b in 0..(n - 1) {
                assert_eq!(u[(n - 1, b)], -1, "U({m}) last row col {b}");
            }
            #[allow(clippy::cast_possible_wrap)]
            let expected = (m - 1) as i64;
            assert_eq!(u[(n - 1, n - 1)], expected, "U({m}) last row last col");
        }
    }

    #[test]
    fn u_matrix_row0_is_all_ones() {
        // a=0 (semigroup row k=1): all entries equal a+1 = 1.
        for m in 2..=8 {
            let u = u_matrix(m);
            for b in 0..(m - 1) {
                assert_eq!(u[(0, b)], 1, "U({m}) row0 col {b}");
            }
        }
    }

    #[test]
    fn u_matrix_col0_pattern() {
        // col 0 (b=0): row 0 = 1, rows a≥1 = a+1−m (since b<a).
        for m in 2..=8 {
            let u = u_matrix(m);
            assert_eq!(u[(0, 0)], 1, "U({m}) col0 row 0");
            #[allow(clippy::cast_possible_wrap)]
            let mi = m as i64;
            for a in 1..(m - 1) {
                #[allow(clippy::cast_possible_wrap)]
                let expected = (a + 1) as i64 - mi;
                assert_eq!(u[(a, 0)], expected, "U({m}) col0 row {a}");
            }
        }
    }

    // ── U(m) determinant ────────────────────────────────────────────────────

    #[test]
    fn u_matrix_det_small_cases() {
        // det(U(m)) = m^(m−2). Same as the m × m unit-bordered form, since
        // dropping a 1-on-diagonal row/col contributes a factor of 1.
        let expected: &[(usize, i64)] = &[
            (2, 1),     // 2^0
            (3, 3),     // 3^1
            (4, 16),    // 4^2
            (5, 125),   // 5^3
            (6, 1296),  // 6^4
            (7, 16807), // 7^5
        ];
        for &(m, det) in expected {
            let u = u_matrix(m);
            assert_eq!(u.det(), det, "det(U({m})) should be {m}^({m}−2) = {det}");
        }
    }

    #[test]
    fn u_matrix_det_equals_m_pow_m_minus_2() {
        // Parametric check for m = 2..=10.
        // Bareiss intermediate products overflow i64 for m ≥ 11.
        for m in 2..=10 {
            let u = u_matrix(m);
            #[allow(clippy::cast_possible_wrap)]
            let expected = (m as i64).pow(u32::try_from(m - 2).unwrap());
            assert_eq!(u.det(), expected, "det(U({m})) ≠ {m}^{}", m - 2);
        }
    }

    #[test]
    fn u_matrix_det_m2_manual() {
        // U(2) = [[1]], det = 1.
        let u = u_matrix(2);
        assert_eq!(u.det(), 1);
    }

    #[test]
    fn u_matrix_det_m3_manual() {
        // U(3) = [[1,1],[-1,2]], det = 1·2 − 1·(−1) = 3.
        let u = u_matrix(3);
        assert_eq!(u.det(), 3);
    }

    // ── L(m) ──────────────────────────────────────────────────────────────────

    #[test]
    fn l_matrix_m2_is_zero() {
        // m=2 → 1×1 strictly lower triangular = [[0]].
        let l = l_matrix(2);
        assert_eq!(l.rows, 1);
        assert_eq!(l.cols, 1);
        assert_eq!(l[(0, 0)], 0);
    }

    #[test]
    fn l_matrix_m5_pattern() {
        let l = l_matrix(5);
        let expected: &[&[i64]] = &[&[0, 0, 0, 0], &[1, 0, 0, 0], &[1, 1, 0, 0], &[1, 1, 1, 0]];
        for (i, row) in expected.iter().enumerate() {
            for (j, &v) in row.iter().enumerate() {
                assert_eq!(l[(i, j)], v, "L(5)[{i}][{j}]");
            }
        }
    }

    #[test]
    fn l_plus_u_matches_block_form() {
        // L(m) + U(m) =
        //   ⎡ U(m-1)   (1, 2, …, m-2)ᵀ ⎤
        //   ⎣ 0 … 0          m-1        ⎦
        for m in 2..=50 {
            let sum = l_matrix(m).mat_add(&u_matrix(m));
            let n = m - 1;

            if m == 2 {
                // Degenerate: U(m-1) = U(1) is 0×0; matrix reduces to [[m-1]] = [[1]].
                #[allow(clippy::cast_possible_wrap)]
                let expected = (m - 1) as i64;
                assert_eq!(sum[(0, 0)], expected, "L+U for m=2");
                continue;
            }

            let u_prev = u_matrix(m - 1);

            // Top-left (m-2)×(m-2) block equals U(m-1).
            for a in 0..(n - 1) {
                for b in 0..(n - 1) {
                    assert_eq!(
                        sum[(a, b)],
                        u_prev[(a, b)],
                        "L+U top-left block at ({a},{b}) for m={m}",
                    );
                }
            }

            // Top-right column entries are 1, 2, …, m-2.
            for a in 0..(n - 1) {
                #[allow(clippy::cast_possible_wrap)]
                let expected = (a + 1) as i64;
                assert_eq!(
                    sum[(a, n - 1)],
                    expected,
                    "L+U top-right col at row {a} for m={m}",
                );
            }

            // Bottom row: zeros except the last entry, which is m-1.
            for b in 0..(n - 1) {
                assert_eq!(sum[(n - 1, b)], 0, "L+U bottom row at col {b} for m={m}");
            }
            #[allow(clippy::cast_possible_wrap)]
            let expected = (m - 1) as i64;
            assert_eq!(
                sum[(n - 1, n - 1)],
                expected,
                "L+U bottom-right corner for m={m}",
            );
        }
    }

    #[test]
    fn l_plus_u_times_c1_recursion_on_random_vectors() {
        // Pure algebraic identity — holds for any (m-1)-vector v, not only c₁.
        // For each m, fabricate a few vectors and check
        //   (L(m)+U(m))·v = ⎛ U(m-1)·v' + v_{m-1} · (1..m-2)ᵀ ⎞
        //                   ⎝          (m-1) · v_{m-1}        ⎠
        // where v' = v[..m-2].
        let cases: &[&[i64]] = &[
            &[1],
            &[3, 7],
            &[2, 5, 11],
            &[4, 1, 9, 2],
            &[7, 3, 5, 8, 2, 6, 4, 9, 1, 10],
        ];
        for v in cases {
            let n = v.len();
            let m = n + 1;
            let lpu = l_matrix(m).mat_add(&u_matrix(m));
            let v_col = DenseMatrix::from_row_slice(n, 1, v);
            let lhs = lpu.mat_mul(&v_col);

            // Build RHS by the block recipe.
            let last = v[n - 1];
            let mut rhs = vec![0i64; n];
            if m == 2 {
                #[allow(clippy::cast_possible_wrap)]
                let scale = (m - 1) as i64;
                rhs[0] = scale * last;
            } else {
                let v_prime = DenseMatrix::from_row_slice(n - 1, 1, &v[..n - 1]);
                let top = u_matrix(m - 1).mat_mul(&v_prime);
                for i in 0..(n - 1) {
                    #[allow(clippy::cast_possible_wrap)]
                    let k = (i + 1) as i64;
                    rhs[i] = top[(i, 0)] + k * last;
                }
                #[allow(clippy::cast_possible_wrap)]
                let scale = (m - 1) as i64;
                rhs[n - 1] = scale * last;
            }

            for i in 0..n {
                assert_eq!(lhs[(i, 0)], rhs[i], "row {i} for m={m}, v={v:?}");
            }
        }
    }

    #[test]
    fn l_plus_u_times_c1_recursion_on_semigroups() {
        // Same identity, applied to c₁ = first row of C_red of real semigroups.
        let gens_cases: &[&[usize]] = &[
            &[2, 5],
            &[3, 4],
            &[3, 7, 11],
            &[4, 6, 9, 11],
            &[5, 7, 11, 13, 17],
            &[6, 9, 20],
            &[7, 9, 11, 13, 15, 17],
            &[8, 11, 13, 14, 17, 19, 21],
        ];
        for gens in gens_cases {
            let s = compute(gens);
            if s.m < 2 {
                continue;
            }
            let m = s.m;
            let n = m - 1;
            let cred = c_red(&s);
            let cred_i = to_i64(&cred);

            // c₁ = first row of C_red, as a column vector.
            let c1: Vec<i64> = (0..n).map(|j| cred_i[(0, j)]).collect();
            let c1_col = DenseMatrix::from_row_slice(n, 1, &c1);

            let lhs = l_matrix(m).mat_add(&u_matrix(m)).mat_mul(&c1_col);

            let last = c1[n - 1];
            let mut rhs = vec![0i64; n];
            if m == 2 {
                #[allow(clippy::cast_possible_wrap)]
                let scale = (m - 1) as i64;
                rhs[0] = scale * last;
            } else {
                let c1_prime = DenseMatrix::from_row_slice(n - 1, 1, &c1[..n - 1]);
                let top = u_matrix(m - 1).mat_mul(&c1_prime);
                for i in 0..(n - 1) {
                    #[allow(clippy::cast_possible_wrap)]
                    let k = (i + 1) as i64;
                    rhs[i] = top[(i, 0)] + k * last;
                }
                #[allow(clippy::cast_possible_wrap)]
                let scale = (m - 1) as i64;
                rhs[n - 1] = scale * last;
            }

            for i in 0..n {
                assert_eq!(lhs[(i, 0)], rhs[i], "row {i} for gens={gens:?}");
            }
        }
    }

    // ── V(m) ──────────────────────────────────────────────────────────────────

    #[test]
    fn v_matrix_m2_is_2() {
        let v = v_matrix(2);
        assert_eq!(v.rows, 1);
        assert_eq!(v.cols, 1);
        assert_eq!(v[(0, 0)], 2);
    }

    #[test]
    fn v_matrix_m3_known() {
        let v = v_matrix(3);
        let expected: &[&[i64]] = &[&[2, -1], &[1, 1]];
        for (a, row) in expected.iter().enumerate() {
            for (b, &val) in row.iter().enumerate() {
                assert_eq!(v[(a, b)], val, "V(3)[{a}][{b}]");
            }
        }
    }

    #[test]
    fn v_matrix_m4_known() {
        let v = v_matrix(4);
        let expected: &[&[i64]] = &[&[2, -1, 0], &[1, 1, -1], &[1, 0, 1]];
        for (a, row) in expected.iter().enumerate() {
            for (b, &val) in row.iter().enumerate() {
                assert_eq!(v[(a, b)], val, "V(4)[{a}][{b}]");
            }
        }
    }

    #[test]
    fn u_matrix_block_decomposition() {
        // Splitting x = (x', x_{m-1}) with x' ∈ ℤ^{m-2} (math 1-indexed):
        //
        //   (U(m)x)_i      = (U(m-1) x')_i  −  Σ_{j<i} x_j  +  i·x_{m-1}     (1 ≤ i ≤ m-2)
        //   (U(m)x)_{m-1}  = (m-1)·x_{m-1}  −  Σ_{j<m-1} x_j
        //
        // Verify on a few small (m, x) pairs.
        fn matvec(mat: &DenseMatrix<i64>, x: &[i64]) -> Vec<i64> {
            (0..mat.nrows())
                .map(|a| (0..mat.ncols()).map(|b| mat[(a, b)] * x[b]).sum())
                .collect()
        }
        let cases: &[(usize, &[i64])] = &[
            (2, &[7]),
            (3, &[2, 5]),
            (3, &[0, 7]),
            (4, &[1, 2, 3]),
            (4, &[0, 0, 5]),
            (5, &[1, 2, 3, 4]),
            (6, &[3, 1, 4, 1, 5]),
            (7, &[-2, 3, -5, 7, -11, 13]),
        ];
        for &(m, x) in cases {
            let n = m - 1;
            assert_eq!(x.len(), n);
            let ux = matvec(&u_matrix(m), x);
            #[allow(clippy::cast_possible_wrap)]
            let x_last = x[n - 1];
            let x_prime = &x[..n - 1];

            // First identity: only meaningful for m ≥ 3 (otherwise n-1 = 0 and the loop is empty).
            if m >= 3 {
                let ux_prime = matvec(&u_matrix(m - 1), x_prime);
                for i in 0..n - 1 {
                    let prefix: i64 = x_prime[..i].iter().sum();
                    #[allow(clippy::cast_possible_wrap)]
                    let expected = ux_prime[i] - prefix + (i + 1) as i64 * x_last;
                    assert_eq!(
                        ux[i],
                        expected,
                        "(U({m})·x)_{i_math} mismatch (block decomposition)",
                        i_math = i + 1,
                    );
                }
            }

            // Last row identity, valid for all m ≥ 2.
            let full_prefix: i64 = x_prime.iter().sum();
            #[allow(clippy::cast_possible_wrap)]
            let expected_last = (n as i64) * x_last - full_prefix;
            assert_eq!(
                ux[n - 1],
                expected_last,
                "(U({m})·x)_{{m-1}} mismatch (block decomposition)",
            );
        }
    }

    #[test]
    fn v_times_u_is_m_identity() {
        // The whole point of V(m): V(m)·U(m) = m·I_{m−1}.
        for m in 2..=10 {
            let v = v_matrix(m);
            let u = u_matrix(m);
            let prod = v * u;
            let n = m - 1;
            for a in 0..n {
                for b in 0..n {
                    #[allow(clippy::cast_possible_wrap)]
                    let expected = if a == b { m as i64 } else { 0 };
                    assert_eq!(
                        prod[(a, b)],
                        expected,
                        "V({m})·U({m})[{a}][{b}] should equal {expected}",
                    );
                }
            }
        }
    }

    // ── c_red ─────────────────────────────────────────────────────────────────

    #[test]
    fn c_red_matches_kunz_submatrix() {
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![6, 9, 20],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let m = s.m;
            let cr = c_red(&s);
            let k = kunz_matrix(&s);
            assert_eq!(cr.rows, m - 1);
            assert_eq!(cr.cols, m - 1);
            for a in 0..(m - 1) {
                for b in 0..(m - 1) {
                    assert_eq!(cr[(a, b)], k[(a + 1, b + 1)], "c_red[{a}][{b}] mismatch");
                }
            }
        }
    }

    // ── u_times_c_red ─────────────────────────────────────────────────────────

    #[test]
    fn u_times_c_red_matches_mat_mul() {
        // Structure-aware product matches the general mat_mul of the (m−1)×(m−1) factors.
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 6, 11],
            vec![5, 7, 11],
            vec![6, 7, 8, 9],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let cr = c_red(&s);
            let fast = u_times_c_red(&cr);
            let u = u_matrix(s.m);
            let cr_i64 = to_i64(&cr);
            let general = u.mat_mul(&cr_i64);
            assert_eq!(fast, general, "u_times_c_red mismatch for ⟨{gens:?}⟩");
        }
    }

    #[test]
    fn u_times_c_red_row0_is_apery() {
        // Row 0 (semigroup row k=1): (U·C_red)[0][b] = w_{b+1}.
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![5, 7, 11],
            vec![6, 9, 20],
        ] {
            let s = compute(gens);
            let m = s.m;
            let product = u_times_c_red(&c_red(&s));
            for b in 0..(m - 1) {
                #[allow(clippy::cast_possible_wrap)]
                let w = s.apery_set[b + 1] as i64;
                assert_eq!(product[(0, b)], w, "row 0 col {b}");
            }
        }
    }

    #[test]
    fn u_times_c_red_col0_is_apery() {
        // Column 0 (semigroup col j=1): (U·C_red)[a][0] = w_{a+1}.
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![5, 7, 11],
            vec![6, 9, 20],
        ] {
            let s = compute(gens);
            let m = s.m;
            let product = u_times_c_red(&c_red(&s));
            for a in 0..(m - 1) {
                #[allow(clippy::cast_possible_wrap)]
                let w = s.apery_set[a + 1] as i64;
                assert_eq!(product[(a, 0)], w, "col 0 row {a}");
            }
        }
    }

    #[test]
    fn u_times_c_red_last_row() {
        // Last row a = m−2: (U·C_red)[m−2][b] = w_{m−1} − w_b for b ≥ 1, w_{m−1} for b = 0.
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![5, 7, 11],
            vec![6, 7, 8, 9],
        ] {
            let s = compute(gens);
            let m = s.m;
            let product = u_times_c_red(&c_red(&s));
            #[allow(clippy::cast_possible_wrap)]
            let w_last = s.apery_set[m - 1] as i64;
            for b in 0..(m - 1) {
                // (U·C_red)[m-2][b] = old (U·C)[m-1][b+1] = w_{m-1} - w_{(m+b) mod m} = w_{m-1} - w_b
                #[allow(clippy::cast_possible_wrap)]
                let expected = w_last - s.apery_set[b] as i64;
                assert_eq!(
                    product[(m - 2, b)],
                    expected,
                    "last row col {b}: w_{{m-1}} - w_{b}",
                );
            }
        }
    }

    #[test]
    fn u_times_c_red_symmetric() {
        // U·C_red is symmetric (it's the i,j ≥ 1 block of the symmetric U·C).
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![5, 7, 11],
            vec![6, 9, 20],
            vec![6, 7, 8, 9],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let m = s.m;
            let product = u_times_c_red(&c_red(&s));
            for a in 0..(m - 1) {
                for b in 0..(m - 1) {
                    assert_eq!(
                        product[(a, b)],
                        product[(b, a)],
                        "⟨{gens:?}⟩: (U·C_red)[{a}][{b}] ≠ (U·C_red)[{b}][{a}]"
                    );
                }
            }
        }
    }

    #[test]
    fn u_times_c_red_general_entry_formula() {
        // (U·C_red)[a][b] = Σ_{l=0}^{a} [w_{(l+b+1) mod m} − w_l].
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![6, 9, 20],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let m = s.m;
            let product = u_times_c_red(&c_red(&s));
            for a in 0..(m - 1) {
                for b in 0..(m - 1) {
                    let mut expected = 0i64;
                    for l in 0..=a {
                        #[allow(clippy::cast_possible_wrap)]
                        let diff = s.apery_set[(l + b + 1) % m] as i64 - s.apery_set[l] as i64;
                        expected += diff;
                    }
                    assert_eq!(product[(a, b)], expected, "⟨{gens:?}⟩ entry ({a},{b})");
                }
            }
        }
    }

    // ── kunz_matrix ───────────────────────────────────────────────────────────

    /// Verify c(i,j) == c(j,i) for every entry.
    fn assert_symmetric(k: &DenseMatrix<usize>, label: &str) {
        let m = k.rows;
        for i in 0..m {
            for j in 0..m {
                assert_eq!(k[(i, j)], k[(j, i)], "{label}: c({i},{j}) ≠ c({j},{i})");
            }
        }
    }

    #[test]
    fn kunz_matrix_3_5_7_symmetric_and_known() {
        // ⟨3,5,7⟩: apery = [0, 7, 5], Frobenius = 4
        let s = compute(&[3, 5, 7]);
        let k = kunz_matrix(&s);
        assert_eq!(k.rows, 3);
        assert_symmetric(&k, "⟨3,5,7⟩");
        // Row 0 is all zeros (apery[0] = 0)
        for j in 0..3 {
            assert_eq!(k[(0, j)], 0, "row 0 must be 0");
        }
        // c(1,1) = (7+7-5)/3 = 3, c(1,2) = (7+5-0)/3 = 4, c(2,2) = (5+5-7)/3 = 1
        assert_eq!(k[(1, 1)], 3);
        assert_eq!(k[(1, 2)], 4);
        assert_eq!(k[(2, 2)], 1);
    }

    #[test]
    fn kunz_matrix_6_9_20_symmetric() {
        let s = compute(&[6, 9, 20]);
        let k = kunz_matrix(&s);
        assert_eq!(k.rows, 6);
        assert_symmetric(&k, "⟨6,9,20⟩");
    }

    #[test]
    fn kunz_matrix_symmetric_for_various_semigroups() {
        for gens in &[
            vec![4usize, 5],
            vec![5, 7, 11],
            vec![6, 7, 8, 9],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let k = kunz_matrix(&s);
            assert_eq!(k.rows, s.m);
            assert_eq!(k.cols, s.m);
            assert_symmetric(&k, "symmetry check");
        }
    }

    #[test]
    fn kunz_matrix_row0_all_zero() {
        // apery[0] = 0 always, so c(0,j) = (0 + w_j - w_j)/m = 0 for all j
        for gens in &[vec![3usize, 5], vec![4, 6, 11], vec![5, 8, 13]] {
            let s = compute(gens);
            let k = kunz_matrix(&s);
            for j in 0..s.m {
                assert_eq!(k[(0, j)], 0, "c(0,{j}) must be 0");
            }
        }
    }

    // ── u_pair_relations ──────────────────────────────────────────────────────

    #[test]
    fn u_pair_relations_dimensions() {
        // Rows = m(m−1)/2 (lex pairs (i, j) with 1 ≤ i ≤ j ≤ m−1).
        for m in 2..=8 {
            let p = u_pair_relations(m);
            let n = m - 1;
            assert_eq!(p.rows, n * (n + 1) / 2, "rows for m={m}");
            assert_eq!(p.cols, n, "cols for m={m}");
        }
    }

    #[test]
    fn u_pair_relations_unscaled_entries_divisible_by_m() {
        // Theory check: every entry of U[i] + U[j] − U[(i+j) mod m] is ≡ 0 (mod m),
        // because each row of U(m) satisfies U[a][c] ≡ a+1 (mod m).
        for m in 2..=10 {
            let u = u_matrix(m);
            let n = m - 1;
            #[allow(clippy::cast_possible_wrap)]
            let mi = m as i64;
            for a in 0..n {
                for b in a..n {
                    let sum = (a + 1) + (b + 1);
                    let k = if sum >= m { sum - m } else { sum };
                    for col in 0..n {
                        let mut v = u[(a, col)] + u[(b, col)];
                        if k != 0 {
                            v -= u[(k - 1, col)];
                        }
                        assert_eq!(
                            v % mi,
                            0,
                            "m={m} pair ({},{}) col {col}: unscaled entry {v} not ≡ 0 (mod {m})",
                            a + 1,
                            b + 1,
                        );
                    }
                }
            }
        }
    }

    /// Lexicographic row index for pair (a, b) with a ≤ b in `0..n`.
    fn pair_row_idx(a: usize, b: usize, n: usize) -> usize {
        debug_assert!(a <= b);
        // sum_{a'=0}^{a-1} (n - a') + (b - a) = a·n − a(a−1)/2 + (b − a)
        a * n - a * (a.saturating_sub(1)) / 2 + (b - a)
    }

    #[test]
    fn u_pair_relations_m3_explicit() {
        // m=3, n=2. Pairs in order: (1,1), (1,2), (2,2).
        // Unscaled: [3,0], [0,3], [-3,3]; divided by 3:
        let p = u_pair_relations(3);
        let expected: &[&[i64]] = &[&[1, 0], &[0, 1], &[-1, 1]];
        assert_eq!(p.rows, 3);
        for (r, row) in expected.iter().enumerate() {
            for (col, &v) in row.iter().enumerate() {
                assert_eq!(p[(r, col)], v, "P(3)[{r}][{col}]");
            }
        }
    }

    #[test]
    fn u_pair_relations_times_c_red_col0_is_kunz() {
        // P · C_red[:,0] gives c(i, j) for each pair (i, j) with i ≤ j.
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![5, 7, 11],
            vec![6, 9, 20],
            vec![6, 7, 8, 9],
        ] {
            let s = compute(gens);
            let m = s.m;
            let n = m - 1;
            let p = u_pair_relations(m);
            let cr = c_red(&s);
            #[allow(clippy::cast_possible_wrap)]
            let col0: Vec<i64> = (0..n).map(|a| cr[(a, 0)] as i64).collect();
            for a in 0..n {
                for b in a..n {
                    let row_idx = pair_row_idx(a, b, n);
                    let mut sum = 0i64;
                    for col in 0..n {
                        sum += p[(row_idx, col)] * col0[col];
                    }
                    #[allow(clippy::cast_possible_wrap)]
                    let expected = s.kunz(a + 1, b + 1) as i64;
                    assert_eq!(
                        sum,
                        expected,
                        "⟨{gens:?}⟩ pair ({},{}): c = {expected}",
                        a + 1,
                        b + 1
                    );
                }
            }
        }
    }

    #[test]
    fn u_pair_relations_diagonal_when_2i_equals_m() {
        // When 2i = m the unscaled row is U[i]+U[i]-U[0] = 2·U[i]; divided by m:
        // entry c is −1 if c < i−1 and 1 otherwise (since U[i−1][c] ∈ {i−m, i} = {−m/2, m/2}).
        for m in [2usize, 4, 6, 8] {
            let p = u_pair_relations(m);
            let n = m - 1;
            let i = m / 2;
            let row_idx = pair_row_idx(i - 1, i - 1, n);
            for col in 0..n {
                let expected = if col + 1 < i { -1 } else { 1 };
                assert_eq!(p[(row_idx, col)], expected, "m={m} col={col}");
            }
        }
    }

    // ── zd_vector ─────────────────────────────────────────────────────────────

    #[test]
    fn zd_vector_m3_known() {
        // zd(3)[b] = 2b − 3 + 3 = 2b; entries: [0, 2]
        let zd = zd_vector(3);
        assert_eq!(zd.nrows(), 1);
        assert_eq!(zd.ncols(), 2);
        assert_eq!(zd[(0, 0)], 0);
        assert_eq!(zd[(0, 1)], 2);
    }

    #[test]
    fn zd_vector_m4_known() {
        // zd(4)[b] = 2b − 4 + 3 = 2b − 1; entries: [−1, 1, 3]
        let zd = zd_vector(4);
        assert_eq!(zd.nrows(), 1);
        assert_eq!(zd.ncols(), 3);
        assert_eq!(zd[(0, 0)], -1);
        assert_eq!(zd[(0, 1)], 1);
        assert_eq!(zd[(0, 2)], 3);
    }

    #[test]
    fn zd_vector_entry_formula() {
        // zd(m)[b] = 2b − m + 3 for b = 0..m−2.
        for m in 2..=9 {
            let zd = zd_vector(m);
            for b in 0..(m - 1) {
                #[allow(clippy::cast_possible_wrap)]
                let expected = 2 * b as i64 - m as i64 + 3;
                assert_eq!(zd[(0, b)], expected, "zd({m})[{b}]");
            }
        }
    }

    #[test]
    fn zd_times_c1_equals_f_plus_m_plus_r() {
        // zd(m) · c₁ = f + m + r  for each test semigroup.
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![6, 9, 20],
            vec![5, 7, 11],
            vec![6, 7, 8, 9],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let m = s.m;
            let n = m - 1;
            let zd = zd_vector(m);
            let cr = c_red(&s);
            #[allow(clippy::cast_possible_wrap)]
            let dot: i64 = (0..n).map(|b| zd[(0, b)] * cr[(b, 0)] as i64).sum();
            #[allow(clippy::cast_possible_wrap)]
            let expected = (s.f + m + s.r) as i64;
            assert_eq!(
                dot, expected,
                "⟨{gens:?}⟩: zd·c₁ = {dot}, f+m+r = {expected}"
            );
        }
    }

    // ── d_matrix ──────────────────────────────────────────────────────────────

    #[test]
    fn d_matrix_equals_zd_minus_u() {
        // D(m)[a][b] = zd(m)[b] − U(m)[a][b].
        for m in 2..=8 {
            let d = d_matrix(m);
            let zd = zd_vector(m);
            let u = u_matrix(m);
            let n = m - 1;
            for a in 0..n {
                for b in 0..n {
                    assert_eq!(d[(a, b)], zd[(0, b)] - u[(a, b)], "D({m})[{a}][{b}]");
                }
            }
        }
    }

    #[test]
    fn d_matrix_times_c1_gives_diag_sums() {
        // D(m) · c₁ = (d₁, …, d_{m−1}) where dᵢ = sg.diag(i).
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![6, 9, 20],
            vec![5, 7, 11],
            vec![6, 7, 8, 9],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let m = s.m;
            let n = m - 1;
            let d = d_matrix(m);
            let cr = c_red(&s);
            for i in 1..=n {
                #[allow(clippy::cast_possible_wrap)]
                let product_row: i64 = (0..n).map(|b| d[(i - 1, b)] * cr[(b, 0)] as i64).sum();
                #[allow(clippy::cast_possible_wrap)]
                let expected = s.diag(i) as i64;
                assert_eq!(
                    product_row, expected,
                    "⟨{gens:?}⟩: D·c₁[{i}] = {product_row}, diag({i}) = {expected}"
                );
            }
        }
    }

    #[test]
    fn w_plus_d_equals_f_plus_m_plus_r() {
        // wᵢ + dᵢ = f + m + r for all i ∈ {1, …, m−1}.
        for gens in &[
            vec![3usize, 5, 7],
            vec![4, 5],
            vec![6, 9, 20],
            vec![5, 7, 11],
            vec![6, 7, 8, 9],
            vec![7, 11, 13, 17, 19],
        ] {
            let s = compute(gens);
            let m = s.m;
            let n = m - 1;
            let d = d_matrix(m);
            let cr = c_red(&s);
            #[allow(clippy::cast_possible_wrap)]
            let target = (s.f + m + s.r) as i64;
            for i in 1..=n {
                #[allow(clippy::cast_possible_wrap)]
                let di: i64 = (0..n).map(|b| d[(i - 1, b)] * cr[(b, 0)] as i64).sum();
                #[allow(clippy::cast_possible_wrap)]
                let wi = s.apery_set[i] as i64;
                assert_eq!(
                    wi + di,
                    target,
                    "⟨{gens:?}⟩: w_{i} + d_{i} = {}, expected f+m+r = {target}",
                    wi + di
                );
            }
        }
    }
}
