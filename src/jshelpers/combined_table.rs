#![warn(clippy::pedantic)]
use super::{JsSemigroup, class_sets};
use std::collections::HashSet;
use std::fmt::Write as _;
use wasm_bindgen::prelude::*;

/// Wrap a classified number in a `<td>` with a residue attribute and a clickable `<span data-n>`.
fn cell_td(cls: &str, n: usize, res: usize) -> String {
    format!("<td data-res=\"{res}\">{}</td>", super::span(cls, n, true))
}

// Determine the CSS class of a cell.
// `kunz = true`: n is a kunz coefficient — mark non-trivial zeros.
// `kunz = false`: n is a natural number — classify by semigroup role.
#[allow(clippy::too_many_arguments)]
pub(crate) fn get_cls(
    n: usize,
    kunz: bool,
    f: usize,
    m: usize,
    apery_set: &[usize],
    gens: &HashSet<usize>,
    pf_set: &HashSet<usize>,
    blobs: &HashSet<usize>,
) -> &'static str {
    if kunz {
        if n == 0 { "kunz-zero" } else { "" }
    } else {
        let apery_val = apery_set[n % m];
        if n > f + m {
            "sg-large"
        } else if n == f {
            "sg-frob"
        } else if gens.contains(&n) {
            "sg-gen"
        } else if n == apery_val {
            "sg-apery"
        } else if n >= apery_val {
            "sg-in"
        } else if pf_set.contains(&n) && blobs.contains(&n) {
            "sg-pf-blob"
        } else if pf_set.contains(&n) {
            "sg-pf"
        } else if blobs.contains(&n) {
            "sg-blob"
        } else {
            "sg-out"
        }
    }
}

/// Build the full combined table: structure grid + repeated header + Apéry row + Kunz matrix.
/// When `tilt == 0` columns span `[0, m)`; when `tilt != 0` they span `[-2m, 2m)` so
/// the wider neighbourhood is visible for a tilted view.
#[wasm_bindgen]
#[must_use]
pub fn combined_table(s: &JsSemigroup, offset: usize, tilt: i32, show_kunz: bool) -> String {
    let sg = &s.0;
    let m = sg.m;
    let f = sg.f;

    // Wide table (-2m…3m-1) only when tilt is active (m <= 15 and tilt != 0);
    // otherwise narrow (0…m-1).
    #[allow(clippy::cast_possible_wrap)]
    let (col_start, col_end): (isize, isize) = if m <= 15 && tilt != 0 {
        (-(2 * m as isize), 3 * m as isize)
    } else {
        (0, m as isize)
    };
    // Residue for each column (Euclidean mod, always in 0..m).
    #[allow(clippy::cast_possible_wrap)]
    let residues: Vec<usize> = (col_start..col_end)
        .map(|c| (offset as isize + c).rem_euclid(m as isize) as usize)
        .collect();

    // Standard m-wide permutation used for the Apéry and Kunz sections.
    let perm: Vec<usize> = (0..m).map(|k| (offset + k) % m).collect();

    let sets = class_sets(sg);
    let cls_of = |n, kunz| {
        get_cls(
            n,
            kunz,
            f,
            m,
            &sg.apery_set,
            &sets.gens,
            &sets.pf_set,
            &sets.blobs,
        )
    };

    #[allow(clippy::format_collect)]
    let header_cells: String = residues.iter().map(|&r| format!("<th>{r}</th>")).collect();
    let header_row = format!("<tr>{header_cells}</tr>");
    // Separator row between structure grid and Kunz: residue headers with data-k for hover
    #[allow(clippy::format_collect)]
    let sep_cells: String = perm
        .iter()
        .map(|&r| format!("<th class=\"residue-sep\" data-k=\"{r}\">{r}</th>"))
        .collect();
    let sep_row = format!("<tr class=\"residue-sep-row\">{sep_cells}</tr>");

    let mut html = String::from("<table class=\"sg-grid\"><thead>");
    html.push_str(&header_row);
    html.push_str("</thead><tbody>");

    // Structure rows (bottom-to-top)
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
            html.push_str(&cell_td(cls_of(n, false), n, res));
        }
        html.push_str("</tr>");
    }

    // Residue separator row with hover support
    html.push_str(&sep_row);

    // Apéry row — data-k carries the residue index for Kunz hover highlighting
    html.push_str("<tr class=\"apery-row\">");
    for &i in &perm {
        let v = sg.apery_set[i];
        let _ = write!(
            html,
            "<td data-k=\"{i}\">{}</td>",
            super::span(cls_of(v, false), v, true)
        );
    }
    html.push_str("</tr>");

    // Kunz matrix rows (optional)
    // data-kunz-i / data-kunz-j / data-kunz-sum enable hover highlighting from the Apéry row
    if show_kunz {
        for &i in &perm {
            let _ = write!(html, "<tr data-kunz-i=\"{i}\">");
            for &j in &perm {
                let v = sg.kunz(i, j);
                let sum = (i + j) % m;
                let _ = write!(
                    html,
                    "<td class=\"{}\" data-kunz-i=\"{i}\" data-kunz-j=\"{j}\" data-kunz-sum=\"{sum}\">{v}</td>",
                    cls_of(v, true)
                );
            }
            html.push_str("</tr>");
        }
    }

    html.push_str("</tbody></table>");
    html
}
