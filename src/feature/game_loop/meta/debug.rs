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
        // Logged on `Changed<Turn>`, i.e. the frame a turn just ended:
        // `pending` hasn't been touched since `TurnPhase::OnePerTurn`
        // earlier in the same frame, so it's the move that just ended the
        // *previous* turn, not one the new player has made yet.
        info!(
            "[gameloop] Match {entity} turn={} player={} last_move={:?}",
            turn.number,
            symbol(turn.player),
            pending.0
        );
    }
}

/// Logs every state transition. `StateTransitionEvent` is a *message* in
/// Bevy 0.19: it is read at the step we choose, like any other.
///
/// `exited`/`entered` are `Option<GameState>` (the very first transition
/// has no `exited` side), but printing `Option`s directly would log every
/// line as `Some(Setup) -> Some(Playing)`. That `Some(...)` is noise in a
/// channel meant to be read, so we unwrap to the variant, falling back to
/// `"none"` for the one transition that can lack a side.
fn log_state_changes(mut transitions: MessageReader<StateTransitionEvent<GameState>>) {
    let name =
        |state: Option<GameState>| state.map_or("none".to_string(), |state| format!("{state:?}"));
    for transition in transitions.read() {
        info!(
            "[gameloop] state {} -> {}",
            name(transition.exited),
            name(transition.entered)
        );
    }
}
