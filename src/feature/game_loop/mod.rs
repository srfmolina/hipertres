//! The game loop: what a turn is, who plays it, in which order the steps of
//! a frame happen, and the life cycle of a match.
//!
//! This feature sits **under** the features it paces (cell, board,
//! hyperboard). It defines the vocabulary they are written in — the phases
//! of `TurnPhase`, the `GameState` — and never names a cell, a board or a
//! hyperboard. `docs/05-the-game-loop.md` explains why.

// Every file of the feature is a private module. The `pub use` lines below
// are the feature's public API: the only names other features can use.
mod meta;
mod state;
mod system;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
// `StatesPlugin` is not in Bevy's prelude, unlike the rest of the state API.
use bevy::state::app::StatesPlugin;

pub use state::GameState;
pub use system::{DrawPhase, TurnPhase};

/// Only for other features' tests (see `meta/testing.rs`).
#[cfg(test)]
pub(crate) use meta::testing::start_playing;

use crate::feature::player::PlayerSystems;
use meta::constant::RESTART_KEY;
use system::request_restart_on_key;

pub struct GameLoopPlugin;

impl Plugin for GameLoopPlugin {
    fn build(&self, app: &mut App) {
        // `DefaultPlugins` brings `StatesPlugin`, but headless test apps
        // don't, and `init_state` needs the `StateTransition` schedule it
        // adds. Same idea as `init_resource::<DebugSettings>()` in the
        // features' `meta/debug.rs`.
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }
        app.init_state::<GameState>()
            // The order of the whole game, in one place.
            .configure_sets(
                Update,
                (
                    TurnPhase::Input,
                    TurnPhase::Mark,
                    TurnPhase::OnePerBoard,
                    TurnPhase::OnePerTurn,
                    TurnPhase::EndTurn,
                    TurnPhase::BoardResults,
                    TurnPhase::MatchResults,
                    TurnPhase::Control,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .configure_sets(PostUpdate, (DrawPhase::State, DrawPhase::Icons).chain())
            // The player feature doesn't know the loop exists, so the loop
            // puts *its* set inside the right phase. A set inside a set
            // inherits its order.
            .configure_sets(PostUpdate, PlayerSystems::Marks.in_set(DrawPhase::Icons))
            // Not in a phase: restarting has to work while the game is
            // finished, and phases only run while playing.
            .add_systems(
                Update,
                request_restart_on_key.run_if(
                    input_just_pressed(RESTART_KEY).and_then(not(in_state(GameState::Setup))),
                ),
            );
    }
}
