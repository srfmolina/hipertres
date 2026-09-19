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

pub use component::{Cell, Muted, PressedColor};
pub use message::CellClicked;
pub use meta::constant::CELL_SIZE;
pub use system::CellSystems;

use system::{apply_clicks, send_cell_clicks, update_cell_colors};

/// Makes every `Cell` entity clickable and keeps its color in sync.
pub struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register the message type, so systems can send and read it.
            .add_message::<CellClicked>()
            // A *global observer*: runs every time a `Pointer<Click>` event
            // is triggered on any entity (see `send_cell_clicks`).
            .add_observer(send_cell_clicks)
            .add_systems(Update, apply_clicks.in_set(CellSystems::Clicks))
            // Visuals run in `PostUpdate`, after all game logic in `Update`
            // (e.g. a board unpressing a cell), so colors always match the
            // final state of the frame.
            .add_systems(PostUpdate, update_cell_colors);
        meta::debug::register(app);
    }
}
