//! The debug feature's systems and run conditions.

use bevy::prelude::*;

use super::resource::{DebugChannel, DebugSettings};

/// A *run condition*: `.run_if(debug_on(DebugChannel::Cells))` makes a system
/// run only while that channel is on. Returns `false` if there are no
/// `DebugSettings` at all.
pub fn debug_on(channel: DebugChannel) -> impl Fn(Option<Res<DebugSettings>>) -> bool + Clone {
    move |settings| settings.is_some_and(|s| s.is_on(channel))
}

pub(super) fn toggle_with_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<DebugSettings>,
) {
    for channel in DebugChannel::ALL {
        if keys.just_pressed(channel.key()) {
            settings.toggle(channel);
            let state = if settings.is_on(channel) { "ON" } else { "OFF" };
            info!("Debug channel '{}' {state}", channel.name());
        }
    }
}
