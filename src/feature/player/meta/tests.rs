// `super::super` is `player/mod.rs`: its plugin and its public API.
use super::super::component::PlayerIcon;
use super::super::*;
use super::constant::{MARK_SCALE, MARK_TEXT_COLOR};
use crate::feature::common::color::mute;

const RED: Color = Color::srgb(1.0, 0.0, 0.0);
const BLUE: Color = Color::srgb(0.0, 0.0, 1.0);

/// An App with only the player feature: no window or rendering needed.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(PlayerPlugin);
    app
}

/// Spawns players with `spawn_players` and returns their entities.
fn spawn(app: &mut App, players: &[(char, Color)]) -> Vec<Entity> {
    let players = players.to_vec();
    app.world_mut()
        .run_system_cached_with(
            |In(players): In<Vec<(char, Color)>>, mut commands: Commands| {
                spawn_players(&mut commands, &players)
            },
            players,
        )
        .unwrap()
}

/// A square of side `size` carrying `mark`.
fn spawn_square(app: &mut App, size: f32, mark: PlayerMark) -> Entity {
    app.world_mut()
        .spawn((Sprite::from_color(Color::WHITE, Vec2::splat(size)), mark))
        .id()
}

/// The icons (children with `PlayerIcon`) of `square`.
fn icons(app: &mut App, square: Entity) -> Vec<Entity> {
    let world = app.world_mut();
    let Some(children) = world.get::<Children>(square) else {
        return Vec::new();
    };
    let children: Vec<Entity> = children.iter().collect();
    children
        .into_iter()
        .filter(|&child| world.get::<PlayerIcon>(child).is_some())
        .collect()
}

/// The only icon of `square`. Panics if it has none or several.
fn icon(app: &mut App, square: Entity) -> Entity {
    let icons = icons(app, square);
    assert_eq!(icons.len(), 1, "expected exactly one icon");
    icons[0]
}

fn icon_text(app: &mut App, square: Entity) -> String {
    let icon = icon(app, square);
    app.world().get::<Text2d>(icon).unwrap().0.clone()
}

#[test]
fn spawn_players_keeps_order_symbol_and_color() {
    let mut app = test_app();
    let players = spawn(&mut app, &[('a', RED), ('b', BLUE)]);

    assert_eq!(players.len(), 2);
    let world = app.world();
    assert_eq!(
        world.get::<Player>(players[0]),
        Some(&Player { symbol: 'a' })
    );
    assert_eq!(
        world.get::<PlayerColor>(players[0]),
        Some(&PlayerColor(RED))
    );
    assert_eq!(
        world.get::<Player>(players[1]),
        Some(&Player { symbol: 'b' })
    );
    assert_eq!(
        world.get::<PlayerColor>(players[1]),
        Some(&PlayerColor(BLUE))
    );
}

#[test]
fn default_players_are_o_and_x() {
    let symbols: Vec<char> = DEFAULT_PLAYERS.iter().map(|&(symbol, _)| symbol).collect();
    assert_eq!(symbols, ['o', 'x']);
}

#[test]
fn mark_with_a_player_gets_one_icon_with_its_symbol() {
    let mut app = test_app();
    let players = spawn(&mut app, &[('x', RED)]);
    let square = spawn_square(
        &mut app,
        100.0,
        PlayerMark {
            player: Some(players[0]),
            muted: false,
        },
    );
    app.update();

    assert_eq!(icon_text(&mut app, square), "x");
    let icon = icon(&mut app, square);
    assert_eq!(
        app.world().get::<TextColor>(icon),
        Some(&TextColor(MARK_TEXT_COLOR))
    );
}

#[test]
fn icon_size_is_a_fraction_of_the_square() {
    let mut app = test_app();
    let players = spawn(&mut app, &[('x', RED)]);
    let square = spawn_square(
        &mut app,
        200.0,
        PlayerMark {
            player: Some(players[0]),
            muted: false,
        },
    );
    app.update();

    let icon = icon(&mut app, square);
    let font = app.world().get::<TextFont>(icon).unwrap();
    // `FontSize` doesn't implement `PartialEq`, so match on it instead.
    assert!(matches!(font.font_size, FontSize::Px(size) if size == 200.0 * MARK_SCALE));
}

#[test]
fn changing_the_player_replaces_the_icon() {
    let mut app = test_app();
    let players = spawn(&mut app, &[('o', RED), ('x', BLUE)]);
    let square = spawn_square(
        &mut app,
        100.0,
        PlayerMark {
            player: Some(players[0]),
            muted: false,
        },
    );
    app.update();

    app.world_mut()
        .get_mut::<PlayerMark>(square)
        .unwrap()
        .player = Some(players[1]);
    app.update();

    // Still exactly one icon (`icon_text` checks), now with the new symbol.
    assert_eq!(icon_text(&mut app, square), "x");
}

#[test]
fn mark_without_a_player_has_no_icon() {
    let mut app = test_app();
    let players = spawn(&mut app, &[('o', RED)]);
    let square = spawn_square(&mut app, 100.0, PlayerMark::default());
    app.update();
    assert!(icons(&mut app, square).is_empty());

    app.world_mut()
        .get_mut::<PlayerMark>(square)
        .unwrap()
        .player = Some(players[0]);
    app.update();
    app.world_mut()
        .get_mut::<PlayerMark>(square)
        .unwrap()
        .player = None;
    app.update();
    assert!(icons(&mut app, square).is_empty());
}

#[test]
fn muted_mark_draws_a_muted_icon() {
    let mut app = test_app();
    let players = spawn(&mut app, &[('o', RED)]);
    let square = spawn_square(
        &mut app,
        100.0,
        PlayerMark {
            player: Some(players[0]),
            muted: true,
        },
    );
    app.update();

    let icon = icon(&mut app, square);
    assert_eq!(
        app.world().get::<TextColor>(icon),
        Some(&TextColor(mute(MARK_TEXT_COLOR)))
    );
}
