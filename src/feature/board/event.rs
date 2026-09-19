//! The board's events.

use bevy::prelude::*;

/// An event a parent sends to a board: "unpress the cell pressed this turn".
///
/// An `EntityEvent` targets one entity (`entity`, the board). Send it with
/// `commands.trigger(ClearTurnPress { entity: board })`.
#[derive(EntityEvent, Debug)]
pub struct ClearTurnPress {
    pub entity: Entity,
}
