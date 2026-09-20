//! The game loop's messages.

use bevy::prelude::*;

/// A *message*: "end the current turn of this match". The end-turn key
/// sends it, and so can anything else (tests, a future "pass" button).
///
/// It is rejected if no move was staged this turn (see `PendingMove`).
#[derive(Message, Debug, Clone, Copy)]
pub struct EndTurnRequested {
    /// The match root: the entity holding the `Turn`.
    pub root: Entity,
}
