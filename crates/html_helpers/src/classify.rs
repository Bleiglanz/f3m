//! Per-integer classification table (n, class, Diff) for the Semigroup tab.

use crate::combined_table::cell_cls;
use crate::spans::{class_sets, span};
use semigroup_math::math::Semigroup;
use std::collections::HashMap;
use std::fmt::Write as _;

/// Returns an HTML table mapping each integer 0..=f+m to its classification,
/// with a "Diff" column showing all representations of n as a difference of
/// two Apéry elements: `w_i` − `w_j` = n.
#[must_use]
pub fn classify_table(sg: &Semigroup) -> String {
    let sets = class_sets(sg);
    let cls_of = |n| cell_cls(n, sg, &sets);

    // Build a map: difference → list of "w_i−w_j" expression strings.
    // Skip j=0 (trivial w_i−0 = w_i). Use sg-gen style for Apéry elements
    // that are also minimal generators.
    let ap = &sg.apery_set;
    let ap_cls = |v: usize| {
        if sets.gens.contains(&v) {
            "sg-gen"
        } else {
            "sg-apery"
        }
    };
    let mut apery_diffs: HashMap<usize, String> = HashMap::new();
    for (i, &wi) in ap.iter().enumerate().skip(1) {
        for (j, &wj) in ap.iter().enumerate().skip(1) {
            if i != j && wi > wj {
                let diff = wi - wj;
                let entry = apery_diffs.entry(diff).or_default();
                if !entry.is_empty() {
                    entry.push(' ');
                }
                let _ = write!(
                    entry,
                    "{}−{}",
                    span(ap_cls(wi), wi, false),
                    span(ap_cls(wj), wj, false),
                );
            }
        }
    }

    let mut out = String::from(
        "<table class=\"classify-table\">\
         <thead><tr><th>n</th><th>class</th><th>Diff</th></tr></thead>\
         <tbody>",
    );
    for n in 0..=(sg.f + sg.m) {
        let n_span = span(cls_of(n), n, true);
        let label = sg.classify(n);
        let cls = match label {
            "zero" => "cl-zero",
            "in S"
            | "in S, Apery"
            | "m=min(S)"
            | "minimal Generator"
            | "f=f(S) Frobenius"
            | "c=c(S)=f+1 Conductor" => "cl-in",
            "reflected gap" => "cl-reflect",
            _ => "cl-gap",
        };
        let diff_cell = apery_diffs.get(&n).map_or("", String::as_str);
        let _ = write!(
            out,
            "<tr><td class=\"cl-n\">{n_span}</td><td class=\"{cls}\">{label}</td>\
             <td class=\"cl-diff\">{diff_cell}</td></tr>",
        );
    }
    out.push_str("</tbody></table>");
    out
}
