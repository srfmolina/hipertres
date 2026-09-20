//! A cell: a clickable square that toggles between unpressed and pressed.
//!
//! This plugin only defines how cells *behave*. It never spawns cells itself:
//! whoever needs cells (a board) spawns them with `Cell::default()`.

// Every file of the feature is a private module. The `pub use` lines below
// are the feature's public API: the only names other features can use.
mod component;
mod message;
mod meta;
mod system;

use bevy::prelude::*;

pub use component::{ActivePlayer, Cell, Muted};
pub use message::CellClicked;
pub use meta::constant::CELL_SIZE;

use super::game_loop::{DrawPhase, TurnPhase};
use system::{apply_clicks, send_cell_clicks, update_cell_colors};

/// Makes every `Cell` entity clickable and keeps its color in sync.
pub struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CellClicked>()
            .add_observer(send_cell_clicks)
            // The game loop decides when clicks are applied: our systems
            // only say which step of a turn they belong to.
            .add_systems(Update, apply_clicks.in_set(TurnPhase::Mark))
            // Visuals run in `PostUpdate`, after all game logic, so colors
            // always match the final state of the frame. The loop orders
            // this phase before the player icons.
            .add_systems(PostUpdate, update_cell_colors.in_set(DrawPhase::State));
        meta::debug::register(app);
    }
}
