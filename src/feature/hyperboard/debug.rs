//! Hyperboard diagnostics, on the `hyperboard` debug channel.

use bevy::prelude::*;

use super::Hyperboard;
use crate::feature::board::{BoardSystems, Turn};
use crate::feature::debug::{DebugChannel, DebugSettings, debug_on};

pub(super) fn register(app: &mut App) {
    app.init_resource::<DebugSettings>().add_systems(
        Update,
        log_hyperboard_changes
            .after(BoardSystems::Control)
            .run_if(debug_on(DebugChannel::Hyperboard)),
    );
}

/// A query filter matching hyperboards whose state *or* turn changed.
/// Giving a long type a name keeps the system signature readable.
type HyperboardOrTurnChanged = Or<(Changed<Hyperboard>, Changed<Turn>)>;

/// Logs whenever the hyperboard or its turn changes.
fn log_hyperboard_changes(
    hyperboards: Query<(Entity, &Hyperboard, &Turn), HyperboardOrTurnChanged>,
) {
    for (entity, hyperboard, turn) in &hyperboards {
        info!(
            "[hyperboard] Hyperboard {entity} turn={} player={:?} active_board={:?} winner={:?}",
            turn.number,
            turn.color.to_srgba(),
            hyperboard.active_board,
            hyperboard.winner().map(|c| c.to_srgba())
        );
    }
}
