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
//! diagnostics live in that feature's own `debug.rs` (e.g. `cell/debug.rs`) and
//! use `debug_on(...)` to run only when their channel is on.

mod log;

use std::collections::HashSet;

use bevy::prelude::*;

/// Name of the environment variable read at startup.
const ENV_VAR: &str = "HIPERTRES_DEBUG";

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

/// A group of related diagnostics that is turned on and off as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugChannel {
    /// Raw mouse buttons and cursor position, before any game logic.
    Input,
    /// Bevy picking: what the pointer is over, and pointer events on entities.
    Picking,
    /// Cell state changes (pressed, color).
    Cells,
    /// Board state changes (turn, pressed cell, winner).
    Boards,
    /// Hyperboard state changes (turn, player, active board, winner).
    Hyperboard,
}

impl DebugChannel {
    pub const ALL: [DebugChannel; 5] = [
        Self::Input,
        Self::Picking,
        Self::Cells,
        Self::Boards,
        Self::Hyperboard,
    ];

    /// Name used in `HIPERTRES_DEBUG` and in log messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Picking => "picking",
            Self::Cells => "cells",
            Self::Boards => "boards",
            Self::Hyperboard => "hyperboard",
        }
    }

    /// Key that toggles this channel while the game runs.
    pub fn key(self) -> KeyCode {
        match self {
            Self::Input => KeyCode::F1,
            Self::Picking => KeyCode::F2,
            Self::Cells => KeyCode::F3,
            Self::Boards => KeyCode::F4,
            Self::Hyperboard => KeyCode::F5,
        }
    }
}

/// Which debug channels are currently on. A *resource*: one global value.
///
/// Any plugin can read it, and should call `app.init_resource::<DebugSettings>()`
/// so the game still works if `DebugPlugin` is not added (everything off).
#[derive(Resource, Debug, Default)]
pub struct DebugSettings {
    enabled: HashSet<DebugChannel>,
}

impl DebugSettings {
    pub fn is_on(&self, channel: DebugChannel) -> bool {
        self.enabled.contains(&channel)
    }

    pub fn toggle(&mut self, channel: DebugChannel) {
        if !self.enabled.remove(&channel) {
            self.enabled.insert(channel);
        }
    }

    /// Reads `HIPERTRES_DEBUG`. Unset means everything off.
    fn from_env() -> Self {
        std::env::var(ENV_VAR)
            .map(|value| Self::parse(&value))
            .unwrap_or_default()
    }

    /// Parses a comma-separated list of channel names, or `all`.
    /// Unknown names are reported and ignored.
    fn parse(value: &str) -> Self {
        let mut enabled = HashSet::new();
        for name in value.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            if name.eq_ignore_ascii_case("all") {
                enabled.extend(DebugChannel::ALL);
            } else if let Some(channel) = DebugChannel::ALL
                .into_iter()
                .find(|c| c.name().eq_ignore_ascii_case(name))
            {
                enabled.insert(channel);
            } else {
                warn!("{ENV_VAR}: unknown debug channel {name:?}, ignored");
            }
        }
        Self { enabled }
    }

    fn enabled_names(&self) -> Vec<&'static str> {
        DebugChannel::ALL
            .into_iter()
            .filter(|c| self.is_on(*c))
            .map(DebugChannel::name)
            .collect()
    }
}

/// A *run condition*: `.run_if(debug_on(DebugChannel::Cells))` makes a system
/// run only while that channel is on. Returns `false` if there are no
/// `DebugSettings` at all.
pub fn debug_on(channel: DebugChannel) -> impl Fn(Option<Res<DebugSettings>>) -> bool + Clone {
    move |settings| settings.is_some_and(|s| s.is_on(channel))
}

fn toggle_with_keys(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<DebugSettings>) {
    for channel in DebugChannel::ALL {
        if keys.just_pressed(channel.key()) {
            settings.toggle(channel);
            let state = if settings.is_on(channel) { "ON" } else { "OFF" };
            info!("Debug channel '{}' {state}", channel.name());
        }
    }
}

#[cfg(test)]
mod tests;
