//! WebAssembly bindings: `JsSemigroup`, HTML rendering helpers, and the global
//! `PageState` (in `pagestate`). All `#[wasm_bindgen]` exports live here.

use crate::math::{Semigroup, compute, gap_block};
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

/// Combined structure-grid + Apéry row + Kunz matrix HTML table.
pub mod combined_table;
/// Expression evaluator over semigroup variables (`e`, `f`, `m`, `a[i]`, …).
pub mod js_eval;
/// Hasse-diagram graph data for the Cayley-graph view.
pub mod jsgraph;
/// Global page state owned by the WASM module (history, toggles, expressions).
pub mod pagestate;
/// "Short props" table on the Semigroup tab.
pub mod shortprops_table;
/// Tilted x-y grid for the Tilt tab.
pub mod tilt;

pub use shortprops_table::{shortprop, shortprop_tds};

// ── shared helpers ────────────────────────────────────────────────────────────

/// Pre-built `HashSets` used for O(1) CSS-class lookups across rendering functions.
#[derive(Debug)]
pub(super) struct ClassSets {
    pub gens: HashSet<usize>,   // minimal generators
    pub pf_set: HashSet<usize>, // pseudo-Frobenius numbers
    pub blobs: HashSet<usize>,  // reflected gaps (blob)
}

/// Build the three classification sets from a semigroup.
/// Call once per render and pass the result to `get_cls` / `span`.
pub(super) fn class_sets(sg: &Semigroup) -> ClassSets {
    ClassSets {
        gens: sg.gen_set.iter().copied().collect(),
        pf_set: sg.pseudo_and_special().pf.into_iter().collect(),
        blobs: sg.blob().into_iter().collect(),
    }
}

/// Render `n` as an HTML `<span>` with the given CSS class.
/// If `data_n` is true, also adds a `data-n` attribute (used for click-to-toggle in the grid).
pub(super) fn span(cls: &str, n: usize, data_n: bool) -> String {
    if data_n {
        format!("<span class=\"{cls}\" data-n=\"{n}\">{n}</span>")
    } else {
        format!("<span class=\"{cls}\">{n}</span>")
    }
}

/// Render `=<span sg-gen>a</span>-<span sg-gen>b</span>` for one SPF generator pair.
/// If `data_n` is true, both spans get `data-n` attributes for click-to-toggle.
pub(super) fn spf_pair(gen_i: usize, gen_j: usize, data_n: bool) -> String {
    format!(
        "={}-{}",
        span("sg-gen", gen_i, data_n),
        span("sg-gen", gen_j, data_n)
    )
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
        self.0.is_symmetric()
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
        to_u32(&self.0.pseudo_and_special().pf)
    }
    /// Type t = |PF(S)|.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn type_t(&self) -> usize {
        self.0.pseudo_and_special().t
    }
    /// Special pseudo-Frobenius numbers (PF that arise as a generator difference).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn special_pf(&self) -> Vec<u32> {
        let diffs: Vec<usize> = self
            .0
            .pseudo_and_special()
            .special
            .iter()
            .map(|&(diff, _)| diff)
            .collect();
        to_u32(&diffs)
    }

    /// Special pseudo-Frobenius numbers grouped as printable `=a−b` strings.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn special_pf_str(&self) -> Vec<String> {
        let ps = self.0.pseudo_and_special();
        shortprops_table::spf_grouped(&ps.special, &self.0.gen_set)
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
    ///
    /// See `crate::math::symmetric_partner` for the construction (Rosales–García-Sánchez 2008).
    #[must_use]
    pub fn symmetric_partner(&self) -> Vec<u32> {
        to_u32(&self.0.compute_symmetric_partner().gen_set)
    }

    /// Generators of the canonical ideal K(S).
    ///
    /// The minimal generators are `{f − p : p ∈ PF(S), p ≠ f}`.
    /// For symmetric S (PF = {f}), K(S) = S and the same generators are returned.
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
        crate::math::glue::can_self_glue(&self.0)
    }

    /// Returns the generators of the self-gluing of this semigroup (α = m, β = first
    /// generator coprime to m), or an empty vec if no such generator exists.
    #[must_use]
    pub fn self_glue(&self) -> Vec<u32> {
        crate::math::glue::self_glue(&self.0)
            .map(|s| to_u32(&s.gen_set))
            .unwrap_or_default()
    }
}

/// Returns the set-containment relationship between two semigroups as a symbol:
/// "⊂" (s1 ⊊ s2), "=" (equal), "⊃" (s1 ⊋ s2), or "?" (incomparable).
#[wasm_bindgen]
#[must_use]
pub fn js_cmp_semigroups(s1: &JsSemigroup, s2: &JsSemigroup) -> String {
    match s1.0.partial_cmp(&s2.0) {
        Some(std::cmp::Ordering::Less) => "⊂".to_string(),
        Some(std::cmp::Ordering::Equal) => "=".to_string(),
        Some(std::cmp::Ordering::Greater) => "⊃".to_string(),
        None => "?".to_string(),
    }
}

