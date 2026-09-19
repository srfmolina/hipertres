// `super::super` is `board/mod.rs`: its plugin and its public API. The
// private items tested here are imported one by one from their files.
use super::super::component::WinnerOverlay;
use super::super::spawn::cell_position;
use super::super::*;
use super::constant::CELL_GAP;
use crate::feature::cell::{ActivePlayer, CELL_SIZE, Cell, CellClicked, CellPlugin, Muted};
use crate::feature::common::color::mute;
use crate::feature::player::{Player, PlayerColor, PlayerMark, PlayerPlugin};

const RED: Color = Color::srgb(1.0, 0.0, 0.0);
const BLUE: Color = Color::srgb(0.0, 0.0, 1.0);

/// An App with the player, cell and board features, and no window or
/// rendering.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((PlayerPlugin, CellPlugin, BoardPlugin));
    app
}

/// The two test players: `red` plays with `RED`, `blue` with `BLUE`.
struct Players {
    red: Entity,
    blue: Entity,
}

/// Spawns two players, a stand-in hyperboard with `Turn { number: 1, player:
/// red }` and a board inside it. Returns (app, parent, board, players).
fn setup() -> (App, Entity, Entity, Players) {
    let mut app = test_app();
    let players = Players {
        red: app
            .world_mut()
            .spawn((Player { symbol: 'r' }, PlayerColor(RED)))
            .id(),
        blue: app
            .world_mut()
            .spawn((Player { symbol: 'b' }, PlayerColor(BLUE)))
            .id(),
    };
    let parent = app
        .world_mut()
        .spawn(Turn {
            number: 1,
            player: players.red,
        })
        .id();
    let board = spawn_test_board(&mut app);
    app.world_mut().entity_mut(board).insert(ChildOf(parent));
    app.update();
    (app, parent, board, players)
}

fn spawn_test_board(app: &mut App) -> Entity {
    app.world_mut()
        .run_system_cached(|mut commands: Commands| {
            spawn_board(&mut commands, Transform::default())
        })
        .unwrap()
}

/// The board's cell entity at (`col`, `row`).
fn cell_at(app: &mut App, board: Entity, col: usize, row: usize) -> Entity {
    let world = app.world_mut();
    let children: Vec<Entity> = world.get::<Children>(board).unwrap().iter().collect();
    children
        .into_iter()
        .find(|&child| world.get::<GridPosition>(child) == Some(&GridPosition { col, row }))
        .unwrap()
}

/// Clicks `cell` the way a real left click does, then runs one frame.
fn click(app: &mut App, cell: Entity) {
    app.world_mut().write_message(CellClicked { cell });
    app.update();
}

/// Ends the turn: the parent moves to the next turn number, played by
/// `player`.
fn next_turn(app: &mut App, parent: Entity, player: Entity) {
    let mut turn = app.world_mut().get_mut::<Turn>(parent).unwrap();
    turn.number += 1;
    turn.player = player;
    app.update();
}

fn pressed(app: &App, cell: Entity) -> Option<Entity> {
    app.world().get::<Cell>(cell).unwrap().pressed
}

fn is_locked(app: &App, cell: Entity) -> bool {
    *app.world().get::<Pickable>(cell).unwrap() == Pickable::IGNORE
}

fn overlay(app: &mut App, board: Entity) -> Option<Entity> {
    let world = app.world_mut();
    let children: Vec<Entity> = world.get::<Children>(board).unwrap().iter().collect();
    children
        .into_iter()
        .find(|&child| world.get::<WinnerOverlay>(child).is_some())
}

fn overlay_color(app: &mut App, board: Entity) -> Option<Color> {
    let overlay = overlay(app, board)?;
    Some(app.world().get::<Sprite>(overlay).unwrap().color)
}

fn overlay_visibility_of(app: &mut App, board: Entity) -> Option<Visibility> {
    let overlay = overlay(app, board)?;
    Some(*app.world().get::<Visibility>(overlay).unwrap())
}

#[test]
fn board_spawns_nine_cells_as_children_with_grid_positions() {
    let (mut app, _, board, _) = setup();

    assert_eq!(app.world().get::<Children>(board).unwrap().len(), 9);
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            cell_at(&mut app, board, col, row); // Panics if missing.
        }
    }
}

#[test]
fn cells_get_pressed_by_the_turn_player() {
    let (mut app, _, board, p) = setup();
    let cell = cell_at(&mut app, board, 0, 0);

    click(&mut app, cell);
    assert_eq!(pressed(&app, cell), Some(p.red));
}

/// Without a parent's `Turn`, nobody is playing: the board has no
/// `ActivePlayer`, so its cells ignore clicks.
#[test]
fn board_without_parent_has_no_active_player() {
    let mut app = test_app();
    let board = spawn_test_board(&mut app);
    app.update();

    assert!(app.world().get::<ActivePlayer>(board).is_none());
}

