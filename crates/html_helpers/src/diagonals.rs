//! HTML tables for the Diag tab: U(m), `U(m)·C_red`, pair-relations, D(m), zd(m).

use crate::combined_table::cell_cls;
use crate::spans::{class_sets, span};
use semigroup_math::math::Semigroup;
use semigroup_math::math::matrix::{
    DenseMatrix, Matrix, c_red, d_matrix, u_matrix, u_pair_relations, u_times_c_red, v_matrix,
    zd_vector,
};
use std::fmt::Write as _;

/// Renders a single `<td>` for an integer cell that takes one of the three
/// small values `{−1, 0, +1}` (matrices like `u_pair_relations(m)` and
/// `RSym`). Other values fall back to plain decimal rendering.
fn pm_cell(val: i64) -> String {
    match val {
        0 => "<td class=\"pm-zero\">0</td>".to_string(),
        1 => "<td class=\"pm-pos1\">1</td>".to_string(),
        -1 => "<td class=\"pm-neg1\">\u{2212}1</td>".to_string(),
        _ => format!("<td>{val}</td>"),
    }
}

/// Render a five-row table for `one`, `one·U(m)`, `c₁`, `one·U(m)·c₁`, and `mg+(m-1)m/2`.
fn render_one_vec(one_u: &[i64], c1: &[i64], apery_sum: i64) -> String {
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

/// Renders the D(m)·c₁ section: the zd row, the d-vector, and the f+m+r identity.
fn render_d_prod(
    zd: &[i64],
    d_prod: &[i64],
    c1: &[i64],
    zd_dot: i64,
    f_plus_m_plus_r: i64,
    two_g_plus_m_minus_1: i64,
) -> String {
    let dim = d_prod.len();
    let mut html = "<table class=\"classify-table u-matrix-table\">\
         <thead><tr><th></th>"
        .to_string();
    for b in 0..dim {
        let _ = write!(html, "<th>{}</th>", b + 1);
    }
    html.push_str("</tr></thead><tbody>");
    html.push_str("<tr><th>c<sub>1</sub></th>");
    for &v in c1 {
        let _ = write!(html, "<td>{v}</td>");
    }
    html.push_str("</tr><tr><th>zd(m)</th>");
    for &v in zd {
        let _ = write!(html, "<td>{v}</td>");
    }
    html.push_str("</tr><tr><th>D(m)\u{b7}c<sub>1</sub></th>");
    for &v in d_prod {
        let _ = write!(html, "<td>{v}</td>");
    }
    let _ = write!(
        html,
        "</tr><tr><th>zd(m)\u{b7}c<sub>1</sub></th>\
         <td colspan=\"{dim}\">{zd_dot}</td>\
         </tr><tr><th>f+m+r</th>\
         <td colspan=\"{dim}\">{f_plus_m_plus_r}</td>\
         </tr><tr><th>2g+m\u{2212}1</th>\
         <td colspan=\"{dim}\">{two_g_plus_m_minus_1}</td>",
    );
    html.push_str("</tr></tbody></table>");
    html
}

/// HTML for the Diag tab: `U(m)`, `one·U(m)`, `U(m)·C_red`, pair-relations,
/// `D(m)`, and the `D(m)·c₁` / zd(m) identity table.
// ALLOW: pure HTML-builder; each block renders one distinct table — splitting
// further would fragment the rendering pipeline without reducing logical complexity.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn diagonals_table(sg: &Semigroup) -> String {
    let mult = sg.m;
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
    let v_mat = v_matrix(mult);
    let html_v = render(&v_mat, "V(m)", &one_based, &one_based, &plain);
    let vu_product = Matrix::mat_mul(&v_mat, &u_mat);
    let html_v_times_u = render(
        &vu_product,
        "V(m)\u{b7}U(m)",
        &one_based,
        &one_based,
        &plain,
    );
    let one_u: Vec<i64> = (0..dim)
        .map(|j| (0..dim).map(|a| u_mat[(a, j)]).sum())
        .collect();
    let cr = c_red(sg);
    #[allow(clippy::cast_possible_wrap)]
    let c1: Vec<i64> = (0..dim).map(|i| cr[(i, 0)] as i64).collect();
    // Selmer's formula: sum of Apéry elements w₁…w_{m−1} = m·g + m·(m−1)/2
    #[allow(clippy::cast_possible_wrap)]
    let apery_sum = (mult * sg.g + mult * (mult - 1) / 2) as i64;
    let html_one = render_one_vec(&one_u, &c1, apery_sum);
    let product = u_times_c_red(&cr);
    let sets = class_sets(sg);
    let classified = |_a: usize, b: usize, val: i64| {
        if b == 0 && val >= 0 {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let v = val as usize;
            format!("<td>{}</td>", span(cell_cls(v, sg, &sets), v, true))
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
    let pair_cell = |_a: usize, _b: usize, val: i64| pm_cell(val);
    let html_pair = render(
        &pair,
        "(U_i+U_j\u{2212}U_{i+j})/m",
        &pair_label,
        &one_based,
        &pair_cell,
    );

    // ── D(m) matrix and zd(m) row vector ────────────────────────────────────
    let d_mat = d_matrix(mult);
    let html_d = render(&d_mat, "D(m)", &one_based, &one_based, &plain);
    let zd = zd_vector(mult);
    // d_i = D(m)[i−1] · c₁  (anti-diagonal sum of Kunz matrix at position i)
    let d_prod: Vec<i64> = (0..dim)
        .map(|i| (0..dim).map(|b| d_mat[(i, b)] * c1[b]).sum())
        .collect();
    // zd · c₁ = f + m + r (number of reflected gaps)
    #[allow(clippy::cast_possible_wrap)]
    let zd_dot: i64 = (0..dim).map(|b| zd[(0, b)] * c1[b]).sum();
    #[allow(clippy::cast_possible_wrap)]
    let f_plus_m_plus_r = (sg.f + mult + sg.r) as i64;
    #[allow(clippy::cast_possible_wrap)]
    let two_g_plus_m_minus_1 = (2 * sg.g + mult - 1) as i64;
    let html_d_prod = render_d_prod(
        zd.as_slice(),
        &d_prod,
        &c1,
        zd_dot,
        f_plus_m_plus_r,
        two_g_plus_m_minus_1,
    );

    let html_rn_block = render_rn_block(sg);

    let mut out = String::from("<div class=\"diagonals-pane\">");
    let _ = write!(out, "<div class=\"table-wrap\">{html_u}</div>");
    let _ = write!(
        out,
        "<p class=\"det-note\">det(U(m)) = m<sup>m\u{2212}2</sup> = \
         {mult}<sup>{}</sup></p>",
        mult - 2,
    );
    let _ = write!(out, "<div class=\"table-wrap\">{html_v}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_v_times_u}</div>");
    let _ = write!(
        out,
        "<p class=\"det-note\">V(m)\u{b7}U(m) = m\u{b7}I<sub>m\u{2212}1</sub> = \
         {mult}\u{b7}I<sub>{}</sub></p>",
        mult - 1,
    );
    let _ = write!(out, "<div class=\"table-wrap\">{html_one}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_uc}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_pair}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_d}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_d_prod}</div>");
    out.push_str("<hr class=\"diagonals-sep\">");
    out.push_str(&html_rn_block);
    out.push_str("</div>");
    out
}

/// Renders the rn table, the `RSym` matrix, the `RSym`·rv verification row,
/// and the two sum identities `1·rv = r` and `1·nv = g − r`.
fn render_rn_block(sg: &Semigroup) -> String {
    let mult = sg.m;
    if mult < 2 {
        return String::new();
    }
    let dim = mult - 1;
    let mu = sg.mu;
    #[allow(clippy::cast_possible_wrap)]
    let r_vec: Vec<i64> = (1..mult).map(|i| sg.r_i(i) as i64).collect();
    #[allow(clippy::cast_possible_wrap)]
    let n_vec: Vec<i64> = (1..mult)
        .map(|i| ((sg.apery_set[i] - i) / mult) as i64 - r_vec[i - 1])
        .collect();

    let mut html = String::new();
    html.push_str(&render_rn_table(&r_vec, &n_vec, mu));

    let cols2 = 2 * dim;
    let r_sym = build_r_sym(mult, mu);
    html.push_str(&render_r_sym_matrix(&r_sym, mult, cols2));

    let prod: Vec<i64> = (0..dim)
        .map(|row| {
            (0..dim)
                .map(|col| r_sym[row * cols2 + col] * r_vec[col])
                .sum()
        })
        .collect();
    html.push_str(&render_r_sym_prod(&prod, mult));

    let sum_r: i64 = r_vec.iter().sum();
    let sum_n: i64 = n_vec.iter().sum();
    #[allow(clippy::cast_possible_wrap)]
    let r_expected = sg.r as i64;
    #[allow(clippy::cast_possible_wrap)]
    let gmr_expected = sg.g as i64 - r_expected;
    let _ = write!(
        html,
        "<p class=\"det-note\">\u{03a3}r<sub>i</sub> = {sum_r}, \
         r = {r_expected} {}</p>\
         <p class=\"det-note\">\u{03a3}n<sub>i</sub> = {sum_n}, \
         g \u{2212} r = {gmr_expected} {}</p>",
        check_glyph(sum_r == r_expected),
        check_glyph(sum_n == gmr_expected),
    );
    html
}

/// `(m−1) × 2(m−1)` flat row-major `RSym` matrix. Row `i` (1-based) has
/// `+1` at column `i`; if `(μ − i) mod m ≠ 0` it also has `−1` at column
/// `(μ − i) mod m`; the lone row `i = μ` ends up all-zeros after the
/// `−1` cancels the `+1` at column `μ`.
fn build_r_sym(mult: usize, mu: usize) -> Vec<i64> {
    let dim = mult - 1;
    let cols2 = 2 * dim;
    let mut r_sym = vec![0i64; dim * cols2];
    for ii in 1..mult {
        let row = ii - 1;
        r_sym[row * cols2 + row] += 1;
        let j_mod = (mu + mult - ii) % mult;
        if j_mod == 0 {
            r_sym[row * cols2 + row] -= 1;
        } else {
            r_sym[row * cols2 + (j_mod - 1)] -= 1;
        }
    }
    r_sym
}

const fn check_glyph(ok: bool) -> &'static str {
    if ok { "\u{2713}" } else { "\u{2717}" }
}

/// Header row `r_1 … r_{m-1} n_1 … n_{m-1}` after a caller-supplied title `<th>`.
/// `mu_class` is added to the `r_μ` cell when non-empty (used to highlight the
/// always-zero μ slot in the rn table).
fn rn_header_row(title: &str, mult: usize, mu: usize, mu_class: &str) -> String {
    let mut h = format!("<th>{title}</th>");
    for i in 1..mult {
        let cls = if i == mu && !mu_class.is_empty() {
            format!(" class=\"{mu_class}\"")
        } else {
            String::new()
        };
        let _ = write!(h, "<th{cls}>r<sub>{i}</sub></th>");
    }
    for i in 1..mult {
        let _ = write!(h, "<th>n<sub>{i}</sub></th>");
    }
    h
}

fn render_rn_table(r_vec: &[i64], n_vec: &[i64], mu: usize) -> String {
    let mult = r_vec.len() + 1;
    let mut h = String::from(
        "<div class=\"table-wrap\">\
         <table class=\"classify-table u-matrix-table\"><thead><tr>",
    );
    h.push_str(&rn_header_row("rn", mult, mu, "rn-mu"));
    h.push_str("</tr></thead><tbody><tr><th></th>");
    for (idx, v) in r_vec.iter().enumerate() {
        let cls = if idx + 1 == mu {
            " class=\"rn-mu\""
        } else {
            ""
        };
        let _ = write!(h, "<td{cls}>{v}</td>");
    }
    for v in n_vec {
        let _ = write!(h, "<td>{v}</td>");
    }
    h.push_str("</tr></tbody></table></div>");
    h
}

fn render_r_sym_matrix(r_sym: &[i64], mult: usize, cols2: usize) -> String {
    let dim = mult - 1;
    let mut h = String::from(
        "<div class=\"table-wrap\">\
         <table class=\"classify-table u-matrix-table\"><thead><tr>",
    );
    h.push_str(&rn_header_row("RSym", mult, 0, ""));
    h.push_str("</tr></thead><tbody>");
    for row in 0..dim {
        let _ = write!(h, "<tr><th>{}</th>", row + 1);
        for col in 0..cols2 {
            h.push_str(&pm_cell(r_sym[row * cols2 + col]));
        }
        h.push_str("</tr>");
    }
    h.push_str("</tbody></table></div>");
    h
}

fn render_r_sym_prod(prod: &[i64], mult: usize) -> String {
    let all_zero = prod.iter().all(|&v| v == 0);
    let mut h = String::from(
        "<div class=\"table-wrap\">\
         <table class=\"classify-table u-matrix-table\"><thead><tr>\
         <th>RSym\u{b7}rv</th>",
    );
    for i in 1..mult {
        let _ = write!(h, "<th>{i}</th>");
    }
    let _ = write!(h, "<th>= 0?</th></tr></thead><tbody><tr><th></th>");
    for v in prod {
        let _ = write!(h, "<td>{v}</td>");
    }
    let _ = write!(
        h,
        "<td>{}</td></tr></tbody></table></div>",
        check_glyph(all_zero)
    );
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use semigroup_math::math::compute;

    fn assert_rn_identities(gens: &[usize]) {
        let sg = compute(gens);
        let html = render_rn_block(&sg);
        // Both identity checks must render the green tick.
        assert!(
            html.contains("\u{2713}") && !html.contains("\u{2717}"),
            "rn block reported a failure for gens={gens:?}:\n{html}"
        );
        // The RSym·rv block always appears (even when the +1/−1 row μ cancels
        // to zero), so the "= 0?" header should be present.
        assert!(html.contains("= 0?"), "missing RSym·rv table for {gens:?}");
    }

    #[test]
    fn rn_identities_small_examples() {
        // Symmetric ⟨3, 5, 7⟩: r = 0.
        assert_rn_identities(&[3, 5, 7]);
        // Almost-symmetric ⟨4, 5, 6⟩.
        assert_rn_identities(&[4, 5, 6]);
        // Generic ⟨5, 7, 9, 11⟩.
        assert_rn_identities(&[5, 7, 9, 11]);
        // Larger multiplicity ⟨7, 9, 11, 13⟩.
        assert_rn_identities(&[7, 9, 11, 13]);
    }
}
