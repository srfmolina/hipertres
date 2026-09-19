//! A board: a 3×3 grid of cells where players try to get three in a row.
//!
//! A board is an entity whose children are its 9 `Cell` entities. It follows
//! the turns of its **parent** (the hyperboard), which holds a `Turn`:
//! - During a turn, only one of its cells can be pressed. Pressing another
//!   one unpresses the previous one.
//! - When the turn ends, the pressed cell is locked (it can't be clicked
//!   anymore), and the board checks for three in a row of the same color.
//! - A board with three in a row records its winner and shows a square of the
//!   winning color over its cells.
//!
//! The parent controls each board through its `BoardControl`: whether the
//! board can be clicked, and whether it shows its winner square. A board that
//! can't be clicked is drawn with muted colors.

// Every file of the feature is a private module. The `pub use` lines below
// are the feature's public API: the only names other features can use.
mod component;
mod event;
mod meta;
mod rule;
mod spawn;
mod system;

use bevy::prelude::*;

pub use component::{Board, BoardControl, GridPosition, Turn};
pub use event::ClearTurnPress;
pub use meta::constant::{BOARD_SIZE, GRID_SIZE};
pub use rule::three_in_a_row;
pub use spawn::spawn_board;
pub use system::BoardSystems;

use super::cell::CellSystems;
use system::{apply_board_control, clear_turn_press, follow_parent_turn, keep_one_press_per_turn};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            // Sets let other features order their systems relative to ours,
            // e.g. `.after(BoardSystems::Turns)`. `.chain()` runs them in order,
            // and all of them run after the clicks of the frame are applied.
            .configure_sets(
                Update,
                (
                    BoardSystems::Presses,
                    BoardSystems::Turns,
                    BoardSystems::Control,
                )
                    .chain()
                    .after(CellSystems::Clicks),
            )
            .add_systems(
                Update,
                (
                    keep_one_press_per_turn.in_set(BoardSystems::Presses),
                    follow_parent_turn.in_set(BoardSystems::Turns),
                    apply_board_control.in_set(BoardSystems::Control),
                ),
            )
            .add_observer(clear_turn_press);
        meta::debug::register(app);
    }
}
