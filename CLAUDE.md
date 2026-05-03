# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

A **Numerical Semigroup Calculator** — a browser-based tool that computes properties of numerical semigroups (embedding dimension, Frobenius number, multiplicity, generators, Apéry set, gaps, etc.) from a user-supplied list of generators. The core algorithm runs in Rust compiled to WebAssembly; the UI is plain HTML/JS/CSS with no framework.

## Build commands

```bash
# Run all unit and integration tests across the workspace
cargo test --workspace

# Run a single test by name
cargo test test_gcd

# Lint the whole workspace (-D warnings keeps clippy strict)
cargo clippy --workspace -- -D warnings

# Build the WASM package (regenerates pkg/ at the repo root)
wasm-pack build crates/semigroup_explorer --target web --out-dir ../../pkg

# Run the native CLI (binary name: waldicone)
cargo run --release --bin waldicone [gmax]

# Serve the app locally (required — ES modules don't work over file://)
python3 -m http.server 8080
# then open http://localhost:8080
```

The `pkg/` directory holds the wasm-pack output and is gitignored — rebuild it locally with the command above after changing any source file in `crates/semigroup_math`, `crates/html_helpers`, or `crates/semigroup_explorer`. The deployed site (GitHub Pages) builds `pkg/` in CI via `.github/workflows/pages.yml`.

## Architecture

This is a Cargo workspace with four crates plus the static frontend:

```
Cargo.toml                               — [workspace] root, shared dependency versions
crates/
  semigroup_math/                        — pure Rust, no wasm, no HTML
    src/lib.rs                           — declares math + eva
    src/math/{mod,glue,matrix,semigroup,symmetric_partner}.rs
                                         — Semigroup, compute(), gcd, GAP code generation,
                                           Kunz matrix utilities, gluing, canonical ideal
    src/eva/mod.rs                       — arithmetic expression evaluator
    tests/integration.rs                 — GAP-cross-checked property tests
  html_helpers/                          — pure-string HTML generators, no wasm-bindgen
    src/{combined_table,shortprops,tilt,classify,diagonals,spans}.rs
                                         — every public fn returns String; both other
                                           crates call these to render views
  semigroup_explorer/                    — wasm-bindgen wrapper crate (cdylib + rlib)
    src/lib.rs                           — JsSemigroup + thin #[wasm_bindgen] shims
                                           around html_helpers + semigroup_math
    src/{pagestate,js_eval,jsgraph}.rs   — WASM-only state, evaluator wrapper, graph data
  semigroup_cones/                       — native CLI binary "waldicone"
    src/main.rs                          — spawns bundled Normaliz, builds aggregate
                                           HTML reports via html_helpers
pkg/                                     — wasm-pack output: semigroup_explorer.js,
                                           semigroup_explorer_bg.wasm, …
gap/                                     — example GAP scripts for manual verification
normaliz/                                — bundled Normaliz binary + cached I/O artifacts
index.html, style.css, jsmodules/        — static frontend (imports pkg/semigroup_explorer.js)
```

### Crate boundaries

- `semigroup_math` has no `wasm_bindgen`, no HTML, no `rayon`. Anything in it must build for both native and `wasm32-unknown-unknown`.
- `html_helpers` depends only on `semigroup_math`; functions take `&Semigroup` and return `String`. No `wasm_bindgen`.
- `semigroup_explorer` is the only crate that uses `wasm_bindgen`. The exported HTML helpers (`combined_table`, `shortprop`, `tilt_table`, `js_classify_table`, `js_diagonals_table`) are one-line wrappers around their `html_helpers` counterparts.
- `semigroup_cones` depends on `semigroup_math` + `html_helpers` + `rayon`. It does NOT depend on `wasm-bindgen`, so `cargo build --bin waldicone` skips that toolchain entirely.

### Data flow

1. User types generators (comma-separated) → `js_compute(input: &str) -> JsSemigroup` (WASM)
2. JS reads properties directly from the `JsSemigroup` object (getters, no JSON)
3. `shortprop`, `combined_table`, `eval_expr` (all WASM) produce HTML strings injected into the page

### Key Rust types

- `semigroup_math::math::Semigroup` — holds all computed properties; `element(x)` / `is_gap(x)` use the Apéry set for O(1) membership
- `semigroup_math::math::compute(input)` — sliding-window algorithm; normalizes by GCD, tracks residue classes mod m
- `semigroup_math::eva::eval(expr)` — recursive-descent parser/evaluator for arithmetic over `usize`
- `semigroup_explorer::js_eval::EvalCtx` — substitutes semigroup variables and `a[i]`/`q[i]` before calling `eva::eval`

