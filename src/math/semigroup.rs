//! [`Semigroup`] — the central data type.
//!
//! Holds every property computed by [`super::compute`] plus the methods that
//! derive further properties on demand.

use super::compute;

/// Pseudo-Frobenius numbers of a semigroup.
#[derive(Debug, Clone)]
pub struct PseudoSpecial {
    /// Pseudo-Frobenius numbers PF(S): gaps `x` such that `x + s ∈ S` for every `s ∈ S \ {0}`.
    pub pf: Vec<usize>,
    /// Type of S = `pf.len()`.
    pub t: usize,
}

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
    /// ae = the largest minimal generator, max of `gen_set`
    pub ae: usize,
    /// ra = number of apery-elements w such that w-m is a reflected gap
    pub ra: usize,
    /// fg = number of fundamental gaps
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
    /// Returns `true` if S is symmetric (genus == `count_set`, equivalently f+1 = 2·genus).
    ///
    /// The well-known equivalence `g = (f+1)/2` is checked in debug builds only;
    /// release builds trust the value computed by [`super::compute`].
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        let sym = self.count_gap == self.count_set;
        debug_assert!(
            !sym || self.f + 1 == 2 * self.count_gap,
            "symmetric semigroup invariant f+1 = 2g violated",
        );
        sym
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

    /// Returns PF(S) and `t = |PF(S)|`.
    #[must_use]
    pub fn pseudo_and_special(&self) -> PseudoSpecial {
        let pf = self.pf_set.clone();
        let t = pf.len();
        PseudoSpecial { pf, t }
    }

    /// Toggle generator `n`: if `n` is a gap, add it as a new generator;
    /// if `n` is a minimal generator, remove it and recompute the generator set.
    /// Returns `self` unchanged if the operation would produce an empty generator set.
    #[must_use]
    pub fn toggle(&self, n: usize) -> Self {
        if self.is_gap(n) {
            let mut newgen = self.gen_set.clone();
            newgen.push(n);
            compute(&newgen)
        } else {
            let is_newgen = |x: usize| {
                (x > n && self.element(x)) || (x < n && self.element(x) && !self.element(n - x))
            };
            let newgen: Vec<usize> = (1..=(self.f + self.m)).filter(|&x| is_newgen(x)).collect();
            if newgen.is_empty() {
                return self.clone();
            }
            compute(&newgen)
        }
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
    /// Returns S/2 = {n ≥ 0 : 2n ∈ S}, the half of `self`.
    #[must_use]
    pub fn compute_s_over_2(&self) -> Self {
        let new_generators: Vec<usize> = (1..usize::midpoint(self.f, self.m) + 2 * self.m)
            .filter(|&x| self.element(2 * x))
            .collect();
        compute(&new_generators)
    }

    /// Returns the semigroup obtained by adding every pseudo-Frobenius number
    /// other than `f` itself as a generator.
    #[must_use]
    pub fn compute_add_all_pf(&self) -> Self {
        let mut current_gen_set = self.gen_set.clone();
        let mut current_pf: Vec<usize> = self
            .pseudo_and_special()
            .pf
            .into_iter()
            .filter(|&x| x != self.f)
            .collect();
        current_gen_set.append(&mut current_pf);
        compute(&current_gen_set)
    }

    /// Returns the semigroup obtained by adding every reflected gap as a generator.
    #[must_use]
    pub fn compute_add_reflected_gaps(&self) -> Self {
        let mut current_gen_set = self.gen_set.clone();
        let mut current_blob = self.blob();
        current_gen_set.append(&mut current_blob);
        compute(&current_gen_set)
    }

    /// Returns the symmetric partner S̄ such that S = S̄/2 = {n | 2n ∈ S̄}.
    ///
    /// See [`crate::math::symmetric_partner`] for the full construction.
    #[must_use]
    pub fn compute_symmetric_partner(&self) -> Self {
        super::symmetric_partner::symmetric_partner(self)
    }
    //TODO compute K(S)
    //generated by f-z for z\notin S
    //generated by f-pf for all pseudo-frobs
    /// Returns the canonical ideal K(S) as a numerical semigroup.
    ///
    /// The minimal generators of K(S) are `{f − p : p ∈ PF(S), p ≠ f}`.
    /// For symmetric semigroups (where PF(S) = {f}), K(S) = S, so `self` is returned.
    #[must_use]
    pub fn canonical_ideal(&self) -> Self {
        // f is always in PF(S), but f − f = 0 is not a semigroup generator.
        let gens: Vec<usize> = self
            .pf_set
            .iter()
            .filter(|&&p| p != self.f)
            .map(|&p| self.f - p)
            .collect();
        if gens.is_empty() {
            // Symmetric semigroup: K(S) = S by definition.
            return self.clone();
        }
        compute(&gens)
    }

    /// Returns the semigroup generated by `m, w₁ + m, w₂, …, w_{m-1}`,
    /// where `wᵢ = apery_set[i]`. (`m` is included to keep the multiplicity
    /// fixed; the Apéry-only notation `〈w₁+m, w₂, …〉` treats `m` as implicit.)
    ///
    /// Motivation (Kunz cone): adding `(2, 1, 1, …, 1)` to row 1 of the Kunz
    /// matrix `C` gives `U(m) · (2, 1, …, 1)ᵀ = (m, 0, …, 0)ᵀ`, so the
    /// candidate new Apéry vector is `(w₁ + m, w₂, …, w_{m-1})`. That move is
    /// feasible iff every antidiagonal entry `c_{a,b}` with `(a + b) ≡ 1 (mod
    /// m)` and `a, b ≠ 1` is ≥ 1. When the move is feasible the result has
    /// multiplicity `m` and exactly that Apéry vector; when it is blocked, the
    /// resulting semigroup equals the original (its old `w₁` is recovered as a
    /// sum of the remaining generators).
    ///
    /// For `m < 2`, returns `self` unchanged.
    #[must_use]
    pub fn compute_apery_shift_first(&self) -> Self {
        if self.m < 2 {
            return self.clone();
        }
        let mut gens = Vec::with_capacity(self.m);
        gens.push(self.m);
        gens.push(self.apery_set[1] + self.m);
        gens.extend_from_slice(&self.apery_set[2..]);
        compute(&gens)
    }
}
