//! The hyperboard's systems and system sets.

use bevy::prelude::*;

use super::component::{Hyperboard, WinnerOverlay};
use super::message::EndTurnRequested;
use super::meta::constant::{HYPERBOARD_SIZE, OVERLAY_Z};
use crate::feature::board::{
    Board, BoardControl, ClearTurnPress, GRID_SIZE, GridPosition, Turn, three_in_a_row,
};
use crate::feature::player::{PlayerColor, PlayerMark};

/// The hyperboard's steps in `Update`. See "Order of a frame" above.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HyperboardSystems {
    /// Keeps one pressed cell per turn in the whole hyperboard.
    Presses,
    /// Handles `EndTurnRequested`.
    EndTurn,
    /// Checks the hyperboard's win, chooses the next board to play in, and
    /// sets each board's `BoardControl`.
    Results,
}

pub(super) fn request_end_turn_on_key(
    hyperboards: Query<Entity, With<Hyperboard>>,
    mut requests: MessageWriter<EndTurnRequested>,
) {
    for hyperboard in &hyperboards {
        requests.write(EndTurnRequested { hyperboard });
    }
}

/// Ends the turn of each hyperboard that asked for it: the turn number goes up
/// by one, and the next player gets the turn.
///
/// Nothing happens if the game is won, or if no cell was pressed this turn
/// (a player can't pass).
pub(super) fn end_requested_turns(
    mut requests: MessageReader<EndTurnRequested>,
    mut hyperboards: Query<(&mut Hyperboard, &mut Turn)>,
) {
    for request in requests.read() {
        let Ok((mut hyperboard, mut turn)) = hyperboards.get_mut(request.hyperboard) else {
            continue;
        };
        if hyperboard.winner.is_some() || hyperboard.active_board.is_none() {
            continue;
        }
        turn.number += 1;
        turn.player = hyperboard.player_for_turn(turn.number);
        hyperboard.played_board = hyperboard.active_board.take();
    }
}

/// Enforces "one pressed cell per turn" in the whole hyperboard, the same way
/// each board does it for its cells: when a second board gets a pressed cell
/// during the turn, the previous board is told to clear its press.
pub(super) fn keep_one_press_per_turn(
    mut commands: Commands,
    mut hyperboards: Query<(&mut Hyperboard, &Children)>,
    boards: Query<&Board>,
) {
    // This runs before `EndTurn` (see "Order of a frame"), so the boards'
    // presses always belong to the current turn.
    for (mut hyperboard, children) in &mut hyperboards {
        let boards_with_press: Vec<Entity> = children
            .iter()
            .filter(|&child| boards.get(child).is_ok_and(|b| b.current().is_some()))
            .collect();

        let newest = match boards_with_press
            .iter()
            .find(|&&b| Some(b) != hyperboard.active_board)
        {
            Some(&newest) => {
                for &other in boards_with_press.iter().filter(|&&b| b != newest) {
                    // "Calls go down": we tell the board, and the board
                    // unpresses its own cell.
                    commands.trigger(ClearTurnPress { entity: other });
                }
                Some(newest)
            }
            None => boards_with_press.first().copied(),
        };
        if hyperboard.active_board != newest {
            hyperboard.active_board = newest;
        }
    }
}

/// After every turn, checks for three boards in a row won by the same player.
pub(super) fn check_for_winner(
    mut commands: Commands,
    mut hyperboards: Query<(Entity, &mut Hyperboard, Ref<Turn>, &Children)>,
    boards: Query<(&Board, &GridPosition)>,
    player_colors: Query<&PlayerColor>,
) {
    for (entity, mut hyperboard, turn, children) in &mut hyperboards {
        // `Ref<Turn>` gives read access plus change detection: the turn
        // changed this frame means a turn just ended. The boards already
        // checked their own wins earlier in this frame (`BoardSystems::Turns`).
        if !turn.is_changed() || hyperboard.winner.is_some() {
            continue;
        }
        let mut grid = [[None; GRID_SIZE]; GRID_SIZE];
        for child in children.iter() {
            if let Ok((board, position)) = boards.get(child) {
                grid[position.row][position.col] = board.winner();
            }
        }
        let Some(winner) = three_in_a_row(&grid) else {
            continue;
        };
        hyperboard.winner = Some(winner);
        let color = player_colors.get(winner).map_or(Color::NONE, |c| c.0);
        commands.entity(entity).with_child((
            WinnerOverlay,
            Sprite::from_color(color, Vec2::splat(HYPERBOARD_SIZE)),
            Transform::from_xyz(0.0, 0.0, OVERLAY_Z),
            // The player feature draws the winner's icon, big, in the middle.
            PlayerMark {
                player: Some(winner),
                muted: false,
            },
        ));
    }
}

/// After a turn ends, chooses where the next player must play: the board at
/// the position of the cell just pressed, or any board if that one is won or
/// full. Runs after the boards checked their wins, so a board won by the
/// move itself already counts as won.
pub(super) fn choose_next_board(
    mut hyperboards: Query<(&mut Hyperboard, &Children)>,
    boards: Query<(&Board, &GridPosition)>,
) {
    for (mut hyperboard, children) in &mut hyperboards {
        // `played_board` is only set by a turn that just ended. Read it first:
        // `take()` would count as a write every frame, even with nothing to
        // take, and mark the hyperboard as changed.
        let Some(played) = hyperboard.played_board else {
            continue;
        };
        hyperboard.played_board = None;
        let Some(target) = boards
            .get(played)
            .ok()
            .and_then(|(board, _)| board.last_move())
        else {
            continue;
        };
        let target_is_open = children.iter().any(|child| {
            boards.get(child).is_ok_and(|(board, position)| {
                *position == target && board.winner().is_none() && !board.is_full()
            })
        });
        hyperboard.next_board = target_is_open.then_some(target);
    }
}

/// Decides what each board may do, through its `BoardControl`.
///
/// A board can be clicked when the game isn't won, the board isn't won or
/// full, and it's where the current player must play (see `next_board`).
/// Boards that can't be clicked are drawn muted. Won boards always show their
/// winner square.
pub(super) fn control_boards(
    hyperboards: Query<(&Hyperboard, &Children)>,
    mut boards: Query<(&Board, &GridPosition, &mut BoardControl)>,
) {
    for (hyperboard, children) in &hyperboards {
        for child in children.iter() {
            if let Ok((board, position, mut control)) = boards.get_mut(child) {
                let open = board.winner().is_none() && !board.is_full();
                let allowed = hyperboard.next_board.is_none_or(|next| next == *position);
                control.set_if_neq(BoardControl {
                    clickable: hyperboard.winner.is_none() && open && allowed,
                    show_winner_overlay: true,
                });
            }
        }
    }
}