### Grid color legend (index.html + style.css)

| CSS class  | Meaning                            |
|------------|------------------------------------|
| `sg-in`    | Element of the semigroup           |
| `sg-out`   | Gap                                |
| `sg-gen`   | Minimal generator                  |
| `sg-apery` | Apéry set element (non-generator)  |
| `sg-frob`  | Frobenius number                   |
| `sg-pf`    | Pseudo-Frobenius number            |
# CLAUDE.md — Rust Development Guidelines

This file governs all Rust code produced or modified in this repository.
Apply every rule below unconditionally. When in doubt, choose the stricter
interpretation. These guidelines take precedence over general defaults.

---

## 1. Toolchain & Edition

- Target **Rust stable**, current edition (`edition = "2021"` in `Cargo.toml`).
- Pin the MSRV explicitly: `rust-version = "1.XX"` in `Cargo.toml`.
- Format with `rustfmt` (default config). Every file must pass `cargo fmt --check`.
- Lint with `cargo clippy -- -D warnings`. Zero clippy warnings are permitted.
- Run `cargo deny check` (licenses, advisories, duplicates) in CI.

---

## 2. Compiler & Lint Configuration

Place the following at the crate root (`lib.rs` / `main.rs`). Do **not** scatter
`#![allow(...)]` suppressions in individual modules without a documented reason
placed in an adjacent comment.

```rust
#![forbid(unsafe_code)]          // see §10 for the single exception path
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
)]
#![warn(
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
)]
```

Suppress a lint only when unavoidable; place the `#[allow]` on the smallest
possible scope and explain why in a `// ALLOW: ...` comment on the line above.

---

## 3. Error Handling

### 3.1 Never use `.unwrap()` or `.expect()` in library code

```rust
// ✗ bad
let val = map.get("key").unwrap();

// ✓ good — propagate with `?`
let val = map.get("key").ok_or(Error::MissingKey("key"))?;
```

`.expect()` is permitted **only** in:
- `main()`, with a message that names the invariant being assumed.
- Test code (`#[cfg(test)]`).
- Build scripts (`build.rs`).

### 3.2 Use typed errors

Define a crate-level `Error` enum with one variant per distinct failure mode.
Derive or implement `std::error::Error` and `Display`. Use `thiserror` for
straightforward cases; implement manually when the message needs runtime context
that `thiserror` cannot express cleanly.

```rust
/// Errors that can occur while parsing a widget descriptor.
#[derive(Debug, thiserror::Error)]
pub enum WidgetError {
    /// The descriptor byte stream was shorter than the minimum header size.
    #[error("descriptor too short: need {min} bytes, got {actual}")]
    TooShort { min: usize, actual: usize },

    /// An unrecognised type tag was encountered.
    #[error("unknown type tag 0x{tag:02X} at offset {offset}")]
    UnknownTag { tag: u8, offset: usize },
}
```

### 3.3 Use `Result` as the return type of all fallible public functions

`pub fn foo(...) -> Result<T, E>` — never `Option` for error paths where
the reason matters to the caller.

---

## 4. Documentation

### 4.1 Every public item must have a doc comment

Doc comments must answer: *what does this do*, *when does it fail*,
*what are the non-obvious constraints*. Include at least one `# Example`
section for non-trivial public functions and all public types.

```rust
/// Decodes a length-prefixed UTF-8 string from `buf` starting at `offset`.
///
/// The first two bytes at `offset` are interpreted as a little-endian `u16`
/// length, followed by exactly that many bytes of UTF-8 data.
///
/// # Errors
///
/// Returns [`DecodeError::BufferTooShort`] when fewer bytes remain than the
/// declared length requires.
/// Returns [`DecodeError::InvalidUtf8`] when the payload is not valid UTF-8.
///
/// # Example
///
/// ```rust
/// let buf = b"\x05\x00hello";
/// assert_eq!(decode_lp_string(buf, 0).unwrap(), "hello");
/// ```
pub fn decode_lp_string(buf: &[u8], offset: usize) -> Result<&str, DecodeError> {
    // ...
}
```

### 4.2 Inline comments

Use `//` inline comments to explain *why*, not *what*. The code itself should
show what is happening; comments should clarify non-obvious decisions,
constraints, or trade-offs.

```rust
// We truncate to i32 here because the downstream C ABI expects a signed int.
// Values above i32::MAX are rejected earlier in validation.
let handle = id as i32;
```

### 4.3 Module-level documentation

