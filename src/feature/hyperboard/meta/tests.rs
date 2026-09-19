// `super::super` is `hyperboard/mod.rs`: its plugin and its public API.
use super::super::component::WinnerOverlay;
use super::super::*;
use crate::feature::board::{Board, BoardControl, BoardPlugin, GRID_SIZE, GridPosition, Turn};
use crate::feature::cell::{Cell, CellClicked, CellPlugin, Muted};
use crate::feature::player::{PlayerMark, PlayerPlugin, spawn_players};

const RED: Color = Color::srgb(1.0, 0.0, 0.0);
const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const BLUE: Color = Color::srgb(0.0, 0.0, 1.0);

/// The test players' entities, with the color each one plays with, so tests
/// can say "the red player" (see `player`).
#[derive(Resource)]
struct TestPlayers(Vec<(Color, Entity)>);

/// The entity of the player who plays with `color`.
fn player(app: &App, color: Color) -> Entity {
    app.world()
        .resource::<TestPlayers>()
        .0
        .iter()
        .find(|(c, _)| *c == color)
        .unwrap()
        .1
}

/// An App with the player, cell, board and hyperboard features, one player
/// per color in `colors` (in turn order), and a hyperboard for them, with no
/// window or rendering. Returns (app, hyperboard).
fn setup(colors: Vec<Color>) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((PlayerPlugin, CellPlugin, BoardPlugin, HyperboardPlugin));
    // Keyboard state, normally created by `DefaultPlugins` (the Space key
    // system reads it). Nothing is pressed in tests: they call `end_turn`.
    app.init_resource::<ButtonInput<KeyCode>>();
    // Cached systems can't capture variables, so `colors` goes in as the
    // system's input instead.
    let (hyperboard, players) = app
        .world_mut()
        .run_system_cached_with(
            |In(colors): In<Vec<Color>>, mut commands: Commands| {
                // Symbols don't matter here: 'a', 'b', 'c'...
                let symbols_and_colors: Vec<(char, Color)> =
                    ('a'..).zip(colors.iter().copied()).collect();
                let players = spawn_players(&mut commands, &symbols_and_colors);
                let hyperboard = spawn_hyperboard(
                    &mut commands,
                    Hyperboard::new(players.clone()),
                    Transform::default(),
                );
                (hyperboard, colors.into_iter().zip(players).collect())
            },
            colors,
        )
        .unwrap();
    app.insert_resource(TestPlayers(players));
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

/// Lets the current player play in any board that isn't won or full, ignoring
/// where the last move sent them. For tests about other rules.
fn allow_any_board(app: &mut App, hyperboard: Entity) {
    app.world_mut()
        .get_mut::<Hyperboard>(hyperboard)
        .unwrap()
        .next_board = None;
    app.update();
}

/// Like `play`, but in any board (see `allow_any_board`).
fn play_anywhere(app: &mut App, hyperboard: Entity, cell: Entity) {
    allow_any_board(app, hyperboard);
    play(app, hyperboard, cell);
}

fn next_board(app: &App, hyperboard: Entity) -> Option<GridPosition> {
    app.world()
        .get::<Hyperboard>(hyperboard)
        .unwrap()
        .next_board
}

/// Positions of the boards that can be clicked right now.
fn clickable_boards(app: &mut App, hyperboard: Entity) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let board = child_at(app, hyperboard, col, row);
            if clickable(app, board) {
                positions.push((col, row));
            }
        }
    }
    positions
}

fn turn(app: &App, hyperboard: Entity) -> Turn {
    *app.world().get::<Turn>(hyperboard).unwrap()
}

