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

    /// True iff every Apéry element is either exactly `f + m` or strictly
    /// less than `f`.
    ///
    /// Informally: adding `f` to `S` (via closure) is "clean" — the only
    /// Apéry element at or above `f` is `f + m` itself, which then becomes
    /// the new conductor. Note that this is *not* equivalent to `deep`:
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
