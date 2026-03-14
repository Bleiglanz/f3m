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

The `pkg/` directory contains committed WASM build artifacts. Rebuild it with `wasm-pack build --target web` after changing `src/lib.rs`.

## Architecture

```
src/lib.rs      — Rust library: all math + wasm-bindgen export
src/main.rs     — Unused binary stub
pkg/            — wasm-pack output: f3m.js, f3m_bg.wasm, f3m.d.ts, package.json
index.html      — Single-page frontend; imports pkg/f3m.js as an ES module
style.css       — Styles for the web UI
```

### Data flow

1. User types generators (comma-separated) → `js_semigroup(input: &str) -> String` (WASM, `src/lib.rs:166`)
2. Returns a JSON string with fields: `e`, `f`, `m`, `count_set`, `count_gap`, `max_gen`, `gen_set`, `apery_set`
3. `index.html` parses the JSON and builds the result table (elements, gaps, Apéry table, color-coded structure grid) entirely in JS

### Core Rust types (`src/lib.rs`)

- `Semigroup` struct — holds all computed properties; `element(x)` and `is_gap(x)` use the Apéry set for O(1) membership test
- `compute(input: &[usize])` — sliding-window algorithm; normalizes by GCD first, then tracks residue classes to find generators and Apéry set without enumerating all elements
- `gcd` / `gcd_vec` — helpers used for normalization

### Grid color legend (index.html + style.css)

| CSS class  | Meaning                            |
|------------|------------------------------------|
| `sg-in`    | Element of the semigroup           |
| `sg-out`   | Gap                                |
| `sg-gen`   | Minimal generator                  |
| `sg-apery` | Apéry set element (non-generator)  |
| `sg-frob`  | Frobenius number                   |
