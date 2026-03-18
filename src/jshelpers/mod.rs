use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use crate::math::{Semigroup, compute};
use crate::eva;

fn span_n(cls: &str, n: usize) -> String {
    format!("<span class=\"{}\" data-n=\"{}\">{}</span>", cls, n, n)
}

fn cell_td(cls: &str, n: usize) -> String {
    format!("<td>{}</td>", span_n(cls, n))
}

// Determine the CSS class of a cell.
// `kunz = true`: n is a kunz coefficient — mark non-trivial zeros.
// `kunz = false`: n is a natural number — classify by semigroup role.
#[allow(clippy::too_many_arguments)]
fn get_cls(
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
        if n > f + m               { "sg-large" }
        else if n == f             { "sg-frob"  }
        else if gens.contains(&n)  { "sg-gen"   }
        else if n == apery_val     { "sg-apery" }
        else if n >= apery_val     { "sg-in"    }
        else if pf_set.contains(&n) { "sg-pf"  }
        else if blobs.contains(&n)  { "sg-blob" }
        else                        { "sg-out"  }
    }
}

/// The shared `<td>` cells for the shortprop columns (m, f, e, g, c-g, t, Sym, gen, PF, SPF).
fn shortprop_cells(sg: &Semigroup) -> String {
    let ((pf, t), (spf, _)) = sg.pft();
    let fmt_spans = |items: &[usize], cls: &str| -> String {
        items.iter()
            .map(|&x| format!("<span class=\"{}\">{}</span>", cls, x))
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
pub fn shortprop_tds(s: &JsSemigroup) -> String {
    shortprop_cells(&s.0)
}

// Replace x[i] substrings (for a given prefix byte) with the i-th element of `set`, or 0.
fn substitute_indexed(expr: &str, prefix: u8, set: &[usize]) -> String {
    let mut result = String::new();
    let bytes = expr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == prefix && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            let start = i + 2;
            let mut j = start;
            while j < bytes.len() && bytes[j].is_ascii_digit() { j += 1; }
            if j < bytes.len() && bytes[j] == b']' && j > start {
                let idx: usize = expr[start..j].parse().unwrap_or(usize::MAX);
                result.push_str(&set.get(idx).copied().unwrap_or(0).to_string());
                i = j + 1;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Replace a[i], q[i] and the letters e, g, f, t, m, E in `expr` with semigroup values:
///   a[i] → i-th Apéry number (0 if i≥m)
///   q[i] → i-th minimal generator (0 if i≥e)
///   e=embedding dim, g=gaps, f=Frobenius, t=type, m=multiplicity, E=largest generator
/// Then evaluate as an integer arithmetic expression (+ - * /, integer division).
/// Returns None on any error.
#[wasm_bindgen]
pub fn eval_expr(expr: &str, s: &JsSemigroup) -> Option<usize> {
    let sg = &s.0;
    let ((_, t), _) = sg.pft();
    let after_a = substitute_indexed(expr, b'a', &sg.apery_set);
    let substituted = substitute_indexed(&after_a, b'q', &sg.gen_set)
        .replace('e', &sg.e.to_string())
        .replace('g', &sg.count_gap.to_string())
        .replace('f', &sg.f.to_string())
        .replace('t', &t.to_string())
        .replace('E', &sg.max_gen.to_string())
        .replace('m', &sg.m.to_string());
    eva::eval(&substituted).ok()
}

/// Build the full combined table: structure grid + repeated header + Apéry row + Kunz matrix.
/// All sections share `m` columns, permuted by `offset` so column `col` shows residue
/// `(offset + col) % m`.
#[wasm_bindgen]
pub fn combined_table(s: &JsSemigroup, offset: usize) -> String {
    let sg = &s.0;
    let m = sg.m;
    let f = sg.f;
    let perm: Vec<usize> = (0..m).map(|k| (offset + k) % m).collect();

    let gens: HashSet<usize> = sg.gen_set.iter().cloned().collect();
    let blobs: HashSet<usize> = sg.blob().into_iter().collect();
    let pf_set: HashSet<usize> = sg.pft().0.0.into_iter().collect();

    let header_cells: String = perm.iter()
        .map(|&r| format!("<th>{}</th>", r))
        .collect();
    let header_row = format!("<tr>{}</tr>", header_cells);

    let mut html = String::from("<table class=\"sg-grid\">");
    html.push_str(&format!("<thead>{}</thead><tbody>", header_row));

    // Structure rows (bottom-to-top)
    let start_row: isize = if offset == 0 { 0 } else { -1 };
    let end_row: isize = (f / m + 3) as isize;
    for row in (start_row..end_row).rev() {
        html.push_str("<tr>");
        for col in 0..m {
            let n_signed: isize = row * m as isize + offset as isize + col as isize;
            if n_signed < 0 {
                html.push_str("<td class=\"sg-empty\"></td>");
                continue;
            }
            let n = n_signed as usize;
            let cls = get_cls(n, false, f, m, &sg.apery_set, &gens, &pf_set, &blobs);
            html.push_str(&cell_td(cls, n));
        }
        html.push_str("</tr>");
    }

    // Repeated header row as separator
    html.push_str(&header_row);

    // Apéry row
    html.push_str("<tr>");
    for &i in &perm {
        let v = sg.apery_set[i];
        let cls = get_cls(v, false, f, m, &sg.apery_set, &gens, &pf_set, &blobs);
        html.push_str(&cell_td(cls, v));
    }
    html.push_str("</tr>");

    // Kunz matrix rows
    for &i in &perm {
        html.push_str("<tr>");
        for &j in &perm {
            let v = sg.kunz(i, j);
            let cls = get_cls(v, true, f, m, &sg.apery_set, &gens, &pf_set, &blobs);
            html.push_str(&format!("<td class=\"{}\">{}</td>", cls, v));
        }
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table>");
    html
}

// ── JsSemigroup ──────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct JsSemigroup(pub(crate) Semigroup);

#[wasm_bindgen]
impl JsSemigroup {
    #[wasm_bindgen(getter)]
    pub fn e(&self) -> usize { self.0.e }
    #[wasm_bindgen(getter)]
    pub fn f(&self) -> usize { self.0.f }
    #[wasm_bindgen(getter)]
    pub fn m(&self) -> usize { self.0.m }
    #[wasm_bindgen(getter)]
    pub fn count_set(&self) -> usize { self.0.count_set }
    #[wasm_bindgen(getter)]
    pub fn count_gap(&self) -> usize { self.0.count_gap }
    #[wasm_bindgen(getter)]
    pub fn max_gen(&self) -> usize { self.0.max_gen }

    #[wasm_bindgen(getter)]
    pub fn gen_set(&self) -> Vec<u32> {
        self.0.gen_set.iter().map(|&x| x as u32).collect()
    }
    #[wasm_bindgen(getter)]
    pub fn apery_set(&self) -> Vec<u32> {
        self.0.apery_set.iter().map(|&x| x as u32).collect()
    }
    #[wasm_bindgen(getter)]
    pub fn blob(&self) -> Vec<u32> {
        self.0.blob().iter().map(|&x| x as u32).collect()
    }

    pub fn is_element(&self, x: usize) -> bool { self.0.element(x) }
    pub fn kunz(&self, i: usize, j: usize) -> usize { self.0.kunz(i, j) }

    #[wasm_bindgen(getter)]
    pub fn is_symmetric(&self) -> bool { self.0.is_symmetric() }
    #[wasm_bindgen(getter)]
    pub fn wilf(&self) -> f64 { self.0.wilf() }

    #[wasm_bindgen(getter)]
    pub fn pf(&self) -> Vec<u32> {
        let ((pf, _), _) = self.0.pft();
        pf.iter().map(|&x| x as u32).collect()
    }
    #[wasm_bindgen(getter)]
    pub fn type_t(&self) -> usize { self.0.pft().0.1 }
    #[wasm_bindgen(getter)]
    pub fn special_pf(&self) -> Vec<u32> {
        let (_, (spf, _)) = self.0.pft();
        spf.iter().map(|&x| x as u32).collect()
    }

    pub fn toggle(&self, n: usize) -> JsSemigroup {
        JsSemigroup(self.0.toggle(n))
    }
}

#[wasm_bindgen]
pub fn js_compute(input: &str) -> JsSemigroup {
    let numbers: Vec<usize> = input
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    JsSemigroup(compute(&numbers))
}
