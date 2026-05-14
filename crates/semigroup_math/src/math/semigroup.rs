//! [`Semigroup`] — the central data type.
//!
//! Holds every property computed by [`super::compute`] plus the inspector
//! methods that read derived properties on demand. Methods that produce a
//! new [`Semigroup`] live in [`super::manipulators`].

/// All computed properties of a numerical semigroup S = <`gen_set`>.
#[derive(Debug, Clone)]
pub struct Semigroup {
    /// Embedding dimension: number of minimal generators.
    pub e: usize,
    /// Frobenius number: largest integer not in S (-1 conventionally for S = N, but we require gcd=1).
    pub f: usize,
    /// Multiplicity: smallest positive element of S (= smallest generator).
    pub m: usize,
    /// t = the type of S, the number of pseudo-Frobenius elements t=#PF(S)
    pub t: usize,
    /// r the number of reflected gaps, r=#RG(S) the number of gaps L such that f-L is a gap
    pub r: usize,
    /// es = number of small minimal generators, the ones < f − m
    pub es: usize,
    /// rl = large reflected gaps, gaps L with f − m < L < f (automatically reflected)
    pub rl: usize,
    /// ae = the largest minimal generator, `max(gen_set)`
    pub ae: usize,
    /// ra = number of apery-elements w such that w-m is a reflected gap
    pub ra: usize,
    /// fg = number of fundamental gaps: gaps `n` with every multiple `kn` (k ≥ 2) in S.
    pub fg: usize,
    /// Number of elements of S in the range (0, f] — the "sporadic" elements (= f+1 - genus).
    pub count_set: usize,
    /// Genus: number of gaps (positive integers not in S).
    pub count_gap: usize,
    /// Largest minimal generator.
    pub max_gen: usize,
    /// Sorted list of minimal generators.
    pub gen_set: Vec<usize>,
    /// Apéry set w.r.t. m: `apery_set`[i] is the smallest element of S congruent to i mod m.
    pub apery_set: Vec<usize>,
    /// sum of all the apery-elements
    pub apery_sum: usize,
    /// Pseudo-Frobenius elements, PF(S) are gaps L such that L+S_+ is contained in S
    pub pf_set: Vec<usize>,
    /// the index of the Frobenius f mod m
    pub mu: usize,
    /// The level of `f`: `level * m < f < (level + 1) * m`. Equivalently
    /// `f / m` (integer division), since `f mod m = μ ≥ 1` whenever `m ≥ 2`.
    pub level: usize,
    /// True iff S is symmetric (equivalently, `t = 1`, `f + 1 = 2g`,
    /// `count_gap = count_set`, and every gap `x` has `f − x ∈ S`).
    pub is_symmetric: bool,
    /// True iff S is almost-symmetric (equivalently, `f + t = 2g`,
    /// `ra = r`, and `PF(S) ∖ {f}` equals the set of reflected gaps).
    /// Symmetric semigroups satisfy this trivially.
    pub is_almost_symmetric: bool,
    /// True iff all elements of S in the range `m+1 … 2m−1` are gaps.
    ///
    /// Equivalently, every Apéry element `w_i = apery_set[i]` with `i ∈ 1..m`
    /// satisfies `w_i > 2m`, i.e. every Kunz quotient `q_i = (w_i − i)/m ≥ 2`,
    /// i.e. `m + i` is a gap for every non-zero residue class. Vacuously true
    /// when `m ≤ 1`.
    pub deep: bool,
}

/// Two semigroups are equal iff they have the same generators, Frobenius number,
/// embedding dimension, and multiplicity.
impl PartialEq for Semigroup {
    fn eq(&self, other: &Self) -> bool {
        self.gen_set == other.gen_set && self.f == other.f && self.e == other.e && self.m == other.m
    }
}

/// Eq is empty, just a marker
impl Eq for Semigroup {}
/// Partial order by set containment: S1 ≤ S2 iff every element of S1 is also in S2.
/// Returns `None` when neither semigroup is a subset of the other.
impl PartialOrd for Semigroup {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let limit = self.f.max(other.f) + self.m.max(other.m);
        let self_in_other = (1..=limit).all(|i| !self.element(i) || other.element(i));
        let other_in_self = (1..=limit).all(|i| !other.element(i) || self.element(i));
        match (self_in_other, other_in_self) {
            (true, true) => Some(std::cmp::Ordering::Equal),
            (true, false) => Some(std::cmp::Ordering::Less),
            (false, true) => Some(std::cmp::Ordering::Greater),
            (false, false) => None,
        }
    }
}

