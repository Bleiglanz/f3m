use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use crate::JsSemigroup;

/// Build the structure grid HTML table for the given semigroup and column offset.
/// The column at position `col` shows residue `(offset + col) % m`.
#[wasm_bindgen]
pub fn structure_table(s: &JsSemigroup, offset: usize) -> String {
    let sg = &s.0;
    let m = sg.m;
    let f = sg.f;
    let num_rows = (f + m - 1) / m + 3; // ceil(f/m) + 3

    let gens: HashSet<usize> = sg.gen_set.iter().cloned().collect();
    let blobs: HashSet<usize> = sg.blob().into_iter().collect();

    let mut html = String::from("<table class=\"sg-grid\">");

    // header row
    html.push_str("<thead><tr>");
    for col in 0..m {
        html.push_str(&format!("<th>{}</th>", (offset + col) % m));
    }
    html.push_str("</tr></thead><tbody>");

    // data rows, highest row index first (top of grid = large numbers)
    for row in (0..num_rows).rev() {
        html.push_str("<tr>");
        for col in 0..m {
            let residue = (offset + col) % m;
            let n = row * m + residue;
            let apery = n == sg.apery_set[residue];
            let cls = if n == f {
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
