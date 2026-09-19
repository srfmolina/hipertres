//! Code *about* the board feature rather than the feature itself: tunable
//! values, diagnostics and tests.
//!
//! `meta` is private to `board` (`mod meta;`), so `pub` in here only means
//! "visible inside the board feature".

pub mod constant;
pub mod debug;

#[cfg(test)]
mod tests;
