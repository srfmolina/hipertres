//! Building players and their icons. These are not systems: systems (or
//! other features' spawn functions) call them.

use bevy::prelude::*;

use super::component::{Player, PlayerColor};

/// Spawns one player per `(symbol, color)`, in turn order, and returns their
/// entities in the same order.
///
/// Players have no parent: they aren't part of any board.
pub fn spawn_players(commands: &mut Commands, players: &[(char, Color)]) -> Vec<Entity> {
    players
        .iter()
        .map(|&(symbol, color)| commands.spawn((Player { symbol }, PlayerColor(color))).id())
        .collect()
}

/// What is drawn on top of something that belongs to `player`: a pressed
/// cell or a won board. To change the icon (an image, a shape...), only
/// change this function.
///
/// `size` is the width and height available, and `color` is already muted
/// when it has to be.
///
/// It returns `impl Bundle`: a group of components to add together to one
/// entity. The caller doesn't know (or care) whether it is text, a sprite or
/// something else.
pub(super) fn player_icon(player: &Player, size: f32, color: Color) -> impl Bundle {
    (
        // `Text2d` draws text in the 2D world, centered on its entity.
        Text2d::new(player.symbol.to_string()),
        // In Bevy 0.19 the size is a `FontSize`: `Px` means logical pixels.
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}
