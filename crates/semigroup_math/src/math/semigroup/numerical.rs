//! Numerical inspectors: methods returning a single scalar (`usize` or `f64`).

use super::Semigroup;

impl Semigroup {
    /// Wilf quotient: `sigma` / (f+1). Wilf's conjecture states this is ≥ 1/e for all S.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn wilf(&self) -> f64 {
        let c = self.f as f64 + 1.0f64;
        let spor = self.sigma as f64;
        spor / c
    }

    /// Kunz coefficient c(i,j) = (apery[i] + apery[j] - apery[(i+j) mod m]) / m.
    /// Forms a symmetric matrix; row sums equal the Apéry elements.
    ///
    /// The Apéry-divisibility invariant (`a_i + a_j − a_{i+j} ≡ 0 (mod m)` and
    /// `a_i + a_j ≥ a_{i+j}`) is checked in debug builds only; release builds
    /// trust the values computed by [`super::super::compute`]. Returns 0 if
    /// either invariant is violated, so this function never panics in release.
    #[must_use]
    pub fn kunz(&self, i: usize, j: usize) -> usize {
        let first = i % self.m;
        let second = j % self.m;
        let idx = (i + j) % self.m;
        let sum = self.apery_set[first] + self.apery_set[second];
        debug_assert!(
            sum >= self.apery_set[idx],
            "Kunz invariant: a_i + a_j ≥ a_{{i+j}}"
        );
        let Some(res) = sum.checked_sub(self.apery_set[idx]) else {
            return 0;
        };
        debug_assert_eq!(
            0,
            res % self.m,
            "Kunz invariant: a_i + a_j − a_{{i+j}} must be divisible by m",
        );
        res / self.m
    }

    /// Returns the sum of the anti-diagonal (minor diagonal) of the Kunz matrix through column `i`.
    ///
    /// For each `j` in `0..m`, sums `kunz(j, i+m-j mod m)`.
    #[must_use]
    pub fn diag(&self, i: usize) -> usize {
        let index = i % self.m;
        (0..self.m).map(|j| self.kunz(j, index + self.m - j)).sum()
    }

    /// Returns the sum of the main diagonal of the Kunz matrix through row `i`.
    ///
    /// For each `j` in `0..m`, sums `kunz(i+j mod m, j)`.
    #[must_use]
    pub fn main_diag(&self, i: usize) -> usize {
        let index: usize = i % self.m;
        (0..self.m).map(|j| self.kunz(index + j, j)).sum()
    }

    /// Number of reflected gaps in residue class `i` (mod `m`): the count of
    /// gaps `x` with `x ≡ i (mod m)` for which `f − x` is also a gap.
    ///
    /// Equals the Kunz coefficient `c(i, j)` where `j = (μ − i) mod m` and
    /// `μ = f mod m = self.mu`. The modular reduction matters because `μ`
    /// may be smaller than `i`; a plain `mu − i` would underflow on `usize`.
    /// Returns 0 when `m < 2` or when `i` is outside `1..m`.
    #[must_use]
    pub fn r_i(&self, i: usize) -> usize {
        if self.m < 2 || i == 0 || i >= self.m {
            return 0;
        }
        let j = (self.mu + self.m - i) % self.m;
        self.kunz(i, j)
    }

    /// ρ(S): smallest `r_i` over residue classes `i ∈ 1..m, i ≠ μ`. The
    /// class `i = μ` is excluded because `r_μ = 0` for every numerical
    /// semigroup (a reflected gap with residue `μ` would have partner
    /// `f − x ≡ 0 (mod m)`, but `0, m, 2m, …` all lie in `S`), so
    /// including it would pin the minimum to 0 unconditionally and carry
    /// no information. Returns 0 when `m < 2` or when `1..m \ {μ}` is
    /// empty (m = 2).
    #[must_use]
    pub fn rho(&self) -> usize {
        (1..self.m)
            .filter(|&i| i != self.mu)
            .map(|i| self.r_i(i))
            .min()
            .unwrap_or(0)
    }
}
