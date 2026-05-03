use super::JsSemigroup;
use semigroup_math::math::{GAP_FOOTER, GAP_HEADER, Semigroup, compute, gap_block};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// Global page state owned by the WASM module.
/// Uses `thread_local! + RefCell` — safe and zero-cost on the single WASM thread.
// ALLOW: the four bools are independent UI display toggles with no meaningful enum grouping.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct PageState {
    history: Vec<Semigroup>,
    current_idx: Option<usize>,
    eva_expr: String,
    gap_blocks: String,
    show_gaps: bool,
    show_s: bool,
    show_kunz: bool,
    show_classification: bool,
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            current_idx: None,
            eva_expr: "f+1".to_string(),
            gap_blocks: String::new(),
            show_gaps: true,
            show_s: true,
            show_kunz: true,
            show_classification: true,
        }
    }
}

thread_local! {
    static STATE: RefCell<PageState> = RefCell::new(PageState::default());
}

fn with_state<F, R>(f: F) -> R
where
    F: FnOnce(&PageState) -> R,
{
    STATE.with(|s| f(&s.borrow()))
}

fn with_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut PageState) -> R,
{
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// ── WASM exports ──────────────────────────────────────────────────────────────

/// Compute a semigroup from comma-separated input, push it to history,
/// update `current_idx`, and return the new index.
///
/// Returns `-1` when the input has no positive integer generators
/// (the underlying [`compute`] function requires at least one).
#[wasm_bindgen]
#[must_use]
pub fn state_push(input: &str) -> i32 {
    let numbers: Vec<usize> = input
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .filter(|&n: &usize| n > 0)
        .collect();
    if numbers.is_empty() {
        return -1;
    }
    let sg = compute(&numbers);
    with_state_mut(|state| {
        let idx = state.history.len();
        state.gap_blocks.push_str(&gap_block(&sg, idx + 1));
        state.history.push(sg);
        state.current_idx = Some(idx);
        // History is bounded by user actions; any plausible value fits in i32.
        i32::try_from(idx).unwrap_or(i32::MAX)
    })
}

/// Return the semigroup at history index `idx`, or `None` if `idx` is out of range.
#[wasm_bindgen]
#[must_use]
pub fn state_get(idx: usize) -> Option<JsSemigroup> {
    with_state(|state| state.history.get(idx).cloned().map(JsSemigroup))
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

/// Get the `show_gaps` display toggle.
#[wasm_bindgen]
#[must_use]
pub fn state_get_show_gaps() -> bool {
    with_state(|s| s.show_gaps)
}
/// Set the `show_gaps` display toggle.
#[wasm_bindgen]
pub fn state_set_show_gaps(v: bool) {
    with_state_mut(|s| s.show_gaps = v);
}

/// Get the `show_s` display toggle.
#[wasm_bindgen]
#[must_use]
pub fn state_get_show_s() -> bool {
    with_state(|s| s.show_s)
}
/// Set the `show_s` display toggle.
#[wasm_bindgen]
pub fn state_set_show_s(v: bool) {
    with_state_mut(|s| s.show_s = v);
}

/// Get the `show_kunz` display toggle.
#[wasm_bindgen]
#[must_use]
pub fn state_get_show_kunz() -> bool {
    with_state(|s| s.show_kunz)
}
/// Set the `show_kunz` display toggle.
#[wasm_bindgen]
pub fn state_set_show_kunz(v: bool) {
    with_state_mut(|s| s.show_kunz = v);
}

/// Get the `show_classification` display toggle.
#[wasm_bindgen]
#[must_use]
pub fn state_get_show_classification() -> bool {
    with_state(|s| s.show_classification)
}
/// Set the `show_classification` display toggle.
#[wasm_bindgen]
pub fn state_set_show_classification(v: bool) {
    with_state_mut(|s| s.show_classification = v);
}

/// Containment-comparison HTML symbol between `history[a]` and `history[b]`.
///
/// Returns `"?"` for any out-of-range index instead of panicking.
#[wasm_bindgen]
#[must_use]
pub fn state_cmp(a: usize, b: usize) -> String {
    with_state(|state| {
        let (Some(sa), Some(sb)) = (state.history.get(a), state.history.get(b)) else {
            return "?".to_string();
        };
        super::containment_glyph(sa.partial_cmp(sb)).to_string()
    })
}

/// HTML for the Comp tab: previous + current entries side-by-side.
///
/// Each pane shows `shortprop` followed by `combined_table`. When fewer than
/// two semigroups are available (history empty, or `current_idx == 0`) returns
/// a placeholder paragraph.
#[wasm_bindgen]
#[must_use]
pub fn state_comp_html() -> String {
    with_state(|state| {
        let Some(idx) = state.current_idx else {
            return COMP_PLACEHOLDER.to_string();
        };
        if idx == 0 {
            return COMP_PLACEHOLDER.to_string();
        }
        let (Some(prev), Some(curr)) = (state.history.get(idx - 1), state.history.get(idx)) else {
            return COMP_PLACEHOLDER.to_string();
        };
        let cmp = super::containment_glyph(prev.partial_cmp(curr));
        // 1-based labels matching the History tab (S₁, S₂, …).
        let prev_lbl = idx;
        let curr_lbl = idx + 1;
        let pane = |label: String, role: &str, sg: &Semigroup| -> String {
            format!(
                "<div class=\"comp-pane\" data-pane=\"{role}\">\
                 <h3>{label}</h3>\
                 <div class=\"comp-shortprop\">{}</div>\
                 <div class=\"comp-grid table-wrap\">{}</div>\
                 </div>",
                html_helpers::shortprop(sg),
                html_helpers::combined_table(sg, 0, 0, state.show_kunz),
            )
        };
        format!(
            "<div class=\"comp-header\">\
             S<sub>{prev_lbl}</sub> {cmp} S<sub>{curr_lbl}</sub>\
             </div>\
             <div class=\"comp-row\">{}{}</div>",
            pane(format!("S<sub>{prev_lbl}</sub> (previous)"), "prev", prev),
            pane(format!("S<sub>{curr_lbl}</sub> (current)"), "curr", curr),
        )
    })
}

const COMP_PLACEHOLDER: &str = "<p><em>Compute at least two semigroups to use the Comp tab. \
     The current and previous entries of the history are shown side-by-side.</em></p>";
