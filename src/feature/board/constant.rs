//! Tunable values for the board.
//!
//! This module is private to `board` (declared as `mod constant;`, without
//! `pub`), so `pub` here only means "visible inside the board feature".

use bevy::color::Color;

/// Number of cells per side of the board.
pub const GRID_SIZE: usize = 3;
/// Width and height of one cell, in world units (= pixels with the default camera).
pub const CELL_SIZE: f32 = 150.0;
/// Empty space between neighbouring cells.
pub const CELL_GAP: f32 = 10.0;

pub const CELL_COLOR: Color = Color::srgb(0.85, 0.85, 0.80);
