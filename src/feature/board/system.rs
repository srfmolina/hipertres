//! The board's systems, observers and system sets.
//!
//! `end_turn`, `overlay_color` and `overlay_visibility` are not systems, but
//! helpers of the systems here, so they stay next to them.

use bevy::prelude::*;

use super::component::{Board, BoardControl, GridPosition, Locked, Turn, WinnerOverlay};
use super::event::ClearTurnPress;
use super::meta::constant::{BOARD_SIZE, GRID_SIZE, OVERLAY_Z};
use super::rule::three_in_a_row;
use crate::feature::cell::{ActivePlayer, Cell, Muted};
use crate::feature::common::color::mute;
use crate::feature::player::{PlayerColor, PlayerMark};

/// The groups of board systems, in the order they run each frame.
/// A `SystemSet` is a label for a group of systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoardSystems {
    /// Handles this frame's clicks: one pressed cell per board and turn.
    Presses,
    /// Follows the parent's `Turn`: ends turns and checks for wins. Runs
    /// after `CellSystems::Clicks` and `Presses`, so the win check always
    /// sees every cell change of the frame.
    Turns,
    /// Applies `BoardControl` to the cells and the winner square. A parent
    /// that changes `BoardControl` between `Turns` and `Control` sees its
    /// change applied in the same frame.
    Control,
}

/// Enforces "one pressed cell per turn" inside each board.
///
/// Cells toggle themselves when clicked. This system notices when a second
/// cell of the same board was pressed during the turn, and unpresses the
/// previous one.
pub(super) fn keep_one_press_per_turn(
    mut boards: Query<(&mut Board, &Children)>,
    mut cells: Query<&mut Cell, Without<Locked>>,
) {
    for (mut board, children) in &mut boards {
        // Cells pressed during this turn. Locked cells were pressed in earlier
        // turns, and `Without<Locked>` leaves them out of the query.
        let pressed_this_turn: Vec<Entity> = children
            .iter()
            .filter(|&child| cells.get(child).is_ok_and(|cell| cell.pressed.is_some()))
            .collect();

        // A pressed cell that isn't `current` was just pressed: it stays, and
        // the others get unpressed. Otherwise nothing new was pressed, and
        // `current` stays (or becomes `None` if it was clicked again to unpress it).
        let newest = match pressed_this_turn
            .iter()
            .find(|&&c| Some(c) != board.current)
        {
            Some(&newest) => {
                for &other in pressed_this_turn.iter().filter(|&&c| c != newest) {
                    if let Ok(mut cell) = cells.get_mut(other) {
                        cell.pressed = None;
                    }
                }
                Some(newest)
            }
            None => pressed_this_turn.first().copied(),
        };
        // Only write when it changes: writing marks the component as changed,
        // even with the same value, and the debug log reacts to changes.
        if board.current != newest {
            board.current = newest;
        }
    }
}

/// Observer for `ClearTurnPress`: unpresses the board's current cell.
pub(super) fn clear_turn_press(
    event: On<ClearTurnPress>,
    mut boards: Query<&mut Board>,
    mut cells: Query<&mut Cell>,
) {
    let Ok(mut board) = boards.get_mut(event.entity) else {
        return;
    };
    if let Some(current) = board.current.take()
        && let Ok(mut cell) = cells.get_mut(current)
    {
        cell.pressed = None;
    }
}

/// Reads the `Turn` of each board's parent and reacts when it changes: ends
/// the previous turn and takes the new turn's player.
pub(super) fn follow_parent_turn(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board, &BoardControl, &ChildOf, &Children)>,
    turns: Query<&Turn>,
    cells: Query<(&Cell, &GridPosition)>,
    player_colors: Query<&PlayerColor>,
) {
    for (entity, mut board, control, child_of, children) in &mut boards {
        let Ok(turn) = turns.get(child_of.parent()) else {
            continue; // The parent has no `Turn`.
        };
        if board.turn == Some(turn.number) {
            continue; // Same turn as before.
        }
        // The first turn the board sees doesn't end anything.
        if board.turn.is_some() {
            end_turn(
                &mut commands,
                entity,
                &mut board,
                control,
                children,
                &cells,
                &player_colors,
            );
        }
        board.turn = Some(turn.number);
        // Cells read `ActivePlayer` from their parent (this board).
        commands.entity(entity).insert(ActivePlayer(turn.player));
    }
}

