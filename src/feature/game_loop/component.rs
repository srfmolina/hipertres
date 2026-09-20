//! The game loop's components. They all go on the **match root**: the
//! entity that holds the `Turn`, which today is the hyperboard.

use bevy::prelude::*;

/// The game's current turn.
///
/// It goes on the match root, and the things below it (the boards) read it
/// from their parent to know whose turn it is and when a turn ended.
///
/// `#[require(PendingMove)]`: a root that has a turn always has a place to
/// record the move staged in it.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(PendingMove)]
pub struct Turn {
    /// Increases by one when a turn ends. Boards compare it to the last
    /// value they saw to notice that a turn ended.
    pub number: u32,
    /// The player whose turn it is.
    pub player: Entity,
}

/// Who plays which turn: the player entities, in turn order.
///
/// The loop only knows that players are entities; what a player *is* lives
/// in the player feature.
#[derive(Component, Debug)]
pub struct TurnOrder {
    pub(super) players: Vec<Entity>,
}

impl TurnOrder {
    /// The turn order for these players (see `spawn_players` in the player
    /// feature). Panics if there are none.
    pub fn new(players: Vec<Entity>) -> Self {
        assert!(!players.is_empty(), "a game needs at least one player");
        Self { players }
    }

    /// The player who plays turn `number` (turns start at 1): players take
    /// turns in list order, starting again from the first.
    pub fn player_for_turn(&self, number: u32) -> Entity {
        let index = (number as usize).saturating_sub(1) % self.players.len();
        self.players[index]
    }

    /// The turn a new match starts at.
    pub fn first_turn(&self) -> Turn {
        Turn {
            number: 1,
            player: self.player_for_turn(1),
        }
    }
}

/// The move the current player has staged this turn, if any: `Some(entity)`
/// where the entity means whatever the feature above considers a move (for
/// the hyperboard, the board holding the pressed cell).
///
/// The loop only asks whether it is `Some`: a turn can only end after the
/// player made their move (no passing). It is **recomputed every frame** by
/// the feature that writes it, and nobody clears it, so during the results
/// of the frame a turn ended it still names the move that was just played.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct PendingMove(pub Option<Entity>);