fn pressed(app: &App, cell: Entity) -> Option<Entity> {
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

/// Moves that fill a board with no three in a row, alternating the first
/// and second player:
///   R B R
///   R B B
///   B R R
const DRAW_MOVES: [(usize, usize); 9] = [
    (0, 0), // first player
    (1, 0), // second player
    (2, 0),
    (1, 1),
    (0, 1),
    (2, 1),
    (1, 2),
    (0, 2),
    (2, 2), // first player
];

/// With two players, makes the first player win `board` along its top row,
/// while the second player makes harmless moves. Moves ignore where the last
/// move sent the player (see `allow_any_board`). It must be the first
/// player's turn. `harmless_moves` counts the second player's moves so far.
fn first_player_wins_board(
    app: &mut App,
    hyperboard: Entity,
    board: Entity,
    harmless_moves: &mut usize,
) {
    for col in 0..GRID_SIZE {
        let cell = child_at(app, board, col, 0);
        play_anywhere(app, hyperboard, cell);
        let reply = harmless_cell(app, hyperboard, *harmless_moves);
        *harmless_moves += 1;
        play_anywhere(app, hyperboard, reply);
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
#[should_panic(expected = "at least one player")]
fn hyperboard_needs_at_least_one_player() {
    Hyperboard::new(Vec::new());
}

#[test]
fn turns_go_through_the_players_in_order() {
    let (mut app, hyperboard) = setup(vec![RED, GREEN, BLUE]);
    assert_eq!(
        turn(&app, hyperboard),
        Turn {
            number: 1,
            player: player(&app, RED)
        }
    );

    for number in 0..3 {
        let cell = harmless_cell(&mut app, hyperboard, number);
        play_anywhere(&mut app, hyperboard, cell);
        if number == 0 {
            assert_eq!(
                turn(&app, hyperboard),
                Turn {
                    number: 2,
                    player: player(&app, GREEN)
                }
            );
        }
    }
    assert_eq!(
        turn(&app, hyperboard),
        Turn {
            number: 4,
            player: player(&app, RED)
        }
    );
}

#[test]
fn cells_get_pressed_by_the_current_player() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let center = child_at(&mut app, hyperboard, 1, 1);
    let first_move = child_at(&mut app, center, 0, 0);
    play(&mut app, hyperboard, first_move);
    // The first move (top-left cell) sends blue to the top-left board.
    let board = child_at(&mut app, hyperboard, 0, 0);
    let cell = child_at(&mut app, board, 0, 0);

    click(&mut app, cell);
    assert_eq!(pressed(&app, cell), Some(player(&app, BLUE)));
}

#[test]
fn only_one_cell_can_be_pressed_per_turn_in_the_whole_hyperboard() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    allow_any_board(&mut app, hyperboard);
    let first_board = child_at(&mut app, hyperboard, 0, 0);
    let second_board = child_at(&mut app, hyperboard, 2, 2);
    let first = child_at(&mut app, first_board, 1, 1);
    let second = child_at(&mut app, second_board, 1, 1);

    click(&mut app, first);
    click(&mut app, second);

    assert_eq!(pressed(&app, first), None);
    assert_eq!(pressed(&app, second), Some(player(&app, RED)));
}

#[test]
fn won_boards_become_unclickable_and_the_rest_stay_clickable() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let won = child_at(&mut app, hyperboard, 0, 0);
    let other = child_at(&mut app, hyperboard, 1, 1);

    first_player_wins_board(&mut app, hyperboard, won, &mut 0);
    // Free choice: every board is allowed, but won ones stay unclickable.
    allow_any_board(&mut app, hyperboard);

    assert_eq!(
        app.world().get::<Board>(won).unwrap().winner(),
        Some(player(&app, RED))
    );
    assert!(!clickable(&app, won));
    assert!(clickable(&app, other));
}

#[test]
fn three_won_boards_in_a_row_win_the_hyperboard() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);

    first_player_wins_game(&mut app, hyperboard);

    let state = app.world().get::<Hyperboard>(hyperboard).unwrap();
    assert_eq!(state.winner(), Some(player(&app, RED)));
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let board = child_at(&mut app, hyperboard, col, row);
            assert!(!clickable(&app, board));
        }
    }
    let red = player(&app, RED);
    let overlays = app
        .world_mut()
        .query_filtered::<(&Sprite, &PlayerMark), With<WinnerOverlay>>()
        .iter(app.world())
        .map(|(sprite, mark)| (sprite.color, *mark))
        .collect::<Vec<_>>();
    // The winner's color, with the winner's icon on top.
    assert_eq!(
        overlays,
        vec![(
            RED,
            PlayerMark {
                player: Some(red),
                muted: false
            }
        )]
    );
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
    let board = child_at(&mut app, hyperboard, 1, 1);
    let cell = child_at(&mut app, board, 0, 0);
    click(&mut app, cell);
    assert_eq!(
        app.world()
            .get::<Hyperboard>(hyperboard)
            .unwrap()
            .active_board,
        Some(board)
    );

    end_turn_now(&mut app, hyperboard);
    assert_eq!(turn(&app, hyperboard).number, 2);
    let state = app.world().get::<Hyperboard>(hyperboard).unwrap();
    assert_eq!(state.active_board, None);
}

