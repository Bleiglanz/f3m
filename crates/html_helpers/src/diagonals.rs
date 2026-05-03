//! HTML tables for the Diag tab: U(m), `U(m)·C_red`, pair-relations, D(m), zd(m).

use crate::combined_table::cell_cls;
use crate::spans::{class_sets, span};
use semigroup_math::math::Semigroup;
use semigroup_math::math::matrix::{
    DenseMatrix, Matrix, c_red, d_matrix, u_matrix, u_pair_relations, u_times_c_red, zd_vector,
};
use std::fmt::Write as _;

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
    let one_u: Vec<i64> = (0..dim)
        .map(|j| (0..dim).map(|a| u_mat[(a, j)]).sum())
        .collect();
    let cr = c_red(sg);
    #[allow(clippy::cast_possible_wrap)]
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
    let two_g_plus_m_minus_1 = (2 * sg.count_gap + mult - 1) as i64;
    let html_d_prod = render_d_prod(
        zd.as_slice(),
        &d_prod,
        &c1,
        zd_dot,
        f_plus_m_plus_r,
        two_g_plus_m_minus_1,
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
    let _ = write!(out, "<div class=\"table-wrap\">{html_d}</div>");
    let _ = write!(out, "<div class=\"table-wrap\">{html_d_prod}</div>");
    out.push_str("</div>");
    out
}
