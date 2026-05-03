//! Colour-class lookup sets and the `<span class="…">n</span>` builder used
//! across every view.

use semigroup_math::math::Semigroup;
use std::collections::HashSet;

/// Pre-built `HashSets` used for O(1) CSS-class lookups across rendering functions.
#[derive(Debug)]
pub struct ClassSets {
    /// Minimal generators of the semigroup.
    pub gens: HashSet<usize>,
    /// Pseudo-Frobenius numbers PF(S).
    pub pf_set: HashSet<usize>,
    /// Reflected gaps (n with f − n also a gap).
    pub blobs: HashSet<usize>,
}

/// Build the three classification sets from a semigroup.
/// Call once per render and pass the result to `crate::combined_table::cell_cls`.
#[must_use]
pub fn class_sets(sg: &Semigroup) -> ClassSets {
    ClassSets {
        gens: sg.gen_set.iter().copied().collect(),
        pf_set: sg.pf_set.iter().copied().collect(),
        blobs: sg.blob().into_iter().collect(),
    }
}

/// Render `n` as an HTML `<span>` with the given CSS class.
/// If `data_n` is true, also adds a `data-n` attribute (used for click-to-toggle in the grid).
#[must_use]
pub fn span(cls: &str, n: usize, data_n: bool) -> String {
    if data_n {
        format!("<span class=\"{cls}\" data-n=\"{n}\">{n}</span>")
    } else {
        format!("<span class=\"{cls}\">{n}</span>")
    }
}
