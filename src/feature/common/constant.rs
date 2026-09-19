//! Tunable values shared by several features.

use bevy::color::Color;

/// Muted colors (see `mute` in `common/color.rs`) are mixed toward this neutral gray.
pub const MUTE_TARGET: Color = Color::srgb(0.30, 0.30, 0.32);
/// How far toward `MUTE_TARGET`: 0.0 keeps the color, 1.0 turns it fully gray.
pub const MUTE_AMOUNT: f32 = 0.65;
