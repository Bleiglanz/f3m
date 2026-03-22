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
            .map(|&(_, (i, j))| format!(
                "=<span class=\"sg-gen\">{}</span>-<span class=\"sg-gen\">{}</span>",
                gen_set[i], gen_set[j]
            ))
            .collect();
        format!("<span class=\"sg-pf\">{diff}</span>{reps}")
    }).collect()
}

/// Render the ten data `<td>` cells shared by the compact summary row and history table rows:
/// m, f, e, g, c-g, t, Sym, gen, PF, SPF — in that order.
pub(super) fn shortprop_cells(sg: &Semigroup) -> String {
    let ((pf, t), (spf, _)) = sg.pseudo_and_special();
    let fmt_spans = |items: &[usize], cls: &str| -> String {
        items.iter().map(|&x| super::span(cls, x, false)).collect::<Vec<_>>().join(", ")
    };
    let spf_html: String = spf_grouped(&spf, &sg.gen_set).join(", ");
    format!(
        "<td>{m}</td><td>{f}</td><td>{e}</td><td>{g}</td><td>{cg}</td>\
         <td>{t}</td><td>{sym}</td>\
         <td class=\"left\">{atoms}</td><td class=\"left\">{pf}</td><td class=\"left\">{spf}</td>",
        m = sg.m,
        f = fmt_spans(&[sg.f], "sg-frob"),
        e = sg.e,
        g = sg.count_gap, cg = sg.count_set, t = t,
        sym = if sg.is_symmetric() { "\u{2705}" } else { "\u{1F6AB}" },
        atoms = fmt_spans(&sg.gen_set, "sg-gen"),
        pf = fmt_spans(&pf, "sg-pf"),
        spf = spf_html,
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
