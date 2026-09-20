// `super::super` is `cell/mod.rs`: its plugin and its public API.
use super::super::*;
use super::constant::UNPRESSED_COLOR;
use crate::feature::common::color::mute;
use crate::feature::game_loop::{GameLoopPlugin, start_playing};
use crate::feature::player::{Player, PlayerColor, PlayerMark, PlayerPlugin};

const RED: Color = Color::srgb(1.0, 0.0, 0.0);
const BLUE: Color = Color::srgb(0.0, 0.0, 1.0);

/// An App with the player, game loop and cell features: no window or
/// rendering needed.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((PlayerPlugin, GameLoopPlugin, CellPlugin));
    // Needed before the first `app.update()` below: once playing, the game
    // loop's restart system reads it every frame.
    app.init_resource::<ButtonInput<KeyCode>>();
    // The phases only run while playing.
    start_playing(&mut app);
    app
}

fn spawn_player(app: &mut App, symbol: char, color: Color) -> Entity {
    app.world_mut()
        .spawn((Player { symbol }, PlayerColor(color)))
        .id()
}

/// Spawns a stand-in board whose `ActivePlayer` is `player`, with one cell
/// inside it. Returns (parent, cell).
fn setup(app: &mut App, player: Entity) -> (Entity, Entity) {
    let parent = app.world_mut().spawn(ActivePlayer(player)).id();
    let cell = app
        .world_mut()
        .spawn((Cell::default(), ChildOf(parent)))
        .id();
    (parent, cell)
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

fn mark_of(app: &App, cell: Entity) -> PlayerMark {
    *app.world().get::<PlayerMark>(cell).unwrap()
}

#[test]
fn new_cell_is_unpressed_and_white() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(Cell::default()).id();
    app.update();

    assert_eq!(cell(&app, entity).pressed, None);
    assert_eq!(color_of(&app, entity), UNPRESSED_COLOR);
    assert_eq!(mark_of(&app, entity), PlayerMark::default());
}

/// Bevy's sprite picking ignores sprites without `Pickable`, and clicks
/// would go to the window instead of the cell.
#[test]
fn new_cell_is_pickable() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(Cell::default()).id();

    assert!(app.world().get::<Pickable>(entity).is_some());
}

/// A cell can't be pressed by nobody: without a parent's `ActivePlayer`,
/// clicks do nothing.
#[test]
fn click_without_active_player_is_ignored() {
    let mut app = test_app();
    let orphan = app.world_mut().spawn(Cell::default()).id();
    let parent = app.world_mut().spawn_empty().id();
    let child = app
        .world_mut()
        .spawn((Cell::default(), ChildOf(parent)))
        .id();

    click(&mut app, orphan);
    click(&mut app, child);
    assert_eq!(cell(&app, orphan).pressed, None);
    assert_eq!(cell(&app, child).pressed, None);
}

#[test]
fn parent_active_player_presses_and_second_click_unpresses() {
    let mut app = test_app();
    let red = spawn_player(&mut app, 'x', RED);
    let (_, entity) = setup(&mut app, red);

    click(&mut app, entity);
    assert_eq!(cell(&app, entity).pressed, Some(red));
    assert_eq!(color_of(&app, entity), RED);
    assert_eq!(
        mark_of(&app, entity),
        PlayerMark {
            player: Some(red),
            muted: false
        }
    );

    click(&mut app, entity);
    assert_eq!(cell(&app, entity).pressed, None);
    assert_eq!(color_of(&app, entity), UNPRESSED_COLOR);
    assert_eq!(mark_of(&app, entity).player, None);
}

#[test]
fn pressed_cell_keeps_its_player_when_active_player_changes() {
    let mut app = test_app();
    let red = spawn_player(&mut app, 'x', RED);
    let blue = spawn_player(&mut app, 'o', BLUE);
    let (parent, entity) = setup(&mut app, red);

    click(&mut app, entity);
    app.world_mut()
        .entity_mut(parent)
        .insert(ActivePlayer(blue));
    app.update();

    assert_eq!(cell(&app, entity).pressed, Some(red));
    assert_eq!(color_of(&app, entity), RED);
}

#[test]
fn muted_cell_is_drawn_with_muted_colors() {
    let mut app = test_app();
    let red = spawn_player(&mut app, 'x', RED);
    let (_, entity) = setup(&mut app, red);
    click(&mut app, entity);

    app.world_mut().entity_mut(entity).insert(Muted(true));
    app.update();
    assert_eq!(color_of(&app, entity), mute(RED));
    assert!(mark_of(&app, entity).muted);

    app.world_mut().entity_mut(entity).insert(Muted(false));
    app.update();
    assert_eq!(color_of(&app, entity), RED);
    assert!(!mark_of(&app, entity).muted);
}

#[test]
fn clicks_on_an_unclickable_cell_are_ignored() {
    let mut app = test_app();
    let red = spawn_player(&mut app, 'x', RED);
    let (_, entity) = setup(&mut app, red);
    app.world_mut().entity_mut(entity).insert(Pickable::IGNORE);

    click(&mut app, entity);
    assert_eq!(cell(&app, entity).pressed, None);
}

/// The player feature draws the icon of the player who pressed the cell,
/// in the same frame as the click.
#[test]
fn pressed_cell_shows_its_player_icon() {
    let mut app = test_app();
    let red = spawn_player(&mut app, 'x', RED);
    let (_, entity) = setup(&mut app, red);

    click(&mut app, entity);
    let world = app.world_mut();
    let children: Vec<Entity> = world.get::<Children>(entity).unwrap().iter().collect();
    let texts: Vec<String> = children
        .into_iter()
        .filter_map(|child| world.get::<Text2d>(child).map(|text| text.0.clone()))
        .collect();
    assert_eq!(texts, ["x"]);
}
