//! Building boards. These are not systems: systems (or other features'
//! spawn functions) call them with their own `Commands`.

use bevy::prelude::*;

use super::component::{Board, GridPosition};
use super::meta::constant::{CELL_GAP, GRID_SIZE};
use crate::feature::cell::{CELL_SIZE, Cell};

/// Spawns a board with its 9 cells as children, and returns the board entity.
///
/// To put it inside a hyperboard, make it a child of the hyperboard:
/// `commands.entity(board).insert(ChildOf(hyperboard))`.
pub fn spawn_board(commands: &mut Commands, transform: Transform) -> Entity {
    commands
        .spawn((Board::default(), transform))
        // Everything spawned inside `with_children` gets `ChildOf(board)`.
        .with_children(|board| {
            for row in 0..GRID_SIZE {
                for col in 0..GRID_SIZE {
                    board.spawn((
                        Cell::default(),
                        GridPosition { col, row },
                        // Relative to the board's center, not to the window.
                        Transform::from_translation(cell_position(col, row).extend(0.0)),
                    ));
                }
            }
        })
        .id()
}

/// Position of the center of the cell at (`col`, `row`), relative to the
/// board's center.
///
/// In Bevy, +X points right and +Y points **up**, so row 0 (the top) gets the
/// largest Y.
pub(super) fn cell_position(col: usize, row: usize) -> Vec2 {
    let step = CELL_SIZE + CELL_GAP;
    // Distance from the grid's center to the center of the outermost cells.
    let offset = (GRID_SIZE as f32 - 1.0) * step / 2.0;
    Vec2::new(col as f32 * step - offset, offset - row as f32 * step)
}
