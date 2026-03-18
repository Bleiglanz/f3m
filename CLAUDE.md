# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

A **Numerical Semigroup Calculator** — a browser-based tool that computes properties of numerical semigroups (embedding dimension, Frobenius number, multiplicity, generators, Apéry set, gaps, etc.) from a user-supplied list of generators. The core algorithm runs in Rust compiled to WebAssembly; the UI is plain HTML/JS/CSS with no framework.

## Build commands

```bash
# Run Rust unit tests (native)
cargo test

# Run a single test by name
cargo test test_gcd

# Lint (fix automatically)
cargo clippy --fix --lib -p f3m

# Build the WASM package (regenerates pkg/)
wasm-pack build --target web

# Serve the app locally (required — ES modules don't work over file://)
python3 -m http.server 8080
# then open http://localhost:8080
```

The `pkg/` directory contains committed WASM build artifacts. Rebuild it with `wasm-pack build --target web` after changing any `src/` file.

## Architecture

```
src/
  lib.rs            — crate root: module declarations + tests
  math/mod.rs       — Semigroup struct, compute(), gcd, GAP code generation
  eva/mod.rs        — arithmetic expression evaluator (usize, recursive indexing)
  jshelpers/mod.rs  — WASM exports: JsSemigroup, HTML rendering, eval_expr
  main.rs           — unused binary stub
pkg/                — wasm-pack output: f3m.js, f3m_bg.wasm, f3m.d.ts, package.json
gap/                — example GAP scripts for manual verification
index.html          — Single-page frontend; imports pkg/f3m.js as an ES module
style.css           — Styles for the web UI
```

### Data flow

1. User types generators (comma-separated) → `js_compute(input: &str) -> JsSemigroup` (WASM)
2. JS reads properties directly from the `JsSemigroup` object (getters, no JSON)
3. `shortprop`, `combined_table`, `eval_expr` (all WASM) produce HTML strings injected into the page

### Key Rust types

- `math::Semigroup` — holds all computed properties; `element(x)` / `is_gap(x)` use the Apéry set for O(1) membership
- `math::compute(input)` — sliding-window algorithm; normalizes by GCD, tracks residue classes mod m
- `eva::eval(expr)` — recursive-descent parser/evaluator for arithmetic over `usize`
- `jshelpers::EvalCtx` — substitutes semigroup variables and `a[i]`/`q[i]` before calling `eva::eval`

### Grid color legend (index.html + style.css)

| CSS class  | Meaning                            |
|------------|------------------------------------|
| `sg-in`    | Element of the semigroup           |
| `sg-out`   | Gap                                |
| `sg-gen`   | Minimal generator                  |
| `sg-apery` | Apéry set element (non-generator)  |
| `sg-frob`  | Frobenius number                   |
| `sg-pf`    | Pseudo-Frobenius number            |
