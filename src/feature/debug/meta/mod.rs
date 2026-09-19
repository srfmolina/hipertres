//! Code *about* the debug feature rather than the feature itself: tunable
//! values and tests.
//!
//! `meta` is private to `debug` (`mod meta;`), so `pub` in here only means
//! "visible inside the debug feature".

pub mod constant;

#[cfg(test)]
mod tests;
