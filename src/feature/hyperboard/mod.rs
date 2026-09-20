//! The hyperboard: a 3×3 grid of boards, and the whole game's rules.
//!
//! A hyperboard is an entity whose children are its 9 `Board` entities. It
//! is the match root: it holds the game loop's `Turn` and `TurnOrder`, and
//! the loop owns ending a turn (the end-turn key, `EndTurnRequested`, and
//! the turn-advance rule). The hyperboard controls the rest of the game:
//! - One pressed cell per turn in the **whole** hyperboard: pressing a cell in
//!   another board unpresses the one pressed earlier in the turn. This is
//!   recorded in the game loop's `PendingMove`, which is what lets the loop
//!   refuse to end a turn with no move staged.
//! - Where the next player must play: the first turn is played in the center
//!   board. After that, the cell pressed in a turn sends the next player to
//!   the board at the same position (pressing the top-right cell of any board
//!   sends them to the top-right board). If that board is won or full, the
//!   next player can play in any board that isn't won or full.
//! - Which boards can be clicked and show their winner square, through each
//!   board's `BoardControl`. Only the boards the player may play in can be
//!   clicked; the others are drawn muted.
//! - After every turn, three won boards in a row of the same color win the
//!   hyperboard: a square of that color covers everything, and no board can
//!   be clicked anymore.
//!
//! # Order of a frame
//!
//! Every step of a turn runs in one of the game loop's phases, and Bevy
//! guarantees the order between them. The authoritative list is `TurnPhase`
//! in `game_loop/system.rs`; this feature owns `OnePerTurn` and
//! `MatchResults`.

// Every file of the feature is a private module. The `pub use` lines below
// are the feature's public API: the only names other features can use.
mod component;
mod meta;
mod spawn;
mod system;

use bevy::prelude::*;

pub use spawn::spawn_hyperboard;

use super::game_loop::TurnPhase;
use system::{check_for_winner, choose_next_board, control_boards, keep_one_press_per_turn};

pub struct HyperboardPlugin;

impl Plugin for HyperboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                keep_one_press_per_turn.in_set(TurnPhase::OnePerTurn),
                (check_for_winner, choose_next_board, control_boards)
                    .chain()
                    .in_set(TurnPhase::MatchResults),
            ),
        );
        meta::debug::register(app);
    }
}
