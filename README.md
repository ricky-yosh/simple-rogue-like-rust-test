# simple-rogue-like-rust-test
Simple rogue-like game to test the capabilities of the bevy game engine

## Run

With Rust 1.99.1 and the [platform prerequisites](docs/setup.md) installed:

```sh
cargo run
```

The first build compiles Bevy and can take several minutes. The app opens a
960 × 640 window with a fixed 20 × 15 dungeon grid and a green player square.

## Controls

| Action | Keys |
| --- | --- |
| Move north | `W` or `↑` |
| Move south | `S` or `↓` |
| Move west | `A` or `←` |
| Move east | `D` or `→` |

Each key press moves the player one tile. Walls and the map border block
movement. Movement is currently the only action; monsters and combat are not
implemented.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

## Planning and learning

Start with [the documentation index](docs/README.md). The current implementation
and its concept explanations are in
[the grid movement checkpoint](docs/checkpoint-grid-movement.md).
