//! Temporary test scene for trying out features on their own.
//!
//! Right now it spawns one board inside a *stand-in hyperboard*: an entity that
//! holds the `Turn`, like the real hyperboard will. Press Space to end the
//! turn; the players' colors alternate. Replace this with the real hyperboard.

use bevy::prelude::*;

use super::board::{Turn, spawn_board};

/// The two players' colors, alternating each turn.
const PLAYER_COLORS: [Color; 2] = [Color::srgb(0.25, 0.55, 0.95), Color::srgb(0.95, 0.55, 0.20)];
const END_TURN_KEY: KeyCode = KeyCode::Space;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scene)
            .add_systems(Update, end_turn_on_key);
    }
}

/// Marks the stand-in hyperboard.
#[derive(Component)]
struct StandInHyperboard;

fn spawn_scene(mut commands: Commands) {
    let hyperboard = commands
        .spawn((
            StandInHyperboard,
            Name::new("Stand-in hyperboard"),
            Turn {
                number: 1,
                color: PLAYER_COLORS[0],
            },
            // A parent needs these so its children can be positioned and seen.
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    let board = spawn_board(&mut commands, Transform::default());
    commands.entity(board).insert(ChildOf(hyperboard));
    info!("Sandbox: turn 1. Press {END_TURN_KEY:?} to end the turn.");
}

fn end_turn_on_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut turns: Query<&mut Turn, With<StandInHyperboard>>,
) {
    if !keys.just_pressed(END_TURN_KEY) {
        return;
    }
    for mut turn in &mut turns {
        turn.number += 1;
        turn.color = PLAYER_COLORS[(turn.number as usize - 1) % PLAYER_COLORS.len()];
        info!("Sandbox: turn {}", turn.number);
    }
}
