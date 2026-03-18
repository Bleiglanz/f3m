use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use super::JsSemigroup;

fn span_n(cls: &str, n: usize) -> String {
    format!("<span class=\"{cls}\" data-n=\"{n}\">{n}</span>")
}

fn cell_td(cls: &str, n: usize) -> String {
    format!("<td>{}</td>", span_n(cls, n))
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
        else if n == f              { "sg-frob"  }
        else if gens.contains(&n)   { "sg-gen"   }
        else if n == apery_val      { "sg-apery" }
        else if n >= apery_val      { "sg-in"    }
        else if pf_set.contains(&n) { "sg-pf"   }
        else if blobs.contains(&n)  { "sg-blob" }
        else                        { "sg-out"  }
    }
}

/// Build the full combined table: structure grid + repeated header + Apéry row + Kunz matrix.
/// All sections share `m` columns, permuted by `offset` so column `col` shows residue
/// `(offset + col) % m`.
#[wasm_bindgen]
#[must_use]
pub fn combined_table(s: &JsSemigroup, offset: usize) -> String {
    let sg = &s.0;
    let m = sg.m;
    let f = sg.f;
    let perm: Vec<usize> = (0..m).map(|k| (offset + k) % m).collect();

    let gens: HashSet<usize> = sg.gen_set.iter().copied().collect();
    let blobs: HashSet<usize> = sg.blob().into_iter().collect();
    let pf_set: HashSet<usize> = sg.pft().0.0.into_iter().collect();

    #[allow(clippy::format_collect)]
    let header_cells: String = perm.iter()
        .map(|&r| format!("<th>{r}</th>"))
        .collect();
    let header_row = format!("<tr>{header_cells}</tr>");

    let mut html = String::from("<table class=\"sg-grid\"><thead>");
    html.push_str(&header_row);
    html.push_str("</thead><tbody>");

    // Structure rows (bottom-to-top)
    #[allow(clippy::cast_possible_wrap)]
    let start_row: isize = if offset == 0 { 0 } else { -1 };
    #[allow(clippy::cast_possible_wrap)]
    let end_row: isize = (f / m + 3) as isize;
    for row in (start_row..end_row).rev() {
        html.push_str("<tr>");
        for col in 0..m {
            #[allow(clippy::cast_possible_wrap)]
            let n_signed: isize = row * m as isize + offset as isize + col as isize;
            if n_signed < 0 {
                html.push_str("<td class=\"sg-empty\"></td>");
                continue;
            }
            #[allow(clippy::cast_sign_loss)]
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
            html.push_str("<td class=\"");
            html.push_str(cls);
            html.push_str("\">");
            html.push_str(&v.to_string());
            html.push_str("</td>");
        }
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table>");
    html
}
