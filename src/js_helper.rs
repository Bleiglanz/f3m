use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use crate::JsSemigroup;

/// Build the structure grid HTML table for the given semigroup.
/// The grid has `width` columns; column `col` shows residue `(offset + col) % width`.
/// Values increase left-to-right and bottom-to-top. When offset > 0 an extra bottom
/// row is prepended so that 0..width-1 are always visible; negative cells are empty.
#[wasm_bindgen]
pub fn structure_table(s: &JsSemigroup, offset: usize, width: usize) -> String {
    let sg = &s.0;
    let f = sg.f;

    let gens: HashSet<usize> = sg.gen_set.iter().cloned().collect();
    let blobs: HashSet<usize> = sg.blob().into_iter().collect();

    let start_row: isize = if offset == 0 { 0 } else { -1 };
    let end_row: isize = (f / width + 3) as isize;

    let mut html = String::from("<table class=\"sg-grid\">");

    html.push_str("<thead><tr>");
    for col in 0..width {
        html.push_str(&format!("<th>{}</th>", (offset + col) % width));
    }
    html.push_str("</tr></thead><tbody>");

    for row in (start_row..end_row).rev() {
        html.push_str("<tr>");
        for col in 0..width {
            let n_signed: isize = row * width as isize + offset as isize + col as isize;
            if n_signed < 0 {
                html.push_str("<td class=\"sg-empty\"></td>");
                continue;
            }
            let n = n_signed as usize;
            let apery = n == sg.apery_set[n % sg.m];
            let cls = if n > f + sg.m {
                "sg-large"
            } else if n == f {
                "sg-frob"
            } else if gens.contains(&n) {
                "sg-gen"
            } else if apery {
                "sg-apery"
            } else if sg.element(n) {
                "sg-in"
            } else if blobs.contains(&n) {
                "sg-blob"
            } else {
                "sg-out"
            };
            html.push_str(&format!("<td class=\"{}\">{}</td>", cls, n));
        }
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table>");
    html
}
