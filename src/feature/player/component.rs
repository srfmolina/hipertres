//! The player's components.

use bevy::prelude::*;

/// A player. Its entity is its identity: cells, boards and turns store this
/// `Entity`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[require(Name = Name::new("Player"))]
pub struct Player {
    /// The char drawn on the player's cells and won boards, e.g. 'x'.
    pub symbol: char,
}

/// The color of the player's cells and won boards.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerColor(pub Color);

/// "Draw this player's icon at my center." It goes on an entity with a
/// `Sprite` (a cell, a board's winner square...), which keeps painting its
/// own square: this component only adds the icon on top.
///
/// The icon is a child entity that the player feature creates and replaces
/// whenever this component changes (see `draw_player_marks`).
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlayerMark {
    /// Whose icon to draw. `None`: no icon.
    pub player: Option<Entity>,
    /// Draw the icon with muted colors, like the square under it.
    pub muted: bool,
}

/// Marks the icon child created for a `PlayerMark`.
#[derive(Component)]
#[require(Name = Name::new("Player icon"))]
pub(super) struct PlayerIcon;
