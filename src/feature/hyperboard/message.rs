//! The hyperboard's messages.

use bevy::prelude::*;

/// A *message*: "end the current turn of this hyperboard". The Space key
/// sends it. It is rejected if no cell was pressed this turn.
#[derive(Message, Debug, Clone, Copy)]
pub struct EndTurnRequested {
    pub hyperboard: Entity,
}
