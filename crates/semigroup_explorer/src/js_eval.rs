//! Expression evaluator over the variables of a numerical semigroup.
//!
//! Supports scalars (`e`, `f`, `g`, `m`, `t`, `Q`, `A`, `s`, `r`) and indexed
//! access (`a[i]` for Apéry, `q[i]` for generators). Index expressions are
//! themselves evaluated recursively, so `a[m-1]`, `q[e-1]`, and `a[a[0]+1]`
//! all work. Substitution is single-pass and operates on character classes,
//! so substituting `e=3` does not mutate digits inserted by an earlier
//! substitution (a problem with the previous chained-`String::replace`
//! implementation).

use super::JsSemigroup;
use semigroup_math::eva;
use wasm_bindgen::prelude::*;

/// Evaluation context: all semigroup scalars and slices needed to substitute
/// named variables (`e`, `f`, `g`, `m`, `t`, `Q`, `A`) and indexed references
/// (`a[i]` for Apéry, `q[i]` for generators) before passing to `eva::eval`.
#[derive(Debug)]
pub(super) struct EvalCtx<'a> {
    apery: &'a [usize],   // Apéry set (a[i])
    gen_set: &'a [usize], // minimal generators (q[i])
    e: usize,
    g: usize,
    f: usize,
    t: usize,
    m: usize,
    max_gen: usize,
    sigma: usize, // σ: semigroup elements below conductor (sigma)
    r: usize,     // reflected gap count
}

impl EvalCtx<'_> {
    /// Resolve a single-character scalar variable to its `usize` value.
    /// Returns `None` for any character that is not a known variable.
    const fn lookup_scalar(&self, b: u8) -> Option<usize> {
        match b {
            b'e' => Some(self.e),
            b'g' => Some(self.g),
            b'f' => Some(self.f),
            b't' => Some(self.t),
            b'Q' => Some(self.max_gen),
            b'A' => Some(self.f + self.m),
            b's' => Some(self.sigma),
            b'r' => Some(self.r),
            b'm' => Some(self.m),
            _ => None,
        }
    }

    /// Find the index of the matching `]` for an opening `[`, handling nested brackets.
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

    /// Substitute all variables and indexed references in `expr` and evaluate.
    ///
    /// Returns `None` if any substitution fails (unknown variable, index out of
    /// range, evaluation overflow, …).
    ///
    /// Substitution is single-pass: each ASCII byte is examined exactly once
    /// in input order, so digits emitted by an earlier substitution can never
    /// be re-interpreted as variables. Implicit-multiplication insertion uses
    /// the *original* token kinds, not the substituted bytes, so `m2` → `m*2`
    /// and `2m` → `2*m` work even when `m` happens to substitute to a digit
    /// that abuts the literal `2`.
    pub(super) fn eval(&self, expr: &str) -> Option<usize> {
        let bytes = expr.as_bytes();
        let mut out = String::with_capacity(bytes.len() * 2);
        let mut prev: TokenKind = TokenKind::None;

        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];

            // a[...] / q[...] indexed reference.
            if (b == b'a' || b == b'q')
                && bytes.get(i + 1) == Some(&b'[')
                && let Some(close) = Self::matching_bracket(bytes, i + 1)
            {
                let inner = &expr[i + 2..close];
                let idx = self.eval(inner)?;
                let set: &[usize] = if b == b'a' { self.apery } else { self.gen_set };
                let value = set.get(idx).copied().unwrap_or(0);
                if prev.needs_mul_before(TokenKind::Var) {
                    out.push('*');
                }
                out.push_str(&value.to_string());
                prev = TokenKind::Var;
                i = close + 1;
                continue;
            }

            // Single-character scalar variable.
            if let Some(value) = self.lookup_scalar(b) {
                if prev.needs_mul_before(TokenKind::Var) {
                    out.push('*');
                }
                out.push_str(&value.to_string());
                prev = TokenKind::Var;
                i += 1;
                continue;
            }

            // Literal pass-through (digits, operators, parentheses, whitespace, …).
            let kind = if b.is_ascii_digit() {
                TokenKind::Digit
            } else {
                TokenKind::Other
            };
            if prev.needs_mul_before(kind) {
                out.push('*');
            }
            out.push(b as char);
            prev = kind;
            i += 1;
        }

        eva::eval(&out).ok()
    }
}

