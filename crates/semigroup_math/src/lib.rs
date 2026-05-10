//! Pure-Rust numerical-semigroup algorithms.
//!
//! This crate implements the core mathematics: the [`Semigroup`](math::Semigroup)
//! type and its derived invariants (Frobenius number, Apéry set, Kunz coefficients,
//! pseudo-Frobenius numbers, glue/canonical-ideal/symmetric-partner constructions),
//! the integer matrix utilities under [`math::matrix`], the recursive-descent
//! arithmetic evaluator in [`eva`], and the GAP-source generators used by the
//! verification scripts.
//!
//! Integration tests cross-check every property against the GAP `NumericalSgps`
//! package (see `gap/test.g`).

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
pub mod math;
pub mod strata;