#[test]
fn second_press_in_same_turn_unpresses_the_first() {
    let (mut app, _, board, p) = setup();
    let first = cell_at(&mut app, board, 0, 0);
    let second = cell_at(&mut app, board, 1, 1);

    click(&mut app, first);
    click(&mut app, second);

    assert_eq!(pressed(&app, first), None);
    assert_eq!(pressed(&app, second), Some(p.red));
}

#[test]
fn unpressing_the_current_cell_then_pressing_another_works() {
    let (mut app, _, board, p) = setup();
    let first = cell_at(&mut app, board, 0, 0);
    let second = cell_at(&mut app, board, 1, 1);

    click(&mut app, first);
    click(&mut app, first);
    click(&mut app, second);

    assert_eq!(pressed(&app, first), None);
    assert_eq!(pressed(&app, second), Some(p.red));
}

#[test]
fn end_of_turn_locks_the_pressed_cell_and_keeps_its_player() {
    let (mut app, parent, board, p) = setup();
    let first = cell_at(&mut app, board, 0, 0);
    let second = cell_at(&mut app, board, 1, 1);

    click(&mut app, first);
    next_turn(&mut app, parent, p.blue);
    assert!(is_locked(&app, first));

    // Pressing a cell in the new turn doesn't touch the locked one.
    click(&mut app, second);
    assert_eq!(pressed(&app, first), Some(p.red));
    assert_eq!(pressed(&app, second), Some(p.blue));
    assert!(!is_locked(&app, second));
}

/// Presses the top row with `player`, one cell per turn, and ends the turn
/// after the last one, so the board is won by `player`. The current turn must
/// already be `player`'s.
fn win_top_row(app: &mut App, parent: Entity, board: Entity, player: Entity) {
    for col in 0..3 {
        let cell = cell_at(app, board, col, 0);
        click(app, cell);
        next_turn(app, parent, player);
    }
}

fn winner(app: &App, board: Entity) -> Option<Entity> {
    app.world().get::<Board>(board).unwrap().winner()
}

fn set_control(app: &mut App, board: Entity, control: BoardControl) {
    app.world_mut().entity_mut(board).insert(control);
    app.update();
}

#[test]
fn three_in_a_row_wins_only_when_the_turn_ends() {
    let (mut app, parent, board, p) = setup();

    for col in 0..3 {
        let cell = cell_at(&mut app, board, col, 0);
        click(&mut app, cell);
        if col < 2 {
            next_turn(&mut app, parent, p.red);
        }
    }
    assert_eq!(winner(&app, board), None);

    next_turn(&mut app, parent, p.blue);
    assert_eq!(winner(&app, board), Some(p.red));
    assert_eq!(overlay_color(&mut app, board), Some(RED));
}

#[test]
fn won_board_stays_clickable_until_its_parent_says_otherwise() {
    let (mut app, parent, board, p) = setup();
    win_top_row(&mut app, parent, board, p.red);
    let untouched = cell_at(&mut app, board, 2, 2);
    assert!(!is_locked(&app, untouched));

    set_control(
        &mut app,
        board,
        BoardControl {
            clickable: false,
            ..default()
        },
    );
    assert!(is_locked(&app, untouched));
}

#[test]
fn making_a_board_clickable_again_keeps_pressed_cells_locked() {
    let (mut app, parent, board, p) = setup();
    let pressed_earlier = cell_at(&mut app, board, 0, 0);
    let untouched = cell_at(&mut app, board, 1, 1);
    click(&mut app, pressed_earlier);
    next_turn(&mut app, parent, p.blue);

    let unclickable = BoardControl {
        clickable: false,
        ..default()
    };
    set_control(&mut app, board, unclickable);
    set_control(&mut app, board, BoardControl::default());

    assert!(is_locked(&app, pressed_earlier));
    assert!(!is_locked(&app, untouched));
}

#[test]
fn parent_can_hide_the_winner_overlay() {
    let (mut app, parent, board, p) = setup();
    let hidden = BoardControl {
        show_winner_overlay: false,
        ..default()
    };
    set_control(&mut app, board, hidden);
    win_top_row(&mut app, parent, board, p.red);

    assert_eq!(winner(&app, board), Some(p.red));
    assert_eq!(
        overlay_visibility_of(&mut app, board),
        Some(Visibility::Hidden)
    );

    set_control(&mut app, board, BoardControl::default());
    assert_eq!(
        overlay_visibility_of(&mut app, board),
        Some(Visibility::Inherited)
    );
}

#[test]
fn clear_turn_press_unpresses_the_current_cell() {
    let (mut app, _, board, _) = setup();
    let cell = cell_at(&mut app, board, 0, 0);
    click(&mut app, cell);

    app.world_mut().trigger(ClearTurnPress { entity: board });
    app.update();

    assert_eq!(pressed(&app, cell), None);
    assert_eq!(app.world().get::<Board>(board).unwrap().current(), None);
}

