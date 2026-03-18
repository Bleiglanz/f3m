# Numerical Semigroup Calculator

A browser-based tool for computing properties of numerical semigroups from a list of generators. The core algorithm is written in Rust and compiled to WebAssembly; the UI is plain HTML/JS/CSS with no framework.

## Features

### Properties computed

| Property | Symbol | Description |
|---|---|---|
| Multiplicity | m | Smallest positive element |
| Frobenius number | f | Largest integer not in the semigroup |
| Embedding dimension | e | Number of minimal generators |
| Genus | g | Number of gaps |
| Sporadic elements | c−g | Elements below f that are in the semigroup |
| Type | t | Cardinality of PF(S) |
| Symmetric | ✅/🚫 | Whether t = 1 (equivalently f + 1 = 2g) |
| Wilf quotient | — | (c−g) / (f+1) vs 1/e; Wilf conjecture says ≥ 1/e |
| Generators | gen | Minimal generating set |
| Pseudo-Frobenius | PF(S) | Gaps x such that x + s ∈ S for all s ∈ S \ {0} |
| Special PF | SPF | PF elements of the form gen[i] − gen[j] not dividing f |

### Structure grid

A color-coded layout of elements and gaps, with an Apéry row and Kunz coefficient matrix c_ij. An offset slider permutes the columns to show any residue class first.

| Color class | Meaning |
|---|---|
| `sg-gen` | Minimal generator |
| `sg-apery` | Apéry set element (non-generator) |
| `sg-frob` | Frobenius number |
| `sg-pf` | Pseudo-Frobenius number |
| `sg-in` | Element of the semigroup |
| `sg-out` | Gap |

### History tab

Every computed semigroup is appended to a summary table. Click any row to restore it as the current input. The table includes the expression and its evaluated value (see below) for each entry.

At the bottom of the history table a full [GAP](https://gap-packages.github.io/numericalsgps/) script is generated that reproduces all computations and can be verified with the `NumericalSgps` package. A **Copy** button copies it to the clipboard.

### Interactive editing

- Click any element in the structure grid to toggle it as a generator (add gap / remove minimal generator).
- Click any Frobenius or PF span in the properties table to toggle it.

### Random generators

| Button | Action |
|---|---|
| Random | 8 random generators in [10, 100] |
| Random3f | Generators chosen to produce a large Frobenius number |
| Symmetric | A random symmetric semigroup |
| RandomPrimes | Generators from a prime range |

---

## Expression evaluator

The properties table contains an input field that evaluates an arithmetic expression over the current semigroup and displays the result. The expression is also recorded in the history table for every entry. Hover the input field for a quick-reference tooltip.

### Scalar variables

| Symbol | Value |
|---|---|
| `m` | Multiplicity |
| `f` | Frobenius number |
| `e` | Embedding dimension |
| `g` | Number of gaps (genus) |
| `t` | Type |
| `Q` | Largest minimal generator (denoted *ae* in the literature) |
| `A` | Largest Apéry element (= f + m) |

### Indexed access

| Syntax | Value |
|---|---|
| `a[i]` | i-th Apéry set element; 0 if i ≥ m |
| `q[i]` | i-th minimal generator; 0 if i ≥ e |

The index expression is itself fully evaluated, so `a[m-1]`, `q[e-1]`, `a[a[0]+1]`, and similar compound expressions all work.

### Operators

`+`, `-`, `*`, `/` over non-negative integers. `/` is integer (truncating) division. Subtraction that would go below zero returns an error (shown as `—`). Standard operator precedence and parentheses are supported.

### Examples

| Expression | Meaning |
|---|---|
| `f + 1` | Conductor |
| `2 * g` | Twice the genus (equals f+1 for symmetric semigroups) |
| `a[m-1]` | Largest Apéry element (same as A = f+m) |
| `q[e-1] - q[0]` | Difference of largest and smallest generator |
| `a[1] * e` | First non-trivial Apéry element times embedding dimension |

---

## Build

Prerequisites: [Rust](https://rustup.rs), [wasm-pack](https://rustwasm.github.io/wasm-pack/)

```bash
# Run Rust unit tests
cargo test

# Lint (auto-fix, pedantic)
cargo clippy --fix --lib -p f3m

# Build the WASM package (regenerates pkg/)
wasm-pack build --target web

# Serve locally (ES modules require a server)
python3 -m http.server 8080
# open http://localhost:8080
```

The `pkg/` directory contains committed build artifacts and is updated by `wasm-pack build`.

---

## Architecture

```
src/
  lib.rs            — crate root: module declarations + tests
  math/mod.rs       — Semigroup struct, compute(), gcd, GAP code generation
  eva/mod.rs        — arithmetic expression evaluator (usize, recursive indexing)
  jshelpers/mod.rs  — WASM exports: JsSemigroup, HTML rendering, eval_expr
  main.rs           — unused binary stub
pkg/                — wasm-pack output (committed): f3m.js, f3m_bg.wasm, *.d.ts
gap/                — example GAP scripts for manual verification
index.html          — single-page frontend
style.css           — styles
```

### Data flow

1. User types generators (comma-separated integers).
2. `js_compute(input)` (WASM) normalises by GCD and runs the sliding-window algorithm, returning a `JsSemigroup` object.
3. The JS frontend reads properties directly from the object and calls `shortprop`, `combined_table`, and `eval_expr` to build the UI.

### Core algorithm

`compute()` in `src/math/mod.rs` uses a sliding window of width 2 × max(generators). It tracks residue classes modulo the multiplicity m to find the Apéry set and the minimal generating set in a single pass, without enumerating all semigroup elements. Membership testing is O(1) via the Apéry set.

### Expression evaluator (`src/eva/mod.rs`)

A hand-written recursive-descent parser over `usize`. No dependencies. Grammar:

```
expr   = term   (('+' | '-') term)*
term   = factor (('*' | '/') factor)*
factor = '(' expr ')' | number
number = [0-9]+
```

`eval_expr` in `jshelpers` pre-processes the input string — substituting `a[i]`/`q[i]` (with recursively evaluated indices) and scalar variables — before passing it to the evaluator.
