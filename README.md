# Hipertres

A 2D board game written in [Rust](https://www.rust-lang.org/) with the
[Bevy](https://bevy.org/) game engine.

> **Status:** early development. A 3×3 hyperboard of 3×3 boards for two
> players: press a cell, end the turn with **Space** (you can't pass without
> pressing a cell). Win three boards in a row to win the game.
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
    ├── debug/           DebugPlugin: switchable diagnostics (see Debugging)
    ├── camera/          CameraPlugin: 2D camera and background color
    ├── cell/            CellPlugin: clickable cells that toggle pressed/unpressed
    ├── board/           BoardPlugin: 3×3 cells, one press per turn, three in a row
    ├── hyperboard/      HyperboardPlugin: 3×3 boards, players, turns, game winner
    └── game/            GamePlugin: spawns the hyperboard at startup
```

Each game feature is a Bevy `Plugin` in its own folder under `src/feature/`,
following the [official plugin guide](https://bevy.org/learn/quick-start/getting-started/plugins/).
A feature folder can contain:

| File          | Contents |
|---------------|----------|
| `mod.rs`      | The plugin, its components and systems |
| `constant.rs` | Tunable values (sizes, colors...) |
| `debug.rs`    | The feature's diagnostics, on its own debug channel |
| `tests.rs`    | The feature's unit tests |

To add a feature, create its folder and register its plugin in
`FeaturePlugins` (`src/feature/mod.rs`). `main.rs` doesn't change.

### Order of a frame

Every game step runs in a named Bevy system set, and the order between them
is guaranteed: clicks are applied to cells, then the one-press-per-turn rules
run, then the end of turn, then **boards** check their wins, then the
**hyperboard** checks its win. The full list is in the docs of
`src/feature/hyperboard/mod.rs`, and tests check that a click, a board win and
a game win all resolve in the same frame.

## Tests

```sh
cargo test
```

Tests live in a `tests.rs` file next to the code they test, declared at the
bottom of `mod.rs` with `#[cfg(test)] mod tests;`. Since `tests` is a child
module, it can test private functions too, and it is only compiled for
`cargo test`.

## Debugging

Diagnostics are grouped in **channels** that you turn on and off. All of them
are off by default.

| Channel   | Key | Logs |
|-----------|-----|------|
| `input`   | F1  | Raw mouse buttons and cursor position |
| `picking` | F2  | What the pointer is over, and pointer events (Over, Press, Click) per entity |
| `cells`   | F3  | Cell state changes (pressed, color) |
| `boards`  | F4  | Board state changes (turn, pressed cell, winner) |
| `hyperboard` | F5 | Hyperboard state changes (turn, player, active board, winner) |

Turn channels on at startup with `HIPERTRES_DEBUG`, or toggle them in game
with their key:

```sh
HIPERTRES_DEBUG=picking,cells cargo run
HIPERTRES_DEBUG=all cargo run
```

To add a channel, add a variant to `DebugChannel` (`src/feature/debug/mod.rs`)
with its name and key. Then, in the feature's `debug.rs`, add systems with
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
