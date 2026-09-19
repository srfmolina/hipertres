//! Tunable values for players.

use bevy::color::Color;

/// The players of a game, in turn order: `(symbol, color)`.
pub const DEFAULT_PLAYERS: [(char, Color); 2] = [
    ('o', Color::srgb(0.25, 0.55, 0.95)), // blue
    ('x', Color::srgb(0.95, 0.55, 0.20)), // orange
];

/// The icon's size, as a fraction of the square it is drawn on.
pub const MARK_SCALE: f32 = 0.7;
/// How far in front of its square the icon is drawn. Small, so a board's
/// winner square (1 unit in front of its cells) still covers the cells' icons.
pub const MARK_Z: f32 = 0.1;
/// Color of the icon (before muting).
pub const MARK_TEXT_COLOR: Color = Color::WHITE;
