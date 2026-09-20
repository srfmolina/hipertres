//! The hyperboard: a 3×3 grid of boards, and the whole game's rules.
//!
//! A hyperboard is an entity whose children are its 9 `Board` entities. It
//! controls the game:
//! - Ending a turn: the game loop's `Turn` advances to the next player,
//!   found in `TurnOrder` (both defined by the game loop feature; the
//!   hyperboard is where they live, as the match root).
//! - One pressed cell per turn in the **whole** hyperboard: pressing a cell in
//!   another board unpresses the one pressed earlier in the turn.
//! - A turn can only end after the player pressed a cell: no passing.
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
//! `MatchResults`, and for now also `EndTurn` (it still reads its own
//! end-turn key and ends the turn itself; a later task moves that into the
//! game loop).

// Every file of the feature is a private module. The `pub use` lines below
// are the feature's public API: the only names other features can use.
mod component;
mod message;
mod meta;
mod spawn;
mod system;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

pub use message::EndTurnRequested;
pub use spawn::spawn_hyperboard;

use super::game_loop::TurnPhase;
use meta::constant::END_TURN_KEY;
use system::{
    check_for_winner, choose_next_board, control_boards, end_requested_turns,
    keep_one_press_per_turn, request_end_turn_on_key,
};

pub struct HyperboardPlugin;

impl Plugin for HyperboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EndTurnRequested>().add_systems(
            Update,
            (
                keep_one_press_per_turn.in_set(TurnPhase::OnePerTurn),
                (
                    // `input_just_pressed` is a run condition from Bevy: the
                    // system only runs on the frame the key goes down.
                    request_end_turn_on_key.run_if(input_just_pressed(END_TURN_KEY)),
                    end_requested_turns,
                )
                    .chain()
                    .in_set(TurnPhase::EndTurn),
                (check_for_winner, choose_next_board, control_boards)
                    .chain()
                    .in_set(TurnPhase::MatchResults),
            ),
        );
        meta::debug::register(app);
    }
}
