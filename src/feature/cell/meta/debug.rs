//! Cell diagnostics, on the `cells` debug channel.

use bevy::prelude::*;

use super::super::component::Cell;
use super::super::system::update_cell_colors;
use crate::feature::debug::{DebugChannel, DebugSettings, debug_on};

pub fn register(app: &mut App) {
    // Creates default settings (all off) if `DebugPlugin` hasn't added them,
    // so cells work even without the debug feature.
    app.init_resource::<DebugSettings>().add_systems(
        PostUpdate,
        log_cell_changes
            // Run after the colors are updated, so the log shows the new color.
            .after(update_cell_colors)
            .run_if(debug_on(DebugChannel::Cells)),
    );
}

fn log_cell_changes(cells: Query<(Entity, &Cell, &Sprite), Changed<Cell>>) {
    for (entity, cell, sprite) in &cells {
        info!(
            "[cells] Cell {entity} pressed={:?} color={:?}",
            cell.pressed.map(|c| c.to_srgba()),
            sprite.color.to_srgba()
        );
    }
}
