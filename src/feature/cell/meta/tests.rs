// `super::super` is `cell/mod.rs`: its plugin and its public API.
use super::super::*;
use super::constant::{DEFAULT_PRESSED_COLOR, UNPRESSED_COLOR};
use crate::feature::common::color::mute;

/// An App with only the cell feature: no window or rendering needed.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(CellPlugin);
    app
}

/// Clicks `cell` the way a real left click does, then runs one frame.
fn click(app: &mut App, cell: Entity) {
    app.world_mut().write_message(CellClicked { cell });
    app.update();
}

fn cell(app: &App, cell: Entity) -> Cell {
    *app.world().get::<Cell>(cell).unwrap()
}

fn color_of(app: &App, cell: Entity) -> Color {
    app.world().get::<Sprite>(cell).unwrap().color
}

#[test]
fn new_cell_is_unpressed_and_white() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(Cell::default()).id();
    app.update();

    assert_eq!(cell(&app, entity).pressed, None);
    assert_eq!(color_of(&app, entity), UNPRESSED_COLOR);
}

/// Bevy's sprite picking ignores sprites without `Pickable`, and clicks
/// would go to the window instead of the cell.
#[test]
fn new_cell_is_pickable() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(Cell::default()).id();

    assert!(app.world().get::<Pickable>(entity).is_some());
}

#[test]
fn click_without_parent_uses_default_color_and_second_click_unpresses() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(Cell::default()).id();

    click(&mut app, entity);
    assert_eq!(cell(&app, entity).pressed, Some(DEFAULT_PRESSED_COLOR));
    assert_eq!(color_of(&app, entity), DEFAULT_PRESSED_COLOR);

    click(&mut app, entity);
    assert_eq!(cell(&app, entity).pressed, None);
    assert_eq!(color_of(&app, entity), UNPRESSED_COLOR);
}

#[test]
fn parent_decides_pressed_color() {
    let mut app = test_app();
    let red = Color::srgb(1.0, 0.0, 0.0);
    let parent = app.world_mut().spawn(PressedColor(red)).id();
    let entity = app
        .world_mut()
        .spawn((Cell::default(), ChildOf(parent)))
        .id();

    click(&mut app, entity);
    assert_eq!(color_of(&app, entity), red);
}

#[test]
fn pressed_cell_keeps_its_color_when_parent_color_changes() {
    let mut app = test_app();
    let red = Color::srgb(1.0, 0.0, 0.0);
    let blue = Color::srgb(0.0, 0.0, 1.0);
    let parent = app.world_mut().spawn(PressedColor(red)).id();
    let entity = app
        .world_mut()
        .spawn((Cell::default(), ChildOf(parent)))
        .id();

    click(&mut app, entity);
    app.world_mut()
        .entity_mut(parent)
        .insert(PressedColor(blue));
    app.update();

    assert_eq!(cell(&app, entity).pressed, Some(red));
}

#[test]
fn muted_cell_is_drawn_with_muted_colors() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(Cell::default()).id();
    click(&mut app, entity);

    app.world_mut().entity_mut(entity).insert(Muted(true));
    app.update();
    assert_eq!(color_of(&app, entity), mute(DEFAULT_PRESSED_COLOR));

    app.world_mut().entity_mut(entity).insert(Muted(false));
    app.update();
    assert_eq!(color_of(&app, entity), DEFAULT_PRESSED_COLOR);
}

#[test]
fn clicks_on_an_unclickable_cell_are_ignored() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((Cell::default(), Pickable::IGNORE))
        .id();

    click(&mut app, entity);
    assert_eq!(cell(&app, entity).pressed, None);
}
