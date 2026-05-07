//! WebAssembly bindings for the numerical-semigroup calculator.
//!
//! This crate provides a [`JsSemigroup`] wrapper exposing the [`Semigroup`]
//! struct to JavaScript, plus thin `#[wasm_bindgen]` shims around the views in
//! [`html_helpers`] and the algorithms in [`semigroup_math`]. State that
//! survives across calls (history, view toggles, the evaluator expression)
//! lives in the [`pagestate`] module's `thread_local!` cell.

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr
)]

/// Expression evaluator over semigroup variables (`e`, `f`, `m`, `a[i]`, …).
pub mod js_eval;
/// Hasse-diagram graph data for the Cayley-graph view.
pub mod jsgraph;
/// Global page state owned by the WASM module (history, toggles, expressions).
pub mod pagestate;

use semigroup_math::math::{Semigroup, compute, gap_block};
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;

/// Symbol for the set-containment relation between two semigroups.
/// `Less` → S₁ ⊊ S₂, `Equal` → equal, `Greater` → S₁ ⊋ S₂, `None` → incomparable.
pub(crate) const fn containment_glyph(ord: Option<Ordering>) -> &'static str {
    match ord {
        Some(Ordering::Less) => "\u{2282}",
        Some(Ordering::Equal) => "=",
        Some(Ordering::Greater) => "\u{2283}",
        None => "?",
    }
}

/// Cast a `usize` slice to `Vec<u32>` for WASM transfer.
///
/// Semigroup values are always small (well below `u32::MAX`) so saturation acts
/// as a defensive cap: a hypothetical out-of-range value clamps to `u32::MAX`
/// rather than silently truncating.
fn to_u32(v: &[usize]) -> Vec<u32> {
    v.iter()
        .map(|&x| u32::try_from(x).unwrap_or(u32::MAX))
        .collect()
}

// ── JsSemigroup ──────────────────────────────────────────────────────────────

/// JavaScript-facing wrapper around [`Semigroup`].
///
/// `wasm_bindgen` exports its getters and methods to JS; the inner `Semigroup`
/// stays Rust-side and is never serialised.
#[wasm_bindgen]
#[derive(Debug)]
pub struct JsSemigroup(pub(crate) Semigroup);

// `wasm_bindgen` getters cannot be `const fn` (they cross the FFI boundary).
// `cast_possible_truncation` is acceptable: semigroup values fit comfortably in u32.
#[allow(clippy::cast_possible_truncation, clippy::missing_const_for_fn)]
#[wasm_bindgen]
impl JsSemigroup {
    /// Embedding dimension (number of minimal generators).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn e(&self) -> usize {
        self.0.e
    }
    /// Frobenius number.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn f(&self) -> usize {
        self.0.f
    }
    /// Multiplicity (smallest positive element).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn m(&self) -> usize {
        self.0.m
    }
    /// σ — number of semigroup elements below the conductor f+1.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn count_set(&self) -> usize {
        self.0.count_set
    }
    /// Genus — number of gaps.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn count_gap(&self) -> usize {
        self.0.count_gap
    }
    /// Largest minimal generator.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn max_gen(&self) -> usize {
        self.0.max_gen
    }

    /// Sorted list of minimal generators.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn gen_set(&self) -> Vec<u32> {
        to_u32(&self.0.gen_set)
    }
    /// Apéry set w.r.t. m.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn apery_set(&self) -> Vec<u32> {
        to_u32(&self.0.apery_set)
    }
    /// Reflected gaps (gaps n with f−n also a gap).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn blob(&self) -> Vec<u32> {
        to_u32(&self.0.blob())
    }

    /// True if `x` belongs to the semigroup.
    #[must_use]
    pub fn is_element(&self, x: usize) -> bool {
        self.0.element(x)
    }
    /// Kunz coefficient c(i, j).
    #[must_use]
    pub fn kunz(&self, i: usize, j: usize) -> usize {
        self.0.kunz(i, j)
    }

    /// True if the semigroup is symmetric.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        self.0.is_symmetric
    }
    /// True if the semigroup is almost-symmetric (f + t = 2g).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn is_almost_symmetric(&self) -> bool {
        self.0.is_almost_symmetric
    }
    /// Number of reflected gaps (gaps n with f−n also a gap).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn r(&self) -> usize {
        self.0.r
    }
    /// Wilf quotient σ/(f+1).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn wilf(&self) -> f64 {
        self.0.wilf()
    }

    /// Pseudo-Frobenius numbers PF(S).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn pf(&self) -> Vec<u32> {
        to_u32(&self.0.pf_set)
    }
    /// Type t = |PF(S)|.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn type_t(&self) -> usize {
        self.0.t
    }
    /// Add `n` as a generator if it is a gap, else remove it from the generating set.
    #[must_use]
    pub fn toggle(&self, n: usize) -> Self {
        Self(self.0.toggle(n))
    }

    /// Generators of S/2 = { x : 2x ∈ S }.
    #[must_use]
    pub fn s_over_2(&self) -> Vec<u32> {
        to_u32(&self.0.compute_s_over_2().gen_set)
    }

    /// Generators of the descent of S — a controlled step down the gaps ladder.
    #[must_use]
    pub fn descent(&self) -> Vec<u32> {
        to_u32(&self.0.descent().gen_set)
    }

    /// Generators of S with every pseudo-Frobenius number ≠ f added.
    #[must_use]
    pub fn add_all_pf(&self) -> Vec<u32> {
        to_u32(&self.0.compute_add_all_pf().gen_set)
    }

    /// Generators of S with every reflected gap added.
    #[must_use]
    pub fn add_reflected_gaps(&self) -> Vec<u32> {
        to_u32(&self.0.compute_add_reflected_gaps().gen_set)
    }

    /// Returns the generators of the symmetric partner S̄, where S = S̄/2.
    #[must_use]
    pub fn symmetric_partner(&self) -> Vec<u32> {
        to_u32(&self.0.compute_symmetric_partner().gen_set)
    }

    /// Generators of the canonical ideal K(S).
    #[must_use]
    pub fn canonical_ideal(&self) -> Vec<u32> {
        to_u32(&self.0.canonical_ideal().gen_set)
    }

    /// Generators of `〈m, w₁ + m, w₂, …, w_{m-1}〉`, the Kunz-cone neighbour
    /// reached by adding `(2, 1, …, 1)` to row 1 of the Kunz coefficient matrix.
    #[must_use]
    pub fn apery_shift_first(&self) -> Vec<u32> {
        to_u32(&self.0.compute_apery_shift_first().gen_set)
    }

    /// Returns `true` if the semigroup has a generator coprime to m (i.e. self-gluing is possible).
    #[must_use]
    pub fn can_self_glue(&self) -> bool {
        semigroup_math::math::glue::can_self_glue(&self.0)
    }

    /// Returns the generators of the self-gluing of this semigroup, or an empty
    /// vec if no generator is coprime to m.
    #[must_use]
    pub fn self_glue(&self) -> Vec<u32> {
        semigroup_math::math::glue::self_glue(&self.0)
            .map(|s| to_u32(&s.gen_set))
            .unwrap_or_default()
    }
}

