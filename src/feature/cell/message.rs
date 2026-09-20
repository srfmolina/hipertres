//! The cell's messages.

use bevy::prelude::*;

/// A *message*: "this cell was clicked". The click observer sends one per left
/// click, and `apply_clicks` applies them during `TurnPhase::Mark`.
///
/// Anything can send it to click a cell without a mouse, e.g. tests:
/// `world.write_message(CellClicked { cell })`.
#[derive(Message, Debug, Clone, Copy)]
pub struct CellClicked {
    pub cell: Entity,
}