Every `mod` declaration that is not trivially self-describing must have a
`//! ...` inner doc comment at the top of its file, summarising purpose and
scope.

---

## 5. Types & API Design

### 5.1 Prefer owned types in public APIs; borrow internally

```rust
// ✓ public API — caller controls lifetime
pub fn new(name: String) -> Self { ... }

// ✓ private helper — borrows for efficiency
fn validate(name: &str) -> Result<(), NameError> { ... }
```

### 5.2 Use the newtype pattern to enforce invariants

```rust
/// A widget name that has been validated as non-empty and ASCII-printable.
pub struct WidgetName(String);

impl WidgetName {
    /// Constructs a `WidgetName`, returning an error if `s` is empty or
    /// contains non-printable ASCII characters.
    pub fn new(s: impl Into<String>) -> Result<Self, NameError> {
        let s = s.into();
        if s.is_empty() { return Err(NameError::Empty); }
        if !s.is_ascii() || s.bytes().any(|b| !b.is_ascii_graphic()) {
            return Err(NameError::NonPrintable);
        }
        Ok(Self(s))
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str { &self.0 }
}
```

### 5.3 Implement standard traits consistently

For every data type, derive or implement as appropriate:
`Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Display`, `Default`.
Do not derive `Copy` for types that carry heap allocation.

### 5.4 Prefer `impl Trait` in argument position over generics when a single trait bound suffices

```rust
// ✓ clearer at call site
pub fn process(input: impl Read) -> Result<Summary, IoError> { ... }
```

### 5.5 Avoid boolean parameters — use enums

```rust
// ✗
fn open(path: &Path, create: bool) -> Result<File, IoError> { ... }

// ✓
pub enum OpenMode { OpenExisting, CreateOrOpen }
fn open(path: &Path, mode: OpenMode) -> Result<File, IoError> { ... }
```

---

## 6. Memory & Resource Safety

### 6.1 Prefer stack allocation; profile before reaching for the heap

Use `Box`, `Vec`, `Arc` only when lifetime or size requirements demand it.
Avoid `Vec` when a fixed-size array suffices.

### 6.2 Drop order matters — document it when non-obvious

If a struct holds resources that must be released in a specific order,
implement `Drop` explicitly and add a comment explaining the ordering
invariant.

### 6.3 Use `Arc<Mutex<T>>` / `Arc<RwLock<T>>` for shared mutable state

Prefer `RwLock` when reads are more frequent than writes. Hold locks for
the shortest possible span; never call external functions while holding a lock.

### 6.4 Avoid cloning to satisfy the borrow checker

A clone that hides a lifetime problem is a design smell. Restructure the
ownership model instead.

---

## 7. Concurrency

- Use `std::sync::mpsc` or `crossbeam-channel` for message passing.
- Annotate `Send + Sync` bounds explicitly when defining thread-safe
  abstractions.
- Never use `unsafe` to implement `Send` or `Sync` without a thorough
  written justification in an adjacent `// SAFETY:` comment.
- Prefer structured concurrency (`std::thread::scope`, `rayon`, `tokio::join!`)
  over detached spawning.
- For async code, keep `async fn` signatures in traits behind
  `#[async_trait]` until native async-in-traits stabilises in the MSRV.

---

## 8. Security

### 8.1 Validate all input at the boundary

Parse, do not validate. Use the type system to make invalid states
unrepresentable. Once data has crossed the public API boundary and been
accepted as a typed value, it must already satisfy all invariants.

### 8.2 No panics in production paths

`panic!`, `unreachable!()`, `todo!()`, and `unimplemented!()` must not appear
in code reachable at runtime outside tests or main. Use `Result`/`Option`
instead. Clippy's `clippy::todo` and `clippy::unimplemented` warnings enforce
this mechanically.

### 8.3 Serialisation / deserialisation

When using `serde`:
- Apply `#[serde(deny_unknown_fields)]` to all `Deserialize` structs that
  represent external (untrusted) data.
- Validate deserialized values in a `TryFrom` impl or a `#[serde(try_from)]`
  attribute; do not rely solely on field types.

### 8.4 Cryptography

Never implement cryptographic primitives. Use audited crates (`ring`,
`rustls`, `argon2`, `chacha20poly1305`). Treat all custom encoding/hashing
as non-cryptographic unless an audited cryptographic crate is involved.

### 8.5 Secrets in memory

Use crates with built-in zeroisation (`secrecy`, `zeroize`) for secret
material. Never log, `Debug`-print, or return secrets from public functions
as plain `String`.

