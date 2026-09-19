//! Code *about* the player feature rather than the feature itself: tunable
//! values and tests.
//!
//! `meta` is private to `player` (`mod meta;`), so `pub` in here only means
//! "visible inside the player feature".

pub mod constant;

#[cfg(test)]
mod tests;
