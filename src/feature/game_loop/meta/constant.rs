//! Tunable values for the game loop.

use bevy::prelude::*;

/// Key that starts a new match, from any state but `Setup`.
pub const RESTART_KEY: KeyCode = KeyCode::KeyR;

/// Key that ends the current turn.
pub const END_TURN_KEY: KeyCode = KeyCode::Space;
