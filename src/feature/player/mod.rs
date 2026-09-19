//! Players: who plays the game, and how a player's things are drawn.
//!
//! A player is an entity with a `Player` (its symbol char) and a
//! `PlayerColor`. The entity itself is the player's identity: cells, boards
//! and turns store the player's `Entity`, not a copy of its color. Future
//! roles or items will be new components on that same entity.
//!
//! This feature doesn't decide *whose turn* it is (that's the hyperboard's
//! rule). It only knows what a player is, creates players, and draws a
//! player's icon on anything that carries a `PlayerMark`.

// Every file of the feature is a private module. The `pub use` lines below
// are the feature's public API: the only names other features can use.
mod component;
mod meta;
mod spawn;
mod system;

use bevy::prelude::*;

pub use component::{Player, PlayerColor, PlayerMark};
pub use meta::constant::DEFAULT_PLAYERS;
pub use spawn::spawn_players;
pub use system::PlayerSystems;

use system::draw_player_marks;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Icons are visuals, so they run in `PostUpdate`, after all game
        // logic. Features that set a `PlayerMark` in `PostUpdate` run their
        // system `.before(PlayerSystems::Marks)`.
        app.add_systems(PostUpdate, draw_player_marks.in_set(PlayerSystems::Marks));
    }
}
