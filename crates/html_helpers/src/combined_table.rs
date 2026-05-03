//! Combined structure-grid + Apéry row + Kunz matrix HTML table for the Semigroup tab.

use crate::spans::{ClassSets, class_sets, span};
use semigroup_math::math::Semigroup;
use std::fmt::Write as _;

/// Wrap a classified number in a `<td>` with a residue attribute and a clickable `<span data-n>`.
fn cell_td(cls: &str, n: usize, res: usize) -> String {
    format!("<td data-res=\"{res}\">{}</td>", span(cls, n, true))
}

/// CSS class for a Kunz-coefficient cell; non-trivial zeros are highlighted.
#[must_use]
pub const fn kunz_cls(n: usize) -> &'static str {
    if n == 0 { "kunz-zero" } else { "" }
}

/// CSS class for a natural number cell, classified by its role in the semigroup.
#[must_use]
pub fn cell_cls(n: usize, sg: &Semigroup, sets: &ClassSets) -> &'static str {
    let apery_val = sg.apery_set[n % sg.m];
    if n > sg.f + sg.m || (n > sg.f && n != apery_val) {
        "sg-large"
    } else if n == sg.f {
        "sg-frob"
    } else if sets.gens.contains(&n) {
        "sg-gen"
    } else if n == apery_val {
        "sg-apery"
    } else if n >= apery_val {
        "sg-in"
    } else if sets.pf_set.contains(&n) && sets.blobs.contains(&n) {
        "sg-pf-blob"
    } else if sets.pf_set.contains(&n) {
        "sg-pf"
    } else if sets.blobs.contains(&n) {
        "sg-blob"
    } else {
        "sg-out"
    }
}

/// Build the full combined table: structure grid, repeated header, Apéry row, Kunz matrix.
///
/// When `tilt == 0` columns span `[0, m)`; when `tilt != 0` they span `[-2m, 2m)`
/// so the wider neighbourhood is visible for a tilted view.
#[must_use]
pub fn combined_table(sg: &Semigroup, offset: usize, tilt: i32, show_kunz: bool) -> String {
    let m = sg.m;
    let f = sg.f;

    #[allow(clippy::cast_possible_wrap)]
    let (col_start, col_end): (isize, isize) = if m <= 15 && tilt != 0 {
        (-(2 * m as isize), 3 * m as isize)
    } else {
        (0, m as isize)
    };
    #[allow(clippy::cast_possible_wrap)]
    let residues: Vec<usize> = (col_start..col_end)
        .map(|c| (offset as isize + c).rem_euclid(m as isize) as usize)
        .collect();
    let perm: Vec<usize> = (0..m).map(|k| (offset + k) % m).collect();

    let sets = class_sets(sg);
    let cls_of = |n| cell_cls(n, sg, &sets);

    #[allow(clippy::format_collect)]
    let header_cells: String = residues.iter().map(|&r| format!("<th>{r}</th>")).collect();
    let header_row = format!("<tr>{header_cells}</tr>");
    #[allow(clippy::format_collect)]
    let sep_cells: String = perm
        .iter()
        .map(|&r| format!("<th class=\"residue-sep\" data-k=\"{r}\">{r}</th>"))
        .collect();
    let sep_row = format!("<tr class=\"residue-sep-row\">{sep_cells}</tr>");

    let mut html = String::from("<table class=\"sg-grid\"><thead>");
    html.push_str(&header_row);
    html.push_str("</thead><tbody>");

    #[allow(clippy::cast_possible_wrap)]
    let start_row: isize = if (m <= 15 && tilt != 0) || offset != 0 {
        -1
    } else {
        0
    };
    #[allow(clippy::cast_possible_wrap)]
    let end_row: isize = (f / m + 3) as isize;
    for row in (start_row..end_row).rev() {
        html.push_str("<tr>");
        for (col_idx, &res) in residues.iter().enumerate() {
            let col = col_start + col_idx.cast_signed();
            #[allow(clippy::cast_possible_wrap)]
            let n_signed: isize = row * m as isize + offset as isize + col - tilt as isize * row;
            if n_signed < 0 {
                html.push_str("<td class=\"sg-empty\"></td>");
                continue;
            }
            #[allow(clippy::cast_sign_loss)]
            let n = n_signed as usize;
            html.push_str(&cell_td(cls_of(n), n, res));
        }
        html.push_str("</tr>");
    }

    html.push_str(&sep_row);

    html.push_str("<tr class=\"apery-row\">");
    for &i in &perm {
        let v = sg.apery_set[i];
        let _ = write!(html, "<td data-k=\"{i}\">{}</td>", span(cls_of(v), v, true));
    }
    html.push_str("</tr>");

    // data-kunz-i / data-kunz-j / data-kunz-sum drive hover highlighting from the Apéry row.
    if show_kunz {
        for &i in &perm {
            let _ = write!(html, "<tr data-kunz-i=\"{i}\">");
            for &j in &perm {
                let v = sg.kunz(i, j);
                let sum = (i + j) % m;
                let _ = write!(
                    html,
                    "<td class=\"{}\" data-kunz-i=\"{i}\" data-kunz-j=\"{j}\" data-kunz-sum=\"{sum}\">{v}</td>",
                    kunz_cls(v)
                );
            }
            html.push_str("</tr>");
        }
    }

    html.push_str("</tbody></table>");
    html
}
