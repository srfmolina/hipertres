// `super::super` is `game_loop/mod.rs`: its plugin and its public API.
use super::super::*;
use super::constant::RESTART_KEY;

/// An App with only the game loop: no window, no match, no other feature.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(GameLoopPlugin);
    // Keyboard state, normally created by `DefaultPlugins`.
    app.init_resource::<ButtonInput<KeyCode>>();
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
    // Stand-in for the match: `game` marks the players and the hyperboard
    // the same way.
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