/// Original-token classifier used to decide whether implicit `*` belongs between
/// two adjacent emissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    None,
    Digit,
    Var,
    Other,
}

impl TokenKind {
    /// `true` when an implicit `*` belongs between `self` (just emitted) and the
    /// `next` token: digit↔var or var↔var.
    const fn needs_mul_before(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Digit | Self::Var, Self::Var) | (Self::Var, Self::Digit),
        )
    }
}

/// Substitute variables in `expr` and evaluate as arithmetic over `usize`.
///
/// Variables: `e` (embedding dim), `g` (gaps), `f` (Frobenius), `t` (type),
/// `m` (multiplicity), `Q` (largest generator), `A = f+m`, `s = σ` (sporadic
/// elements), `r` (reflected gap count). Indexed: `a[i]` (Apéry set, `0` if
/// out of range), `q[i]` (generators, `0` if out of range). Returns `None` on
/// any error (unknown variable, malformed brackets, evaluation overflow, …).
#[wasm_bindgen]
#[must_use]
pub fn eval_expr(expr: &str, s: &JsSemigroup) -> Option<usize> {
    let sg = &s.0;
    let ctx = EvalCtx {
        apery: &sg.apery_set,
        gen_set: &sg.gen_set,
        e: sg.e,
        g: sg.g,
        f: sg.f,
        t: sg.t,
        m: sg.m,
        max_gen: sg.max_gen,
        sigma: sg.sigma,
        r: sg.r,
    };
    ctx.eval(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use semigroup_math::math::{Semigroup, compute};

    use super::super::JsSemigroup;

    fn ctx_for(gens: &[usize]) -> (JsSemigroup, Semigroup) {
        let sg = compute(gens);
        let js = JsSemigroup(sg.clone());
        (js, sg)
    }

    #[test]
    fn scalars_resolve() {
        let (js, _) = ctx_for(&[6, 9, 20]);
        assert_eq!(eval_expr("m", &js), Some(6));
        assert_eq!(eval_expr("f", &js), Some(43));
        assert_eq!(eval_expr("e", &js), Some(3));
        assert_eq!(eval_expr("f+1", &js), Some(44));
        assert_eq!(eval_expr("A", &js), Some(49)); // f + m
    }

    #[test]
    fn indexed_access() {
        let (js, _) = ctx_for(&[6, 9, 20]);
        assert_eq!(eval_expr("a[0]", &js), Some(0));
        // Recursive index: a[m-1] = a[5]; Apéry set is [0,49,20,9,40,29], a[5]=29.
        assert_eq!(eval_expr("a[m-1]", &js), Some(29));
        assert_eq!(eval_expr("q[e-1]", &js), Some(20));
        assert_eq!(eval_expr("q[99]", &js), Some(0)); // out of range → 0
    }

    #[test]
    fn substitution_is_single_pass() {
        // Regression test: with the old `String::replace('e', ...).replace('g', ...)`
        // chain, substituting e=3 then g=12 would mutate digits in any earlier
        // numeric literal that contained '3' or 'g'. Our single-pass walker can't.
        let (js, _) = ctx_for(&[6, 9, 20]);
        // 100 + e = 100 + 3 = 103. Old impl would also rewrite the '3' if any
        // later substitution happened to spell '3'.
        assert_eq!(eval_expr("100+e", &js), Some(103));
    }

    #[test]
    fn implicit_multiplication() {
        let (js, _) = ctx_for(&[6, 9, 20]);
        assert_eq!(eval_expr("2m", &js), Some(12));
        assert_eq!(eval_expr("m2", &js), Some(12));
        assert_eq!(eval_expr("2e+1", &js), Some(7));
    }

    #[test]
    fn malformed_returns_none() {
        let (js, _) = ctx_for(&[6, 9, 20]);
        assert_eq!(eval_expr("a[", &js), None); // unmatched bracket
        assert_eq!(eval_expr("?", &js), None); // unknown character
    }
}