#[test]
fn a_turn_cannot_end_without_a_move() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);

    end_turn_now(&mut app, hyperboard);
    assert_eq!(turn(&app, hyperboard).number, 1);

    // Pressing a cell and unpressing it again is not a move either.
    let center = child_at(&mut app, hyperboard, 1, 1);
    let cell = child_at(&mut app, center, 0, 0);
    click(&mut app, cell);
    assert_eq!(pressed(&app, cell), Some(player(&app, RED)));
    click(&mut app, cell);
    end_turn_now(&mut app, hyperboard);
    assert_eq!(turn(&app, hyperboard).number, 1);
}

#[test]
fn full_board_without_winner_is_disabled() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let board = child_at(&mut app, hyperboard, 1, 1);
    for (col, row) in DRAW_MOVES {
        let cell = child_at(&mut app, board, col, row);
        play_anywhere(&mut app, hyperboard, cell);
    }

    let state = app.world().get::<Board>(board).unwrap();
    assert!(state.is_full());
    assert_eq!(state.winner(), None);
    assert!(!clickable(&app, board));
    // Cells keep their colors (drawn muted, like every disabled board), and
    // no square covers them.
    let top_left = child_at(&mut app, board, 0, 0);
    assert_eq!(pressed(&app, top_left), Some(player(&app, RED)));
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
        play_anywhere(&mut app, hyperboard, cell);
        let reply = harmless_cell(&mut app, hyperboard, harmless_moves);
        harmless_moves += 1;
        play_anywhere(&mut app, hyperboard, reply);
    }
    allow_any_board(&mut app, hyperboard);

    // The winning click and the end of the turn, in one single frame.
    let last_cell = child_at(&mut app, last_board, 2, 0);
    app.world_mut()
        .write_message(CellClicked { cell: last_cell });
    app.world_mut()
        .write_message(EndTurnRequested { hyperboard });
    app.update();

    assert_eq!(
        app.world().get::<Board>(last_board).unwrap().winner(),
        Some(player(&app, RED))
    );
    assert_eq!(
        app.world().get::<Hyperboard>(hyperboard).unwrap().winner(),
        Some(player(&app, RED))
    );
}

#[test]
fn first_turn_is_played_in_the_center_board() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);

    assert_eq!(clickable_boards(&mut app, hyperboard), vec![(1, 1)]);
    let center = child_at(&mut app, hyperboard, 1, 1);
    let corner = child_at(&mut app, hyperboard, 0, 0);
    let muted = |app: &mut App, board: Entity| {
        let cell = child_at(app, board, 1, 1);
        app.world().get::<Muted>(cell).unwrap().0
    };
    assert!(!muted(&mut app, center));
    assert!(muted(&mut app, corner));
}

#[test]
fn clicks_outside_the_allowed_board_are_ignored() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let corner = child_at(&mut app, hyperboard, 0, 0);
    let cell = child_at(&mut app, corner, 1, 1);

    click(&mut app, cell);
    assert_eq!(pressed(&app, cell), None);
}

