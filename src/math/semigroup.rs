#![warn(clippy::pedantic)]

use std::collections;
use super::compute;

/// All computed properties of a numerical semigroup S = <`gen_set`>.
#[derive(Debug, Clone)]
pub struct Semigroup {
    /// Embedding dimension: number of minimal generators.
    pub e: usize,
    /// Frobenius number: largest integer not in S (-1 conventionally for S = N, but we require gcd=1).
    pub f: usize,
    /// Multiplicity: smallest positive element of S (= smallest generator).
    pub m: usize,
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
}

/// Two semigroups are equal iff they have the same generators, Frobenius number,
/// embedding dimension, and multiplicity.
impl PartialEq for Semigroup {
    fn eq(&self, other: &Self) -> bool {
        self.gen_set == other.gen_set
            && self.f == other.f
            && self.e == other.e
            && self.m == other.m
    }
}

impl Eq for Semigroup {}

/// Partial order by set containment: S1 ≤ S2 iff every element of S1 is also in S2.
/// Returns `None` when neither semigroup is a subset of the other.
impl PartialOrd for Semigroup {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let limit = self.f.max(other.f) + self.m.max(other.m);
        let self_in_other = (1..=limit).all(|i| !self.element(i)  || other.element(i));
        let other_in_self = (1..=limit).all(|i| !other.element(i) || self.element(i));
        match (self_in_other, other_in_self) {
            (true,  true)  => Some(std::cmp::Ordering::Equal),
            (true,  false) => Some(std::cmp::Ordering::Less),
            (false, true)  => Some(std::cmp::Ordering::Greater),
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
    /// # Panics
    /// Panics if the symmetric invariant `f + 1 == 2 * g` is violated (internal consistency check).
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        let sym = self.count_gap == self.count_set;
        assert!(!sym || self.f + 1 == 2 * self.count_gap);
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
    /// # Panics
    /// Panics if the Kunz divisibility invariant is violated (internal consistency check).
    #[must_use]
    pub fn kunz(&self, i: usize, j: usize) -> usize {
        let first = i % self.m;
        let second = j % self.m;
        let idx = (i + j) % self.m;
        let sum = self.apery_set[first] + self.apery_set[second];
        assert!(sum >= self.apery_set[idx]);
        let res = sum - self.apery_set[idx];
        assert_eq!(0, res % self.m, "ai+aj-a(i+j) immer duch m teilbar!");
        res / self.m
    }
    /// Computes PF(S) and the special pseudo-Frobenius numbers.
    ///
    /// Returns `((pf, t), (spf, st))` where:
    /// - `pf` = pseudo-Frobenius numbers: gaps x such that x + s ∈ S for every s ∈ S \ {0}.
    /// - `t`  = |PF(S)| (the type of S).
    /// - `spf` = special PF: elements of PF(S) expressible as gen[i] - gen[j] (i > j) that don't divide f,
    ///   paired with the generator indices `(i, j)`.
    /// - `st` = |SPF(S)|.
    #[must_use]
    #[allow(clippy::type_complexity)]
    pub fn pseudo_and_special(&self) ->
                                     ((Vec<usize>, usize), // PF and its length
                                      (Vec<(usize, (usize, usize))>, usize) // SPF with what diff it is and the length of SPF
                                     )
    {
        let mut pf: Vec<usize> = self
            .blob()
            .into_iter()
            .filter(|&g| self.gen_set.iter().all(|&a| self.element(a + g)))
            .collect();
        pf.push(self.f);

        let t = pf.len();
        let normal_pseudofrobenius = (pf, t);

        // Special PF: elements of PF(S) that equal gen[i]-gen[j] (i>j) and don't divide f
        let pf_set: collections::HashSet<usize> =
            normal_pseudofrobenius.0.iter().copied().collect();
        let mut special_set:Vec<(usize, (usize, usize))>= Vec::new();
        for i in 1..self.gen_set.len() {
            for j in 0..i {
                let diff = self.gen_set[i] - self.gen_set[j];
                if pf_set.contains(&diff) && !self.f.is_multiple_of(diff) {
                    special_set.push((diff,(i,j))    );
                }
            }
        }
        let special: Vec<(usize,(usize,usize))> = special_set.into_iter().collect();
        // todo: sort by the first number x in the pair (x(-,-))
        let st = special.len();
        (normal_pseudofrobenius, (special, st))
    }

    /// Toggle generator `n`: if `n` is a gap, add it as a new generator;
    /// if `n` is a minimal generator, remove it and recompute the generator set.
    /// Returns `self` unchanged if the operation would produce an empty generator set.
    #[must_use]
    pub fn toggle(&self, n: usize) -> Semigroup {
        if self.is_gap(n) {
            let mut newgen = self.gen_set.clone();
            newgen.push(n);
            compute(&newgen)
        } else {
            let is_newgen = |x: usize| {
                (x > n && self.element(x))
                    || (x < n && self.element(x) && !self.element(n - x))
            };
            let newgen: Vec<usize> = (1..=(self.f + self.m))
                .filter(|&x| is_newgen(x))
                .collect();
            if newgen.is_empty() { return self.clone(); }
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
            n if self.element(n) => "in S",
            n if n < self.f && !self.element(self.f - n) => "reflected gap",
            n if !self.element(n) => "gap",
            _ => "unknown"
        }
    }
}
