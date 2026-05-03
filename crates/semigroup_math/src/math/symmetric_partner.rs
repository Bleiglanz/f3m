//! Construction of the symmetric partner S̄ of a numerical semigroup S.
//!
//! Theorem 1 of Rosales & García-Sánchez (*Every numerical semigroup is one half
//! of a symmetric numerical semigroup*, Proc. AMS 136, 2008, pp. 475–477) states:
//! for every numerical semigroup S there exists a symmetric numerical semigroup S̄
//! such that S = S̄/2 = {n ∈ ℕ | 2n ∈ S̄}.
//!
//! ## Construction (g ≥ 2, where g = F(S))
//!
//! Let H(S) = {h₁, …, h\_t} be the gaps of S. Define
//!
//! ```text
//! S̄ = 2S  ∪  {4g − 2h − 1 | h ∈ H(S)}  ∪  {n | n ≥ 4g}
//! ```
//!
//! Then F(S̄) = 4g − 1 and S̄ is symmetric with S = S̄/2.
//!
//! **Membership rule** (avoids materialising the full set):
//! - n ≥ 4g → always in S̄
//! - n even → n ∈ S̄ iff n/2 ∈ S  (because only even elements come from 2S)
//! - n odd, n < 4g → n ∈ S̄ iff (4g − 1 − n)/2 is a gap of S and > 0
//!
//! **Special case g = 1:** the only numerical semigroup with F = 1 is ⟨2, 3⟩.
//! The general formula would introduce 1 as a member (making S̄ = ℕ), so the
//! paper handles this separately: S̄ = ⟨4, 5, 6⟩ (F(S̄) = 7, symmetric).

use super::{Semigroup, compute};

/// Returns the symmetric partner S̄ of `s`.
///
/// S̄ is the symmetric numerical semigroup guaranteed by Theorem 1 of
/// Rosales & García-Sánchez (2008), satisfying S = {n ∈ ℕ | 2n ∈ S̄}.
///
/// # Panics
///
/// Cannot panic for any valid `Semigroup` produced by [`super::compute`].
#[must_use]
pub fn symmetric_partner(s: &Semigroup) -> Semigroup {
    let g = s.f;

    // g = 0: S = ℕ (F stored as 0 when input is [1]); already symmetric.
    if g == 0 {
        return compute(&s.gen_set);
    }

    // g = 1: unique semigroup with F = 1 is ⟨2,3⟩.
    // The general formula inserts 1 ∈ S̄ (partner of gap h=1 gives 4·1−2·1−1=1),
    // which would make S̄ = ℕ and break closure. Paper gives S̄ = ⟨4,5,6⟩.
    if g == 1 {
        return compute(&[4, 5, 6]);
    }

    // g ≥ 2: general construction.
    let four_g = 4 * g;

    // Membership predicate for S̄ restricted to 0..4g; everything ≥ 4g is in S̄.
    let in_sbar = |n: usize| -> bool {
        if n >= four_g {
            return true;
        }
        if n.is_multiple_of(2) {
            // n ∈ 2S  iff  n/2 ∈ S
            return s.element(n / 2);
        }
        // n is odd and n < 4g.
        // n ∈ {4g−2h−1 | h gap} iff h = (4g−1−n)/2 is a gap of S and h > 0.
        // (4g−1−n) is even because 4g and 1 are even/odd, n is odd → even−odd=odd
        // wait: 4g (even) − 1 (odd) = odd; odd − n (odd) = even. ✓
        let h = (four_g - 1 - n) / 2;
        h > 0 && s.is_gap(h)
    };

    // Find minimal generators of S̄: elements x ∈ S̄ (1 ≤ x ≤ 4g) that cannot
    // be written as a + b with a, b ∈ S̄, a > 0, b > 0.
    //
    // We scan x in increasing order; for each x ∈ S̄ we check whether any
    // previously-seen positive element a satisfies in_sbar(x − a).
    // Since a ∈ [1, x−1] and x − a ∈ [1, x−1], no underflow occurs.
    let mut sbar_pos: Vec<usize> = Vec::new();
    let mut generators: Vec<usize> = Vec::new();

    for x in 1..=four_g {
        if !in_sbar(x) {
            continue;
        }
        let decomposable = sbar_pos.iter().any(|&a| in_sbar(x - a));
        sbar_pos.push(x);
        if !decomposable {
            generators.push(x);
        }
    }

    compute(&generators)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::compute;

    /// Verify S = S̄/2 and S̄ is symmetric.
    fn check_partner(gens: &[usize]) {
        let s = compute(gens);
        let s_bar = symmetric_partner(&s);

        // S̄ must be symmetric.
        assert!(
            s_bar.is_symmetric(),
            "S̄ not symmetric for S=<{gens:?}>, F(S̄)={}",
            s_bar.f
        );

        // S = S̄/2: check for all n in 0..=F(S̄)/2 + 2.
        let limit = s_bar.f / 2 + 2;
        for n in 0..=limit {
            let in_s = s.element(n);
            let in_s_bar_2 = s_bar.element(2 * n);
            assert_eq!(
                in_s, in_s_bar_2,
                "S ≠ S̄/2 at n={n} for S=<{gens:?}>: element(n)={in_s}, element(2n)={in_s_bar_2}"
            );
        }
    }

    #[test]
    fn test_partner_3_5() {
        // ⟨3,5⟩ → S̄ = ⟨6,10,13⟩  (F(S̄)=27=4·7−1)
        let s = compute(&[3, 5]);
        let s_bar = symmetric_partner(&s);
        assert_eq!(s_bar.f, 4 * s.f - 1);
        check_partner(&[3, 5]);
    }

    #[test]
    fn test_partner_4_6_11() {
        check_partner(&[4, 6, 11]);
    }

    #[test]
    fn test_partner_6_9_20() {
        check_partner(&[6, 9, 20]);
    }

    #[test]
    fn test_partner_symmetric_is_self() {
        // A symmetric semigroup ⟨3,5⟩ is already symmetric, but its partner
        // S̄ (over S̄/2 = S) is a new symmetric semigroup one level up.
        // Just verify the round-trip invariant holds.
        check_partner(&[3, 5]);
        check_partner(&[5, 7]); // symmetric (F=23)
        check_partner(&[4, 5, 7]); // symmetric
    }

    #[test]
    fn test_partner_g1() {
        // g = 1: S = ⟨2,3⟩, S̄ = ⟨4,5,6⟩
        let s = compute(&[2, 3]);
        let s_bar = symmetric_partner(&s);
        assert_eq!(s_bar.gen_set, vec![4, 5, 6]);
        check_partner(&[2, 3]);
    }

    #[test]
    fn test_partner_large() {
        check_partner(&[13, 17, 27]);
        check_partner(&[10, 21, 22, 23]);
    }
}
