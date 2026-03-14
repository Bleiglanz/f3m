# Numerical Semigroup Calculator

A browser-based tool for computing properties of numerical semigroups from a list of generators. The core algorithm is written in Rust and compiled to WebAssembly; the UI is plain HTML/JS/CSS with no framework.

## What it computes

| Property | Description |
|---|---|
| Wilf | Sporadic count / (f+1) vs 1/e (Wilf conjecture) |
| Embedding dimension (e) | Number of minimal generators |
| Frobenius number (f) | Largest integer not in the semigroup |
| PF(S) | Pseudo-Frobenius numbers |
| Type t | Cardinality of PF(S) |
| Symmetric | Whether the semigroup is symmetric (t = 1) |
| Multiplicity (m) | Smallest positive element |
| Apéry set | One representative per residue class mod m |
| Structure grid | Color-coded element/gap layout |
| Kunz coordinates | c_ij table derived from the Apéry set |

## Usage

Enter a comma-separated list of generators, e.g. `3, 5, 7`. The GCD is normalized automatically. Click any gap, Frobenius number, or pseudo-Frobenius element to add it as a generator. Click a generator to remove it.

The random buttons generate:
- **Random** — 8 random generators in [10, 100]
- **Random3f** — generators that force a large Frobenius number
- **Symmetric** — a random symmetric semigroup
- **RandomPrimes** — generators from a prime range

## Build

Prerequisites: [Rust](https://rustup.rs), [wasm-pack](https://rustwasm.github.io/wasm-pack/)

```bash
# Run Rust unit tests
cargo test

# Build the WASM package (regenerates pkg/)
wasm-pack build --target web

# Serve locally (ES modules require a server)
python3 -m http.server 8080
# open http://localhost:8080
```

The `pkg/` directory contains committed build artifacts. Rebuild after changing `src/lib.rs`.

## Architecture

```
src/lib.rs   — Rust: all semigroup math + wasm-bindgen exports
pkg/         — wasm-pack output (committed): f3m.js, f3m_bg.wasm, *.d.ts
index.html   — Single-page frontend
style.css    — Styles
```

Membership testing uses the Apéry set for O(1) lookup. The sliding-window algorithm in `compute()` finds the minimal generating set and Apéry set without enumerating all elements.
