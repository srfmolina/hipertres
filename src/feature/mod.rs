//! All game features. Each submodule is one feature folder exposing a `Plugin`.
//!
//! To add a feature: create `feature/<name>/mod.rs` with its plugin, declare
//! it below with `mod <name>;`, and `.add()` it to `FeaturePlugins`.

use bevy::{app::PluginGroupBuilder, prelude::*};

// Private: nothing outside `feature` needs to reach into them directly.
// Make one `pub` when another part of the code needs its types (e.g. `board::Cell`).
mod board;
mod camera;

/// Every Hipertres feature, bundled like Bevy's own `DefaultPlugins`.
///
/// A `PluginGroup` is a list of plugins added in one go. `main.rs` only needs
/// `.add_plugins(FeaturePlugins)`.
pub struct FeaturePlugins;

impl PluginGroup for FeaturePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(camera::CameraPlugin)
            .add(board::BoardPlugin)
    }
}
