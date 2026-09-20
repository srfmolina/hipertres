//! The game scene: what exists when the game starts.
//!
//! Right now that's the default players ('o' and 'x') and one hyperboard for
//! them in the middle of the window. A future setup screen (how many players,
//! their symbols...) would only change this feature. Restarting (see the
//! game loop's `RESTART_KEY`) just re-enters `GameState::Setup`, which runs
//! this same spawn again after despawning the old match.

use bevy::prelude::*;

use super::game_loop::GameState;
use super::hyperboard::{Hyperboard, spawn_hyperboard};
use super::player::{DEFAULT_PLAYERS, spawn_players};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Not `Startup`: the match is built whenever the game enters
        // `Setup`, so restarting is just going back to that state.
        app.add_systems(OnEnter(GameState::Setup), spawn_game);
    }
}

fn spawn_game(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    let players = spawn_players(&mut commands, &DEFAULT_PLAYERS);
    // `DespawnOnEnter` (Bevy 0.19's replacement for `StateScoped`) despawns
    // the entity when the game enters that state again. Despawning is
    // recursive, so the hyperboard takes its boards and cells with it.
    for &player in &players {
        commands
            .entity(player)
            .insert(DespawnOnEnter(GameState::Setup));
    }
    let hyperboard = spawn_hyperboard(
        &mut commands,
        Hyperboard::new(players),
        Transform::default(),
    );
    commands
        .entity(hyperboard)
        .insert(DespawnOnEnter(GameState::Setup));
    // Nothing else to set up, so the match starts right away. A future
    // setup screen would set `Playing` when the player presses "start".
    next_state.set(GameState::Playing);
}
