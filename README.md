# Hipertres

A 2D board game written in [Rust](https://www.rust-lang.org/) with the
[Bevy](https://bevy.org/) game engine.

> **Status:** early development. A 3×3 hyperboard of 3×3 boards for two
> players, **o** (blue) and **x** (orange): press a cell, end the turn with
> **Space** (you can't pass without pressing a cell). Pressed cells and won
> boards show their player's color and symbol. Win three boards in a row to
> win the game. Press **R** at any time to restart the match.
>
> The first move is played in the center board. After that, the cell you press
> sends the next player to the board at the same position; if that board is
> won or full, they can play in any open board. Boards you can't play in are
> drawn with muted colors.

## Requirements

- **Rust**: latest stable (Bevy's minimum supported Rust version is always
  the latest stable). Install it with [rustup](https://rustup.rs/), and
  update it with `rustup update`.
- **System libraries** (Linux only). On Fedora:

  ```sh
  sudo dnf install gcc-c++ libX11-devel alsa-lib-devel systemd-devel wayland-devel libxkbcommon-devel
  ```

  For other distributions, see Bevy's
  [Linux dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md) list.
  Windows needs the Visual Studio C++ Build Tools, and macOS needs
  `xcode-select --install`.

## Running

```sh
cargo run
```

The **first build takes several minutes** (about 10 on a recent laptop)
because it compiles all of Bevy. After that, rebuilds only compile the game
code and take a few seconds.

For an optimized build:

```sh
cargo run --release
```

## Project structure

```
src/
├── main.rs              Builds the Bevy App: DefaultPlugins + FeaturePlugins
└── feature/
    ├── mod.rs           FeaturePlugins: plugin group with every feature
    ├── common/          Shared code that no feature owns (e.g. `mute` colors)
    ├── debug/           DebugPlugin: switchable diagnostics (see Debugging)
    ├── camera/          CameraPlugin: 2D camera and background color
    ├── game_loop/       GameLoopPlugin: turns, turn order, the match's state, and the order of a frame
    ├── player/          PlayerPlugin: players (symbol, color) and their icons
    ├── cell/            CellPlugin: clickable cells that toggle pressed/unpressed
    ├── board/           BoardPlugin: 3×3 cells, one press per turn, three in a row
    ├── hyperboard/      HyperboardPlugin: 3×3 boards, one press per turn, where the next player plays, game winner
    └── game/            GamePlugin: spawns the players and the hyperboard when a match starts
```

Each game feature is a Bevy `Plugin` in its own folder under `src/feature/`,
following the [official plugin guide](https://bevy.org/learn/quick-start/getting-started/plugins/).
Inside a feature, each kind of Bevy item has its own file, so you know where
to look. A feature only has the files it needs:

| File           | Contents |
|----------------|----------|
| `mod.rs`       | The plugin, and the feature's public API (`pub use`) |
| `component.rs` | Components |
| `resource.rs`  | Resources |
| `message.rs`   | Messages (read with `MessageReader`) |
| `event.rs`     | Events (handled by observers) |
| `state.rs`     | Bevy states (`#[derive(States)]`, `State<T>`/`NextState<T>`, `OnEnter`/`OnExit`) |
| `system.rs`    | Systems, observers, system sets, and helpers used only by them |
| `spawn.rs`     | `spawn_*` functions that build entities (not systems) |
| `rule.rs`      | Game rules as plain functions (not systems) |
| `meta/`        | Code *about* the feature: `constant.rs` (tunable values), `debug.rs` (diagnostics on its debug channel), `testing.rs` (test helpers for other features), `tests.rs` |

Every file is a private module. `mod.rs` re-exports with `pub use` the names
other features may use, so the top of `mod.rs` is the feature's API. Items
only the feature itself uses are `pub(super)`.

A type goes in the feature that **defines what it means**, even if other
features use it: `ActivePlayer` is in `cell` because cells read it, although
boards insert it; `Turn` is in `game_loop` because the loop defines what a
turn is, even though it lives on the hyperboard entity and the boards are
what react to it. `common/` is only for code that no feature owns.

Small features (`camera`, `game`) stay in a single `mod.rs`.

To add a feature, create its folder and register its plugin in
`FeaturePlugins` (`src/feature/mod.rs`). `main.rs` doesn't change.

### Order of a frame

The `game_loop` feature owns the order of a turn as one chained `TurnPhase`
(`Input, Mark, OnePerBoard, OnePerTurn, EndTurn, BoardResults, MatchResults,
Control`), gated to run only while the match is being played: clicks are
applied to cells, then the one-press-per-turn rules run, then the end of
turn, then **boards** check their wins, then the **hyperboard** checks its
win, chooses the next board and sets each board's `BoardControl`, then
boards apply it. A separate `DrawPhase` (`State`, then `Icons`) draws the
result in `PostUpdate`, and keeps running even once the match is finished.
The full list is in `src/feature/game_loop/system.rs`, and tests check that
a click, a board win and a game win all resolve in the same frame.

## Tests

```sh
cargo test
```

Each feature's tests live in its `meta/tests.rs`, declared in `meta/mod.rs`
with `#[cfg(test)] mod tests;`. Since `tests` is inside the feature, it can
test the feature's private items too, and it is only compiled for
`cargo test`. Tests for `common/` are in `common/tests.rs`.

## Debugging

Diagnostics are grouped in **channels** that you turn on and off. All of them
are off by default.

| Channel   | Key | Logs |
|-----------|-----|------|
| `input`   | F1  | Raw mouse buttons and cursor position |
| `picking` | F2  | What the pointer is over, and pointer events (Over, Press, Click) per entity |
| `cells`   | F3  | Cell state changes (pressed, color) |
| `boards`  | F4  | Board state changes (turn, pressed cell, winner) |
| `hyperboard` | F5 | Hyperboard state changes (next board, winner) |
| `gameloop` | F6 | Turn changes and match state transitions (`Setup`/`Playing`/`Finished`) |

Turn channels on at startup with `HIPERTRES_DEBUG`, or toggle them in game
with their key:

```sh
HIPERTRES_DEBUG=picking,cells cargo run
HIPERTRES_DEBUG=all cargo run
```

To add a channel, add a variant to `DebugChannel` (`src/feature/debug/resource.rs`)
with its name and key. Then, in the feature's `meta/debug.rs`, add systems with
`.run_if(debug_on(DebugChannel::YourChannel))`.

## Tech stack

| What        | Version |
|-------------|---------|
| Bevy        | 0.19.1  |
| Rust        | edition 2024 |

Bevy changes a lot between versions, and tutorials or answers written for an
older version often won't compile. When looking things up, check which
version they target. The source for this exact version, including its
`examples/` folder, is in `~/.cargo/registry/src/*/bevy-0.19.1/` after the
first build.

### Build profile

`Cargo.toml` follows the Bevy quick-start recommendations:

- `[profile.dev] opt-level = 1`: light optimization for our own code, so
  debug builds stay fast to compile.
- `[profile.dev.package."*"] opt-level = 3`: full optimization for
  dependencies (Bevy). They are compiled only once, and an unoptimized Bevy
  runs very slowly.

## Useful links

- [Bevy quick start](https://bevy.org/learn/quick-start/getting-started/)
- [Bevy examples](https://bevy.org/examples/): browsable and runnable in the browser
- [API docs (docs.rs)](https://docs.rs/bevy/0.19.1/bevy/)
- [Unofficial Bevy Cheat Book](https://bevy-cheatbook.github.io/): good concept
  explanations, but some code targets older versions

## License

The **source code** of Hipertres is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. This is the same dual license used by Bevy and most of the
Rust ecosystem.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

**Not covered by this license:** the name "Hipertres", its logo, and the
game's art, music and sound assets, as they are added. They are
© Serafín López Molina (srfmolina), all rights reserved, unless a file says
otherwise.
