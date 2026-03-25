#![warn(clippy::pedantic)]
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use super::{JsSemigroup, class_sets};

/// Wrap a classified number in a `<td>` containing a clickable `<span data-n>`.
fn cell_td(cls: &str, n: usize) -> String {
    format!("<td>{}</td>", super::span(cls, n, true))
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
        if n > f + m                { "sg-large" }
        else if n == f                                      { "sg-frob"    }
        else if gens.contains(&n)                          { "sg-gen"     }
        else if n == apery_val                             { "sg-apery"   }
        else if n >= apery_val                             { "sg-in"      }
        else if pf_set.contains(&n) && blobs.contains(&n) { "sg-pf-blob" }
        else if pf_set.contains(&n)                        { "sg-pf"      }
        else if blobs.contains(&n)                         { "sg-blob"    }
        else                        { "sg-out"  }
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
    let num_cols = (col_end - col_start).cast_unsigned();

    // Residue for each column (Euclidean mod, always in 0..m).
    #[allow(clippy::cast_possible_wrap)]
    let residues: Vec<usize> = (col_start..col_end)
        .map(|c| (offset as isize + c).rem_euclid(m as isize) as usize)
        .collect();

    // Standard m-wide permutation used for the Apéry and Kunz sections.
    let perm: Vec<usize> = (0..m).map(|k| (offset + k) % m).collect();

    let sets = class_sets(sg);
    let cls_of = |n, kunz| get_cls(n, kunz, f, m, &sg.apery_set, &sets.gens, &sets.pf_set, &sets.blobs);

    #[allow(clippy::format_collect)]
    let header_cells: String = residues.iter()
        .map(|&r| format!("<th>{r}</th>"))
        .collect();
    let header_row = format!("<tr>{header_cells}</tr>");

    let mut html = String::from("<table class=\"sg-grid\"><thead>");
    html.push_str(&header_row);
    html.push_str("</thead><tbody>");

    // Structure rows (bottom-to-top)
    #[allow(clippy::cast_possible_wrap)]
    let start_row: isize = if (m <= 15 && tilt != 0) || offset != 0 { -1 } else { 0 };
    #[allow(clippy::cast_possible_wrap)]
    let end_row: isize = (f / m + 3) as isize;
    for row in (start_row..end_row).rev() {
        html.push_str("<tr>");
        for col_idx in 0..num_cols {
            let col = col_start + col_idx.cast_signed();
            #[allow(clippy::cast_possible_wrap)]
            let n_signed: isize = row * m as isize + offset as isize + col - tilt as isize * row;
            if n_signed < 0 {
                html.push_str("<td class=\"sg-empty\"></td>");
                continue;
            }
            #[allow(clippy::cast_sign_loss)]
            let n = n_signed as usize;
            html.push_str(&cell_td(cls_of(n, false), n));
        }
        html.push_str("</tr>");
    }

    // Repeated header row as separator
    html.push_str(&header_row);

    // Apéry row
    html.push_str("<tr>");
    for &i in &perm {
        let v = sg.apery_set[i];
        html.push_str(&cell_td(cls_of(v, false), v));
    }
    html.push_str("</tr>");

    // Kunz matrix rows (optional)
    if show_kunz {
        for &i in &perm {
            html.push_str("<tr>");
            for &j in &perm {
                let v = sg.kunz(i, j);
                html.push_str("<td class=\"");
                html.push_str(cls_of(v, true));
                html.push_str("\">");
                html.push_str(&v.to_string());
                html.push_str("</td>");
            }
            html.push_str("</tr>");
        }
    }

    html.push_str("</tbody></table>");
    html
}

