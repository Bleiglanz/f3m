//! Manipulators: methods on [`Semigroup`] that return another [`Semigroup`].
//!
//! Every method here has the shape `(&self, …) -> Semigroup`. They live in a
//! second `impl Semigroup` block to keep `semigroup.rs` focused on the data
//! type and its inspectors.

use super::Semigroup;
use super::compute;
use super::symmetric_partner::symmetric_partner;

impl Semigroup {
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
    /// TODO: doc need to be rewritten, code changes!
    ///
    /// Ascent: dual of [`Self::descent`], inverting both descent branches.
    ///
    /// 1. **`!is_descent` inverse**: pick the largest minimal generator
    ///    `w` with `m < w < f` and toggle it. Mirrors descent's
    ///    "smallest Apéry > f" rule on `gen_set` instead of `apery_set`.
    /// 2. **`is_descent` inverse** (fallback): if no such `w` exists and
    ///    `f + m` is itself a minimal generator, remove `f + m` as an
    ///    element of `S`. Toggle's standard `1..=(f+m)` enumeration
    ///    range is too narrow here (the new max Apéry of `S \ {f+m}` can
    ///    reach `f + 2m`), so the closure is computed over the elements
    ///    of `S` in `1..=f+2m` minus `f+m`.
    ///
    /// The two clauses match the two cases of [`Self::is_descent_image`]:
    /// `ascent` is non-trivial whenever `is_descent_image()` is true.
    ///
    /// Returns `self` unchanged when neither clause applies.
    ///
    /// #?? Duality with [`Self::descent`]
    #[must_use]
    pub fn ascent(&self) -> Self {
        for k in 1..=self.level {
            let mut atoms = self.gen_set.clone();
            atoms.sort_unstable();
            atoms.reverse();
            if let Some(w) = atoms
                .iter()
                .copied()
                .filter(|&a| k * a > self.m && k * a < self.f && (k * a) + self.m > self.f)
                .filter(|&a| self.apery_set.contains(&(k * a)))
                .max()
            {
                return self.toggle(w);
            }
            if self.gen_set.contains(&(self.f + self.m)) {
                // S \ {f+m}: enumerate up to f + 2m so the next Apéry (≤ f + 2m
                // in residue μ) is included for `compute` to pick up.
                // only because toggle(f+m) would not work!
                let elements: Vec<usize> = (1..=self.f + 2 * self.m)
                    .filter(|&x| x != self.f + self.m && self.element(x))
                    .collect();
                if elements.is_empty() {
                    return self.clone();
                }
                return compute(&elements);
            }
        }
        self.clone()
    }

    /// TODO rewrite this doc - code change
    /// Descent: a controlled step down the gaps ladder.
    ///
    /// Returns `self` when `f < m` (only the trivial `S = ℕ` case).
    /// Otherwise picks the smallest Apéry element `x` with `x > f` and
    /// adds `x − m` (a gap in the same residue class as `x`) as a new
    /// generator.
    ///
    /// The two-branch presentation in the literature (add `f` when
    /// [`Self::is_descent`], otherwise add `x − m` for some `x ∈ (f, f+m)`)
    /// is the same rule: when `is_descent` holds the only Apéry element
    /// above `f` is `a_μ = f+m`, and `(f+m) − m = f`.
    ///
    /// # Duality with [`Self::ascent`]
    ///
    /// `descent` is a left inverse of `ascent` **only** on the subset of
    /// semigroups where `ascent` enters its branch (2): `f + m ∈ gen_set`
    /// and no minimal generator lies in `(f − m, f)`. See [`Self::ascent`]
    /// for the proof sketch and a counterexample.
    #[must_use]
    pub fn descent(&self) -> Self {
        if self.f < self.m {
            return self.clone();
        }
        let mut newgen = self.gen_set.clone();
        // a_μ = f + m is always an Apéry element above f, so this iterator is
        // non-empty whenever m ≥ 1.
        let smallest = *self
            .apery_set
            .iter()
            .filter(|&&x| x > self.f)
            .min()
            .unwrap_or(&0);
        if smallest == self.f + self.m {
            newgen.push(smallest - self.m);
        } else {
            // there exists one smaller than f+m
            let largest_smaller_fpm = *self
                .apery_set
                .iter()
                .filter(|&&x| x > self.f)
                .max()
                .unwrap_or(&0);
            newgen.push(largest_smaller_fpm - self.m);
        }
        compute(&newgen)
    }

    /// Fast descent: collapses every [`Self::descent`] step needed to drop
    /// `f` by exactly `m` into a single closure computation.
    ///
    /// Returns `self` when `f < 2m`. Otherwise extends the generator set
    /// with `x − m` for every Apéry element `x > f`, then closes. The
    /// `x = f + m` case contributes `f` itself; cases with `x ∈ (f, f+m)`
    /// contribute the gap below `x` in the same residue class. The result
    /// satisfies `result.f == self.f − self.m` and `result.mu == self.mu`.
    #[must_use]
    pub fn fast_descent(&self) -> Self {
        if self.f < 2 * self.m {
            return self.clone();
        }
        let mut newgen = self.gen_set.clone();
        // x > f ≥ 2m here, so x − m ≥ m + 1.
        newgen.extend(
            self.apery_set
                .iter()
                .filter(|&&x| x > self.f)
                .map(|&x| x - self.m),
        );
        compute(&newgen)
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
        current_gen_set.extend(self.pf_set.iter().copied().filter(|&x| x != self.f));
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
        symmetric_partner(self)
    }

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
