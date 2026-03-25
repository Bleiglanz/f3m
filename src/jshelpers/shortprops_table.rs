#![warn(clippy::pedantic)]
use wasm_bindgen::prelude::*;
use crate::math::Semigroup;
use super::JsSemigroup;

/// Groups special-PF entries by diff value and formats each as
/// `<sg-pf>p</sg-pf>=<sg-gen>a</sg-gen>-<sg-gen>b</sg-gen>=...`
pub(super) fn spf_grouped(spf: &[(usize, (usize, usize))], gen_set: &[usize]) -> Vec<String> {
    // collect unique diffs in order of first appearance
    let mut seen: Vec<usize> = Vec::new();
    for &(diff, _) in spf {
        if !seen.contains(&diff) { seen.push(diff); }
    }
    seen.iter().map(|&diff| {
        #[allow(clippy::format_collect)]
        let reps: String = spf.iter()
            .filter(|&&(d, _)| d == diff)
            .map(|&(_, (i, j))| super::spf_pair(gen_set[i], gen_set[j], false))
            .collect();
        format!("<span class=\"sg-pf\">{diff}</span>{reps}")
    }).collect()
}

/// Render a `<td>` with a count that reveals a hover popup listing the items.
/// `left` adds the `left` alignment class used for generator/PF columns.
fn popup_cell(left: bool, count: usize, content: &str) -> String {
    let cls = if left { "left has-popup" } else { "has-popup" };
    format!(
        "<td class=\"{cls}\"><span class=\"popup-count\">{count}</span>\
         <div class=\"popup\">{content}</div></td>",
    )
}

/// Render the ten data `<td>` cells shared by the compact summary row and history table rows:
/// m, f, e, g, c-g, t, Sym, gen, PF, SPF — in that order.
/// gen, PF and (when non-zero) SPF show counts with a hover popup listing the actual values.
pub(super) fn shortprop_cells(sg: &Semigroup) -> String {
    let ((pf, t), (spf_vec, st)) = sg.pseudo_and_special();
    let fmt_spans = |items: &[usize], cls: &str| -> String {
        items.iter().map(|&x| super::span(cls, x, false)).collect::<Vec<_>>().join(", ")
    };
    let gen_td  = popup_cell(true,  sg.e, &fmt_spans(&sg.gen_set, "sg-gen"));
    let pf_td   = popup_cell(true,  t,    &fmt_spans(&pf, "sg-pf"));
    let spf_td  = if st > 0 {
        popup_cell(false, st, &spf_grouped(&spf_vec, &sg.gen_set).join("&nbsp; "))
    } else {
        "<td>0</td>".to_string()
    };
    format!(
        "<td>{m}</td><td>{f}</td><td>{e}</td><td>{g}</td><td>{cg}</td>\
         <td>{t}</td><td>{sym}</td>{gen_td}{pf_td}{spf_td}",
        m = sg.m,
        f = fmt_spans(&[sg.f], "sg-frob"),
        e = sg.e,
        g = sg.count_gap, cg = sg.count_set, t = t,
        sym = if sg.is_symmetric() { "\u{2705}" } else { "\u{1F6AB}" },
    )
}

/// Compact summary row for the properties table: nested table with header + one data row.
#[wasm_bindgen]
#[must_use]
pub fn shortprop(s: &JsSemigroup) -> String {
    format!(
        "<table class=\"shortprop-table\"><thead><tr>\
         <th>m</th><th>f</th><th>e</th><th>g</th><th>c-g</th><th>t</th><th>Sym</th>\
         <th>gen</th><th>PF</th><th>SPF</th>\
         </tr></thead><tbody><tr>{}</tr></tbody></table>",
        shortprop_cells(&s.0)
    )
}

/// Flat `<td>` cells for use in the history table row (no nested table, no header).
#[wasm_bindgen]
#[must_use]
pub fn shortprop_tds(s: &JsSemigroup) -> String {
    shortprop_cells(&s.0)
}
