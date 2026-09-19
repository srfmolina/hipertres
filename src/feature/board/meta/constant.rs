//! Tunable values for the board.

use crate::feature::cell::CELL_SIZE;

/// Number of cells per side of the board.
pub const GRID_SIZE: usize = 3;
/// Empty space between neighbouring cells.
pub const CELL_GAP: f32 = 10.0;
/// Width and height of the whole board: the cells plus the gaps between them.
pub const BOARD_SIZE: f32 = GRID_SIZE as f32 * CELL_SIZE + (GRID_SIZE as f32 - 1.0) * CELL_GAP;
/// How far in front of the cells the winner overlay is drawn. In 2D, a higher
/// `z` is drawn on top.
pub const OVERLAY_Z: f32 = 1.0;
