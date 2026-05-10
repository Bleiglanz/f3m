//! Strata chains: `M_0 ⊆ M_1 ⊆ … ⊆ M_lmax ⊆ {1, …, N}`.
//!
//! Used by the standalone strata-explorer page. A chain is just a `Vec<Vec<usize>>`
//! of length `lmax + 1`, with each inner vector sorted ascending.

use rand::Rng;

/// Generate a random monotonic chain of length `lmax + 1`.
///
/// `M_0` is always empty. `M_1` is a uniformly random subset of `{1, …, N}`
/// drawn at probability ≈ 0.4 per element. Each subsequent level adds further
/// elements not yet present at probability ≈ 0.25, so sets only grow.
#[must_use]
pub fn random_strata(n: usize, lmax: usize) -> Vec<Vec<usize>> {
    let mut chain: Vec<Vec<usize>> = Vec::with_capacity(lmax + 1);
    chain.push(Vec::new()); // M_0 = ∅
    if lmax == 0 || n == 0 {
        for _ in 1..=lmax {
            chain.push(Vec::new());
        }
        return chain;
    }

    let mut rng = rand::thread_rng();
    let mut present = vec![false; n];

    // M_1: random subset.
    for slot in &mut present {
        if rng.gen_bool(0.4) {
            *slot = true;
        }
    }
    chain.push(snapshot(&present));

    // M_2 … M_lmax: monotonically grow.
    for _ in 2..=lmax {
        for slot in &mut present {
            if !*slot && rng.gen_bool(0.25) {
                *slot = true;
            }
        }
        chain.push(snapshot(&present));
    }

    chain
}

fn snapshot(present: &[bool]) -> Vec<usize> {
    present
        .iter()
        .enumerate()
        .filter_map(|(i, &p)| if p { Some(i + 1) } else { None })
        .collect()
}

/// Encode a chain as `;`-separated rows of `,`-separated values.
///
/// Empty rows produce empty strings between separators, so a chain of length
/// `lmax + 1` always encodes to exactly `lmax` semicolons.
#[must_use]
pub fn encode_chain(chain: &[Vec<usize>]) -> String {
    chain
        .iter()
        .map(|row| {
            row.iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Decode a chain string produced by [`encode_chain`].
///
/// Tokens that fail to parse are silently dropped, matching the tolerant
/// parsing used elsewhere for user-facing inputs.
#[must_use]
pub fn decode_chain(s: &str) -> Vec<Vec<usize>> {
    s.split(';')
        .map(|row| {
            if row.is_empty() {
                Vec::new()
            } else {
                row.split(',')
                    .filter_map(|t| t.trim().parse().ok())
                    .collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let chain = vec![vec![], vec![1, 3], vec![1, 3, 5], vec![1, 2, 3, 5]];
        let s = encode_chain(&chain);
        assert_eq!(s, ";1,3;1,3,5;1,2,3,5");
        assert_eq!(decode_chain(&s), chain);
    }

    #[test]
    fn empty_chain_encodes_to_separator_string() {
        let chain = vec![vec![], vec![], vec![], vec![]];
        assert_eq!(encode_chain(&chain), ";;;");
        assert_eq!(decode_chain(";;;"), chain);
    }

    #[test]
    fn random_strata_is_monotone() {
        for _ in 0..50 {
            let chain = random_strata(8, 5);
            assert_eq!(chain.len(), 6);
            assert!(chain[0].is_empty());
            for w in chain.windows(2) {
                let prev: std::collections::HashSet<_> = w[0].iter().copied().collect();
                let curr: std::collections::HashSet<_> = w[1].iter().copied().collect();
                assert!(prev.is_subset(&curr));
            }
            for row in &chain {
                assert!(row.iter().all(|&v| (1..=8).contains(&v)));
            }
        }
    }

    #[test]
    fn random_strata_handles_zero_inputs() {
        assert_eq!(random_strata(0, 3).len(), 4);
        assert_eq!(random_strata(5, 0).len(), 1);
    }
}
