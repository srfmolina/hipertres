//! Helpers for the tests of *other* features, which all need a game that is
//! already playing.

use bevy::prelude::*;

use super::super::state::GameState;

/// Puts a freshly built test `App` into `GameState::Playing`.
///
/// The real game gets there by itself: `OnEnter(GameState::Setup)` spawns
/// the match and asks for `Playing`. A test app has no `game` feature, so
/// it would stay in `Setup` forever, and no `TurnPhase` would run.
pub(crate) fn start_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    // One frame: `StateTransition` runs before `Update`, so the phases of
    // this very frame already run in `Playing`.
    app.update();
}
