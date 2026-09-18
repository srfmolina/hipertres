//! Entry point of Hipertres.
//!
//! `main` only builds the Bevy `App` and plugs in the game's features.
//! Each feature lives in its own module and exposes a single `Plugin`.

// Declare the modules (files) that make up the game.
// `mod board;` tells Rust to compile `src/board.rs` as the `board` module.
mod board;
mod camera;

// The prelude re-exports the Bevy types you use most (App, Commands, Transform...).
use bevy::prelude::*;

use board::BoardPlugin;
use camera::CameraPlugin;

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
        // Our own plugins. The order doesn't matter here: plugins only
        // *register* things. Nothing runs until `.run()` is called.
        .add_plugins((CameraPlugin, BoardPlugin))
        // Start the game loop. This call only returns when the window closes.
        .run();
}
