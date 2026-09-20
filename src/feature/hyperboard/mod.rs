//! The hyperboard: a 3×3 grid of boards, and the whole game's rules.
//!
//! A hyperboard is an entity whose children are its 9 `Board` entities. It
//! controls the game:
//! - The players (a list of colors) and the `Turn`: which turn it is and
//!   which player plays it. Ending a turn gives it to the next player.
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
//! Every step runs in a named set, and Bevy guarantees the order between them:
//!
//! 1. `CellSystems::Clicks`: clicks are applied to cells.
//! 2. `BoardSystems::Presses`, then `HyperboardSystems::Presses`: one press
//!    per turn, in each board and then in the whole hyperboard.
//! 3. `HyperboardSystems::EndTurn`: a request to end the turn is accepted or rejected.
//! 4. `BoardSystems::Turns`: boards end the turn and check their win. Every
//!    cell change of the frame has already happened.
//! 5. `HyperboardSystems::Results`: the hyperboard checks its win, chooses
//!    where the next player plays, and sets each board's `BoardControl`.
//!    Every board has already checked its win.
//! 6. `BoardSystems::Control`: boards apply their `BoardControl`.

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
pub use system::HyperboardSystems;

use super::board::BoardSystems;
use meta::constant::END_TURN_KEY;
use system::{
    check_for_winner, choose_next_board, control_boards, end_requested_turns,
    keep_one_press_per_turn, request_end_turn_on_key,
};

pub struct HyperboardPlugin;

impl Plugin for HyperboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EndTurnRequested>()
            // Place our steps between the board's (see "Order of a frame" above).
            .configure_sets(
                Update,
                (
                    HyperboardSystems::Presses
                        .after(BoardSystems::Presses)
                        .before(HyperboardSystems::EndTurn),
                    HyperboardSystems::EndTurn.before(BoardSystems::Turns),
                    HyperboardSystems::Results
                        .after(BoardSystems::Turns)
                        .before(BoardSystems::Control),
                ),
            )
            .add_systems(
                Update,
                (
                    keep_one_press_per_turn.in_set(HyperboardSystems::Presses),
                    (
                        // `input_just_pressed` is a run condition from Bevy: the
                        // system only runs on the frame the key goes down.
                        request_end_turn_on_key.run_if(input_just_pressed(END_TURN_KEY)),
                        end_requested_turns,
                    )
                        .chain()
                        .in_set(HyperboardSystems::EndTurn),
                    (check_for_winner, choose_next_board, control_boards)
                        .chain()
                        .in_set(HyperboardSystems::Results),
                ),
            );
        meta::debug::register(app);
    }
}
