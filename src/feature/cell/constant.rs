//! Tunable values for cells.

use bevy::color::Color;

/// Width and height of a cell, in world units (= pixels with the default camera).
pub const CELL_SIZE: f32 = 150.0;

/// Color of a cell that is not pressed.
pub const UNPRESSED_COLOR: Color = Color::WHITE;

/// Color of a pressed cell when its parent doesn't choose one
/// (see `PressedColor` in `cell/mod.rs`).
pub const DEFAULT_PRESSED_COLOR: Color = Color::srgb(0.25, 0.55, 0.95);

/// Muted colors (see `mute` in `cell/mod.rs`) are mixed toward this neutral gray.
pub const MUTE_TARGET: Color = Color::srgb(0.30, 0.30, 0.32);
/// How far toward `MUTE_TARGET`: 0.0 keeps the color, 1.0 turns it fully gray.
pub const MUTE_AMOUNT: f32 = 0.65;
