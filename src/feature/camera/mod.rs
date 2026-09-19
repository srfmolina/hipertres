//! Camera and background: everything about *how* the world is looked at.

// A small feature: everything fits in this one file, tunable values first.

use bevy::prelude::*;

/// Color shown wherever nothing else is drawn.
const BACKGROUND_COLOR: Color = Color::srgb(0.10, 0.10, 0.12);

/// Sets up the 2D camera and the background color.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    // `build` is called once, when the plugin is added to the App.
    // Here we register what the plugin brings: resources and systems.
    fn build(&self, app: &mut App) {
        app
            // `ClearColor` is a *resource* (a single global value) that the
            // renderer reads to paint the background every frame.
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            // Run `spawn_camera` once, in the `Startup` schedule.
            .add_systems(Startup, spawn_camera);
    }
}

/// A *system*: a plain function that Bevy calls for us.
/// It asks for `Commands` (a queue of world changes) as a parameter,
/// and Bevy provides it automatically.
fn spawn_camera(mut commands: Commands) {
    // Spawn an entity with the `Camera2d` component. Without a camera,
    // nothing is rendered. By default it looks at the origin (0, 0), and
    // 1 world unit = 1 pixel.
    commands.spawn(Camera2d);
}
