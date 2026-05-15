//! Boolean inspectors: membership tests and structural predicates.
//!
//! Also hosts the routing helper [`Semigroup::classify`], which returns a
//! short `&str` label naming which structural role a number plays in S.
//! It lives here because it is a categorisation predicate (the cases are
//! mutually exclusive boolean checks), even though its return type is
//! `&str` rather than `bool`.

use super::Semigroup;

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

    /// Returns `true` if `x` is a reflected gap: both `x` and `f - x` are gaps.
    #[must_use]
    pub fn is_reflected_gap(&self, x: usize) -> bool {
        self.is_gap(x) && self.is_gap(self.f - x)
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

    /// Classify a number: short `&str` label naming which structural role
    /// `n` plays in S (Frobenius, conductor, atom, Apéry, plain element,
    /// reflected gap, ordinary gap, …).
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
}
