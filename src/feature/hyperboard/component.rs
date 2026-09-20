//! The hyperboard's components.

use bevy::prelude::*;

use crate::feature::board::{GRID_SIZE, GridPosition};

/// The hyperboard's state. Its boards are its children, and it also holds the
/// game's `Turn` (defined by the game loop feature).
///
/// The type and its fields are `pub(super)`: visible in the whole hyperboard
/// feature, but not to other features. Other features go through
/// `spawn_hyperboard` and `Hyperboard::winner`.
#[derive(Component, Debug)]
#[require(Transform, Visibility, Name = Name::new("Hyperboard"))]
pub(super) struct Hyperboard {
    /// The board with the cell pressed during the current turn, if any.
    pub(super) active_board: Option<Entity>,
    /// The board played in the turn that just ended, until the next board is
    /// chosen from it.
    pub(super) played_board: Option<Entity>,
    /// Where the current player must play: the board at this position, or
    /// any board that isn't won or full when `None`.
    pub(super) next_board: Option<GridPosition>,
    /// The player who got three boards in a row.
    pub(super) winner: Option<Entity>,
}

impl Default for Hyperboard {
    fn default() -> Self {
        Self {
            active_board: None,
            played_board: None,
            // The first turn is played in the center board.
            next_board: Some(GridPosition {
                col: GRID_SIZE / 2,
                row: GRID_SIZE / 2,
            }),
            winner: None,
        }
    }
}

impl Hyperboard {
    /// The player who won the game, if any.
    pub fn winner(&self) -> Option<Entity> {
        self.winner
    }
}

/// Marks the square that covers a won hyperboard.
#[derive(Component)]
#[require(Name = Name::new("Hyperboard winner overlay"))]
pub(super) struct WinnerOverlay;
