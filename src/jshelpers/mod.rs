#![warn(clippy::pedantic)]
use wasm_bindgen::prelude::*;
use crate::math::{Semigroup, compute, gap_block};
pub mod combined_table;
pub mod js_eval;
pub mod jsgraph;
pub mod shortprops_table;
pub use shortprops_table::{shortprop, shortprop_tds};

// ── JsSemigroup ──────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct JsSemigroup(pub(crate) Semigroup);

// Semigroup values are always small; truncation to u32 is safe in practice.
#[allow(clippy::cast_possible_truncation)]
#[wasm_bindgen]
impl JsSemigroup {
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn e(&self) -> usize { self.0.e }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn f(&self) -> usize { self.0.f }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn m(&self) -> usize { self.0.m }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn count_set(&self) -> usize { self.0.count_set }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn count_gap(&self) -> usize { self.0.count_gap }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn max_gen(&self) -> usize { self.0.max_gen }

    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn gen_set(&self) -> Vec<u32> {
        self.0.gen_set.iter().map(|&x| x as u32).collect()
    }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn apery_set(&self) -> Vec<u32> {
        self.0.apery_set.iter().map(|&x| x as u32).collect()
    }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn blob(&self) -> Vec<u32> {
        self.0.blob().iter().map(|&x| x as u32).collect()
    }

    #[must_use] 
    pub fn is_element(&self, x: usize) -> bool { self.0.element(x) }
    #[must_use] 
    pub fn kunz(&self, i: usize, j: usize) -> usize { self.0.kunz(i, j) }

    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn is_symmetric(&self) -> bool { self.0.is_symmetric() }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn wilf(&self) -> f64 { self.0.wilf() }

    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn pf(&self) -> Vec<u32> {
        let ((pf, _), _): ((Vec<usize>, usize), (Vec<usize>, usize)) = self.0.pft();
        pf.iter().map(|&x| x as u32).collect()
    }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn type_t(&self) -> usize { self.0.pft().0.1 }
    #[wasm_bindgen(getter)]
    #[must_use] 
    pub fn special_pf(&self) -> Vec<u32> {
        let (_, (spf, _)): ((Vec<usize>, usize), (Vec<usize>, usize)) = self.0.pft();
        spf.iter().map(|&x| x as u32).collect()
    }

    #[must_use] 
    pub fn toggle(&self, n: usize) -> JsSemigroup {
        JsSemigroup(self.0.toggle(n))
    }
}

/// Return the GAP assertion block for a single semigroup, numbered `idx`.
#[wasm_bindgen]
#[must_use] 
pub fn js_gap_block(s: &JsSemigroup, idx: usize) -> String {
    gap_block(&s.0, idx)
}

#[wasm_bindgen]
#[must_use] 
pub fn js_compute(input: &str) -> JsSemigroup {
    let numbers: Vec<usize> = input
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    JsSemigroup(compute(&numbers))
}
