//! Self-gluing of a numerical semigroup.
//!
//! A **gluing** of two copies of S using scalars α and β produces the semigroup
//! α·S + β·S = ⟨ α·g, β·g  |  g ∈ gen(S) ⟩.
//!
//! The gluing condition requires gcd(α, β) = 1, α ∈ S, and β ∈ S.
//! For the self-gluing implemented here α = m (the multiplicity) and β = x,
//! where x is the first minimal generator of S with gcd(m, x) = 1.

use super::{Semigroup, compute, gcd};

/// Returns the first minimal generator of `s` (other than m) that is coprime to m,
/// or `None` if no such generator exists.
fn find_coprime_gen(s: &Semigroup) -> Option<usize> {
    let m = s.m;
    s.gen_set
        .iter()
        .copied()
        .find(|&g| g != m && gcd(m, g) == 1)
}

/// Returns `true` if the semigroup has a generator coprime to m, i.e. self-gluing is possible.
#[must_use]
pub fn can_self_glue(s: &Semigroup) -> bool {
    find_coprime_gen(s).is_some()
}

/// Returns the self-gluing of `s` using α = m and β = x, where x is the first
/// minimal generator of `s` satisfying gcd(m, x) = 1.
///
/// Returns `None` when no such generator exists (i.e. every generator shares a
/// common factor with m, which cannot happen for a primitive semigroup but is
/// possible for edge cases supplied by the caller).
///
/// # Example
///
/// ```rust
/// use semigroup_math::math::{compute, glue::self_glue};
/// let s = compute(&[3, 5]);
/// let glued = self_glue(&s).unwrap();
/// // generators are 3·{3,5} ∪ 5·{3,5} = {9,15} ∪ {15,25} = {9,15,25}
/// assert_eq!(glued.gen_set, vec![9, 15, 25]);
/// ```
#[must_use]
pub fn self_glue(s: &Semigroup) -> Option<Semigroup> {
    let m = s.m;
    let x = find_coprime_gen(s)?;

    // New generators: m·g for each g in gen(S), plus x·g for each g in gen(S).
    // Duplicates are harmless — compute() deduplicates via the sliding-window.
    let new_gens: Vec<usize> = s.gen_set.iter().flat_map(|&g| [m * g, x * g]).collect();

    Some(compute(&new_gens))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::compute;

    #[test]
    fn self_glue_3_5() {
        // <3,5>: m=3, first x with gcd(3,x)=1 is 5.
        // New gens: 3·{3,5} ∪ 5·{3,5} = {9,15,25}.
        let s = compute(&[3, 5]);
        let g = self_glue(&s).unwrap();
        assert_eq!(g.gen_set, vec![9, 15, 25]);
    }

    #[test]
    fn self_glue_6_9_20() {
        // <6,9,20>: m=6, generators are [6,9,20]. gcd(6,9)=3 ≠ 1, gcd(6,20)=2 ≠ 1.
        // No generator besides m itself is coprime with 6, so None.
        let s = compute(&[6, 9, 20]);
        assert!(self_glue(&s).is_none());
    }

    #[test]
    fn self_glue_4_7() {
        // <4,7>: m=4, first x with gcd(4,x)=1 is 7.
        // New gens: 4·{4,7} ∪ 7·{4,7} = {16,28} ∪ {28,49} = {16,28,49}.
        let s = compute(&[4, 7]);
        let g = self_glue(&s).unwrap();
        assert_eq!(g.gen_set, vec![16, 28, 49]);
    }
}
