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

use super::creators::PRIMES_LIST;

const RAND_LO: usize = 10;
const RAND_HI: usize = 100;
const RAND_COUNT: usize = 8;

/// Eight uniformly random integers in `[10, 100]`.
#[must_use]
pub fn random_generators() -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..RAND_COUNT)
        .map(|_| rng.gen_range(RAND_LO..=RAND_HI))
        .collect()
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
