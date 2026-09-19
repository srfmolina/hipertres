//! The game scene: what exists when the game starts.
//!
//! Right now that's the default players ('o' and 'x') and one hyperboard for
//! them in the middle of the window. A future setup screen (how many players,
//! their symbols...) would only change this feature.

use bevy::prelude::*;

use super::hyperboard::{Hyperboard, spawn_hyperboard};
use super::player::{DEFAULT_PLAYERS, spawn_players};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_game);
    }
}

fn spawn_game(mut commands: Commands) {
    let players = spawn_players(&mut commands, &DEFAULT_PLAYERS);
    spawn_hyperboard(
        &mut commands,
        Hyperboard::new(players),
        Transform::default(),
    );
}
