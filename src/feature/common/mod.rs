//! Code shared by several features that belongs to none of them.
//!
//! This is not a feature: it has no plugin. Only put something here when no
//! feature owns it. A type that a feature *defines and reacts to* (like the
//! cell's `PressedColor`, which boards set) belongs to that feature, even when
//! other features use it.

pub mod color;
pub mod constant;

#[cfg(test)]
mod tests;
