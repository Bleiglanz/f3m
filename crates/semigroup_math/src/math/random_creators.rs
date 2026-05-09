//! Randomized creators: `() -> Semigroup` constructions backed by
//! [`rand::thread_rng`].
//!
//! Each function pair exposes the raw generator list (`*_generators`) and
//! the computed [`Semigroup`]. The wasm bindings call the `*_generators`
//! variants so the input box keeps showing the same numbers the user sees
//! when "Compute" is hit on that text — preserving the prior UX.
//!
//! On wasm32 the entropy source is the browser's `crypto.getRandomValues`,
//! routed through the `getrandom` crate's `js` feature (see `Cargo.toml`).

use rand::Rng;
use rand::seq::SliceRandom;

use super::creators::PRIMES_LIST;
use super::{Semigroup, compute};

/// Inclusive lower bound for entries produced by [`random_generators`].
const RAND_LO: usize = 10;
/// Inclusive upper bound for entries produced by [`random_generators`].
const RAND_HI: usize = 100;
/// Number of integers produced by [`random_generators`].
const RAND_COUNT: usize = 8;
/// Cap on retries for predicate-matching helpers like [`random_symmetric`].
const RAND_MATCH_MAX_ATTEMPTS: usize = 10_000;

/// Eight uniformly random integers in `[10, 100]`. Mirrors the JavaScript
/// `randNums()` helper — the seed for every other random creator here.
#[must_use]
pub fn random_generators() -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..RAND_COUNT)
        .map(|_| rng.gen_range(RAND_LO..=RAND_HI))
        .collect()
}

/// Random semigroup: [`compute`] applied to [`random_generators`].
#[must_use]
pub fn random() -> Semigroup {
    compute(&random_generators())
}

/// Generator list that pushes the Frobenius number near `k·m`.
///
/// Returns eight random numbers followed by the block
/// `[k·m, k·m+1, …, k·m + k·m]` (length `k·m + 1`, matching the original
/// JS `randWithMultiplier(k)`). `m` is the multiplicity of the random
/// sample.
#[must_use]
pub fn random_with_multiplier_generators(k: usize) -> Vec<usize> {
    let nums = random_generators();
    let peek = compute(&nums);
    let block = k * peek.m;
    let mut gens = nums;
    gens.reserve(block + 1);
    for i in 0..=block {
        gens.push(block + i);
    }
    gens
}

/// [`compute`] applied to [`random_with_multiplier_generators`].
#[must_use]
pub fn random_with_multiplier(k: usize) -> Semigroup {
    compute(&random_with_multiplier_generators(k))
}

/// Repeatedly draws a fresh [`random_generators`] list, computes its
/// semigroup, and returns the first generator list whose semigroup
/// satisfies `predicate`. Returns `None` after
/// [`RAND_MATCH_MAX_ATTEMPTS`] failures.
fn random_matching_generators(predicate: impl Fn(&Semigroup) -> bool) -> Option<Vec<usize>> {
    for _ in 0..RAND_MATCH_MAX_ATTEMPTS {
        let nums = random_generators();
        if predicate(&compute(&nums)) {
            return Some(nums);
        }
    }
    None
}

/// Generators of a random symmetric semigroup (`t = 1`), or `None`.
#[must_use]
pub fn random_symmetric_generators() -> Option<Vec<usize>> {
    random_matching_generators(|s| s.is_symmetric)
}

/// Random symmetric semigroup, or `None` if no sample was found.
#[must_use]
pub fn random_symmetric() -> Option<Semigroup> {
    random_symmetric_generators().map(|g| compute(&g))
}

/// Generators of a random pseudo-symmetric semigroup (`r = 1`), or `None`.
#[must_use]
pub fn random_pseudo_symmetric_generators() -> Option<Vec<usize>> {
    random_matching_generators(|s| s.r == 1)
}

/// Random pseudo-symmetric semigroup (`r = 1`), or `None`.
#[must_use]
pub fn random_pseudo_symmetric() -> Option<Semigroup> {
    random_pseudo_symmetric_generators().map(|g| compute(&g))
}

/// Generators of a random proper almost-symmetric semigroup (almost-symmetric
/// with `r ≥ 2`, excluding the symmetric and pseudo-symmetric cases), or `None`.
#[must_use]
pub fn random_almost_symmetric_generators() -> Option<Vec<usize>> {
    random_matching_generators(|s| s.is_almost_symmetric && s.r >= 2)
}

/// Random proper almost-symmetric semigroup, or `None`.
#[must_use]
pub fn random_almost_symmetric() -> Option<Semigroup> {
    random_almost_symmetric_generators().map(|g| compute(&g))
}

/// 4 to 8 primes drawn uniformly at random (without replacement) from
/// [`PRIMES_LIST`], returned in increasing order.
#[must_use]
pub fn random_primes_subset() -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(4..=8);
    let mut chosen: Vec<usize> = PRIMES_LIST
        .choose_multiple(&mut rng, count)
        .copied()
        .collect();
    chosen.sort_unstable();
    chosen
}

/// Random "P" semigroup: [`compute`] applied to [`random_primes_subset`].
#[must_use]
pub fn random_primes() -> Semigroup {
    compute(&random_primes_subset())
}
