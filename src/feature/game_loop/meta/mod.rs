//! Code *about* the game loop feature rather than the feature itself:
//! tunable values, test helpers and tests.
//!
//! `meta` is private to `game_loop`, so `pub` in here only means "visible
//! inside the game loop feature".

pub mod constant;
pub mod debug;

#[cfg(test)]
pub(super) mod testing;

#[cfg(test)]
mod tests;
