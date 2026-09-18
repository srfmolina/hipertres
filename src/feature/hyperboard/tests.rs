use super::*;
use crate::feature::board::BoardPlugin;
use crate::feature::cell::{Cell, CellClicked, CellPlugin};

const RED: Color = Color::srgb(1.0, 0.0, 0.0);
const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const BLUE: Color = Color::srgb(0.0, 0.0, 1.0);

/// An App with the cell, board and hyperboard features and a hyperboard for
/// `players`, with no window or rendering. Returns (app, hyperboard).
fn setup(players: Vec<Color>) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((CellPlugin, BoardPlugin, HyperboardPlugin));
    // Keyboard state, normally created by `DefaultPlugins` (the Space key
    // system reads it). Nothing is pressed in tests: they call `end_turn`.
    app.init_resource::<ButtonInput<KeyCode>>();
    // Cached systems can't capture variables, so `players` goes in as the
    // system's input instead.
    let hyperboard = app
        .world_mut()
        .run_system_cached_with(
            |In(players): In<Vec<Color>>, mut commands: Commands| {
                spawn_hyperboard(
                    &mut commands,
                    Hyperboard::new(players),
                    Transform::default(),
                )
            },
            players,
        )
        .unwrap();
    app.update();
    (app, hyperboard)
}

/// The child of `parent` at (`col`, `row`): a board of a hyperboard, or a
/// cell of a board.
fn child_at(app: &mut App, parent: Entity, col: usize, row: usize) -> Entity {
    let world = app.world_mut();
    let children: Vec<Entity> = world.get::<Children>(parent).unwrap().iter().collect();
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

/// Asks to end the turn the way the Space key does, then runs one frame.
fn end_turn_now(app: &mut App, hyperboard: Entity) {
    app.world_mut()
        .write_message(EndTurnRequested { hyperboard });
    app.update();
}

/// Presses `cell` and ends the turn.
fn play(app: &mut App, hyperboard: Entity, cell: Entity) {
    click(app, cell);
    end_turn_now(app, hyperboard);
}

fn turn(app: &App, hyperboard: Entity) -> Turn {
    *app.world().get::<Turn>(hyperboard).unwrap()
}

fn pressed(app: &App, cell: Entity) -> Option<Color> {
    app.world().get::<Cell>(cell).unwrap().pressed
}

fn clickable(app: &App, board: Entity) -> bool {
    app.world().get::<BoardControl>(board).unwrap().clickable
}

/// Moves for a player who must play but shouldn't win anything: 3 cells in
/// each bottom-row board, placed so they never make three in a row.
fn harmless_cell(app: &mut App, hyperboard: Entity, move_number: usize) -> Entity {
    const CELLS: [(usize, usize); 3] = [(0, 0), (1, 0), (0, 1)];
    let board = child_at(app, hyperboard, move_number / 3, 2);
    let (col, row) = CELLS[move_number % 3];
    child_at(app, board, col, row)
}

/// With two players, makes the first player win `board` along its top row,
/// while the second player makes harmless moves. It must be the first
/// player's turn. `harmless_moves` counts the second player's moves so far.
fn first_player_wins_board(
    app: &mut App,
    hyperboard: Entity,
    board: Entity,
    harmless_moves: &mut usize,
) {
    for col in 0..GRID_SIZE {
        let cell = child_at(app, board, col, 0);
        play(app, hyperboard, cell);
        let reply = harmless_cell(app, hyperboard, *harmless_moves);
        *harmless_moves += 1;
        play(app, hyperboard, reply);
    }
}

/// Wins the top-row boards for the first player: the whole game.
fn first_player_wins_game(app: &mut App, hyperboard: Entity) {
    let mut harmless_moves = 0;
    for col in 0..GRID_SIZE {
        let board = child_at(app, hyperboard, col, 0);
        first_player_wins_board(app, hyperboard, board, &mut harmless_moves);
    }
}

#[test]
fn hyperboard_spawns_nine_boards_as_children() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);

    assert_eq!(app.world().get::<Children>(hyperboard).unwrap().len(), 9);
    let board = child_at(&mut app, hyperboard, 2, 2);
    assert!(app.world().get::<Board>(board).is_some());
}

#[test]
fn default_hyperboard_has_two_players() {
    assert_eq!(Hyperboard::default().players.len(), 2);
}

#[test]
fn turns_go_through_the_players_in_order() {
    let (mut app, hyperboard) = setup(vec![RED, GREEN, BLUE]);
    assert_eq!(
        turn(&app, hyperboard),
        Turn {
            number: 1,
            color: RED
        }
    );

    for number in 0..3 {
        let cell = harmless_cell(&mut app, hyperboard, number);
        play(&mut app, hyperboard, cell);
        if number == 0 {
            assert_eq!(
                turn(&app, hyperboard),
                Turn {
                    number: 2,
                    color: GREEN
                }
            );
        }
    }
    assert_eq!(
        turn(&app, hyperboard),
        Turn {
            number: 4,
            color: RED
        }
    );
}

#[test]
fn cells_get_pressed_with_the_current_players_color() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let first_move = harmless_cell(&mut app, hyperboard, 0);
    play(&mut app, hyperboard, first_move);
    let board = child_at(&mut app, hyperboard, 0, 0);
    let cell = child_at(&mut app, board, 0, 0);

    click(&mut app, cell);
    assert_eq!(pressed(&app, cell), Some(BLUE));
}