impl Semigroup {
    /// Returns `true` if `x` is an element of S.
    /// Uses the Apéry set for O(1) membership: x ∈ S iff x ≥ `apery_set`[x mod m].
    #[must_use]
    pub fn element(&self, x: usize) -> bool {
        let modulus = x % self.m;
        let ap = self.apery_set[modulus];
        x >= ap
    }
    /// Returns `true` if `x` is a gap of S (positive integer not in S).
    #[must_use]
    pub fn is_gap(&self, x: usize) -> bool {
        !self.element(x)
    }
    /// Wilf quotient: `count_set` / (f+1). Wilf's conjecture states this is ≥ 1/e for all S.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn wilf(&self) -> f64 {
        let c = self.f as f64 + 1.0f64;
        let spor = self.count_set as f64;
        spor / c
    }
    /// Returns `true` if `x` is a reflected gap: both `x` and `f - x` are gaps.
    #[must_use]
    pub fn is_reflected_gap(&self, x: usize) -> bool {
        self.is_gap(x) && self.is_gap(self.f - x)
    }
    /// The "blob": sorted list of all reflected gaps (gaps g with f-g also a gap).
    #[must_use]
    pub fn blob(&self) -> Vec<usize> {
        (0..self.f).filter(|&x| self.is_reflected_gap(x)).collect()
    }
    /// Kunz coefficient c(i,j) = (apery[i] + apery[j] - apery[(i+j) mod m]) / m.
    /// Forms a symmetric matrix; row sums equal the Apéry elements.
    ///
    /// The Apéry-divisibility invariant (`a_i + a_j − a_{i+j} ≡ 0 (mod m)` and
    /// `a_i + a_j ≥ a_{i+j}`) is checked in debug builds only; release builds
    /// trust the values computed by [`super::compute`]. Returns 0 if either
    /// invariant is violated, so this function never panics in release.
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

    /// classify a number
    #[must_use]
    pub fn classify(&self, n: usize) -> &str {
        match n {
            0 => "zero",
            n if n == self.m => "m=min(S)",
            n if n == self.f => "f=f(S) Frobenius",
            n if n == self.f + 1 => "c=c(S)=f+1 Conductor",
            n if self.gen_set.contains(&n) => "minimal Generator",
            n if self.apery_set[n % self.m] == n => "in S, Apery",
            n if self.element(n) => "S",
            n if n < self.f && !self.element(self.f - n) => "reflected gap",
            n if !self.element(n) => "gap",
            _ => "unknown",
        }
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
    /// True iff every Apéry
    /// element is either exactly `f + m` or strictly less than `f`.
    ///
    /// Informally: adding `f` to `S` (via closure) is "clean" — the only
    /// Apéry element at or above `f` is `f + m` itself, which then becomes
    /// the new conductor.  Note that this is *not* equivalent to `deep`:
    /// for example `<3, 4, 8>` satisfies `is_descent` but has `4 = m+1 ∈ S`.
    #[must_use]
    pub fn is_descent(&self) -> bool {
        self.apery_set
            .iter()
            .all(|&w| w == self.f + self.m || w < self.f)
    }

    /// True iff `S` lies in the image of [`Self::descent`] — i.e. some
    /// minimal generator falls in the half-open window `(f − m, f)`
    /// (the `!is_descent` regime: descent added it as `x − m` for some
    /// Apéry `x ∈ (f, f+m)`) or equals `f + m` (the `is_descent` regime:
    /// descent added `T.f` which became the new max Apéry element).
    #[must_use]
    pub fn is_descent_image(&self) -> bool {
        let max_apery = self.f + self.m;
        // g + m > f ⇔ g > f − m, underflow-free.
        self.gen_set
            .iter()
            .any(|&g| g == max_apery || (g < self.f && g + self.m > self.f))
    }

    /// True iff the interval `V(S) = {f − m + 1, …, f − 1}` is entirely
    /// contained in `S`. Returns `false` when `f < m` (interval undefined
    /// / out-of-range under `usize`).
    ///
    /// `V` is the "ceiling row" of the Kunz strip just below `f`; when it
    /// is full, the descent `S ∪ {f}` collapses many Apéry elements at
    /// once. Used by the up-down property tests in `tests/integration.rs`.
    #[must_use]
    pub fn v_in_s(&self) -> bool {
        self.f >= self.m && (self.f - self.m + 1..self.f).all(|i| self.element(i))
    }
}