```rust
use secrecy::{ExposeSecret, Secret};

pub struct Credentials {
    pub username: String,
    // The password is wrapped in `Secret` so its `Debug` impl redacts it.
    pub password: Secret<String>,
}
```

### 8.6 Integer arithmetic

Prefer checked or saturating arithmetic (`checked_add`, `saturating_mul`)
over bare `+`, `*` for values whose bounds are not statically obvious.
In release builds, Rust does *not* trap on integer overflow.

### 8.7 File-system and path handling

Never construct paths from untrusted strings using string concatenation.
Use `std::path::Path::join` and validate that the resolved path stays
within the intended root (path-traversal check).

### 8.8 Environment variables and configuration

Treat environment variables as untrusted input. Parse them into typed values
immediately; do not carry raw `OsString`/`String` values through the
application.

---

## 9. Dependencies

- Prefer the standard library over external crates for simple tasks.
- Before adding a new dependency, check it against `cargo deny` advisories.
- Justify each non-trivial dependency in a `# Dependencies` section of the
  relevant module's or binary's doc comment, or in `Cargo.toml` as an
  inline comment.
- Avoid crates with `unsafe`-heavy internals unless there is no safe
  alternative and the crate is widely audited (e.g., `memmap2`, `ring`).
- Keep dependency version constraints as tight as is practically reasonable:
  prefer `"^1.2"` over `"*"`.

---

## 10. `unsafe` Code

`#![forbid(unsafe_code)]` is set at the crate level. To introduce `unsafe`:

1. Remove the forbid attribute from the crate root.
2. Add `#![deny(unsafe_code)]` in its place.
3. Scope the `#[allow(unsafe_code)]` to the single module that requires it.
4. For **every** `unsafe` block, write a `// SAFETY:` comment immediately
   above it that:
   - Names every invariant that must hold for the block to be sound.
   - Explains why each invariant is upheld at this call site.
5. Add a unit test that exercises the unsafe code path under miri
   (`cargo +nightly miri test`).

```rust
// SAFETY: `ptr` was obtained from `Box::into_raw` in `Widget::new` and has
// not been aliased or freed since then. The calling thread holds the only
// live reference to this allocation.
let widget = unsafe { Box::from_raw(ptr) };
```

---

## 11. Testing

### 11.1 Coverage expectations

- Every public function must have at least one test.
- Every `Err` variant of a public `Result`-returning function must have a
  test that triggers it.
- Property-based tests (`proptest`, `quickcheck`) are encouraged for parsing
  and arithmetic-heavy code.

### 11.2 Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests live in the same file as the code they test, in this module.
    // Integration tests live in `tests/`.

    #[test]
    fn decode_lp_string_returns_correct_slice() {
        let buf = b"\x05\x00hello";
        assert_eq!(decode_lp_string(buf, 0).unwrap(), "hello");
    }

    #[test]
    fn decode_lp_string_errors_on_truncated_buffer() {
        let buf = b"\x05\x00hel";   // 3 bytes instead of 5
        assert!(matches!(
            decode_lp_string(buf, 0),
            Err(DecodeError::BufferTooShort { .. })
        ));
    }
}
```

### 11.3 No `unwrap` / `expect` in tests that can produce false negatives

Use `assert!`, `assert_eq!`, or `?` with `-> Result<(), Box<dyn Error>>` as
the test return type so failures surface as test failures rather than panics
that obscure the root cause.

---

## 12. Formatting & Style

- **Line length**: 100 characters (configured in `rustfmt.toml`:
  `max_width = 100`).
- **Imports**: group as `std` → external crates → internal crates/modules,
  separated by blank lines. Use `rustfmt`'s `imports_granularity = "Crate"`.
- **Naming**: follow Rust API guidelines — `snake_case` for functions and
  variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- **Trailing commas**: always in multi-line lists, match arms, and function
  argument lists.
- **No `return` at the end of a function body** unless needed for clarity in
  complex control flow.

---

## 13. Workflow Expected from Claude Code

When writing or modifying Rust in this repository, Claude Code must:

1. **Run the full check suite** before presenting code as complete:
   ```
   cargo fmt --check && cargo clippy -- -D warnings && cargo test
   ```
2. **Never introduce a compilation warning**, even temporarily.
3. **Explain non-trivial design decisions** in a brief comment within the
   code, not only in the chat.
4. **Prefer minimal diffs** when editing existing files: change only what is
   necessary to satisfy the task.
5. **Update documentation** for any public item whose behaviour changes.
6. **Flag security-sensitive code** by adding a `// SECURITY:` comment that
   briefly describes the threat model consideration at that point.
