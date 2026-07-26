//! Shared presentation primitives for AUV command frontends.
//!
//! This crate owns rendering and format routing. App crates continue to own
//! their command result types and JSON contracts.

extern crate self as auv_cli_common;

pub mod outputs;

/// Derive [`outputs::formats::table::TableRow`] from named struct fields.
///
/// Fields default to an uppercase header derived from their Rust name. Use
/// `#[table(header = "...")]`, `#[table(hidden)]`, `#[table(wide)]`, or
/// `#[table(display_with = "path::to::function")]` to override that schema.
/// `display_with` also accepts an inline closure. `display_zero` provides an
/// explicit fallback for false, absent, numeric-zero, or empty values. Inline
/// closures may read other fields from the same row through `self`.
pub use auv_cli_common_macros::TableRow;
