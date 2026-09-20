//! The hyperboard's systems.

use bevy::prelude::*;

use super::component::{Hyperboard, WinnerOverlay};
use super::meta::constant::{HYPERBOARD_SIZE, OVERLAY_Z};
use crate::feature::board::{
    Board, BoardControl, ClearTurnPress, GRID_SIZE, GridPosition, three_in_a_row,
};
use crate::feature::game_loop::{GameState, PendingMove, Turn};
use crate::feature::player::{PlayerColor, PlayerMark};

/// Enforces "one pressed cell per turn" in the whole hyperboard, the same
/// way each board does it for its cells, and records the result in the game
/// loop's `PendingMove`: without it, the loop refuses to end the turn.
pub(super) fn keep_one_press_per_turn(
    mut commands: Commands,
    mut hyperboards: Query<(&mut PendingMove, &Children), With<Hyperboard>>,
    boards: Query<&Board>,
) {
    // This runs before `TurnPhase::EndTurn`, so the boards' presses always
    // belong to the current turn.
    for (mut pending, children) in &mut hyperboards {
        let boards_with_press: Vec<Entity> = children
            .iter()
            .filter(|&child| boards.get(child).is_ok_and(|b| b.current().is_some()))
            .collect();

        let newest = match boards_with_press.iter().find(|&&b| Some(b) != pending.0) {
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
        pending.set_if_neq(PendingMove(newest));
    }
}

/// After every turn, checks for three boards in a row won by the same player.
pub(super) fn check_for_winner(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut hyperboards: Query<(Entity, &mut Hyperboard, Ref<Turn>, &Children)>,
    boards: Query<(&Board, &GridPosition)>,
    player_colors: Query<&PlayerColor>,
) {
    for (entity, mut hyperboard, turn, children) in &mut hyperboards {
        // `Ref<Turn>` gives read access plus change detection: the turn
        // changed this frame means a turn just ended. The boards already
        // checked their own wins earlier in this frame
        // (`TurnPhase::BoardResults`).
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
        // The match is over: from the next frame no `TurnPhase` runs, so
        // clicks and turns are frozen. The boards are muted in this same
        // frame, by `control_boards` below.
        next_state.set(GameState::Finished);
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
/// the position of the cell just pressed, or any board if that one is won
/// or full.
///
/// `PendingMove` still names the board played in the turn that just ended:
/// the hyperboard only recomputes it next frame, in `TurnPhase::OnePerTurn`.
pub(super) fn choose_next_board(
    mut hyperboards: Query<(&mut Hyperboard, Ref<Turn>, &PendingMove, &Children)>,
    boards: Query<(&Board, &GridPosition)>,
) {
    for (mut hyperboard, turn, pending, children) in &mut hyperboards {
        // `Ref<Turn>` gives read access plus change detection: the turn
        // changed this frame means a turn just ended.
        if !turn.is_changed() {
            continue;
        }
        let Some(played) = pending.0 else {
            continue; // The first frame of a match: nothing was played yet.
        };
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
