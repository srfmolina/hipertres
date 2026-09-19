//! The player's systems and system sets.

use bevy::prelude::*;

use super::component::{Player, PlayerIcon, PlayerMark};
use super::meta::constant::{MARK_SCALE, MARK_TEXT_COLOR, MARK_Z};
use super::spawn::player_icon;
use crate::feature::common::color::mute;

/// The player's steps in `PostUpdate`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerSystems {
    /// Draws the icons of the `PlayerMark`s that changed. Systems that set a
    /// `PlayerMark` in `PostUpdate` must run before it.
    Marks,
}

/// Replaces the icon of every entity whose `PlayerMark` changed (or was just
/// added: new components count as changed).
///
/// It doesn't edit the old icon: it despawns it and spawns a new one from
/// `player_icon`. So this system works the same whatever the icon is made of.
/// It only happens when a mark changes (a press, a win, muting), not every
/// frame.
pub(super) fn draw_player_marks(
    mut commands: Commands,
    marks: Query<(Entity, &PlayerMark, &Sprite, Option<&Children>), Changed<PlayerMark>>,
    icons: Query<(), With<PlayerIcon>>,
    players: Query<&Player>,
) {
    for (entity, mark, sprite, children) in &marks {
        for child in children.into_iter().flatten() {
            if icons.contains(*child) {
                commands.entity(*child).despawn();
            }
        }
        let Some(player) = mark.player.and_then(|p| players.get(p).ok()) else {
            continue;
        };
        // The icon fills a fraction of the square under it.
        let size = sprite.custom_size.map_or(0.0, |s| s.min_element()) * MARK_SCALE;
        let color = if mark.muted {
            mute(MARK_TEXT_COLOR)
        } else {
            MARK_TEXT_COLOR
        };
        commands.entity(entity).with_child((
            PlayerIcon,
            Transform::from_xyz(0.0, 0.0, MARK_Z),
            player_icon(player, size, color),
        ));
    }
}
