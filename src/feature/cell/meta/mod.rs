//! Code *about* the cell feature rather than the feature itself: tunable
//! values, diagnostics and tests.
//!
//! `meta` is private to `cell` (`mod meta;`), so `pub` in here only means
//! "visible inside the cell feature".

pub mod constant;
pub mod debug;

#[cfg(test)]
mod tests;
