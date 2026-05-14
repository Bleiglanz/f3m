//! [`Semigroup`] — the central data type.
//!
//! Holds every property computed by [`super::compute`] plus the inspector
//! methods that read derived properties on demand. The methods are split
//! across three sibling modules by return type:
//!
//! - [`numerical`] — methods returning a single number (`usize` or `f64`).
//! - [`vec_props`] — methods returning a `Vec<usize>` (sub-set views).
//! - [`bool_props`] — predicates returning `bool` (or, for the routing
//!   helper `classify`, a `&str`).
//!
//! Each module re-opens `impl Semigroup` with its own block; doc comments
//! sit on the method bodies where they are defined.
//!
//! Methods that produce a *new* [`Semigroup`] live in
//! [`super::manipulators`].

pub mod bool_props;
pub mod numerical;
pub mod vec_props;

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
    pub sigma: usize,
    /// Genus: number of gaps (positive integers not in S).
    pub g: usize,
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
    /// `g = sigma`, and every gap `x` has `f − x ∈ S`).
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
