//! A cell: a clickable square that toggles between unpressed and pressed.
//!
//! This plugin only defines how cells *behave*. It never spawns cells itself:
//! whoever needs cells (the future board) spawns them with `Cell::default()`.

mod constant;
mod debug;

use bevy::prelude::*;

use constant::{CELL_SIZE, DEFAULT_PRESSED_COLOR, UNPRESSED_COLOR};

/// Makes every `Cell` entity clickable and keeps its color in sync.
pub struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app
            // A *global observer*: runs every time a `Pointer<Click>` event
            // is triggered on any entity (see `toggle_on_click`).
            .add_observer(toggle_on_click)
            // A normal system, running every frame.
            .add_systems(Update, update_cell_colors);
        debug::register(app);
    }
}

/// A board cell. It remembers whether it is pressed.
///
/// `#[require(...)]` lists *required components*, which are added
/// automatically when a `Cell` is spawned:
/// - `Sprite`: a white square, unless you provide your own. `Sprite` in turn
///   requires `Transform` and `Visibility`.
/// - `Pickable`: lets Bevy's picking detect the cell under the pointer.
///   Sprites without it are ignored, and clicks go to the window behind them.
/// - `Name`: a label for debugging, so logs say "Cell 374v0" instead of "374v0".
///
/// So `commands.spawn(Cell::default())` gives a complete, visible, clickable cell.
#[derive(Component, Debug, Default)]
#[require(
    Sprite = Sprite::from_color(UNPRESSED_COLOR, Vec2::splat(CELL_SIZE)),
    Pickable,
    Name = Name::new("Cell")
)]
pub struct Cell {
    pub pressed: bool,
}

/// The color a cell turns when pressed.
///
/// It goes on a cell's **parent** (the future board), not on the cell: the
/// parent decides the color for all its cells. A cell with no parent, or
/// whose parent has no `PressedColor`, uses `DEFAULT_PRESSED_COLOR`.
#[derive(Component, Debug, Clone, Copy)]
pub struct PressedColor(pub Color);

/// Observer: toggles a cell's `pressed` state when it is left-clicked.
///
/// Observers are systems that run in reaction to an *event*, instead of
/// every frame. The first parameter, `On<Pointer<Click>>`, says which event:
/// a pointer (mouse or touch) click. Bevy's picking finds which entity is
/// under the pointer and triggers the event on it. `click.entity` is that
/// entity.
///
/// Clicking works because `DefaultPlugins` includes sprite picking, and every
/// `Cell` has a `Sprite` and a `Pickable` (see `Cell`).
fn toggle_on_click(click: On<Pointer<Click>>, mut cells: Query<&mut Cell>) {
    if click.button != PointerButton::Primary {
        return;
    }
    // The observer sees clicks on *every* entity. `get_mut` returns `Err` when
    // the clicked entity has no `Cell`, and then we ignore the click.
    if let Ok(mut cell) = cells.get_mut(click.entity) {
        cell.pressed = !cell.pressed;
    }
}

/// Paints each cell according to its `pressed` state.
///
/// Keeping the *state* (`Cell::pressed`, changed by clicks) separate from the
/// *visuals* (this system) means anything can press a cell, such as a click,
/// the game rules or a future power, and the color will always follow.
///
/// `Changed<Cell>` is a *query filter*: the query only returns cells whose
/// `Cell` component changed since this system last ran, so idle cells cost
/// nothing. Newly spawned cells count as changed too.
fn update_cell_colors(
    mut cells: Query<(&Cell, &mut Sprite, Option<&ChildOf>), Changed<Cell>>,
    pressed_colors: Query<&PressedColor>,
) {
    for (cell, mut sprite, child_of) in &mut cells {
        sprite.color = if cell.pressed {
            // `ChildOf` exists only if the cell has a parent. Ask the parent
            // for its `PressedColor`, and fall back to the default.
            child_of
                .and_then(|child_of| pressed_colors.get(child_of.parent()).ok())
                .map_or(DEFAULT_PRESSED_COLOR, |pressed_color| pressed_color.0)
        } else {
            UNPRESSED_COLOR
        };
    }
}

#[cfg(test)]
mod tests;