/// Locks the cell pressed this turn, and checks whether the board is full and
/// whether it has three in a row.
fn end_turn(
    commands: &mut Commands,
    entity: Entity,
    board: &mut Board,
    control: &BoardControl,
    children: &Children,
    cells: &Query<(&Cell, &GridPosition)>,
    player_colors: &Query<&PlayerColor>,
) {
    if let Some(pressed) = board.current.take() {
        commands.entity(pressed).insert(Locked);
        board.last_move = cells.get(pressed).ok().map(|(_, position)| *position);
    }

    let mut grid = [[None; GRID_SIZE]; GRID_SIZE];
    for child in children.iter() {
        if let Ok((cell, position)) = cells.get(child) {
            grid[position.row][position.col] = cell.pressed;
        }
    }
    let full = grid.iter().flatten().all(Option::is_some);
    if board.full != full {
        board.full = full;
    }
    if board.winner.is_some() {
        return; // The first three in a row is the one that counts.
    }
    let Some(winner) = three_in_a_row(&grid) else {
        return;
    };

    board.winner = Some(winner);
    let color = player_colors.get(winner).map_or(Color::NONE, |c| c.0);
    commands.entity(entity).with_child((
        WinnerOverlay,
        Sprite::from_color(overlay_color(color, control), Vec2::splat(BOARD_SIZE)),
        Transform::from_xyz(0.0, 0.0, OVERLAY_Z),
        overlay_visibility(control),
        // The player feature draws the winner's icon on the square.
        overlay_mark(winner, control),
    ));
}

/// Applies each board's `BoardControl` to its children: which cells can be
/// clicked, which colors are muted, and whether the winner square is shown.
///
/// A cell is clickable when its board is clickable and the cell isn't locked.
/// "Unclickable" means `Pickable::IGNORE`: picking skips the cell entirely.
/// A board that can't be clicked mutes all its colors, cells and winner square
/// (with its icon).
pub(super) fn apply_board_control(
    boards: Query<(&Board, &BoardControl, &Children)>,
    mut cells: Query<(&mut Pickable, &mut Muted, Has<Locked>), With<Cell>>,
    mut overlays: Query<(&mut Visibility, &mut Sprite, &mut PlayerMark), With<WinnerOverlay>>,
    player_colors: Query<&PlayerColor>,
) {
    for (board, control, children) in &boards {
        for child in children.iter() {
            if let Ok((mut pickable, mut muted, locked)) = cells.get_mut(child) {
                let wanted = if control.clickable && !locked {
                    Pickable::default()
                } else {
                    Pickable::IGNORE
                };
                // `set_if_neq` only writes (and marks as changed) when different.
                pickable.set_if_neq(wanted);
                muted.set_if_neq(Muted(!control.clickable));
            }
            if let Ok((mut visibility, mut sprite, mut mark)) = overlays.get_mut(child) {
                visibility.set_if_neq(overlay_visibility(control));
                if let Some(winner) = board.winner {
                    let color = player_colors.get(winner).map_or(Color::NONE, |c| c.0);
                    sprite.color = overlay_color(color, control);
                    mark.set_if_neq(overlay_mark(winner, control));
                }
            }
        }
    }
}

/// The winner square's color: muted while the board can't be clicked.
fn overlay_color(winner: Color, control: &BoardControl) -> Color {
    if control.clickable {
        winner
    } else {
        mute(winner)
    }
}

/// The winner square's icon: the winner's, muted like the square.
fn overlay_mark(winner: Entity, control: &BoardControl) -> PlayerMark {
    PlayerMark {
        player: Some(winner),
        muted: !control.clickable,
    }
}

fn overlay_visibility(control: &BoardControl) -> Visibility {
    if control.show_winner_overlay {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}
