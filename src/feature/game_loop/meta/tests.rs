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
