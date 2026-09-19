//! Diagnostics you can turn on and off, grouped in *channels*.
//!
//! Turn channels on at startup with the `HIPERTRES_DEBUG` environment variable:
//!
//! ```sh
//! HIPERTRES_DEBUG=picking,cells cargo run   # some channels
//! HIPERTRES_DEBUG=all cargo run             # every channel
//! ```
//!
//! Or toggle them while the game runs with the F-keys (see `DebugChannel::key`).
//!
//! This module holds the switchboard (`DebugSettings`) and the diagnostics
//! that don't belong to any feature (raw input, picking). Feature-specific
//! diagnostics live in that feature's own `meta/debug.rs` (e.g.
//! `cell/meta/debug.rs`) and
//! use `debug_on(...)` to run only when their channel is on.

// Every file of the feature is a private module. The `pub use` lines below
// are the feature's public API: the only names other features can use.
mod log;
mod meta;
mod resource;
mod system;

use bevy::prelude::*;

pub use resource::{DebugChannel, DebugSettings};
pub use system::debug_on;

use system::toggle_with_keys;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        let settings = DebugSettings::from_env();
        info!(
            "Debug channels on: {:?}. Toggle with {}.",
            settings.enabled_names(),
            DebugChannel::ALL
                .map(|c| format!("{:?}={}", c.key(), c.name()))
                .join(", ")
        );

        // `insert_resource` (not `init_resource`) so our env-based settings
        // replace the default ones that other plugins may have created.
        app.insert_resource(settings)
            .add_systems(Update, toggle_with_keys);
        log::register(app);
    }
}
