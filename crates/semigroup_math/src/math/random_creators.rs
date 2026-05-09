//! Randomized creators backed by [`rand::thread_rng`].
//!
//! Every function returns a *raw generator list*; callers run [`compute`]
//! on the result. This matches the wasm/UI flow where the input box
//! keeps showing the user's seed.
//!
//! On wasm32 the entropy source is the browser's `crypto.getRandomValues`,
//! routed through the `getrandom` crate's `js` feature (see `Cargo.toml`).

use rand::Rng;
use rand::seq::SliceRandom;

use super::compute;
use super::creators::PRIMES_LIST;
use super::gcd_vec;
use super::semigroup::Semigroup;

const RAND_LO: usize = 10;
const RAND_HI: usize = 100;
const RAND_COUNT: usize = 8;
const RAND_MATCH_MAX_ATTEMPTS: usize = 10_000;

/// Eight uniformly random integers in `[10, 100]`.
#[must_use]
pub fn random_generators() -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..RAND_COUNT)
        .map(|_| rng.gen_range(RAND_LO..=RAND_HI))
        .collect()
}

/// Generator list that pushes the Frobenius number near `k·m`.
///
/// Returns eight random numbers followed by the block
/// `[k·m, k·m+1, …, k·m + k·m]` (length `k·m + 1`, matching the original
/// JS `randWithMultiplier(k)`). `m` is taken from the random sample
/// directly via `min/gcd` instead of a full [`compute`] peek.
#[must_use]
pub fn random_with_multiplier_generators(k: usize) -> Vec<usize> {
    let nums = random_generators();
    // m = min/gcd matches what compute() would report after gcd-reduction,
    // without paying for the sliding-window run.
    let g = gcd_vec(&nums);
    let m = nums.iter().min().copied().unwrap_or(g) / g;
    let block = k * m;
    let mut gens = nums;
    gens.reserve(block + 1);
    for i in 0..=block {
        gens.push(block + i);
    }
    gens
}

/// Repeatedly draws a fresh [`random_generators`] list and returns the
/// first one whose [`Semigroup`] satisfies `predicate`. Returns `None`
/// after [`RAND_MATCH_MAX_ATTEMPTS`] failures.
#[must_use]
pub fn random_matching_generators(predicate: impl Fn(&Semigroup) -> bool) -> Option<Vec<usize>> {
    for _ in 0..RAND_MATCH_MAX_ATTEMPTS {
        let nums = random_generators();
        if predicate(&compute(&nums)) {
            return Some(nums);
        }
    }
    None
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