/// Returns an HTML table mapping each integer 0..=f+m to its classification,
/// with a "Diff" column showing all representations of n as a difference of
/// two Apéry elements: `w_i` − `w_j` = n.
#[wasm_bindgen]
#[must_use]
pub fn js_classify_table(s: &JsSemigroup) -> String {
    use std::collections::HashMap;
    use std::fmt::Write as _;
    let sg = &s.0;
    let sets = class_sets(sg);
    let cls_of = |n| {
        combined_table::get_cls(
            n,
            false,
            sg.f,
            sg.m,
            &sg.apery_set,
            &sets.gens,
            &sets.pf_set,
            &sets.blobs,
        )
    };

    // Build a map: difference → list of "w_i−w_j" expression strings.
    // Skip j=0 (trivial w_i−0 = w_i). Use sg-gen style for Apéry elements
    // that are also minimal generators.
    let ap = &sg.apery_set;
    let ap_cls = |v: usize| {
        if sets.gens.contains(&v) {
            "sg-gen"
        } else {
            "sg-apery"
        }
    };
    let mut apery_diffs: HashMap<usize, String> = HashMap::new();
    for (i, &wi) in ap.iter().enumerate().skip(1) {
        for (j, &wj) in ap.iter().enumerate().skip(1) {
            if i != j && wi > wj {
                let diff = wi - wj;
                let entry = apery_diffs.entry(diff).or_default();
                if !entry.is_empty() {
                    entry.push(' ');
                }
                let _ = write!(
                    entry,
                    "{}−{}",
                    span(ap_cls(wi), wi, false),
                    span(ap_cls(wj), wj, false),
                );
            }
        }
    }

    let mut out = String::from(
        "<table class=\"classify-table\">\
         <thead><tr><th>n</th><th>class</th><th>Diff</th></tr></thead>\
         <tbody>",
    );
    for n in 0..=(sg.f + sg.m) {
        let n_span = span(cls_of(n), n, true);
        let label = sg.classify(n);
        let cls = match label {
            "zero" => "cl-zero",
            "in S"
            | "in S, Apery"
            | "m=min(S)"
            | "minimal Generator"
            | "f=f(S) Frobenius"
            | "c=c(S)=f+1 Conductor" => "cl-in",
            "reflected gap" => "cl-reflect",
            _ => "cl-gap",
        };
        let diff_cell = apery_diffs.get(&n).map_or("", String::as_str);
        let _ = write!(
            out,
            "<tr><td class=\"cl-n\">{n_span}</td><td class=\"{cls}\">{label}</td>\
             <td class=\"cl-diff\">{diff_cell}</td></tr>",
        );
    }
    out.push_str("</tbody></table>");
    out
}

/// Returns U(m), `U(m)·C_red`, and the U(m) pair-relations matrix as HTML tables.
/// Render a five-row table for `one`, `one·U(m)`, `c₁`, `one·U(m)·c₁`, and `mg+(m-1)m/2`.
///
/// - `one_u[j]`   = column-j sum of U(m) (the product `one·U(m)`)
/// - `c1[i]`      = first column of `C_red` (Kunz coefficients `c(i+1, 1)`)
/// - `apery_sum`  = m·g + m·(m−1)/2 = sum of Apéry elements w₁,…,w_{m−1}
/// - rows 4 and 5 each span all data columns; they should always be equal.
fn render_one_vec(one_u: &[i64], c1: &[i64], apery_sum: i64) -> String {
    use std::fmt::Write as _;
    let dim = one_u.len();
    let scalar: i64 = one_u.iter().zip(c1.iter()).map(|(&u, &c)| u * c).sum();
    let mut html = "<table class=\"classify-table u-matrix-table\">\
         <thead><tr><th></th>"
        .to_string();
    for b in 0..dim {
        let _ = write!(html, "<th>{}</th>", b + 1);
    }
    html.push_str("</tr></thead><tbody><tr><th>one</th>");
    for _ in 0..dim {
        html.push_str("<td>1</td>");
    }
    html.push_str("</tr><tr><th>one\u{b7}U(m)</th>");
    for &v in one_u {
        let _ = write!(html, "<td>{v}</td>");
    }
    html.push_str("</tr><tr><th>c<sub>1</sub></th>");
    for &v in c1 {
        let _ = write!(html, "<td>{v}</td>");
    }
    let _ = write!(
        html,
        "</tr><tr><th>one\u{b7}U(m)\u{b7}c<sub>1</sub></th>\
         <td colspan=\"{dim}\">{scalar}</td>\
         </tr><tr><th>mg + (m\u{2212}1)m/2</th>\
         <td colspan=\"{dim}\">{apery_sum}</td>",
    );
    html.push_str("</tr></tbody></table>");
    html
}

