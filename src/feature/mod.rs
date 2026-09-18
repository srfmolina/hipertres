//! All game features. Each submodule is one feature folder exposing a `Plugin`.
//!
//! To add a feature: create `feature/<name>/mod.rs` with its plugin, declare
//! it below with `mod <name>;`, and `.add()` it to `FeaturePlugins`.

use bevy::{app::PluginGroupBuilder, prelude::*};

// Private to `feature`: features can use each other (`super::cell::Cell`),
// but code outside `feature` can't. Nothing outside needs them yet.
mod board;
mod camera;
mod cell;
mod debug;
mod game;
mod hyperboard;

/// Every Hipertres feature, bundled like Bevy's own `DefaultPlugins`.
///
/// A `PluginGroup` is a list of plugins added in one go. `main.rs` only needs
/// `.add_plugins(FeaturePlugins)`.
pub struct FeaturePlugins;

impl PluginGroup for FeaturePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // Diagnostics, all off unless turned on (see `debug/mod.rs`).
            .add(debug::DebugPlugin)
            .add(camera::CameraPlugin)
            .add(cell::CellPlugin)
            .add(board::BoardPlugin)
            .add(hyperboard::HyperboardPlugin)
            .add(game::GamePlugin)
    }
}
