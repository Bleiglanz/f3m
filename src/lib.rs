//! Numerical-semigroup calculator: properties (Frobenius, Apéry, gaps, Kunz, …)
//! exposed as a Rust library and as WebAssembly bindings via `wasm-bindgen`.
//!
//! Integration tests live in `tests/integration.rs` and cross-check every
//! property against the GAP `NumericalSgps` package (see `gap/test.g`).

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr
)]

pub mod eva;
pub mod jshelpers;
pub mod math;
