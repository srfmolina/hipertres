//! The hyperboard's components.

use bevy::prelude::*;

use super::meta::constant::DEFAULT_PLAYERS;
use crate::feature::board::{GRID_SIZE, GridPosition};

/// The hyperboard's state. Its boards are its children, and it also holds the
/// game's `Turn`.
///
/// The fields are `pub(super)`: visible in the whole hyperboard feature, but
/// not to other features.
#[derive(Component, Debug)]
#[require(Transform, Visibility, Name = Name::new("Hyperboard"))]
pub struct Hyperboard {
    /// Players in turn order, identified by their color.
    pub(super) players: Vec<Color>,
    /// The board with the cell pressed during the current turn, if any.
    pub(super) active_board: Option<Entity>,
    /// The board played in the turn that just ended, until the next board is
    /// chosen from it.
    pub(super) played_board: Option<Entity>,
    /// Where the current player must play: the board at this position, or
    /// any board that isn't won or full when `None`.
    pub(super) next_board: Option<GridPosition>,
    /// The color that got three boards in a row.
    pub(super) winner: Option<Color>,
}

impl Hyperboard {
    /// A hyperboard for these players, in turn order. Panics if there are none.
    pub fn new(players: Vec<Color>) -> Self {
        assert!(
            !players.is_empty(),
            "a hyperboard needs at least one player"
        );
        Self {
            players,
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

    /// The color that won the game, if any.
    pub fn winner(&self) -> Option<Color> {
        self.winner
    }

    /// The player who plays turn `number` (turns start at 1): players take
    /// turns in list order, starting again from the first.
    pub fn player_for_turn(&self, number: u32) -> Color {
        let index = (number as usize).saturating_sub(1) % self.players.len();
        self.players[index]
    }
}

impl Default for Hyperboard {
    fn default() -> Self {
        Self::new(DEFAULT_PLAYERS.to_vec())
    }
}

/// Marks the square that covers a won hyperboard.
#[derive(Component)]
#[require(Name = Name::new("Hyperboard winner overlay"))]
pub(super) struct WinnerOverlay;
