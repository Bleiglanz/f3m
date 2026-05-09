//! Parameterized creators.
//!
//! Free functions that build a generator list from explicit numeric
//! arguments. Callers run [`super::compute`] on the result.
//!
//! The randomized counterparts live in [`super::random_creators`].

/// Fixed list of primes the "P" / `random_primes_subset` creator samples from.
pub const PRIMES_LIST: &[usize] = &[
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
];

/// `T(m, f)` generator list `[m, f+1, f+2, …, f+m]`.
///
/// Standard construction whose semigroup has multiplicity `m`; the
/// Frobenius number equals `f` whenever `f mod m ≠ 0` (otherwise `f`
/// itself is a sum of `m`s and the Frobenius drops).
#[must_use]
pub fn tmf_generators(m: usize, f: usize) -> Vec<usize> {
    let mut gens = Vec::with_capacity(m + 1);
    gens.push(m);
    for i in 0..m {
        gens.push(f + 1 + i);
    }
    gens
}

/// `A(m, d, n)` generator list `[m, m+d, m+2d, …, m+nd]`.
///
/// Callers should ensure `gcd(m, d) = 1` for a proper numerical
/// semigroup; otherwise [`compute`] divides out the common factor.
#[must_use]
pub fn arith_generators(m: usize, d: usize, n: usize) -> Vec<usize> {
    (0..=n).map(|i| m + i * d).collect()
}

/// "Rolf primes" generator list: `p_n` together with every prime `p`
/// with `p_n < p ≤ 5·p_n`, where `p_1 = 2`. `n` is clamped to `≥ 1`.
#[must_use]
pub fn rolf_primes(n: usize) -> Vec<usize> {
    let idx = n.max(1);
    #[allow(clippy::cast_possible_truncation)]
    let upper = primal::estimate_nth_prime(idx as u64).1 as usize;
    let sieve = primal::Sieve::new(upper * 5);
    let pn = sieve.primes_from(0).nth(idx - 1).unwrap_or(2);
    sieve.primes_from(pn).take_while(|&p| p <= 5 * pn).collect()
}
