//! Board diagnostics, on the `boards` debug channel.

use bevy::prelude::*;

use super::{Board, follow_parent_turn};
use crate::feature::debug::{DebugChannel, DebugSettings, debug_on};

pub(super) fn register(app: &mut App) {
    app.init_resource::<DebugSettings>().add_systems(
        Update,
        log_board_changes
            .after(follow_parent_turn)
            .run_if(debug_on(DebugChannel::Boards)),
    );
}

fn log_board_changes(boards: Query<(Entity, &Board), Changed<Board>>) {
    for (entity, board) in &boards {
        info!(
            "[boards] Board {entity} turn={:?} current={:?} winner={:?}",
            board.turn,
            board.current,
            board.winner.map(|c| c.to_srgba())
        );
    }
}
