//! The debug feature's resource, `DebugSettings`, and `DebugChannel`, the
//! type it is made of.

use std::collections::HashSet;

use bevy::prelude::*;

use super::meta::constant::ENV_VAR;

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
    /// The game loop: turn changes and state transitions.
    GameLoop,
}

impl DebugChannel {
    pub const ALL: [DebugChannel; 6] = [
        Self::Input,
        Self::Picking,
        Self::Cells,
        Self::Boards,
        Self::Hyperboard,
        Self::GameLoop,
    ];

    /// Name used in `HIPERTRES_DEBUG` and in log messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Picking => "picking",
            Self::Cells => "cells",
            Self::Boards => "boards",
            Self::Hyperboard => "hyperboard",
            Self::GameLoop => "gameloop",
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
            Self::GameLoop => KeyCode::F6,
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
    pub(super) fn from_env() -> Self {
        std::env::var(ENV_VAR)
            .map(|value| Self::parse(&value))
            .unwrap_or_default()
    }

    /// Parses a comma-separated list of channel names, or `all`.
    /// Unknown names are reported and ignored.
    pub(super) fn parse(value: &str) -> Self {
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

    pub(super) fn enabled_names(&self) -> Vec<&'static str> {
        DebugChannel::ALL
            .into_iter()
            .filter(|c| self.is_on(*c))
            .map(DebugChannel::name)
            .collect()
    }
}
