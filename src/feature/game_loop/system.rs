//! The game loop's systems and system sets: the order of a frame.

use bevy::prelude::*;

use super::component::{PendingMove, Turn, TurnOrder};
use super::message::EndTurnRequested;
use super::state::GameState;

/// The steps of a turn, in the order they run each frame. **This enum is
/// the order of the game**: every feature puts its systems in one of these
/// sets, and nobody else orders anything.
///
/// A `SystemSet` is a label for a group of systems. `GameLoopPlugin`
/// `.chain()`s these sets, so Bevy guarantees that every system of one
/// phase runs before every system of the next one, whichever feature they
/// come from. They all run only while `GameState::Playing`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TurnPhase {
    /// Requests are collected: `request_end_turn_on_key` reads the end-turn
    /// key and turns it into an `EndTurnRequested` message for every match
    /// root in the world.
    Input,
    /// This frame's clicks are applied to the cells (cell). After this
    /// phase no cell changes because of a click until the next frame.
    Mark,
    /// One pressed cell per board (board).
    OnePerBoard,
    /// One pressed cell in the whole match, recorded in `PendingMove`
    /// (hyperboard).
    OnePerTurn,
    /// The turn advances, if a move is staged: `end_requested_turns` reads
    /// the requests collected in `Input` and, for each one whose match has
    /// a `PendingMove`, moves the turn to the next player.
    EndTurn,
    /// Boards end the turn: they lock the pressed cell and check whether
    /// they are full or won (board). Every cell change of the frame has
    /// already happened.
    BoardResults,
    /// The match's results: its own win, where the next player must play,
    /// and what each board is allowed to do (hyperboard). Every board has
    /// already checked its win.
    MatchResults,
    /// Boards apply what they are allowed to do to their cells (board).
    /// Running after `MatchResults` means a `BoardControl` change made
    /// there is applied in the same frame it happens.
    Control,
}

/// The steps that turn state into visuals, in `PostUpdate`, after all the
/// logic of the frame.
///
/// Not gated by `GameState`: a finished match still has to be drawn.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DrawPhase {
    /// Things paint themselves from their own state (cell).
    State,
    /// Player icons are drawn on whatever asked for one (player).
    Icons,
}

/// Sends the game back to `Setup`, which despawns the old match (everything
/// marked with `despawn_on_setup()`'s `DespawnWhen`, see `game/mod.rs`) and
/// builds a new one.
///
/// `NextState` is a queue: the change happens in the next frame's
/// `StateTransition`, never in the middle of `Update`.
pub(super) fn request_restart_on_key(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Setup);
}

/// Asks to end the turn of every match in the world.
pub(super) fn request_end_turn_on_key(
    roots: Query<Entity, With<Turn>>,
    mut requests: MessageWriter<EndTurnRequested>,
) {
    for root in &roots {
        requests.write(EndTurnRequested { root });
    }
}

/// Ends the turn of each match that asked for it: the turn number goes up
/// by one, and the next player gets the turn.
///
/// A turn can only end after the player staged a move: no passing. There is
/// no "is the game over" check here — a finished match runs no phases at
/// all.
pub(super) fn end_requested_turns(
    mut requests: MessageReader<EndTurnRequested>,
    mut roots: Query<(&TurnOrder, &mut Turn, &PendingMove)>,
) {
    for request in requests.read() {
        let Ok((order, mut turn, pending)) = roots.get_mut(request.root) else {
            continue;
        };
        if pending.0.is_none() {
            continue;
        }
        turn.number += 1;
        turn.player = order.player_for_turn(turn.number);
    }
}
