# simple-rogue-like-rust-test
Simple rogue-like game to test the capabilities of the bevy game engine

## Run

With Rust 1.99.1 and the [platform prerequisites](docs/setup.md) installed:

```sh
cargo run
```

The first build compiles Bevy and can take several minutes. The app opens a
960 × 640 window with a green player square centered on a dark background.
Close the window to exit. Movement and dungeon gameplay are not implemented yet.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Planning and learning

Start with [the documentation index](docs/README.md).
