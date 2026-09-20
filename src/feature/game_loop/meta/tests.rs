// `super::super` is `game_loop/mod.rs`: its plugin and its public API.
use super::super::*;
use super::constant::RESTART_KEY;

/// An App with only the game loop: no window, no match, no other feature.
fn test_app() -> App {
    let mut app = App::new();
    // The plugin itself provides the keyboard state (see its `build`).
    app.add_plugins(GameLoopPlugin);
    app
}

/// The state the game is in right now.
fn state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

#[test]
fn a_game_starts_in_setup() {
    let mut app = test_app();
    app.update();
    assert_eq!(state(&app), GameState::Setup);
}

#[test]
fn start_playing_puts_the_test_app_in_playing() {
    let mut app = test_app();
    start_playing(&mut app);
    assert_eq!(state(&app), GameState::Playing);
}

#[test]
fn the_restart_key_goes_back_to_setup_and_despawns_the_match() {
    let mut app = test_app();
    start_playing(&mut app);
    // Stand-in for the match: unlike `game`, which despawns the players and
    // the hyperboard with `DespawnWhen` (via `despawn_on_setup()`), this
    // uses the plain `DespawnOnEnter` Bevy provides — good enough for this
    // test, which only checks that a restart despawns the old match.
    let root = app.world_mut().spawn(DespawnOnEnter(GameState::Setup)).id();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(RESTART_KEY);
    app.update(); // the restart system queues `Setup`
    app.update(); // the transition runs, and despawns the old match

    assert_eq!(state(&app), GameState::Setup);
    assert!(app.world().get_entity(root).is_err());
}

#[test]
fn the_restart_key_does_nothing_while_setting_up() {
    let mut app = test_app();
    app.update();
    let root = app.world_mut().spawn(DespawnOnEnter(GameState::Setup)).id();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(RESTART_KEY);
    app.update();
    app.update();

    assert_eq!(state(&app), GameState::Setup);
    assert!(app.world().get_entity(root).is_ok());
}

/// Stand-in for `spawn_game`'s match root: spawned *from inside*
/// `OnEnter(GameState::Setup)`, the same shape as the real match, marked to
/// despawn on that same transition. Kept local to this test module instead
/// of pulling in `game` or `hyperboard`.
#[derive(Component)]
struct MatchStandIn;

fn spawn_match_stand_in(mut commands: Commands) {
    commands.spawn((
        MatchStandIn,
        DespawnWhen::new(|transition: &StateTransitionEvent<GameState>| {
            transition.entered == Some(GameState::Setup)
        }),
    ));
}

fn match_stand_ins(app: &mut App) -> Vec<Entity> {
    app.world_mut()
        .query_filtered::<Entity, With<MatchStandIn>>()
        .iter(app.world())
        .collect()
}

/// Regression test for an ordering hazard `DespawnOnEnter` has and
/// `DespawnWhen` does not: a match spawned by `OnEnter(Setup)` must not be
/// despawned by that very same entry. `DespawnOnEnter`'s despawn system and
/// the systems of `OnEnter(Setup)` both live in
/// `StateTransitionSystems::EnterSchedules` with no order between them, so
/// this could fail depending on the schedule's toposort; `DespawnWhen`
/// instead runs from `TransitionSchedules`, which Bevy chains strictly
/// before `EnterSchedules`.
#[test]
fn a_match_spawned_on_entering_setup_survives_that_same_entry() {
    let mut app = test_app();
    app.add_systems(OnEnter(GameState::Setup), spawn_match_stand_in);

    app.update(); // first entry into `Setup`: spawns the stand-in

    let matches = match_stand_ins(&mut app);
    assert_eq!(
        matches.len(),
        1,
        "the match built by this entry into Setup must not despawn itself"
    );
}

#[test]
fn restarting_despawns_the_old_match_and_setup_builds_a_new_one() {
    let mut app = test_app();
    app.add_systems(OnEnter(GameState::Setup), spawn_match_stand_in);

    app.update(); // first entry into `Setup`
    let original = match_stand_ins(&mut app)[0];

    start_playing(&mut app);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(RESTART_KEY);
    app.update(); // the restart system queues `Setup`
    app.update(); // the old match despawns, then `Setup` spawns a new one

    let matches = match_stand_ins(&mut app);
    assert_eq!(matches.len(), 1);
    assert_ne!(
        matches[0], original,
        "restart must spawn a new match, not keep the old one"
    );
}

#[test]
#[should_panic(expected = "at least one player")]
fn a_turn_order_needs_at_least_one_player() {
    TurnOrder::new(Vec::new());
}

#[test]
fn players_take_turns_in_order_and_start_again() {
    let mut app = test_app();
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let order = TurnOrder::new(vec![a, b]);

    assert_eq!(order.player_for_turn(1), a);
    assert_eq!(order.player_for_turn(2), b);
    assert_eq!(order.player_for_turn(3), a);
    assert_eq!(
        order.first_turn(),
        Turn {
            number: 1,
            player: a
        }
    );
}

#[test]
fn a_turn_brings_a_pending_move_with_it() {
    let mut app = test_app();
    let player = app.world_mut().spawn_empty().id();
    let root = app.world_mut().spawn(Turn { number: 1, player }).id();
    assert_eq!(
        app.world().get::<PendingMove>(root),
        Some(&PendingMove(None))
    );
}

/// A match root: a turn for `player`, an order, and no staged move yet.
fn spawn_root(app: &mut App, players: Vec<Entity>) -> Entity {
    let order = TurnOrder::new(players);
    let turn = order.first_turn();
    app.world_mut().spawn((order, turn)).id()
}

fn turn(app: &App, root: Entity) -> Turn {
    *app.world().get::<Turn>(root).unwrap()
}

#[test]
fn a_turn_does_not_end_without_a_staged_move() {
    let mut app = test_app();
    start_playing(&mut app);
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let root = spawn_root(&mut app, vec![a, b]);

    app.world_mut().write_message(EndTurnRequested { root });
    app.update();

    assert_eq!(
        turn(&app, root),
        Turn {
            number: 1,
            player: a
        }
    );
}

#[test]
fn a_staged_move_lets_the_turn_end_and_gives_it_to_the_next_player() {
    let mut app = test_app();
    start_playing(&mut app);
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let root = spawn_root(&mut app, vec![a, b]);
    let move_site = app.world_mut().spawn_empty().id();
    app.world_mut()
        .entity_mut(root)
        .insert(PendingMove(Some(move_site)));

    app.world_mut().write_message(EndTurnRequested { root });
    app.update();

    assert_eq!(
        turn(&app, root),
        Turn {
            number: 2,
            player: b
        }
    );
}

#[test]
fn turns_do_not_end_once_the_match_is_finished() {
    let mut app = test_app();
    start_playing(&mut app);
    let a = app.world_mut().spawn_empty().id();
    let root = spawn_root(&mut app, vec![a]);
    app.world_mut()
        .entity_mut(root)
        .insert(PendingMove(Some(a)));
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Finished);
    app.update();

    app.world_mut().write_message(EndTurnRequested { root });
    app.update();

    assert_eq!(turn(&app, root).number, 1);
}
