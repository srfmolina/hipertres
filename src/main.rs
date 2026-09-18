use bevy::prelude::*;

const GRID_SIZE: usize = 3;
const CELL_SIZE: f32 = 150.0;
const CELL_GAP: f32 = 10.0;

const BACKGROUND_COLOR: Color = Color::srgb(0.10, 0.10, 0.12);
const CELL_COLOR: Color = Color::srgb(0.85, 0.85, 0.80);

/// A single cell of the board, identified by its column and row.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    col: usize,
    row: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hipertres".into(),
                resolution: (800, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let cell_mesh = meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE));
    let cell_material = materials.add(CELL_COLOR);

    // Center the grid on the origin, where the camera looks.
    let step = CELL_SIZE + CELL_GAP;
    let offset = (GRID_SIZE as f32 - 1.0) * step / 2.0;

    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let x = col as f32 * step - offset;
            // Row 0 is the top row; world Y points up.
            let y = offset - row as f32 * step;
            commands.spawn((
                Cell { col, row },
                Mesh2d(cell_mesh.clone()),
                MeshMaterial2d(cell_material.clone()),
                Transform::from_xyz(x, y, 0.0),
            ));
        }
    }
}
