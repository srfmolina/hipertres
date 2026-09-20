//! Game loop diagnostics, on the `gameloop` debug channel.

use bevy::prelude::*;

use super::super::component::{PendingMove, Turn};
use super::super::state::GameState;
use super::super::system::TurnPhase;
use crate::feature::debug::{DebugChannel, DebugSettings, debug_on};
use crate::feature::player::Player;

pub fn register(app: &mut App) {
    app.init_resource::<DebugSettings>().add_systems(
        Update,
        (log_turn_changes, log_state_changes)
            .after(TurnPhase::Control)
            .run_if(debug_on(DebugChannel::GameLoop)),
    );
}

/// Logs the turn of every match whenever it changes.
fn log_turn_changes(
    roots: Query<(Entity, &Turn, &PendingMove), Changed<Turn>>,
    players: Query<&Player>,
) {
    // The player's symbol, for the log ('?' if the entity isn't a player).
    let symbol = |player: Entity| players.get(player).map_or('?', |p| p.symbol);
    for (entity, turn, pending) in &roots {
        info!(
            "[gameloop] Match {entity} turn={} player={} pending_move={:?}",
            turn.number,
            symbol(turn.player),
            pending.0
        );
    }
}

/// Logs every state transition. `StateTransitionEvent` is a *message* in
/// Bevy 0.19: it is read at the step we choose, like any other.
fn log_state_changes(mut transitions: MessageReader<StateTransitionEvent<GameState>>) {
    for transition in transitions.read() {
        info!(
            "[gameloop] state {:?} -> {:?}",
            transition.exited, transition.entered
        );
    }
}