#[test]
fn only_one_cell_can_be_pressed_per_turn_in_the_whole_hyperboard() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let first_board = child_at(&mut app, hyperboard, 0, 0);
    let second_board = child_at(&mut app, hyperboard, 2, 2);
    let first = child_at(&mut app, first_board, 1, 1);
    let second = child_at(&mut app, second_board, 1, 1);

    click(&mut app, first);
    click(&mut app, second);

    assert_eq!(pressed(&app, first), None);
    assert_eq!(pressed(&app, second), Some(RED));
}

#[test]
fn won_boards_become_unclickable_and_the_rest_stay_clickable() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let won = child_at(&mut app, hyperboard, 0, 0);
    let other = child_at(&mut app, hyperboard, 1, 1);

    first_player_wins_board(&mut app, hyperboard, won, &mut 0);

    assert_eq!(app.world().get::<Board>(won).unwrap().winner(), Some(RED));
    assert!(!clickable(&app, won));
    assert!(clickable(&app, other));
}

#[test]
fn three_won_boards_in_a_row_win_the_hyperboard() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);

    first_player_wins_game(&mut app, hyperboard);

    let state = app.world().get::<Hyperboard>(hyperboard).unwrap();
    assert_eq!(state.winner(), Some(RED));
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let board = child_at(&mut app, hyperboard, col, row);
            assert!(!clickable(&app, board));
        }
    }
    let overlays = app
        .world_mut()
        .query_filtered::<&Sprite, With<WinnerOverlay>>()
        .iter(app.world())
        .map(|sprite| sprite.color)
        .collect::<Vec<_>>();
    assert_eq!(overlays, vec![RED]);
}

#[test]
fn turns_stop_once_the_game_is_won() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    first_player_wins_game(&mut app, hyperboard);
    let before = turn(&app, hyperboard);

    end_turn_now(&mut app, hyperboard);
    assert_eq!(turn(&app, hyperboard), before);
}

#[test]
fn ending_the_turn_clears_the_active_board() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let board = child_at(&mut app, hyperboard, 0, 0);
    let cell = child_at(&mut app, board, 0, 0);
    click(&mut app, cell);

    end_turn_now(&mut app, hyperboard);
    let state = app.world().get::<Hyperboard>(hyperboard).unwrap();
    assert_eq!(state.active_board, None);
}

#[test]
fn a_turn_cannot_end_without_a_move() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);

    end_turn_now(&mut app, hyperboard);
    assert_eq!(turn(&app, hyperboard).number, 1);

    // Pressing a cell and unpressing it again is not a move either.
    let cell = harmless_cell(&mut app, hyperboard, 0);
    click(&mut app, cell);
    click(&mut app, cell);
    end_turn_now(&mut app, hyperboard);
    assert_eq!(turn(&app, hyperboard).number, 1);
}

#[test]
fn full_board_without_winner_is_disabled_but_looks_the_same() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let board = child_at(&mut app, hyperboard, 1, 1);
    // A draw, filled alternating red and blue:
    //   R B R
    //   R B B
    //   B R R
    let moves = [
        (0, 0), // red
        (1, 0), // blue
        (2, 0),
        (1, 1),
        (0, 1),
        (2, 1),
        (1, 2),
        (0, 2),
        (2, 2), // red
    ];
    for (col, row) in moves {
        let cell = child_at(&mut app, board, col, row);
        play(&mut app, hyperboard, cell);
    }

    let state = app.world().get::<Board>(board).unwrap();
    assert!(state.is_full());
    assert_eq!(state.winner(), None);
    assert!(!clickable(&app, board));
    // It looks the same: cells keep their colors, and no square covers them.
    let top_left = child_at(&mut app, board, 0, 0);
    assert_eq!(pressed(&app, top_left), Some(RED));
    let children: Vec<Entity> = app.world().get::<Children>(board).unwrap().iter().collect();
    assert!(
        children
            .iter()
            .all(|&c| app.world().get::<Cell>(c).is_some())
    );
}

/// Everything happens in one frame, in order: the click reaches the cell, the
/// board sees it and is won, then the hyperboard sees the board and is won.
#[test]
fn click_board_win_and_game_win_resolve_in_the_same_frame() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let mut harmless_moves = 0;
    for col in 0..2 {
        let board = child_at(&mut app, hyperboard, col, 0);
        first_player_wins_board(&mut app, hyperboard, board, &mut harmless_moves);
    }
    // Red has 2 cells of the third board's top row.
    let last_board = child_at(&mut app, hyperboard, 2, 0);
    for col in 0..2 {
        let cell = child_at(&mut app, last_board, col, 0);
        play(&mut app, hyperboard, cell);
        let reply = harmless_cell(&mut app, hyperboard, harmless_moves);
        harmless_moves += 1;
        play(&mut app, hyperboard, reply);
    }

    // The winning click and the end of the turn, in one single frame.
    let last_cell = child_at(&mut app, last_board, 2, 0);
    app.world_mut()
        .write_message(CellClicked { cell: last_cell });
    app.world_mut()
        .write_message(EndTurnRequested { hyperboard });
    app.update();

    assert_eq!(
        app.world().get::<Board>(last_board).unwrap().winner(),
        Some(RED)
    );
    assert_eq!(
        app.world().get::<Hyperboard>(hyperboard).unwrap().winner(),
        Some(RED)
    );
}