/// HTML table for the Diag tab: `U(m)`, `one·U(m)`, `U(m)·C_red`, pair-relations.
// ALLOW: pure HTML-builder; each block renders one distinct table — splitting further
// would fragment the rendering pipeline without reducing logical complexity.
#[allow(clippy::too_many_lines)]
#[wasm_bindgen]
#[must_use]
pub fn js_diagonals_table(s: &JsSemigroup) -> String {
    use crate::math::matrix::{
        DenseMatrix, Matrix, c_red, u_matrix, u_pair_relations, u_times_c_red,
    };
    use std::fmt::Write as _;
    let sg = &s.0;
    let mult = sg.m;
    // Render a DenseMatrix<i64> with custom row/col header labels and cell formatter.
    let render = |mat: &DenseMatrix<i64>,
                  caption: &str,
                  row_label: &dyn Fn(usize) -> String,
                  col_label: &dyn Fn(usize) -> String,
                  cell: &dyn Fn(usize, usize, i64) -> String|
     -> String {
        let mut html = format!(
            "<table class=\"classify-table u-matrix-table\">\
             <thead><tr><th>{caption}</th>",
        );
        for b in 0..mat.ncols() {
            let _ = write!(html, "<th>{}</th>", col_label(b));
        }
        html.push_str("</tr></thead><tbody>");
        for a in 0..mat.nrows() {
            let _ = write!(html, "<tr><th>{}</th>", row_label(a));
            for b in 0..mat.ncols() {
                html.push_str(&cell(a, b, mat[(a, b)]));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");
        html
    };
    let plain = |_a: usize, _b: usize, val: i64| format!("<td>{val}</td>");
    let one_based = |i: usize| (i + 1).to_string();
    let dim = mult - 1;
    // Lex-ordered labels for pairs (i, j) with 1 ≤ i ≤ j ≤ m−1.
    let pair_labels: Vec<String> = (0..dim)
        .flat_map(|a| (a..dim).map(move |b| format!("({},{})", a + 1, b + 1)))
        .collect();
    let pair_label = |r: usize| pair_labels[r].clone();
    let u_mat = u_matrix(mult);
    let html_u = render(&u_mat, "U(m)", &one_based, &one_based, &plain);
    let one_u: Vec<i64> = (0..dim)
        .map(|j| (0..dim).map(|a| u_mat[(a, j)]).sum())
        .collect();
    let cr = c_red(sg);
    #[allow(clippy::cast_possible_wrap)] // Kunz coefficients are always small
    let c1: Vec<i64> = (0..dim).map(|i| cr[(i, 0)] as i64).collect();
    // Selmer's formula: sum of Apéry elements w₁…w_{m−1} = m·g + m·(m−1)/2
    #[allow(clippy::cast_possible_wrap)]
    let apery_sum = (mult * sg.count_gap + mult * (mult - 1) / 2) as i64;
    let html_one = render_one_vec(&one_u, &c1, apery_sum);
    let product = u_times_c_red(&cr);
    let sets = class_sets(sg);
    let classified = |_a: usize, b: usize, val: i64| {
        if b == 0 && val >= 0 {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let v = val as usize;
            let cls = combined_table::get_cls(
                v,
                false,
                sg.f,
                mult,
                &sg.apery_set,
                &sets.gens,
                &sets.pf_set,
                &sets.blobs,
            );
            format!("<td>{}</td>", span(cls, v, true))
        } else {
            format!("<td>{val}</td>")
        }
    };
    let html_uc = render(
        &product,
        "U(m)\u{b7}C_red",
        &one_based,
        &one_based,
        &classified,
    );
    let pair = u_pair_relations(mult);
    let pair_cell = |_a: usize, _b: usize, val: i64| match val {
        0 => "<td class=\"pm-zero\">0</td>".to_string(),
        1 => "<td class=\"pm-pos1\">1</td>".to_string(),
        -1 => "<td class=\"pm-neg1\">\u{2212}1</td>".to_string(),
        _ => format!("<td>{val}</td>"),
    };
    let html_pair = render(
        &pair,
        "(U_i+U_j\u{2212}U_{i+j})/m",
        &pair_label,
        &one_based,
        &pair_cell,
    );

    let mut out = String::from("<div class=\"diagonals-pane\">");
    let _ = write!(out, "<div class=\"table-wrap\">{html_u}</div>");
    let _ = write!(
        out,
        "<p class=\"det-note\">det(U(m)) = m<sup>m\u{2212}2</sup> = \
         {mult}<sup>{}</sup></p>",
        mult - 2,
    );
    let _ = write!(out, "<div class=\"table-wrap\">{html_one}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_uc}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_pair}</div>");
    out.push_str("</div>");
    out
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
    crate::math::GAP_HEADER.to_string()
}

/// GAP script footer (final assertion-success print).
#[wasm_bindgen]
#[must_use]
pub fn gap_footer() -> String {
    crate::math::GAP_FOOTER.to_string()
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
