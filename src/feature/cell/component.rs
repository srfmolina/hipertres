//! The cell's components.

use bevy::prelude::*;

use super::meta::constant::{CELL_SIZE, UNPRESSED_COLOR};

/// A cell's state.
///
/// `#[require(...)]` lists *required components*, which are added
/// automatically when a `Cell` is spawned:
/// - `Sprite`: a white square, unless you provide your own. `Sprite` in turn
///   requires `Transform` and `Visibility`.
/// - `Pickable`: lets Bevy's picking detect the cell under the pointer.
///   Sprites without it are ignored, and clicks go to the window behind them.
///   To make a cell unclickable, set it to `Pickable::IGNORE`: picking then
///   skips the cell completely, so it never receives clicks.
/// - `Name`: a label for debugging, so logs say "Cell 374v0" instead of "374v0".
/// - `Muted`: whether the cell is drawn with muted colors (not muted by default).
///
/// So `commands.spawn(Cell::default())` gives a complete, visible, clickable cell.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
#[require(
    Sprite = Sprite::from_color(UNPRESSED_COLOR, Vec2::splat(CELL_SIZE)),
    Pickable,
    Name = Name::new("Cell"),
    Muted
)]
pub struct Cell {
    /// `None` when unpressed. `Some(color)` when pressed, remembering the
    /// color it was pressed with, even if the parent's `PressedColor` changes later.
    pub pressed: Option<Color>,
}

/// Whether a cell is drawn with muted colors (see `mute` in `common/color.rs`).
/// Its parent (the board) sets it, e.g. to show that the cell can't be played
/// right now.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Muted(pub bool);

/// The color a cell turns when pressed.
///
/// It goes on a cell's **parent** (the board), not on the cell: the parent
/// decides the color for all its cells. A cell with no parent, or whose
/// parent has no `PressedColor`, uses `DEFAULT_PRESSED_COLOR`.
///
/// It lives in the cell feature, not in `common`, even though the board is
/// the one that inserts it: the cell defines what it means and reacts to it.
/// The board just uses the cell's API, like it does with `Muted`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PressedColor(pub Color);
