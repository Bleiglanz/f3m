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

/// Returns the `diag`/`main_diag` tables, U(m), U(m)−(m−1), U(m)·C, and U(m)·C−(m−1)w₁ tables.
// ALLOW: matrix variable names are intentionally similar (they refer to related matrices).
#[allow(clippy::similar_names, clippy::too_many_lines)]
#[wasm_bindgen]
#[must_use]
pub fn js_diagonals_table(s: &JsSemigroup) -> String {
    use crate::math::matrix::{kunz_matrix, mat_mul_unsigned, to_i64, u_matrix};
    use std::fmt::Write as _;
    let sg = &s.0;
    let m = sg.m;
    // ── diag / main_diag columns ─────────────────────────────────────────────
    let build = |header: &str, f: &dyn Fn(usize) -> usize| -> String {
        let mut t = format!(
            "<table class=\"classify-table diagonals-table\">\
             <thead><tr><th>i</th><th>{header}</th></tr></thead><tbody>",
        );
        let mut sum = 0usize;
        for i in 0..m {
            let v = f(i);
            sum += v;
            let _ = write!(t, "<tr><td class=\"cl-n\">{i}</td><td>{v}</td></tr>");
        }
        let _ = write!(
            t,
            "<tr><td class=\"cl-n\"><b>Σ</b></td><td><b>{sum}</b></td></tr>"
        );
        t.push_str("</tbody></table>");
        t
    };
    let minor = build("diag(i)", &|i| sg.diag(i));
    let main = build("main_diag(i)", &|i| sg.main_diag(i));
    // ── helper: render a usize DenseMatrix; `cell` maps an entry value to a <td>…</td> string ──
    let render_mat = |mat: &crate::math::matrix::DenseMatrix<usize>,
                      caption: &str,
                      cell: &dyn Fn(usize) -> String|
     -> String {
        let mut h = format!(
            "<table class=\"classify-table u-matrix-table\">\
             <thead><tr><th>{caption}</th>",
        );
        for j in 0..m {
            let _ = write!(h, "<th>{j}</th>");
        }
        h.push_str("</tr></thead><tbody>");
        for i in 0..m {
            let _ = write!(h, "<tr><th>{i}</th>");
            for j in 0..m {
                h.push_str(&cell(mat[(i, j)]));
            }
            h.push_str("</tr>");
        }
        h.push_str("</tbody></table>");
        h
    };
    let sets = class_sets(sg);
    let plain_cell = |n: usize| format!("<td>{n}</td>");
    let classified_cell = |n: usize| {
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
    };
    // ── helper: render a DenseMatrix<i64> as a plain HTML table ─────────────
    let render_i64_mat = |mat: &crate::math::matrix::DenseMatrix<i64>, caption: &str| -> String {
        let mut h = format!(
            "<table class=\"classify-table u-matrix-table\">\
                 <thead><tr><th>{caption}</th>",
        );
        for j in 0..m {
            let _ = write!(h, "<th>{j}</th>");
        }
        h.push_str("</tr></thead><tbody>");
        for i in 0..m {
            let _ = write!(h, "<tr><th>{i}</th>");
            for j in 0..m {
                let _ = write!(h, "<td>{}</td>", mat[(i, j)]);
            }
            h.push_str("</tr>");
        }
        h.push_str("</tbody></table>");
        h
    };
    let kunz = kunz_matrix(sg);
    let full_u = u_matrix(m);
    let u_i64 = to_i64(&full_u);
    let det_u = crate::math::matrix::Matrix::det(&u_i64);
    let html_u = render_mat(&full_u, &format!("U(m)  det={det_u}"), &plain_cell);
    // U(m) − (m−1): subtract (m−1) from every entry, yielding an i64 matrix.
    // ALLOW: semigroup multiplicity m is always small; wrapping is impossible in practice.
    #[allow(clippy::cast_possible_wrap)]
    let shift = (m - 1) as i64;
    let mut u_shifted = u_i64;
    for i in 0..m {
        for j in 0..m {
            u_shifted[(i, j)] -= shift;
        }
    }
    let det_us = crate::math::matrix::Matrix::det(&u_shifted);
    let html_us = render_i64_mat(&u_shifted, &format!("U(m)−(m−1)  det={det_us}"));
    let mut product_full = mat_mul_unsigned(&full_u, &kunz);
    let html_uc = render_mat(&product_full, "U(m)·C", &plain_cell);
    // Subtract the scalar (m−1)·w₁ from every entry of U·C in-place.
    let sub_val = (m - 1) * sg.apery_set[1];
    for i in 0..m {
        for j in 0..m {
            product_full[(i, j)] = product_full[(i, j)].saturating_sub(sub_val);
        }
    }
    let html_adj = render_mat(&product_full, "U(m)·C − (m−1)w₁", &classified_cell);
    format!("<div class=\"diagonals-pane\">{minor}{main}{html_u}{html_us}{html_uc}{html_adj}</div>")
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
