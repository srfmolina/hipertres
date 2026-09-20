//! The game loop's systems and system sets: the order of a frame.

use bevy::prelude::*;

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
    /// Requests are collected: the end-turn key (game loop).
    Input,
    /// This frame's clicks are applied to the cells (cell). After this
    /// phase no cell changes because of a click until the next frame.
    Mark,
    /// One pressed cell per board (board).
    OnePerBoard,
    /// One pressed cell in the whole match, recorded in `PendingMove`
    /// (hyperboard).
    OnePerTurn,
    /// The turn advances, if a move is staged (game loop).
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

/// Sends the game back to `Setup`, which despawns the match (everything
/// marked `DespawnOnEnter(GameState::Setup)`) and builds a new one.
///
/// `NextState` is a queue: the change happens in the next frame's
/// `StateTransition`, never in the middle of `Update`.
pub(super) fn request_restart_on_key(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Setup);
}
