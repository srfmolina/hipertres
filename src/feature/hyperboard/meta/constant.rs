//! Tunable values for the hyperboard.

use bevy::prelude::*;

use crate::feature::board::{BOARD_SIZE, GRID_SIZE};

/// Key that ends the current turn.
pub const END_TURN_KEY: KeyCode = KeyCode::Space;

/// Boards are drawn at this fraction of their full size, so all 9 fit in the window.
pub const BOARD_SCALE: f32 = 0.5;
/// Empty space between neighbouring boards (after scaling).
pub const BOARD_GAP: f32 = 20.0;
/// Width and height of the whole hyperboard.
pub const HYPERBOARD_SIZE: f32 =
    GRID_SIZE as f32 * BOARD_SIZE * BOARD_SCALE + (GRID_SIZE as f32 - 1.0) * BOARD_GAP;
/// The winner overlay's `z`: above the boards' own winner overlays (`z = 1`).
pub const OVERLAY_Z: f32 = 2.0;
