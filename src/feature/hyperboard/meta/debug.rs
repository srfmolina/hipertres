//! Hyperboard diagnostics, on the `hyperboard` debug channel.

use bevy::prelude::*;

use super::super::component::Hyperboard;
use crate::feature::debug::{DebugChannel, DebugSettings, debug_on};
use crate::feature::game_loop::{Turn, TurnPhase};
use crate::feature::player::Player;

pub fn register(app: &mut App) {
    app.init_resource::<DebugSettings>().add_systems(
        Update,
        log_hyperboard_changes
            .after(TurnPhase::Control)
            .run_if(debug_on(DebugChannel::Hyperboard)),
    );
}

/// A query filter matching hyperboards whose state *or* turn changed.
/// Giving a long type a name keeps the system signature readable.
type HyperboardOrTurnChanged = Or<(Changed<Hyperboard>, Changed<Turn>)>;

/// Logs whenever the hyperboard or its turn changes.
fn log_hyperboard_changes(
    hyperboards: Query<(Entity, &Hyperboard, &Turn), HyperboardOrTurnChanged>,
    players: Query<&Player>,
) {
    // The player's symbol, for the log ('?' if the entity isn't a player).
    let symbol = |player: Entity| players.get(player).map_or('?', |p| p.symbol);
    for (entity, hyperboard, turn) in &hyperboards {
        info!(
            "[hyperboard] Hyperboard {entity} turn={} player={:?} active_board={:?} next_board={:?} winner={:?}",
            turn.number,
            symbol(turn.player),
            hyperboard.active_board,
            hyperboard.next_board,
            hyperboard.winner().map(symbol)
        );
    }
}
