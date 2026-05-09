//! HTML fragment generators for numerical-semigroup views.
//!
//! Every public function in this crate returns a `String` of HTML and is free
//! of `wasm_bindgen` attributes. The `semigroup_explorer` crate wraps these
//! functions with thin `#[wasm_bindgen]` shims for the browser; the
//! `semigroup_cones` binary reuses the same implementations to produce its
//! aggregate HTML reports without going through the JS bridge.

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

pub mod classify;
pub mod combined_table;
pub mod diagonals;
pub mod shortprops;
mod spans;
pub mod tilt;

pub use classify::classify_table;
pub use combined_table::combined_table;
pub use diagonals::diagonals_table;
pub use shortprops::{shortprop, shortprop_cells};
pub use spans::{ClassSets, class_sets, span};
pub use tilt::tilt_table;

/// Render a boolean as the bright check / red prohibited glyph used
/// throughout the property tables and waldicone reports.
#[must_use]
pub const fn glyph(b: bool) -> &'static str {
    if b { "\u{2705}" } else { "\u{1F6AB}" }
}
