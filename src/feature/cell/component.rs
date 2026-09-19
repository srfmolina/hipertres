//! The cell's components.

use bevy::prelude::*;

use super::meta::constant::{CELL_SIZE, UNPRESSED_COLOR};
use crate::feature::player::PlayerMark;

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
/// - `PlayerMark`: draws the icon of the player who pressed the cell (see
///   the player feature). `update_cell_colors` keeps it in sync.
///
/// So `commands.spawn(Cell::default())` gives a complete, visible, clickable cell.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
#[require(
    Sprite = Sprite::from_color(UNPRESSED_COLOR, Vec2::splat(CELL_SIZE)),
    Pickable,
    Name = Name::new("Cell"),
    Muted,
    PlayerMark
)]
pub struct Cell {
    /// `None` when unpressed. `Some(player)` when pressed: the player entity
    /// that pressed it, remembered even if the parent's `ActivePlayer`
    /// changes later.
    pub pressed: Option<Entity>,
}

/// Whether a cell is drawn with muted colors (see `mute` in `common/color.rs`).
/// Its parent (the board) sets it, e.g. to show that the cell can't be played
/// right now.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Muted(pub bool);

/// The player whose cells get pressed right now.
///
/// It goes on a cell's **parent** (the board), not on the cell: the parent
/// decides who presses all its cells. A cell with no parent, or whose parent
/// has no `ActivePlayer`, ignores clicks: a cell can't be pressed by nobody.
///
/// It lives in the cell feature, not in the board, even though the board is
/// the one that inserts it: the cell defines what it means and reacts to it.
/// The board just uses the cell's API, like it does with `Muted`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivePlayer(pub Entity);
