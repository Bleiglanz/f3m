use wasm_bindgen::prelude::*;
use crate::math::Semigroup;
use super::JsSemigroup;

/// The shared `<td>` cells for the shortprop columns (m, f, e, g, c-g, t, Sym, gen, PF, SPF).
pub(super) fn shortprop_cells(sg: &Semigroup) -> String {
    let ((pf, t), (spf, _)) = sg.pft();
    let fmt_spans = |items: &[usize], cls: &str| -> String {
        items.iter()
            .map(|&x| format!("<span class=\"{cls}\">{x}</span>"))
            .collect::<Vec<_>>()
            .join(", ")
    };
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
        spf = fmt_spans(&spf, "sg-pf"),
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
