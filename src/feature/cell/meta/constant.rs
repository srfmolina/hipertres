//! Tunable values for cells.

use bevy::color::Color;

/// Width and height of a cell, in world units (= pixels with the default camera).
pub const CELL_SIZE: f32 = 150.0;

/// Color of a cell that is not pressed.
pub const UNPRESSED_COLOR: Color = Color::WHITE;
