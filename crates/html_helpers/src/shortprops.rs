//! HTML rendering for the compact summary row and the per-history-entry row.

use crate::spans::span;
use semigroup_math::math::Semigroup;

/// Render a `<td>` with a count that reveals a hover popup listing the items.
fn popup_cell(count: usize, content: &str) -> String {
    format!(
        "<td class=\"left has-popup\"><span class=\"popup-count\">{count}</span>\
         <div class=\"popup\">{content}</div></td>",
    )
}

/// Render the data `<td>` cells shared by the compact summary row and history table rows.
///
/// Columns: m, f, e, g, σ, r, ra, fg, t, Sym, di, gen, PF, Wilf, 1/e — in that order.
/// `gen` and `PF` show counts with a hover popup listing the actual values.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn shortprop_cells(sg: &Semigroup) -> String {
    let fmt_spans = |items: &[usize], cls: &str| -> String {
        items
            .iter()
            .map(|&x| span(cls, x, false))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let gen_td = popup_cell(sg.e, &fmt_spans(&sg.gen_set, "sg-gen"));
    let pf_td = popup_cell(sg.t, &fmt_spans(&sg.pf_set, "sg-pf"));
    let glyph = |b: bool| {
        if b { "\u{2705}" } else { "\u{1F6AB}" }
    };
    format!(
        "<td>{m}</td><td>{f}</td><td>{e}</td><td>{g}</td><td>{cg}</td>\
         <td>{r}</td><td>{ra}</td><td>{fg}</td><td>{t}</td><td>{sym}</td><td>{di}</td>\
         {gen_td}{pf_td}\
         <td>{wilf:.4}</td><td>{inv_e:.4}</td>",
        m = sg.m,
        f = fmt_spans(&[sg.f], "sg-frob"),
        e = sg.e,
        g = sg.count_gap,
        cg = sg.count_set,
        r = sg.r,
        ra = sg.ra,
        fg = sg.fg,
        t = sg.t,
        sym = glyph(sg.is_symmetric),
        di = glyph(sg.is_descent_image()),
        wilf = sg.wilf(),
        inv_e = 1.0 / sg.e as f64,
    )
}

/// Compact summary row for the properties table: nested table with header + one data row.
#[must_use]
pub fn shortprop(sg: &Semigroup) -> String {
    format!(
        "<table class=\"shortprop-table\"><thead><tr>\
         <th title=\"Multiplicity: smallest positive element\">m</th>\
         <th title=\"Frobenius number: largest gap\">f</th>\
         <th title=\"Embedding dimension: number of minimal generators\">e</th>\
         <th title=\"Genus: number of gaps\">g</th>\
         <th title=\"Sporadic elements: elements of S below the conductor f+1\">\u{03C3}</th>\
         <th title=\"Reflected gaps: gaps n where f\u{2212}n is also a gap\">r</th>\
         <th title=\"Reflected Ap\u{00E9}ry: Ap\u{00E9}ry elements w where w\u{2212}m is a reflected gap\">ra</th>\
         <th title=\"Fundamental gaps: gaps not expressible as sum of two smaller gaps\">fg</th>\
         <th title=\"Type: number of pseudo-Frobenius numbers\">t</th>\
         <th title=\"Symmetric: t=1 and g=(f+1)/2\">Sym</th>\
         <th title=\"Descent image: \u{2203} T with T.descent()=S; equivalently a min-gen lies in (f\u{2212}m, f) or at f+m\">di</th>\
         <th title=\"Minimal generators\">gen</th>\
         <th title=\"Pseudo-Frobenius numbers: maximals of \u{2124} \u{2216} S\">PF</th>\
         <th title=\"Wilf quotient: \u{03C3}/(f+1) \u{2265} 1/e (conjecture)\">Wilf</th>\
         <th title=\"Wilf conjecture lower bound: 1/e\">1/e</th>\
         </tr></thead><tbody><tr>{}</tr></tbody></table>",
        shortprop_cells(sg)
    )
}
