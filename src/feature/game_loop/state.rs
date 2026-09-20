//! The game loop's state: the life cycle of a match.

use bevy::prelude::*;

/// Where the game is in its life cycle.
///
/// A *state* is a Bevy state machine: one global value (the `State<T>`
/// resource) that systems can run in (`run_if(in_state(...))`) and react to
/// (`OnEnter(...)` / `OnExit(...)`). To change it, a system sets the
/// `NextState<T>` resource; Bevy applies it in the `StateTransition`
/// schedule, once per frame, before `Update`.
///
/// Deriving `States` needs `Clone + PartialEq + Eq + Hash + Debug`, and
/// `Default` decides the state the game starts in.
#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    /// The match is being built (see `spawn_game` in the game feature).
    /// Entering `Setup` again is how the game restarts: everything marked
    /// `DespawnOnEnter(GameState::Setup)` is despawned first.
    #[default]
    Setup,
    /// Turns run. The only state in which `TurnPhase` runs.
    Playing,
    /// Somebody won. Clicks and turns are frozen, visuals are not.
    // Not constructed yet: a later task in this plan sets it once the
    // hyperboard's win is checked in `TurnPhase::MatchResults`.
    #[allow(dead_code)]
    Finished,
}
