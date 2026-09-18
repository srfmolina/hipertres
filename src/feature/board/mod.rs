//! The game board: a GRID_SIZE × GRID_SIZE grid of cells.

mod constant;

use bevy::prelude::*;

use constant::{CELL_COLOR, CELL_GAP, CELL_SIZE, GRID_SIZE};

/// Spawns the board when the game starts.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_board);
    }
}

/// A *component*: data attached to an entity. It marks an entity as a
/// board cell and stores its position in the grid.
/// `#[derive(Component)]` is all Bevy needs to accept it as a component.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub col: usize,
    /// Row 0 is the top row.
    pub row: usize,
}

/// Spawns one entity per cell.
///
/// Besides `Commands`, this system asks for two *resources*:
/// - `Assets<Mesh>`: storage for all meshes (shapes made of triangles).
/// - `Assets<ColorMaterial>`: storage for all 2D materials (how a shape is colored).
///
/// `ResMut` means mutable access, because we add new assets to them.
fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Add the shape and the color *once*. `add` returns a `Handle`, a cheap
    // reference to the stored asset. All 9 cells share the same mesh and
    // material through cloned handles.
    let cell_mesh = meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE));
    let cell_material = materials.add(CELL_COLOR);

    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            // Spawn an entity with a tuple of components (a "bundle").
            commands.spawn((
                Cell { col, row },
                // `Mesh2d` + `MeshMaterial2d` are what makes the entity visible.
                Mesh2d(cell_mesh.clone()),
                MeshMaterial2d(cell_material.clone()),
                // Where the entity is. A Transform places the *center* of the mesh.
                Transform::from_translation(cell_position(col, row).extend(0.0)),
            ));
        }
    }
}

/// World position of the center of the cell at (`col`, `row`).
///
/// The grid is centered on the origin, where the camera looks.
/// In Bevy, +X points right and +Y points **up**, so row 0 (the top)
/// gets the largest Y.
fn cell_position(col: usize, row: usize) -> Vec2 {
    let step = CELL_SIZE + CELL_GAP;
    // Distance from the grid's center to the center of the outermost cells.
    let offset = (GRID_SIZE as f32 - 1.0) * step / 2.0;
    Vec2::new(col as f32 * step - offset, offset - row as f32 * step)
}

// Unit tests live in `tests.rs`, next to this file. `#[cfg(test)]` means they
// are only compiled for `cargo test`, never into the game.
#[cfg(test)]
mod tests;
