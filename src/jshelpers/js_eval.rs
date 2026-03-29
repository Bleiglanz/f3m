#![warn(clippy::pedantic)]
use super::JsSemigroup;
use crate::eva;
use wasm_bindgen::prelude::*;

/// Evaluation context: all semigroup scalars and slices needed to substitute
/// named variables (`e`, `f`, `g`, `m`, `t`, `Q`, `A`) and indexed references
/// (`a[i]` for Apéry, `q[i]` for generators) before passing to `eva::eval`.
pub(super) struct EvalCtx<'a> {
    apery: &'a [usize],   // Apéry set (a[i])
    gen_set: &'a [usize], // minimal generators (q[i])
    e: usize,
    g: usize,
    f: usize,
    t: usize,
    m: usize,
    max_gen: usize,
}

impl EvalCtx<'_> {
    // Find the index of the matching ']', handling nested brackets.
    fn matching_bracket(bytes: &[u8], open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (i, &byte) in bytes.iter().enumerate().skip(open) {
            match byte {
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    // Replace prefix[<expr>] with the element at the evaluated index, or 0 if out of range.
    fn substitute_indexed(&self, expr: &str, prefix: u8, set: &[usize]) -> String {
        let mut result = String::new();
        let bytes = expr.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == prefix
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'['
                && let Some(close) = Self::matching_bracket(bytes, i + 1)
            {
                let inner = &expr[i + 2..close];
                let idx = self.eval(inner).unwrap_or(usize::MAX);
                result.push_str(&set.get(idx).copied().unwrap_or(0).to_string());
                i = close + 1;
                continue;
            }
            result.push(bytes[i] as char);
            i += 1;
        }
        result
    }

    /// Substitute all variables/indices in `expr` and evaluate as a `usize` arithmetic expression.
    /// Returns `None` if substitution or evaluation fails (unknown variable, overflow, etc.).
    pub(super) fn eval(&self, expr: &str) -> Option<usize> {
        let s = self.substitute_indexed(expr, b'a', self.apery);
        let s = self.substitute_indexed(&s, b'q', self.gen_set);
        // Insert implicit '*' between a digit and a letter (e.g. "2e" → "2*e")
        // and between a letter and a digit (e.g. "e2" → "e*2"), so that
        // expressions like "2e" evaluate as 2*e rather than concatenating.
        let s = Self::insert_implicit_mul(&s);
        let s = s
            .replace('e', &self.e.to_string())
            .replace('g', &self.g.to_string())
            .replace('f', &self.f.to_string())
            .replace('t', &self.t.to_string())
            .replace('Q', &self.max_gen.to_string())
            .replace('A', &(self.f + self.m).to_string())
            .replace('m', &self.m.to_string());
        eva::eval(&s).ok()
    }

    fn insert_implicit_mul(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut result = String::with_capacity(s.len() + 8);
        for (i, &b) in bytes.iter().enumerate() {
            if i > 0 {
                let prev = bytes[i - 1];
                // digit then letter, or letter then digit → insert '*'
                if (prev.is_ascii_digit() && b.is_ascii_alphabetic())
                    || (prev.is_ascii_alphabetic() && b.is_ascii_digit())
                {
                    result.push('*');
                }
            }
            result.push(b as char);
        }
        result
    }
}

/// Replace a[expr], q[expr] and scalars in `expr` with semigroup values:
///   a[i] → i-th Apéry number (0 if i≥m),  q[i] → i-th generator (0 if i≥e)
///   e=embedding dim, g=gaps, f=Frobenius, t=type, m=multiplicity,
///   Q=largest generator (max gen), A=max Apéry element (= f+m)
/// Index expressions are evaluated recursively. Returns None on any error.
#[wasm_bindgen]
#[must_use]
pub fn eval_expr(expr: &str, s: &JsSemigroup) -> Option<usize> {
    let sg = &s.0;
    let ((_, t), _) = sg.pseudo_and_special();
    let ctx = EvalCtx {
        apery: &sg.apery_set,
        gen_set: &sg.gen_set,
        e: sg.e,
        g: sg.count_gap,
        f: sg.f,
        t,
        m: sg.m,
        max_gen: sg.max_gen,
    };
    ctx.eval(expr)
}
