//! The game scene: what exists when the game starts.
//!
//! Right now that's one hyperboard with the default settings (two players,
//! Space ends the turn) in the middle of the window.

use bevy::prelude::*;

use super::hyperboard::{Hyperboard, spawn_hyperboard};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_game);
    }
}

fn spawn_game(mut commands: Commands) {
    spawn_hyperboard(&mut commands, Hyperboard::default(), Transform::default());
}
