//! General diagnostics: raw input and picking. Each one only runs while its
//! channel is on.

use bevy::ecs::name::NameOrEntityItem;
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

use super::{DebugChannel, DebugSettings, debug_on};

pub(super) fn register(app: &mut App) {
    app.add_systems(
        Update,
        (
            log_mouse_buttons.run_if(debug_on(DebugChannel::Input)),
            log_hover_changes.run_if(debug_on(DebugChannel::Picking)),
        ),
    )
    // Observers can't use `run_if`, so each one checks its channel itself.
    .add_observer(log_pointer::<Over>)
    .add_observer(log_pointer::<Press>)
    .add_observer(log_pointer::<Click>);
}

fn log_mouse_buttons(buttons: Res<ButtonInput<MouseButton>>, windows: Query<&Window>) {
    let cursor = windows.iter().next().and_then(Window::cursor_position);
    for button in buttons.get_just_pressed() {
        info!("[input] mouse {button:?} pressed, cursor={cursor:?}");
    }
}

/// Logs what the pointer is over, only when that changes (not every frame).
fn log_hover_changes(hover: Res<HoverMap>, names: Query<NameOrEntity>, mut last: Local<String>) {
    let now = hover
        .values()
        .flat_map(|hits| hits.keys())
        .filter_map(|&entity| names.get(entity).ok())
        .map(|item| label(&item))
        .collect::<Vec<_>>()
        .join(", ");
    if *last != now {
        info!("[picking] hovering: [{now}]");
        *last = now;
    }
}

/// One generic observer for several pointer event types `E` (Over, Press, Click).
///
/// Pointer events *bubble*: after firing on the entity under the pointer, they
/// fire again on its parent, and so on up to the window. So one click on a
/// cell logs a Click on the cell, then another on the window.
fn log_pointer<E>(
    event: On<Pointer<E>>,
    settings: Option<Res<DebugSettings>>,
    names: Query<NameOrEntity>,
) where
    E: std::fmt::Debug + Clone + Reflect,
{
    if !settings.is_some_and(|s| s.is_on(DebugChannel::Picking)) {
        return;
    }
    // "bevy_picking::events::Click" -> "Click"
    let kind = std::any::type_name::<E>()
        .rsplit("::")
        .next()
        .unwrap_or("?");
    if let Ok(item) = names.get(event.entity) {
        info!("[picking] {kind} on {}", label(&item));
    }
}

/// "Cell 374v0" for named entities, "65v0" for the rest.
fn label(item: &NameOrEntityItem) -> String {
    match item.name {
        Some(name) => format!("{name} {}", item.entity),
        None => item.entity.to_string(),
    }
}
