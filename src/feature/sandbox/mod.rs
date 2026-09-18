//! Temporary test scene for trying out features on their own.
//!
//! Right now it spawns a single cell in the middle of the window. Remove this
//! feature (and its line in `FeaturePlugins`) once the board uses cells.

use bevy::prelude::*;

use super::cell::Cell;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_single_cell);
    }
}

fn spawn_single_cell(mut commands: Commands) {
    // The required components on `Cell` add the Sprite, Transform and
    // Visibility. With no Transform given, the cell sits at the origin: the
    // center of the window.
    commands.spawn(Cell::default());
}
