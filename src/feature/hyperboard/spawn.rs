//! Building hyperboards. These are not systems: systems call them with their
//! own `Commands` (see `spawn_game` in the game feature).

use bevy::prelude::*;

use super::component::Hyperboard;
use super::meta::constant::{BOARD_GAP, BOARD_SCALE};
use crate::feature::board::{BOARD_SIZE, GRID_SIZE, GridPosition, spawn_board};
use crate::feature::game_loop::TurnOrder;

/// Spawns a hyperboard with its 9 boards as children, starting at turn 1,
/// and returns the hyperboard entity. The hyperboard entity is the match
/// root: it carries the game loop's `TurnOrder` and `Turn`.
pub fn spawn_hyperboard(
    commands: &mut Commands,
    turn_order: TurnOrder,
    transform: Transform,
) -> Entity {
    let turn = turn_order.first_turn();
    let root = commands
        .spawn((Hyperboard::default(), turn_order, turn, transform))
        .id();
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let transform = Transform::from_translation(board_position(col, row).extend(0.0))
                // Scale x and y only. Scaling z too would squash the boards'
                // own layers (their winner square is at z = 1).
                .with_scale(Vec3::new(BOARD_SCALE, BOARD_SCALE, 1.0));
            let board = spawn_board(commands, transform);
            commands
                .entity(board)
                .insert((GridPosition { col, row }, ChildOf(root)));
        }
    }
    root
}

/// Position of the center of the board at (`col`, `row`), relative to the
/// hyperboard's center. Row 0 is at the top.
fn board_position(col: usize, row: usize) -> Vec2 {
    let step = BOARD_SIZE * BOARD_SCALE + BOARD_GAP;
    let offset = (GRID_SIZE as f32 - 1.0) * step / 2.0;
    Vec2::new(col as f32 * step - offset, offset - row as f32 * step)
}
