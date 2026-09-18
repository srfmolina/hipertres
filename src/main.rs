//! Entry point of Hipertres.
//!
//! `main` only builds the Bevy `App` and plugs in the game's features.
//! Each feature lives in its own module and exposes a single `Plugin`.

// Declare the top-level modules of the game.
// `mod feature;` tells Rust to compile the folder `src/feature/` as the
// `feature` module, starting from `src/feature/mod.rs`.
mod feature;

// The prelude re-exports the Bevy types you use most (App, Commands, Transform...).
use bevy::prelude::*;

use crate::feature::FeaturePlugins;

fn main() {
    App::new()
        // DefaultPlugins is Bevy's built-in bundle: window, rendering, input,
        // assets, audio... `.set(...)` replaces the settings of one plugin in
        // the bundle. Here we configure the main window.
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hipertres".into(),
                resolution: (800, 800).into(),
                // `..default()` fills every other field with its default value.
                ..default()
            }),
            ..default()
        }))
        // All our game features, as one plugin group (see `feature/mod.rs`).
        // Plugins only *register* things. Nothing runs until `.run()` is called.
        .add_plugins(FeaturePlugins)
        // Start the game loop. This call only returns when the window closes.
        .run();
}
