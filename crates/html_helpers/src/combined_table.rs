//! Combined structure-grid + Apéry row + Kunz matrix HTML table for the Semigroup tab.

use crate::spans::{ClassSets, class_sets, span};
use semigroup_math::math::Semigroup;
use std::fmt::Write as _;

/// Wrap a classified number in a `<td>` with a residue attribute and a clickable `<span data-n>`.
fn cell_td(cls: &str, n: usize, res: usize) -> String {
    format!("<td data-res=\"{res}\">{}</td>", span(cls, n, true))
}

/// Variant of [`cell_td`] that adds extra CSS classes (for strata borders).
fn cell_td_with(cls: &str, n: usize, res: usize, extra_cls: &str) -> String {
    if extra_cls.is_empty() {
        cell_td(cls, n, res)
    } else {
        format!(
            "<td class=\"{extra_cls}\" data-res=\"{res}\">{}</td>",
            span(cls, n, true)
        )
    }
}

/// Strata classification of an integer `n` relative to a semigroup.
///
/// `level = floor(f/m)` is the largest level we care about. With `mu = f mod m`,
/// the residues split into `left = {1, ..., mu-1}` and `right = {mu+1, ..., m-1}`.
/// Two cells share the same [`Strata`] value iff they live in the same rectangle
/// (same level, same side).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strata {
    Left { level: usize },
    Right { level: usize },
}

/// Classify `n` into a strata rectangle, or return `None` if `n` lies outside
/// every `M_l` / `N_l` (i.e. `n` is in column 0 or μ, or above the level cutoff).
const fn strata_of(n: usize, sg: &Semigroup) -> Option<Strata> {
    if sg.m < 2 {
        return None;
    }
    let l = n / sg.m;
    let i = n % sg.m;
    if l > sg.level || i == 0 || i == sg.mu {
        return None;
    }
    if i < sg.mu {
        Some(Strata::Left { level: l })
    } else {
        Some(Strata::Right { level: l })
    }
}

/// Compute the four border-class additions for a strata cell at `n_signed`.
/// Returns a string like ` strata-t strata-l` (with leading spaces) or empty.
fn strata_borders(n_signed: isize, sg: &Semigroup, m_isize: isize, tilt: i32) -> String {
    if n_signed < 0 {
        return String::new();
    }
    #[allow(clippy::cast_sign_loss)]
    let s_self = strata_of(n_signed as usize, sg);
    if s_self.is_none() {
        return String::new();
    }
    let strata_at = |k: isize| -> Option<Strata> {
        if k < 0 {
            None
        } else {
            #[allow(clippy::cast_sign_loss)]
            strata_of(k as usize, sg)
        }
    };
    let tilt_isize = tilt as isize;
    let mut out = String::new();
    if strata_at(n_signed + m_isize - tilt_isize) != s_self {
        out.push_str(" strata-t");
    }
    if strata_at(n_signed - m_isize + tilt_isize) != s_self {
        out.push_str(" strata-b");
    }
    if strata_at(n_signed - 1) != s_self {
        out.push_str(" strata-l");
    }
    if strata_at(n_signed + 1) != s_self {
        out.push_str(" strata-r");
    }
    out
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
///
/// When `show_strata` is `true`, every cell whose value lies in some `M_l` (the
/// left strata `{lm + i : 1 ≤ i < μ, 0 ≤ l ≤ level}`) or `N_l` (the right
/// strata `{lm + i : μ < i < m}`) receives `strata-{t,b,l,r}` classes on the
/// sides where the visual neighbour is *not* in the same rectangle, so the
/// rendered grid carries a thick black border around each `M_l` / `N_l`.
#[must_use]
pub fn combined_table(
    sg: &Semigroup,
    offset: usize,
    tilt: i32,
    show_kunz: bool,
    show_strata: bool,
) -> String {
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

    let table_cls = if show_strata {
        "sg-grid sg-grid-strata"
    } else {
        "sg-grid"
    };
    let mut html = format!("<table class=\"{table_cls}\"><thead>");
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
    #[allow(clippy::cast_possible_wrap)]
    let m_isize = m as isize;
    for row in (start_row..end_row).rev() {
        html.push_str("<tr>");
        for (col_idx, &res) in residues.iter().enumerate() {
            let col = col_start + col_idx.cast_signed();
            #[allow(clippy::cast_possible_wrap)]
            let n_signed: isize = row * m_isize + offset as isize + col - tilt as isize * row;
            if n_signed < 0 {
                html.push_str("<td class=\"sg-empty\"></td>");
                continue;
            }
            #[allow(clippy::cast_sign_loss)]
            let n = n_signed as usize;
            let extra = if show_strata {
                strata_borders(n_signed, sg, m_isize, tilt)
            } else {
                String::new()
            };
            html.push_str(&cell_td_with(cls_of(n), n, res, extra.trim()));
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