#[test]
fn the_pressed_cell_sends_the_next_player_to_the_matching_board() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let center = child_at(&mut app, hyperboard, 1, 1);

    // Red presses the top-right cell of the center board...
    let cell = child_at(&mut app, center, 2, 0);
    play(&mut app, hyperboard, cell);

    // ...so blue must play in the top-right board.
    assert_eq!(
        next_board(&app, hyperboard),
        Some(GridPosition { col: 2, row: 0 })
    );
    assert_eq!(clickable_boards(&mut app, hyperboard), vec![(2, 0)]);

    // Blue presses the bottom-left cell there: red goes to the bottom-left board.
    let top_right = child_at(&mut app, hyperboard, 2, 0);
    let cell = child_at(&mut app, top_right, 0, 2);
    play(&mut app, hyperboard, cell);
    assert_eq!(clickable_boards(&mut app, hyperboard), vec![(0, 2)]);
}

#[test]
fn being_sent_to_a_won_board_allows_any_open_board() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let won = child_at(&mut app, hyperboard, 0, 0);
    first_player_wins_board(&mut app, hyperboard, won, &mut 0);

    // Red plays the top-left cell of the center board: that sends blue to
    // the top-left board, which is won.
    let center = child_at(&mut app, hyperboard, 1, 1);
    let cell = child_at(&mut app, center, 0, 0);
    play_anywhere(&mut app, hyperboard, cell);

    assert_eq!(next_board(&app, hyperboard), None);
    let open: Vec<_> = clickable_boards(&mut app, hyperboard);
    assert_eq!(open.len(), 8);
    assert!(!open.contains(&(0, 0)));
}

#[test]
fn being_sent_to_a_full_board_allows_any_open_board() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let full = child_at(&mut app, hyperboard, 1, 1);
    for (col, row) in DRAW_MOVES {
        let cell = child_at(&mut app, full, col, row);
        play_anywhere(&mut app, hyperboard, cell);
    }
    assert!(app.world().get::<Board>(full).unwrap().is_full());

    // Blue plays the center cell of the top-left board: that sends red to the
    // center board, which is full.
    let corner = child_at(&mut app, hyperboard, 0, 0);
    let cell = child_at(&mut app, corner, 1, 1);
    play_anywhere(&mut app, hyperboard, cell);

    assert_eq!(next_board(&app, hyperboard), None);
    let open = clickable_boards(&mut app, hyperboard);
    assert_eq!(open.len(), 8);
    assert!(!open.contains(&(1, 1)));
}

/// The move that wins a board can also send the next player to that same
/// board: it must count as won already.
#[test]
fn a_move_that_wins_the_board_it_points_to_allows_any_open_board() {
    let (mut app, hyperboard) = setup(vec![RED, BLUE]);
    let board = child_at(&mut app, hyperboard, 0, 0);
    for (harmless_move, col) in (1..3).enumerate() {
        let cell = child_at(&mut app, board, col, 0);
        play_anywhere(&mut app, hyperboard, cell);
        let reply = harmless_cell(&mut app, hyperboard, harmless_move);
        play_anywhere(&mut app, hyperboard, reply);
    }

    // Red completes the top row with the top-left cell of the top-left board.
    let cell = child_at(&mut app, board, 0, 0);
    play_anywhere(&mut app, hyperboard, cell);

    assert_eq!(
        app.world().get::<Board>(board).unwrap().winner(),
        Some(player(&app, RED))
    );
    assert_eq!(next_board(&app, hyperboard), None);
}

/// Counts, frame by frame, how often `Changed<Hyperboard>` matched.
#[derive(Resource, Default)]
struct HyperboardChanges(usize);

/// Systems and debug logs react to `Changed<Hyperboard>`, so a frame where
/// nothing happens must not mark it as changed.
#[test]
fn idle_frames_do_not_mark_the_hyperboard_as_changed() {
    let (mut app, _) = setup(vec![RED, BLUE]);
    // A system sees changes the way the game's systems do: "changed since
    // this system last ran". It runs in `Last`, after everything else.
    app.init_resource::<HyperboardChanges>().add_systems(
        Last,
        |changed: Query<(), Changed<Hyperboard>>, mut count: ResMut<HyperboardChanges>| {
            count.0 += changed.iter().count();
        },
    );
    app.update(); // First run: everything counts as changed, since it's new.
    app.world_mut().resource_mut::<HyperboardChanges>().0 = 0;

    for _ in 0..3 {
        app.update();
    }
    assert_eq!(app.world().resource::<HyperboardChanges>().0, 0);
}
