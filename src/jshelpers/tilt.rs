#![warn(clippy::pedantic)]
use wasm_bindgen::prelude::*;
use super::{JsSemigroup, class_sets};
use super::combined_table::get_cls;

/// Pure x-y grid for the Tilt tab: no Apéry row, no Kunz matrix.
/// x (columns) and y (rows) both run from -3m to 3m.
/// y increases upward (highest y at top). x increases left to right.
/// Element at (x, y) = y*m + x - tilt*y.
#[wasm_bindgen]
#[must_use]
pub fn tilt_table(s: &JsSemigroup, tilt: i32) -> String {
    let sg = &s.0;
    let m = sg.m;
    let f = sg.f;

    #[allow(clippy::cast_possible_wrap)]
    let range = 3 * m as isize;
    let col_start = -range;
    let col_end   =  range + 1;
    let row_start = -2;
    let row_end   = (f / m + 3) as isize;  // 2 rows above the Frobenius row
    let num_cols  = (col_end - col_start).cast_unsigned();

    let sets = class_sets(sg);
    let cls_of = |n| get_cls(n, false, f, m, &sg.apery_set, &sets.gens, &sets.pf_set, &sets.blobs);

    // Header: "y\x" corner + x-coordinate labels; x=0 column highlighted
    #[allow(clippy::format_collect)]
    let header_cells: String = (col_start..col_end)
        .map(|x| if x == 0 {
            format!("<th class=\"tilt-axis\">{x}</th>")
        } else {
            format!("<th>{x}</th>")
        })
        .collect();
    let header_row = format!("<tr><th>y\\x</th>{header_cells}</tr>");

    let mut html = String::from("<table class=\"sg-grid tilt-grid\"><thead>");
    html.push_str(&header_row);
    html.push_str("</thead><tbody>");

    // y reversed: highest y at top
    for row in (row_start..row_end).rev() {
        let row_axis = row == 0;
        let th_cls = if row_axis { " class=\"tilt-axis\"" } else { "" };
        html.push_str(&format!("<tr><th{th_cls}>{row}</th>"));
        for col_idx in 0..num_cols {
            let x = col_start + col_idx.cast_signed();
            let axis = row_axis || x == 0;
            #[allow(clippy::cast_possible_wrap)]
            let n_signed: isize = row * m as isize + x - tilt as isize * row;
            if n_signed < 0 {
                let cls = if axis { "sg-empty tilt-axis" } else { "sg-empty" };
                html.push_str(&format!("<td class=\"{cls}\"></td>"));
                continue;
            }
            #[allow(clippy::cast_sign_loss)]
            let n = n_signed as usize;
            let inner = super::span(cls_of(n), n, true);
            if axis {
                html.push_str(&format!("<td class=\"tilt-axis\">{inner}</td>"));
            } else {
                html.push_str(&format!("<td>{inner}</td>"));
            }
        }
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table>");
    html
}
