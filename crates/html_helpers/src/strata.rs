//! HTML renderer for the strata-explorer table.
//!
//! Layout: a `<th>` row showing the columns `1..=n`, followed by chain rows
//! ordered top-down from `lmax` to `0` (so the largest set sits on top), and
//! a repeat of the column header at the bottom.

use std::fmt::Write as _;

/// Render a strata chain as an HTML table with `n` columns.
///
/// Each cell carries `data-l="<level>" data-v="<value>"` attributes so the
/// JS click handler can locate the toggled element without re-parsing the
/// cell text. Cells in the set use the `strata-in` class; gaps use
/// `strata-out`.
#[must_use]
pub fn strata_table(chain: &[Vec<usize>], n: usize) -> String {
    let header_cells = {
        let mut s = String::new();
        for v in 1..=n {
            let _ = write!(s, "<th>{v}</th>");
        }
        s
    };

    let mut html = String::from("<table class=\"strata-grid\"><thead>");
    let _ = write!(html, "<tr><th></th>{header_cells}</tr>");
    html.push_str("</thead><tbody>");

    for l in (0..chain.len()).rev() {
        let _ = write!(html, "<tr><th>{l}</th>");
        for v in 1..=n {
            let in_set = chain[l].binary_search(&v).is_ok();
            let cls = if in_set { "strata-in" } else { "strata-out" };
            let label = if in_set { v.to_string() } else { String::new() };
            let _ = write!(
                html,
                "<td class=\"{cls}\" data-l=\"{l}\" data-v=\"{v}\">{label}</td>"
            );
        }
        html.push_str("</tr>");
    }

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
        let html = strata_table(&chain, 4);
        // Levels in reverse order: 2 first, then 1, then 0.
        let l2_pos = html.find("<th>2</th><td").unwrap();
        let l1_pos = html.find("<th>1</th><td").unwrap();
        let l0_pos = html.find("<th>0</th><td").unwrap();
        assert!(l2_pos < l1_pos && l1_pos < l0_pos);
        assert!(html.contains("data-l=\"1\" data-v=\"3\">3</td>"));
        assert!(html.contains("strata-foot"));
    }

    #[test]
    fn empty_chain_still_renders_headers() {
        let html = strata_table(&[], 3);
        assert!(html.contains("<th>1</th>"));
        assert!(html.contains("strata-foot"));
    }
}
