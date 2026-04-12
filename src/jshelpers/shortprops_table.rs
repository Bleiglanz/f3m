#![warn(clippy::pedantic)]
use super::JsSemigroup;
use crate::math::Semigroup;
use wasm_bindgen::prelude::*;

/// Groups special-PF entries by diff value and formats each as
/// `<sg-pf>p</sg-pf>=<sg-gen>a</sg-gen>-<sg-gen>b</sg-gen>=...`
pub(super) fn spf_grouped(spf: &[(usize, (usize, usize))], gen_set: &[usize]) -> Vec<String> {
    // collect unique diffs in order of first appearance
    let mut seen: Vec<usize> = Vec::new();
    for &(diff, _) in spf {
        if !seen.contains(&diff) {
            seen.push(diff);
        }
    }
    seen.iter()
        .map(|&diff| {
            #[allow(clippy::format_collect)]
            let reps: String = spf
                .iter()
                .filter(|&&(d, _)| d == diff)
                .map(|&(_, (i, j))| super::spf_pair(gen_set[i], gen_set[j], false))
                .collect();
            format!("<span class=\"sg-pf\">{diff}</span>{reps}")
        })
        .collect()
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

/// Render the data `<td>` cells shared by the compact summary row and history table rows:
/// m, f, e, g, σ, r, t, Sym, gen, PF, SPF, Wilf — in that order.
/// gen, PF and (when non-zero) SPF show counts with a hover popup listing the actual values.
#[allow(clippy::cast_precision_loss)]
pub(super) fn shortprop_cells(sg: &Semigroup) -> String {
    let ((pf, t), (spf_vec, st)) = sg.pseudo_and_special();
    let fmt_spans = |items: &[usize], cls: &str| -> String {
        items
            .iter()
            .map(|&x| super::span(cls, x, false))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let gen_td = popup_cell(true, sg.e, &fmt_spans(&sg.gen_set, "sg-gen"));
    let pf_td = popup_cell(true, t, &fmt_spans(&pf, "sg-pf"));
    let spf_td = if st > 0 {
        popup_cell(
            false,
            st,
            &spf_grouped(&spf_vec, &sg.gen_set).join("&nbsp; "),
        )
    } else {
        "<td>0</td>".to_string()
    };
    let r = sg.r;
    format!(
        "<td>{m}</td><td>{f}</td><td>{e}</td><td>{g}</td><td>{cg}</td>\
         <td>{r}</td><td>{t}</td><td>{sym}</td>{gen_td}{pf_td}{spf_td}\
         <td>{wilf:.4}</td><td>{inv_e:.4}</td>",
        m = sg.m,
        f = fmt_spans(&[sg.f], "sg-frob"),
        e = sg.e,
        g = sg.count_gap,
        cg = sg.count_set,
        r = r,
        t = t,
        sym = if sg.is_symmetric() {
            "\u{2705}"
        } else {
            "\u{1F6AB}"
        },
        wilf = sg.wilf(),
        inv_e = 1.0 / sg.e as f64,
    )
}

/// Compact summary row for the properties table: nested table with header + one data row.
#[wasm_bindgen]
#[must_use]
pub fn shortprop(s: &JsSemigroup) -> String {
    format!(
        "<table class=\"shortprop-table\"><thead><tr>\
         <th title=\"Multiplicity: smallest positive element\">m</th>\
         <th title=\"Frobenius number: largest gap\">f</th>\
         <th title=\"Embedding dimension: number of minimal generators\">e</th>\
         <th title=\"Genus: number of gaps\">g</th>\
         <th title=\"Sporadic elements: elements of S below the conductor f+1\">\u{03C3}</th>\
         <th title=\"Reflected gaps: gaps n where f\u{2212}n is also a gap\">r</th>\
         <th title=\"Type: number of pseudo-Frobenius numbers\">t</th>\
         <th title=\"Symmetric: t=1 and g=(f+1)/2\">Sym</th>\
         <th title=\"Minimal generators\">gen</th>\
         <th title=\"Pseudo-Frobenius numbers: maximals of \u{2124} \u{2216} S\">PF</th>\
         <th title=\"Special pseudo-Frobenius: PF that are differences of generators\">SPF</th>\
         <th title=\"Wilf quotient: \u{03C3}/(f+1) \u{2265} 1/e (conjecture)\">Wilf</th>\
         <th title=\"Wilf conjecture lower bound: 1/e\">1/e</th>\
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
