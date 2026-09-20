//! Hyperboard diagnostics, on the `hyperboard` debug channel.

use bevy::prelude::*;

use super::super::component::Hyperboard;
use crate::feature::debug::{DebugChannel, DebugSettings, debug_on};
use crate::feature::game_loop::TurnPhase;
use crate::feature::player::Player;

pub fn register(app: &mut App) {
    app.init_resource::<DebugSettings>().add_systems(
        Update,
        log_hyperboard_changes
            .after(TurnPhase::Control)
            .run_if(debug_on(DebugChannel::Hyperboard)),
    );
}

/// Logs whenever the hyperboard changes.
fn log_hyperboard_changes(
    hyperboards: Query<(Entity, &Hyperboard), Changed<Hyperboard>>,
    players: Query<&Player>,
) {
    // The player's symbol, for the log ('?' if the entity isn't a player).
    let symbol = |player: Entity| players.get(player).map_or('?', |p| p.symbol);
    for (entity, hyperboard) in &hyperboards {
        info!(
            "[hyperboard] Hyperboard {entity} next_board={:?} winner={:?}",
            hyperboard.next_board,
            hyperboard.winner().map(symbol)
        );
    }
}
