#![warn(clippy::pedantic)]
use crate::math::{Semigroup, compute, gap_block};
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
pub mod combined_table;
pub mod js_eval;
pub mod jsgraph;
pub mod pagestate;
pub mod shortprops_table;
pub mod tilt;
pub use shortprops_table::{shortprop, shortprop_tds};

// ── shared helpers ────────────────────────────────────────────────────────────

/// Pre-built `HashSets` used for O(1) CSS-class lookups across rendering functions.
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
        pf_set: sg.pseudo_and_special().0.0.into_iter().collect(),
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

/// Cast a `usize` slice to `Vec<u32>` for WASM transfer (values are always small in practice).
#[allow(clippy::cast_possible_truncation)]
fn to_u32(v: &[usize]) -> Vec<u32> {
    v.iter().map(|&x| x as u32).collect()
}

// ── JsSemigroup ──────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct JsSemigroup(pub(crate) Semigroup);

// Semigroup values are always small; truncation to u32 is safe in practice.
#[allow(clippy::cast_possible_truncation)]
#[wasm_bindgen]
impl JsSemigroup {
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn e(&self) -> usize {
        self.0.e
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn f(&self) -> usize {
        self.0.f
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn m(&self) -> usize {
        self.0.m
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn count_set(&self) -> usize {
        self.0.count_set
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn count_gap(&self) -> usize {
        self.0.count_gap
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn max_gen(&self) -> usize {
        self.0.max_gen
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn gen_set(&self) -> Vec<u32> {
        to_u32(&self.0.gen_set)
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn apery_set(&self) -> Vec<u32> {
        to_u32(&self.0.apery_set)
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn blob(&self) -> Vec<u32> {
        to_u32(&self.0.blob())
    }

    #[must_use]
    pub fn is_element(&self, x: usize) -> bool {
        self.0.element(x)
    }
    #[must_use]
    pub fn kunz(&self, i: usize, j: usize) -> usize {
        self.0.kunz(i, j)
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        self.0.is_symmetric()
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn wilf(&self) -> f64 {
        self.0.wilf()
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn pf(&self) -> Vec<u32> {
        let ((pf, _), _) = self.0.pseudo_and_special();
        to_u32(&pf)
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn type_t(&self) -> usize {
        self.0.pseudo_and_special().0.1
    }
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn special_pf(&self) -> Vec<u32> {
        let (_, (spf, _)) = self.0.pseudo_and_special();
        to_u32(&spf.iter().map(|&(diff, _)| diff).collect::<Vec<_>>())
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn special_pf_str(&self) -> Vec<String> {
        let (_, (spf, _)) = self.0.pseudo_and_special();
        shortprops_table::spf_grouped(&spf, &self.0.gen_set)
    }

    #[must_use]
    pub fn toggle(&self, n: usize) -> JsSemigroup {
        JsSemigroup(self.0.toggle(n))
    }

    #[must_use]
    pub fn s_over_2(&self) -> Vec<u32> {
        to_u32(&self.0.compute_s_over_2().gen_set)
    }

    #[must_use]
    pub fn add_all_pf(&self) -> Vec<u32> {
        to_u32(&self.0.compute_add_all_pf().gen_set)
    }

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

/// Returns the U(m) matrix and U(m)·C product as HTML tables.
#[wasm_bindgen]
#[must_use]
pub fn js_diagonals_table(s: &JsSemigroup) -> String {
    use crate::math::matrix::{kunz_matrix, u_matrix, u_times_kunz};
    use std::fmt::Write as _;
    let sg = &s.0;
    let m = sg.m;
    // Render a DenseMatrix<i64> as an HTML table.
    // `cell` maps (row, col, value) to a <td>…</td> string.
    let render = |mat: &crate::math::matrix::DenseMatrix<i64>,
                  caption: &str,
                  cell: &dyn Fn(usize, usize, i64) -> String|
     -> String {
        let mut html = format!(
            "<table class=\"classify-table u-matrix-table\">\
             <thead><tr><th>{caption}</th>",
        );
        for col in 0..m {
            let _ = write!(html, "<th>{col}</th>");
        }
        html.push_str("</tr></thead><tbody>");
        for row in 0..m {
            let _ = write!(html, "<tr><th>{row}</th>");
            for col in 0..m {
                html.push_str(&cell(row, col, mat[(row, col)]));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");
        html
    };
    let plain = |_row: usize, _col: usize, val: i64| format!("<td>{val}</td>");
    let u = u_matrix(m);
    let html_u = render(&u, "U(m)", &plain);
    // U(m)·C via structure-aware O(m²) formula.
    let kunz = kunz_matrix(sg);
    let product = u_times_kunz(&kunz, m);
    let sets = class_sets(sg);
    let classified = |_row: usize, col: usize, val: i64| {
        if col == 1 && val >= 0 {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let n = val as usize;
            let cls = combined_table::get_cls(
                n,
                false,
                sg.f,
                m,
                &sg.apery_set,
                &sets.gens,
                &sets.pf_set,
                &sets.blobs,
            );
            format!("<td>{}</td>", span(cls, n, true))
        } else {
            format!("<td>{val}</td>")
        }
    };
    let html_uc = render(&product, "U(m)\u{b7}C", &classified);
    let mut out = String::from("<div class=\"diagonals-pane\">");
    out.push_str(&html_u);
    let _ = write!(
        out,
        "<p class=\"det-note\">det(U(m)) = m<sup>m\u{2212}2</sup> = \
         {m}<sup>{}</sup></p>",
        m - 2,
    );
    out.push_str(&html_uc);
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

#[wasm_bindgen]
#[must_use]
pub fn gap_header() -> String {
    crate::math::GAP_HEADER.to_string()
}

#[wasm_bindgen]
#[must_use]
pub fn gap_footer() -> String {
    crate::math::GAP_FOOTER.to_string()
}

#[wasm_bindgen]
#[must_use]
pub fn js_compute(input: &str) -> JsSemigroup {
    let numbers: Vec<usize> = input
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    JsSemigroup(compute(&numbers))
}
