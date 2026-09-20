//! The game scene: what exists when the game starts.
//!
//! Right now that's the default players ('o' and 'x') and one hyperboard for
//! them in the middle of the window. A future setup screen (how many players,
//! their symbols...) would only change this feature. Restarting (see the
//! game loop's `RESTART_KEY`) just re-enters `GameState::Setup`, which runs
//! this same spawn again after despawning the old match.

use bevy::prelude::*;

use super::game_loop::{GameState, TurnOrder};
use super::hyperboard::spawn_hyperboard;
use super::player::{DEFAULT_PLAYERS, spawn_players};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Not `Startup`: the match is built whenever the game enters
        // `Setup`, so restarting is just going back to that state.
        app.add_systems(OnEnter(GameState::Setup), spawn_game);
    }
}

/// Marks an entity to be despawned the moment the game (re-)enters `Setup`.
///
/// This is spawned *in* `OnEnter(GameState::Setup)` itself, which rules out
/// `DespawnOnEnter`: that component is despawned by a system in
/// `StateTransitionSystems::EnterSchedules`, the very set that also runs
/// `OnEnter(Setup)` — nothing orders one before the other, so a match built
/// by this same entry could be despawned instantly, depending on the
/// schedule's toposort. `DespawnWhen` instead runs from
/// `StateTransitionSystems::TransitionSchedules`, which Bevy chains
/// *before* `EnterSchedules`, so the old match is always gone before
/// `spawn_game` builds the new one.
fn despawn_on_setup() -> DespawnWhen<GameState> {
    DespawnWhen::new(|transition: &StateTransitionEvent<GameState>| {
        transition.entered == Some(GameState::Setup)
    })
}

fn spawn_game(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    let players = spawn_players(&mut commands, &DEFAULT_PLAYERS);
    // Despawning is recursive, so the hyperboard takes its boards and cells
    // with it.
    for &player in &players {
        commands.entity(player).insert(despawn_on_setup());
    }
    let hyperboard = spawn_hyperboard(&mut commands, TurnOrder::new(players), Transform::default());
    commands.entity(hyperboard).insert(despawn_on_setup());
    // Nothing else to set up, so the match starts right away. A future
    // setup screen would set `Playing` when the player presses "start".
    next_state.set(GameState::Playing);
}
