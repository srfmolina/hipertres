//! The cell's systems, observers and system sets.

use bevy::prelude::*;

use super::component::{ActivePlayer, Cell, Muted};
use super::message::CellClicked;
use super::meta::constant::UNPRESSED_COLOR;
use crate::feature::common::color::mute;
use crate::feature::player::{PlayerColor, PlayerMark};

/// The cell's steps in `Update`, so other features can run after them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellSystems {
    /// Applies this frame's clicks to the cells. After this step, no cell
    /// changes because of a click until the next frame.
    Clicks,
}

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
pub(super) fn send_cell_clicks(click: On<Pointer<Click>>, mut clicks: MessageWriter<CellClicked>) {
    if click.button == PointerButton::Primary {
        clicks.write(CellClicked { cell: click.entity });
    }
}

/// Toggles every clicked cell: unpressed becomes pressed by the parent's
/// `ActivePlayer`, and pressed becomes unpressed.
///
/// A `MessageReader` returns each message once: the messages sent since this
/// system last read them.
pub(super) fn apply_clicks(
    mut clicks: MessageReader<CellClicked>,
    mut cells: Query<(&mut Cell, &Pickable, Option<&ChildOf>)>,
    active_players: Query<&ActivePlayer>,
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
        if cell.pressed.is_some() {
            cell.pressed = None;
            continue;
        }
        // `ChildOf` exists only if the cell has a parent. Ask the parent
        // who is playing. Nobody playing: the click is ignored.
        if let Some(active) =
            child_of.and_then(|child_of| active_players.get(child_of.parent()).ok())
        {
            cell.pressed = Some(active.0);
        }
    }
}

/// A query filter matching cells whose `Cell` *or* `Muted` changed.
type CellOrMutedChanged = Or<(Changed<Cell>, Changed<Muted>)>;

/// Paints each cell according to its state: white when unpressed, the color
/// of the player who pressed it otherwise, muted if `Muted`. It also tells the
/// player feature whose icon to draw on it (`PlayerMark`).
///
/// Keeping the *state* (`Cell`, changed by clicks and by the board) separate
/// from the *visuals* (this system) means anything can change a cell and the
/// color will always follow.
///
/// `Changed<Cell>` is a *query filter*: the query only returns cells whose
/// `Cell` component changed since this system last ran, so idle cells cost
/// nothing. Newly spawned cells count as changed too.
pub(super) fn update_cell_colors(
    mut cells: Query<(&Cell, &Muted, &mut Sprite, &mut PlayerMark), CellOrMutedChanged>,
    player_colors: Query<&PlayerColor>,
) {
    for (cell, muted, mut sprite, mut mark) in &mut cells {
        let color = cell
            .pressed
            .and_then(|player| player_colors.get(player).ok())
            .map_or(UNPRESSED_COLOR, |player_color| player_color.0);
        sprite.color = if muted.0 { mute(color) } else { color };
        // `set_if_neq` only marks the mark as changed (and redraws the icon)
        // when it really changes.
        mark.set_if_neq(PlayerMark {
            player: cell.pressed,
            muted: muted.0,
        });
    }
}
