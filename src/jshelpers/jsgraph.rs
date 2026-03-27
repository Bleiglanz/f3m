#![warn(clippy::pedantic)]
use super::combined_table::get_cls;
use super::{JsSemigroup, class_sets};
use crate::math::Semigroup;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

/// Hasse-diagram covering relation: a <_S b iff b - a is a minimal generator of S.
fn leq(a: usize, b: usize, ng: &Semigroup) -> bool {
    if a >= b {
        false
    } else {
        let delta = b - a;
        ng.element(delta) && ng.gen_set.contains(&delta)
    }
}

/// Returns all edges (a, b) with a <_S b in the given slice (Hasse-style partial order).
#[must_use]
pub fn graph_edges(numbers: &[usize], ng: &Semigroup) -> (Vec<usize>, Vec<(usize, usize)>) {
    let edges: Vec<(usize, usize)> = numbers
        .iter()
        .enumerate()
        .flat_map(|(idx, &i)| numbers[idx + 1..].iter().map(move |&j| (i, j)))
        .filter(|&(a, b)| leq(a, b, ng))
        .collect();

    let nodes: Vec<usize> = edges
        .iter()
        .flat_map(|&(i, j)| [i, j])
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    (nodes, edges)
}

/// Graph edges up to `upto` as plain text pairs, one per line.
#[wasm_bindgen]
#[must_use]
pub fn js_graph_edges_text(s: &JsSemigroup, upto: usize) -> String {
    let sg = &s.0;
    let numbers: Vec<usize> = (0..=upto).collect();
    let (_nodes, edges) = graph_edges(&numbers, sg);
    edges
        .iter()
        .map(|(a, b)| format!("({a},{b})"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Node IDs (as u32) that appear in the graph for 0..=upto.
#[wasm_bindgen]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn js_graph_node_ids(s: &JsSemigroup, upto: usize) -> Vec<u32> {
    let sg = &s.0;
    let numbers: Vec<usize> = (0..=upto).collect();
    let (nodes, _) = graph_edges(&numbers, sg);
    nodes.iter().map(|&n| n as u32).collect()
}

/// Edges as a flat [from, to, from, to, ...] u32 array for 0..=upto.
#[wasm_bindgen]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn js_graph_edge_pairs(s: &JsSemigroup, upto: usize) -> Vec<u32> {
    let sg = &s.0;
    let numbers: Vec<usize> = (0..=upto).collect();
    let (_, edges) = graph_edges(&numbers, sg);
    edges
        .iter()
        .flat_map(|&(a, b)| [a as u32, b as u32])
        .collect()
}

/// CSS class name for node `n` using the same classification as the combined table.
#[wasm_bindgen]
#[must_use]
pub fn js_node_class(s: &JsSemigroup, n: usize) -> String {
    let sg = &s.0;
    let sets = class_sets(sg);
    get_cls(
        n,
        false,
        sg.f,
        sg.m,
        &sg.apery_set,
        &sets.gens,
        &sets.pf_set,
        &sets.blobs,
    )
    .to_string()
}
