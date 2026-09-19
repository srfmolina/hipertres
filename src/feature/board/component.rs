//! The board's components.

use bevy::prelude::*;

/// The game's current turn. It goes on a board's **parent** (the hyperboard),
/// which tells its boards what turn it is by changing it.
///
/// It's defined here, in the board feature, because it is what a board needs
/// from its parent. The hyperboard uses the board feature, not the other way
/// round.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Turn {
    /// Increases by one when a turn ends. Boards compare it to the last value
    /// they saw to notice that a turn ended.
    pub number: u32,
    /// Color of the player whose turn it is. The board's cells get pressed
    /// with it.
    pub color: Color,
}

/// What a board's parent allows it to do. It goes on the board, and only the
/// parent should change it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardControl {
    /// When `false`, none of the board's cells can be clicked. When `true`,
    /// only its cells that aren't locked can be clicked.
    pub clickable: bool,
    /// Whether a won board shows its square of the winning color.
    pub show_winner_overlay: bool,
}

impl Default for BoardControl {
    fn default() -> Self {
        Self {
            clickable: true,
            show_winner_overlay: true,
        }
    }
}

/// A board's state. Its cells are its children (Bevy's `Children` component).
///
/// The fields are `pub(super)`: visible in the whole board feature (`board`
/// and its modules), but not to other features. Only the board's own systems
/// change them.
#[derive(Component, Debug, Default)]
#[require(Transform, Visibility, BoardControl, Name = Name::new("Board"))]
pub struct Board {
    /// Last turn number seen from the parent. `None` until the board sees one.
    pub(super) turn: Option<u32>,
    /// The cell pressed during the current turn, if any.
    pub(super) current: Option<Entity>,
    /// The color that got three in a row first.
    pub(super) winner: Option<Color>,
    /// Every cell is pressed. Updated when a turn ends.
    pub(super) full: bool,
    /// Position of the cell locked at the last turn end, if one was.
    pub(super) last_move: Option<GridPosition>,
}

impl Board {
    /// The cell pressed during the current turn, if any.
    pub fn current(&self) -> Option<Entity> {
        self.current
    }

    /// The color that won this board, if any.
    pub fn winner(&self) -> Option<Color> {
        self.winner
    }

    /// Whether every cell is pressed, as of the last turn end. A full board
    /// without a winner is a draw: nobody can win it anymore.
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// Position of the cell pressed in the last turn that ended with a move
    /// in this board.
    pub fn last_move(&self) -> Option<GridPosition> {
        self.last_move
    }
}

/// Where something sits in a 3×3 grid, with row 0 at the top: a cell in its
/// board, or a board in its hyperboard.
///
/// The board adds it to each of its cells. The cell feature doesn't need to
/// know about grids: an entity can have components from several features.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub col: usize,
    pub row: usize,
}

/// Marks a cell that was pressed in an earlier turn. It can't be clicked
/// anymore, and keeps its color for the rest of the game.
#[derive(Component)]
pub(super) struct Locked;

/// Marks the square that covers a won board.
#[derive(Component)]
#[require(Name = Name::new("Winner overlay"))]
pub(super) struct WinnerOverlay;
