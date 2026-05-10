//! HTML renderer for the strata-explorer table.
//!
//! Layout: a `<th>` row showing the columns `1..=n`, followed by chain rows
//! ordered top-down from `lmax` to `0` (so the largest set sits on top), a
//! pair of "minimum" rows showing the symbolic `w_i` labels and the smallest
//! `lm + i` value per column, and a final repeat of the column header so the
//! grid is visually framed top and bottom.

use std::fmt::Write as _;

/// Render a strata chain as an HTML table with `n` columns and stride `m`.
///
/// Each filled cell shows the value `l*m + i` (matching the semigroup
/// convention where row `l` covers values `lm + 1, …, lm + (m-1)`). Every
/// cell carries `data-l`, `data-v`, and `data-val` attributes so the JS
/// hover/click handlers can locate elements without parsing cell text.
/// Headers (top, bottom, and the in-table column index `<th>` cells) all
/// carry `data-v` so column-symmetry hover lights them up too.
///
/// Two extra rows precede the bottom header: the `w` label row showing
/// `w_1, …, w_n`, and the `w`-value row showing the minimum filled value
/// per column (the smallest `l*m + i` over all `l` with `i ∈ M_l`), or an
/// em dash when the column never appears in the chain.
#[must_use]
pub fn strata_table(chain: &[Vec<usize>], n: usize, m: usize) -> String {
    // w_i = (smallest level l with i ∈ M_l) * m + i, or None when i never appears.
    let w: Vec<Option<usize>> = (1..=n)
        .map(|i| {
            chain.iter().enumerate().find_map(|(l, row)| {
                if row.binary_search(&i).is_ok() {
                    Some(l * m + i)
                } else {
                    None
                }
            })
        })
        .collect();

    let header_cells = {
        let mut s = String::new();
        for v in 1..=n {
            let _ = write!(s, "<th data-v=\"{v}\">{v}</th>");
        }
        s
    };

    let mut html = String::from("<table class=\"strata-grid\"><thead>");
    let _ = write!(html, "<tr><th></th>{header_cells}</tr>");
    html.push_str("</thead><tbody>");

    for l in (0..chain.len()).rev() {
        let _ = write!(html, "<tr><th>{l}</th>");
        for i in 1..=n {
            let in_set = chain[l].binary_search(&i).is_ok();
            let cls = if in_set { "strata-in" } else { "strata-out" };
            let val = l * m + i;
            let label = if in_set {
                val.to_string()
            } else {
                String::new()
            };
            let _ = write!(
                html,
                "<td class=\"{cls}\" data-l=\"{l}\" data-v=\"{i}\" data-val=\"{val}\">{label}</td>"
            );
        }
        html.push_str("</tr>");
    }

    // w-label row: subscripted symbolic names.
    html.push_str("<tr class=\"strata-w-label\"><th>w</th>");
    for i in 1..=n {
        let _ = write!(html, "<th data-v=\"{i}\">w<sub>{i}</sub></th>");
    }
    html.push_str("</tr>");

    // w-value row: the minimum lm + i per column (or em dash).
    html.push_str("<tr class=\"strata-w-val\"><th></th>");
    for (idx, w_i) in w.iter().enumerate() {
        let i = idx + 1;
        match w_i {
            Some(val) => {
                let _ = write!(
                    html,
                    "<td class=\"strata-w-cell\" data-v=\"{i}\" data-w=\"{val}\">{val}</td>"
                );
            }
            None => {
                let _ = write!(
                    html,
                    "<td class=\"strata-w-cell strata-w-undef\" data-v=\"{i}\">&mdash;</td>"
                );
            }
        }
    }
    html.push_str("</tr>");

    let _ = write!(
        html,
        "<tr class=\"strata-foot\"><th></th>{header_cells}</tr>"
    );
    html.push_str("</tbody></table>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_header_and_rows_in_top_down_order() {
        let chain = vec![vec![], vec![1, 3], vec![1, 2, 3]];
        let html = strata_table(&chain, 4, 5);
        // Levels in reverse order: 2 first, then 1, then 0.
        let l2_pos = html.find("<th>2</th><td").unwrap();
        let l1_pos = html.find("<th>1</th><td").unwrap();
        let l0_pos = html.find("<th>0</th><td").unwrap();
        assert!(l2_pos < l1_pos && l1_pos < l0_pos);
        // Filled cell at l=1, i=3 shows lm+i = 1*5 + 3 = 8.
        assert!(html.contains("data-l=\"1\" data-v=\"3\" data-val=\"8\">8</td>"));
        assert!(html.contains("strata-foot"));
    }

    #[test]
    fn empty_chain_still_renders_headers() {
        let html = strata_table(&[], 3, 4);
        assert!(html.contains("<th data-v=\"1\">1</th>"));
        assert!(html.contains("strata-foot"));
        // No data rows ⇒ every w_i is undefined.
        assert!(html.contains("strata-w-undef"));
    }

    #[test]
    fn w_row_holds_min_value_per_column() {
        // Column 1 first appears at l=2 (w_1 = 2*5+1 = 11).
        // Column 2 first appears at l=1 (w_2 = 1*5+2 = 7).
        // Column 3 never appears ⇒ undefined.
        let chain = vec![vec![], vec![2], vec![1, 2]];
        let html = strata_table(&chain, 3, 5);
        assert!(html.contains("data-w=\"11\">11</td>"));
        assert!(html.contains("data-w=\"7\">7</td>"));
        // w_3 is undefined.
        assert!(html.contains("data-v=\"3\">&mdash;</td>"));
    }
}
