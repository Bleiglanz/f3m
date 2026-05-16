//! Semigroup-flavoured matrix constructors.
//!
//! Includes U(m), L(m), V(m), the Kunz coefficient matrix, the reduced
//! Kunz matrix `C_red`, the pair-relations matrix, the zero-diagonal
//! row vector, and the anti-diagonal coefficient matrix D(m). Plus the
//! `to_i64` widening helper used to lift `usize` matrices into the
//! signed Bareiss-friendly representation.

use super::dense::DenseMatrix;

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
    DenseMatrix::from_vec(n, n, data)
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
    DenseMatrix::from_vec(n, n, data)
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
        return DenseMatrix::from_vec(n, n, data);
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
    DenseMatrix::from_vec(n, n, data)
}

// ── Kunz matrix ───────────────────────────────────────────────────────────────

/// Constructs the m × m Kunz coefficient matrix for semigroup `s`.
///
/// Entry `(i, j)` equals `c(i,j) = (w_i + w_j − w_{(i+j) mod m}) / m`
/// where `w_k` is the `k`-th Apéry set element.  The matrix is symmetric
/// and has non-negative integer entries.
#[must_use]
pub fn kunz_matrix(s: &super::super::Semigroup) -> DenseMatrix<usize> {
    let m = s.m;
    let mut data = vec![0usize; m * m];
    for i in 0..m {
        for j in 0..m {
            data[i * m + j] = s.kunz(i, j);
        }
    }
    DenseMatrix::from_vec(m, m, data)
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
pub fn c_red(s: &super::super::Semigroup) -> DenseMatrix<usize> {
    let m = s.m;
    assert!(m >= 2, "c_red requires m ≥ 2");
    let n = m - 1;
    let mut data = vec![0usize; n * n];
    for a in 0..n {
        for b in 0..n {
            data[a * n + b] = s.kunz(a + 1, b + 1);
        }
    }
    DenseMatrix::from_vec(n, n, data)
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
    let n = c_red.nrows();
    assert_eq!(c_red.ncols(), n, "u_times_c_red expects a square matrix");
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
    DenseMatrix::from_vec(n, n, data)
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
    DenseMatrix::from_vec(total_rows, n, data)
}

// ── Pair-relations in augmented (r, n, 1) coordinates ────────────────────────

/// Constructs the `m(m−1)/2 × (2m−1)` pair-relations matrix in augmented
/// `(r, n, 1)` coordinates.
///
/// Columns index `(r_1, …, r_{m−1}, n_1, …, n_{m−1}, 1)`. Each row is one
/// `(i, j)` pair with `1 ≤ i ≤ j ≤ m−1` in lex order; the entry pattern is
///
/// ```text
/// +1 at r_i, r_j, n_i, n_j   (collapses to +2 on the diagonal i = j)
/// −1 at r_k, n_k             (where k = (i+j) mod m, only when k ≠ 0)
/// +1 in the constant column  (when i + j ≥ m, to absorb the wrap +1)
/// ```
///
/// Multiplying by the augmented `rn = (r_1, …, r_{m−1}, n_1, …, n_{m−1}, 1)`
/// vector yields the vector of Kunz coefficients `c(i, j)` in lex order. By
/// the Kunz cone, every entry of that product is `≥ 0` for any numerical
/// semigroup of the given `(m, μ, g)`.
///
/// # Panics
///
/// Panics if `m < 2`.
#[must_use]
pub fn pair_relations_rn(m: usize) -> DenseMatrix<i64> {
    assert!(m >= 2, "pair_relations_rn requires m ≥ 2");
    let n = m - 1;
    let cols = 2 * n + 1;
    let total_rows = n * (n + 1) / 2;
    let mut data = vec![0i64; total_rows * cols];
    let mut row_idx = 0;
    for a in 0..n {
        // a-based 0..n corresponds to one-based i = a+1.
        for b in a..n {
            let off = row_idx * cols;
            let sum = (a + 1) + (b + 1);
            let k = if sum >= m { sum - m } else { sum };
            // +1 at r_i, r_j and n_i, n_j (collapses to +2 if a == b).
            data[off + a] += 1;
            data[off + b] += 1;
            data[off + n + a] += 1;
            data[off + n + b] += 1;
            // −1 at r_k, n_k when k ≠ 0.
            if k != 0 {
                data[off + (k - 1)] -= 1;
                data[off + n + (k - 1)] -= 1;
            }
            // Wraparound +1 in the constant slot.
            if sum >= m {
                data[off + 2 * n] += 1;
            }
            row_idx += 1;
        }
    }
    DenseMatrix::from_vec(total_rows, cols, data)
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
    DenseMatrix::from_vec(1, n, data)
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
    DenseMatrix::from_vec(n, n, data)
}

// ── usize → i64 conversion ────────────────────────────────────────────────────

/// Converts a `DenseMatrix<usize>` to `DenseMatrix<i64>` so that signed
/// operations (determinant, inverse) can be applied.
#[must_use]
pub fn to_i64(mat: &DenseMatrix<usize>) -> DenseMatrix<i64> {
    let n_rows = mat.nrows();
    let n_cols = mat.ncols();
    #[allow(clippy::cast_possible_wrap)]
    let data: Vec<i64> = mat.as_slice().iter().map(|&x| x as i64).collect();
    DenseMatrix::from_vec(n_rows, n_cols, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::compute;
    use crate::math::matrix::Matrix;

    // ── U(m) ──────────────────────────────────────────────────────────────────

    #[test]
    fn u_matrix_m2_is_1x1() {
        // m=2 → (m−1)×(m−1) = 1×1 = [[1]]
        let u = u_matrix(2);
        assert_eq!(u.nrows(), 1);
        assert_eq!(u.ncols(), 1);
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
        assert_eq!(u.nrows(), 4);
        assert_eq!(u.ncols(), 4);
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
        assert_eq!(l.nrows(), 1);
        assert_eq!(l.ncols(), 1);
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
        assert_eq!(v.nrows(), 1);
        assert_eq!(v.ncols(), 1);
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
            assert_eq!(cr.nrows(), m - 1);
            assert_eq!(cr.ncols(), m - 1);
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
        let m = k.nrows();
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
        assert_eq!(k.nrows(), 3);
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
        assert_eq!(k.nrows(), 6);
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
            assert_eq!(k.nrows(), s.m);
            assert_eq!(k.ncols(), s.m);
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
            assert_eq!(p.nrows(), n * (n + 1) / 2, "rows for m={m}");
            assert_eq!(p.ncols(), n, "cols for m={m}");
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
        assert_eq!(p.nrows(), 3);
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
    fn pair_relations_rn_dimensions_and_row_structure() {
        // Shape: m(m−1)/2 rows × (2m−1) cols. Per row:
        // exactly four +1 entries among the r/n columns (collapses to two +2
        // entries on the diagonal i = j); zero or two −1 entries (one for r_k,
        // one for n_k when k = (i+j) mod m ≠ 0); the constant column is +1
        // exactly when i + j ≥ m.
        for m in 2..=8 {
            let p = pair_relations_rn(m);
            let n = m - 1;
            assert_eq!(p.nrows(), n * (n + 1) / 2, "rows for m={m}");
            assert_eq!(p.ncols(), 2 * n + 1, "cols for m={m}");
            let mut row = 0;
            for a in 0..n {
                for b in a..n {
                    let sum = (a + 1) + (b + 1);
                    let k = if sum >= m { sum - m } else { sum };
                    // Count positives in the r/n half only — the constant
                    // column is checked separately because its +1 from i+j ≥ m
                    // shouldn't muddle the diagonal-vs-off-diagonal accounting.
                    let pos_rn = (0..2 * n).filter(|&c| p[(row, c)] > 0).count();
                    let neg = (0..p.ncols()).filter(|&c| p[(row, c)] < 0).count();
                    let row_sum: i64 = (0..p.ncols()).map(|c| p[(row, c)]).sum();
                    if a == b {
                        // Diagonal: +2 at r_a and n_a (one column each).
                        assert_eq!(pos_rn, 2, "diag pair ({},{}) m={m}", a + 1, b + 1);
                        assert_eq!(p[(row, a)], 2, "diag r-cell m={m} a={a}");
                        assert_eq!(p[(row, n + a)], 2, "diag n-cell m={m} a={a}");
                    } else {
                        // Off-diagonal: +1 at four distinct r/n columns.
                        assert_eq!(pos_rn, 4, "off-diag pair ({},{}) m={m}", a + 1, b + 1);
                    }
                    let expected_neg = if k == 0 { 0 } else { 2 };
                    assert_eq!(
                        neg,
                        expected_neg,
                        "neg-count for ({},{}) m={m} k={k}",
                        a + 1,
                        b + 1,
                    );
                    let const_col = p[(row, 2 * n)];
                    assert_eq!(
                        const_col,
                        i64::from(u8::from(sum >= m)),
                        "const-col for ({},{}) m={m}",
                        a + 1,
                        b + 1,
                    );
                    // Row sum identity: 4 − 2·[k≠0] + [sum≥m] = c(i,j) when all
                    // r/n coefficients are 1 — i.e., this row applied to the
                    // augmented all-ones vector reproduces 4 − 2·[k≠0] + δ.
                    let expected_sum =
                        4 - 2 * i64::from(u8::from(k != 0)) + i64::from(u8::from(sum >= m));
                    assert_eq!(
                        row_sum,
                        expected_sum,
                        "row sum for ({},{}) m={m}",
                        a + 1,
                        b + 1
                    );
                    row += 1;
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
