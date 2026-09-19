//! Code *about* the hyperboard feature rather than the feature itself:
//! tunable values, diagnostics and tests.
//!
//! `meta` is private to `hyperboard` (`mod meta;`), so `pub` in here only
//! means "visible inside the hyperboard feature".

pub mod constant;
pub mod debug;

#[cfg(test)]
mod tests;
