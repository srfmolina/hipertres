//! A cell: a clickable square that toggles between unpressed and pressed.
//!
//! This plugin only defines how cells *behave*. It never spawns cells itself:
//! whoever needs cells (a board) spawns them with `Cell::default()`.

mod constant;
mod debug;

use bevy::prelude::*;

pub use constant::CELL_SIZE;
use constant::{DEFAULT_PRESSED_COLOR, MUTE_AMOUNT, MUTE_TARGET, UNPRESSED_COLOR};

/// Makes every `Cell` entity clickable and keeps its color in sync.
pub struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register the message type, so systems can send and read it.
            .add_message::<CellClicked>()
            // A *global observer*: runs every time a `Pointer<Click>` event
            // is triggered on any entity (see `send_cell_clicks`).
            .add_observer(send_cell_clicks)
            .add_systems(Update, apply_clicks.in_set(CellSystems::Clicks))
            // Visuals run in `PostUpdate`, after all game logic in `Update`
            // (e.g. a board unpressing a cell), so colors always match the
            // final state of the frame.
            .add_systems(PostUpdate, update_cell_colors);
        debug::register(app);
    }
}

/// The cell's steps in `Update`, so other features can run after them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellSystems {
    /// Applies this frame's clicks to the cells. After this step, no cell
    /// changes because of a click until the next frame.
    Clicks,
}

/// A *message*: "this cell was clicked". The click observer sends one per left
/// click, and `apply_clicks` applies them during `CellSystems::Clicks`.
///
/// Anything can send it to click a cell without a mouse, e.g. tests:
/// `world.write_message(CellClicked { cell })`.
#[derive(Message, Debug, Clone, Copy)]
pub struct CellClicked {
    pub cell: Entity,
}

/// A cell's state.
///
/// `#[require(...)]` lists *required components*, which are added
/// automatically when a `Cell` is spawned:
/// - `Sprite`: a white square, unless you provide your own. `Sprite` in turn
///   requires `Transform` and `Visibility`.
/// - `Pickable`: lets Bevy's picking detect the cell under the pointer.
///   Sprites without it are ignored, and clicks go to the window behind them.
///   To make a cell unclickable, set it to `Pickable::IGNORE`: picking then
///   skips the cell completely, so it never receives clicks.
/// - `Name`: a label for debugging, so logs say "Cell 374v0" instead of "374v0".
/// - `Muted`: whether the cell is drawn with muted colors (not muted by default).
///
/// So `commands.spawn(Cell::default())` gives a complete, visible, clickable cell.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
#[require(
    Sprite = Sprite::from_color(UNPRESSED_COLOR, Vec2::splat(CELL_SIZE)),
    Pickable,
    Name = Name::new("Cell"),
    Muted
)]
pub struct Cell {
    /// `None` when unpressed. `Some(color)` when pressed, remembering the
    /// color it was pressed with, even if the parent's `PressedColor` changes later.
    pub pressed: Option<Color>,
}

/// Whether a cell is drawn with muted colors (see `mute`). Its parent (the
/// board) sets it, e.g. to show that the cell can't be played right now.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Muted(pub bool);

/// A muted version of `color`: mixed toward a neutral gray, so it looks
/// desaturated and dimmer. Works for white too (plain desaturation would
/// leave white unchanged, since white has no saturation).
///
/// `mix` comes from Bevy's `Mix` trait, in the prelude.
pub fn mute(color: Color) -> Color {
    color.mix(&MUTE_TARGET, MUTE_AMOUNT)
}

/// The color a cell turns when pressed.
///
/// It goes on a cell's **parent** (the board), not on the cell: the parent
/// decides the color for all its cells. A cell with no parent, or whose
/// parent has no `PressedColor`, uses `DEFAULT_PRESSED_COLOR`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PressedColor(pub Color);

/// Observer: turns each left click into a `CellClicked` message.
///
/// Observers are systems that run in reaction to an *event*, instead of
/// every frame. The first parameter, `On<Pointer<Click>>`, says which event:
/// a pointer (mouse or touch) click. Bevy's picking finds which entity is
/// under the pointer and triggers the event on it. `click.entity` is that
/// entity.
///
/// Picking runs early in the frame, at no step we control. So the observer
/// doesn't change the cell itself: it sends a message, and `apply_clicks`
/// applies it at a known point of the frame (`CellSystems::Clicks`).
fn send_cell_clicks(click: On<Pointer<Click>>, mut clicks: MessageWriter<CellClicked>) {
    if click.button == PointerButton::Primary {
        clicks.write(CellClicked { cell: click.entity });
    }
}

/// Toggles every clicked cell: unpressed becomes pressed with the parent's
/// `PressedColor`, and pressed becomes unpressed.
///
/// A `MessageReader` returns each message once: the messages sent since this
/// system last read them.
fn apply_clicks(
    mut clicks: MessageReader<CellClicked>,
    mut cells: Query<(&mut Cell, &Pickable, Option<&ChildOf>)>,
    pressed_colors: Query<&PressedColor>,
) {
    for click in clicks.read() {
        // `get_mut` returns `Err` when the entity has no `Cell`: the click
        // was on something else (e.g. the window). Ignore it.
        let Ok((mut cell, pickable, child_of)) = cells.get_mut(click.cell) else {
            continue;
        };
        // An unclickable cell (`Pickable::IGNORE`) never gets real clicks
        // from picking. Clicks sent as messages follow the same rule.
        if !pickable.is_hoverable {
            continue;
        }
        cell.pressed = match cell.pressed {
            Some(_) => None,
            // `ChildOf` exists only if the cell has a parent. Ask the parent
            // for its `PressedColor`, and fall back to the default.
            None => Some(
                child_of
                    .and_then(|child_of| pressed_colors.get(child_of.parent()).ok())
                    .map_or(DEFAULT_PRESSED_COLOR, |pressed_color| pressed_color.0),
            ),
        };
    }
}

/// A query filter matching cells whose `Cell` *or* `Muted` changed.
type CellOrMutedChanged = Or<(Changed<Cell>, Changed<Muted>)>;

/// Paints each cell according to its state.
///
/// Keeping the *state* (`Cell`, changed by clicks and by the board) separate
/// from the *visuals* (this system) means anything can change a cell and the
/// color will always follow.
///
/// `Changed<Cell>` is a *query filter*: the query only returns cells whose
/// `Cell` component changed since this system last ran, so idle cells cost
/// nothing. Newly spawned cells count as changed too.
fn update_cell_colors(mut cells: Query<(&Cell, &Muted, &mut Sprite), CellOrMutedChanged>) {
    for (cell, muted, mut sprite) in &mut cells {
        let color = cell.pressed.unwrap_or(UNPRESSED_COLOR);
        sprite.color = if muted.0 { mute(color) } else { color };
    }
}

#[cfg(test)]
mod tests;