// ── Thin wrappers around html_helpers ────────────────────────────────────────

/// Combined structure-grid + Apéry row + Kunz matrix HTML table.
#[wasm_bindgen]
#[must_use]
pub fn combined_table(s: &JsSemigroup, offset: usize, tilt: i32, show_kunz: bool) -> String {
    html_helpers::combined_table(&s.0, offset, tilt, show_kunz)
}

/// Compact summary row for the properties table.
#[wasm_bindgen]
#[must_use]
pub fn shortprop(s: &JsSemigroup) -> String {
    html_helpers::shortprop(&s.0)
}

/// Flat `<td>` cells for use in the history table row (no nested table, no header).
#[wasm_bindgen]
#[must_use]
pub fn shortprop_tds(s: &JsSemigroup) -> String {
    html_helpers::shortprop_cells(&s.0)
}

/// Sheared x-y grid view (Tilt tab).
#[wasm_bindgen]
#[must_use]
pub fn tilt_table(s: &JsSemigroup, tilt: i32) -> String {
    html_helpers::tilt_table(&s.0, tilt)
}

/// Per-integer classification table (Semigroup tab).
#[wasm_bindgen]
#[must_use]
pub fn js_classify_table(s: &JsSemigroup) -> String {
    html_helpers::classify_table(&s.0)
}

/// Diag-tab tables: U(m), one·U(m)·c₁, `U(m)·C_red`, pair-relations, D(m), zd(m)·c₁.
#[wasm_bindgen]
#[must_use]
pub fn js_diagonals_table(s: &JsSemigroup) -> String {
    html_helpers::diagonals_table(&s.0)
}

// ── Misc wasm exports ────────────────────────────────────────────────────────

/// Returns the set-containment relationship between two semigroups as a symbol:
/// "⊂" (s1 ⊊ s2), "=" (equal), "⊃" (s1 ⊋ s2), or "?" (incomparable).
#[wasm_bindgen]
#[must_use]
pub fn js_cmp_semigroups(s1: &JsSemigroup, s2: &JsSemigroup) -> String {
    containment_glyph(s1.0.partial_cmp(&s2.0)).to_string()
}

/// Return `p_n` and all primes > `p_n` up to `5·p_n` (1-indexed: n=1 → `p_1`=2).
#[wasm_bindgen]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn js_rolf_primes(n: usize) -> Vec<u32> {
    let idx = n.max(1);
    let upper = primal::estimate_nth_prime(idx as u64).1 as usize;
    let sieve = primal::Sieve::new(upper * 5);
    let pn = sieve.primes_from(0).nth(idx - 1).unwrap_or(2);
    sieve
        .primes_from(pn)
        .take_while(|&p| p <= 5 * pn)
        .map(|p| p as u32)
        .collect()
}

/// Return the GAP assertion block for a single semigroup, numbered `idx`.
#[wasm_bindgen]
#[must_use]
pub fn js_gap_block(s: &JsSemigroup, idx: usize) -> String {
    gap_block(&s.0, idx)
}

/// GAP script header (load + package declarations) for the verification script.
#[wasm_bindgen]
#[must_use]
pub fn gap_header() -> String {
    semigroup_math::math::GAP_HEADER.to_string()
}

/// GAP script footer (final assertion-success print).
#[wasm_bindgen]
#[must_use]
pub fn gap_footer() -> String {
    semigroup_math::math::GAP_FOOTER.to_string()
}

/// Parse a comma-separated generator list and compute its semigroup.
///
/// Returns `None` when the input contains no positive integer generators
/// (empty / whitespace-only / all zeros), since [`compute`] requires at least
/// one positive generator.
#[wasm_bindgen]
#[must_use]
pub fn js_compute(input: &str) -> Option<JsSemigroup> {
    let numbers: Vec<usize> = input
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .filter(|&n: &usize| n > 0)
        .collect();
    if numbers.is_empty() {
        return None;
    }
    Some(JsSemigroup(compute(&numbers)))
}