#[test]
fn three_in_a_row_finds_rows_columns_and_diagonals() {
    let r = Some(RED);
    let b = Some(BLUE);
    let row = [[None, None, None], [b, b, b], [r, None, r]];
    let column = [[r, b, None], [r, b, None], [None, b, r]];
    let diagonal = [[r, b, None], [b, r, None], [None, None, r]];
    let anti_diagonal = [[None, None, b], [r, b, None], [b, r, None]];

    assert_eq!(three_in_a_row(&row), b);
    assert_eq!(three_in_a_row(&column), b);
    assert_eq!(three_in_a_row(&diagonal), r);
    assert_eq!(three_in_a_row(&anti_diagonal), b);
}

#[test]
fn three_in_a_row_needs_three_of_the_same_value() {
    let r = Some(RED);
    let b = Some(BLUE);
    let mixed = [[r, r, b], [None, None, None], [None, None, None]];
    let empty: [[Option<Color>; 3]; 3] = [[None; 3]; 3];

    assert_eq!(three_in_a_row(&mixed), None);
    assert_eq!(three_in_a_row(&empty), None);
}

#[test]
fn center_cell_is_at_board_center() {
    assert_eq!(cell_position(1, 1), Vec2::ZERO);
}

#[test]
fn top_left_cell_is_up_and_left() {
    let step = CELL_SIZE + CELL_GAP;
    assert_eq!(cell_position(0, 0), Vec2::new(-step, step));
}

#[test]
fn board_reports_full_when_every_cell_is_pressed() {
    let (mut app, parent, board, p) = setup();
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            assert!(!app.world().get::<Board>(board).unwrap().is_full());
            let cell = cell_at(&mut app, board, col, row);
            click(&mut app, cell);
            next_turn(&mut app, parent, p.red);
        }
    }
    assert!(app.world().get::<Board>(board).unwrap().is_full());
}

/// The win check must see a cell pressed in the same frame the turn ends.
#[test]
fn win_check_sees_a_click_from_the_same_frame() {
    let (mut app, parent, board, p) = setup();
    for col in 0..2 {
        let cell = cell_at(&mut app, board, col, 0);
        click(&mut app, cell);
        next_turn(&mut app, parent, p.red);
    }
    let last = cell_at(&mut app, board, 2, 0);

    // Click and end the turn in one single frame.
    app.world_mut().write_message(CellClicked { cell: last });
    app.world_mut().get_mut::<Turn>(parent).unwrap().number += 1;
    app.update();

    assert_eq!(winner(&app, board), Some(p.red));
}

#[test]
fn end_of_turn_records_the_position_of_the_move() {
    let (mut app, parent, board, p) = setup();
    let cell = cell_at(&mut app, board, 2, 1);
    click(&mut app, cell);
    assert_eq!(app.world().get::<Board>(board).unwrap().last_move(), None);

    next_turn(&mut app, parent, p.blue);
    assert_eq!(
        app.world().get::<Board>(board).unwrap().last_move(),
        Some(GridPosition { col: 2, row: 1 })
    );
}

#[test]
fn unclickable_board_mutes_its_cells_and_winner_square() {
    let (mut app, parent, board, p) = setup();
    win_top_row(&mut app, parent, board, p.red);
    let cell = cell_at(&mut app, board, 2, 2);
    assert_eq!(app.world().get::<Muted>(cell), Some(&Muted(false)));

    let unclickable = BoardControl {
        clickable: false,
        ..default()
    };
    set_control(&mut app, board, unclickable);
    assert_eq!(app.world().get::<Muted>(cell), Some(&Muted(true)));
    assert_eq!(overlay_color(&mut app, board), Some(mute(RED)));

    set_control(&mut app, board, BoardControl::default());
    assert_eq!(app.world().get::<Muted>(cell), Some(&Muted(false)));
    assert_eq!(overlay_color(&mut app, board), Some(RED));
}

/// The rule is generic: it works with any value that can be compared, not
/// only players.
#[test]
fn three_in_a_row_works_with_numbers() {
    let grid = [
        [Some(1), Some(2), None],
        [None, Some(2), Some(1)],
        [None, Some(2), None],
    ];
    assert_eq!(three_in_a_row(&grid), Some(2));
}

fn overlay_mark(app: &mut App, board: Entity) -> Option<PlayerMark> {
    let overlay = overlay(app, board)?;
    app.world().get::<PlayerMark>(overlay).copied()
}

#[test]
fn winner_square_shows_the_winner_icon_muted_with_the_board() {
    let (mut app, parent, board, p) = setup();
    win_top_row(&mut app, parent, board, p.red);
    assert_eq!(
        overlay_mark(&mut app, board),
        Some(PlayerMark {
            player: Some(p.red),
            muted: false
        })
    );

    let unclickable = BoardControl {
        clickable: false,
        ..default()
    };
    set_control(&mut app, board, unclickable);
    assert_eq!(
        overlay_mark(&mut app, board),
        Some(PlayerMark {
            player: Some(p.red),
            muted: true
        })
    );
}
