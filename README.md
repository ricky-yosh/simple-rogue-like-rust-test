# simple-rogue-like-rust-test

A short, handcrafted, turn-based dungeon roguelike built with Bevy 0.20.1. It is
also a learning experiment: the implementation is deliberately small, modular,
and documented for someone learning Rust and ECS.

## Run

With Rust 1.99.1 and the [platform prerequisites](docs/setup.md) installed:

```sh
cargo run
```

The first build compiles Bevy and can take several minutes. The app opens a
1280 × 800 window with a 32 × 18 dungeon, pixel-art actors from Kenney's CC0 Tiny
Dungeon pack, a persistent HUD, an upgrade panel, and win/loss screens.

## Controls

| Action | Input |
| --- | --- |
| Move north / south / west / east | `W` `S` `A` `D` or arrow keys |
| Wait one turn | `Space` |
| Attack | Walk into an enemy |
| Choose a level-up upgrade | `1` `2` `3` or click a card |
| Restart after victory/defeat | `R` or click Restart |
| Close the game | Window close button |

Each fresh movement press is one tile. Walls, map boundaries, and the locked exit
block movement without consuming a turn. Moving into an enemy attacks instead.

## The run

- Explore a fixed dungeon with two regular enemy types: a pursuing skeleton and a
  slower, tougher brute that acts every other turn.
- Defeat enemies for XP. Crossing a threshold freezes the dungeon and offers three
  repeatable upgrades: Vitality, Strength, and Recovery.
- Reach the boss's exit alcove. The Warden telegraphs a heavy attack on a red
  tile: move before the next enemy phase to dodge it, then strike during its
  recovery turn.
- Defeat the Warden to unlock the door, step through to win, or lose at zero HP.
- Restart at any result without relaunching the app.

## Checks

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

The current suite has 47 headless tests. The final results and limitations are in
the [experiment record](docs/experiment-record.md). Automated checks do not
replace the [manual playtest checklist](docs/playtest-checklist.md).

## Artwork

The unmodified Kenney Tiny Dungeon pack is preserved under
[`assets/third_party/kenney_tiny_dungeon/`](assets/third_party/kenney_tiny_dungeon/),
including its [CC0 license](assets/third_party/kenney_tiny_dungeon/License.txt).
Source: <https://kenney.nl/assets/tiny-dungeon>.

## Documentation

Start with [the documentation index](docs/README.md). The main walkthrough is
[the architecture guide](docs/architecture-walkthrough.md); it links the finished
code, explains components/resources/states, and follows a key press through
combat, XP, upgrades, the boss, and restart.
