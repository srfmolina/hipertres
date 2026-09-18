use super::*;

/// An App with only the color system: no window or rendering needed.
fn test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, update_cell_colors);
    app
}

fn set_pressed(app: &mut App, cell: Entity, pressed: bool) {
    app.world_mut().get_mut::<Cell>(cell).unwrap().pressed = pressed;
    // Run all schedules once, like one frame of the game.
    app.update();
}

fn color_of(app: &App, cell: Entity) -> Color {
    app.world().get::<Sprite>(cell).unwrap().color
}

#[test]
fn new_cell_is_unpressed_and_white() {
    let mut app = test_app();
    let cell = app.world_mut().spawn(Cell::default()).id();
    app.update();

    assert!(!app.world().get::<Cell>(cell).unwrap().pressed);
    assert_eq!(color_of(&app, cell), UNPRESSED_COLOR);
}

/// Bevy's sprite picking ignores sprites without `Pickable`, and clicks
/// would go to the window instead of the cell.
#[test]
fn new_cell_is_pickable() {
    let mut app = test_app();
    let cell = app.world_mut().spawn(Cell::default()).id();

    assert!(app.world().get::<Pickable>(cell).is_some());
}

#[test]
fn cell_without_parent_uses_default_pressed_color_and_toggles_back() {
    let mut app = test_app();
    let cell = app.world_mut().spawn(Cell::default()).id();

    set_pressed(&mut app, cell, true);
    assert_eq!(color_of(&app, cell), DEFAULT_PRESSED_COLOR);

    set_pressed(&mut app, cell, false);
    assert_eq!(color_of(&app, cell), UNPRESSED_COLOR);
}

#[test]
fn parent_decides_pressed_color() {
    let mut app = test_app();
    let red = Color::srgb(1.0, 0.0, 0.0);
    let parent = app.world_mut().spawn(PressedColor(red)).id();
    let cell = app
        .world_mut()
        .spawn((Cell::default(), ChildOf(parent)))
        .id();

    set_pressed(&mut app, cell, true);
    assert_eq!(color_of(&app, cell), red);
}
