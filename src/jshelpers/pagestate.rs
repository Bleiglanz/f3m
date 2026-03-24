#![warn(clippy::pedantic)]
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use crate::math::{Semigroup, compute, gap_block, GAP_HEADER, GAP_FOOTER};
use super::JsSemigroup;

/// Global page state owned by the WASM module.
/// Uses `thread_local! + RefCell` — safe and zero-cost on the single WASM thread.
pub struct PageState {
    history:     Vec<Semigroup>,
    current_idx: Option<usize>,
    eva_expr:    String,
    gap_blocks:  String,
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            history:     Vec::new(),
            current_idx: None,
            eva_expr:    "f+1".to_string(),
            gap_blocks:  String::new(),
        }
    }
}

thread_local! {
    static STATE: RefCell<PageState> = RefCell::new(PageState::default());
}

fn with_state<F, R>(f: F) -> R where F: FnOnce(&PageState) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn with_state_mut<F, R>(f: F) -> R where F: FnOnce(&mut PageState) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// ── WASM exports ──────────────────────────────────────────────────────────────

/// Compute a semigroup from comma-separated input, push it to history,
/// update `current_idx`, and return the new index.
#[wasm_bindgen]
#[must_use]
pub fn state_push(input: &str) -> usize {
    let numbers: Vec<usize> = input
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    let sg = compute(&numbers);
    with_state_mut(|state| {
        let idx = state.history.len();
        state.gap_blocks.push_str(&gap_block(&sg, idx + 1));
        state.history.push(sg);
        state.current_idx = Some(idx);
        idx
    })
}

/// Return the semigroup at history index `idx`.
#[wasm_bindgen]
#[must_use]
pub fn state_get(idx: usize) -> JsSemigroup {
    with_state(|state| JsSemigroup(state.history[idx].clone()))
}

/// Number of semigroups in history.
#[wasm_bindgen]
#[must_use]
pub fn state_len() -> usize {
    with_state(|state| state.history.len())
}

/// Current history index, or -1 if history is empty.
#[wasm_bindgen]
#[must_use]
pub fn state_current_idx() -> i32 {
    // History size is always tiny; truncation to i32 is safe in practice.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    with_state(|state| state.current_idx.map_or(-1, |i| i as i32))
}

/// Set the current history index (call when the user re-focuses a history entry).
#[wasm_bindgen]
pub fn state_set_current_idx(idx: usize) {
    with_state_mut(|state| state.current_idx = Some(idx));
}

/// Get the evaluator expression string.
#[wasm_bindgen]
#[must_use]
pub fn state_get_eva_expr() -> String {
    with_state(|state| state.eva_expr.clone())
}

/// Set the evaluator expression string.
#[wasm_bindgen]
pub fn state_set_eva_expr(expr: &str) {
    with_state_mut(|state| state.eva_expr = expr.to_string());
}

/// Full GAP script: header + all accumulated blocks + footer.
#[wasm_bindgen]
#[must_use]
pub fn state_gap_output() -> String {
    with_state(|state| format!("{}{}{}", GAP_HEADER, state.gap_blocks, GAP_FOOTER))
}

/// Containment-comparison HTML symbol between `history[a]` and `history[b]`.
#[wasm_bindgen]
#[must_use]
pub fn state_cmp(a: usize, b: usize) -> String {
    with_state(|state| match state.history[a].partial_cmp(&state.history[b]) {
        Some(std::cmp::Ordering::Less)    => "⊂".to_string(),
        Some(std::cmp::Ordering::Equal)   => "=".to_string(),
        Some(std::cmp::Ordering::Greater) => "⊃".to_string(),
        None                              => "?".to_string(),
    })
}
