# What is Bevy?

Bevy is a free, open-source game engine written in Rust, licensed under MIT or
Apache-2.0. It supports both 2D and 3D applications. You use it as a dependency in
a Rust project; a separate Unity-style editor installation is not required.

The normal starting workflow is: edit code → build with Cargo → run the game →
inspect the result. Asset creation can still involve external art/audio tools.

## The mental model

Bevy uses an Entity Component System (ECS), rather than a hierarchy of game-object classes.

| Concept | Meaning | Dungeon example |
| --- | --- | --- |
| Entity | An identifier for a thing in the game | The player or a particular monster |
| Component | Data attached to an entity | Grid position, health, player marker |
| System | A Rust function that operates on game data | Resolve movement or monster turns |
| Resource | Shared data in the game world | The dungeon map or current turn |
| Schedule | When systems run and how they are ordered | Input before movement before enemy actions |
| Plugin | A way to group related engine/game functionality | Dungeon gameplay setup and systems |

Bevy supplies facilities such as rendering, windows, input, assets, audio, UI,
and scheduling. Our game's combat rules, dungeon generation, and enemy behavior
are still ours to implement. We do not need a physics engine for grid movement.

## Why code-first may help agents

- Game setup and behavior can be represented in reviewable source files.
- Cargo offers repeatable commands for compilation, tests, formatting, and linting.
- Rust's compiler catches many type and borrowing mistakes before launch.
- Gameplay rules and some ECS interactions can be tested without opening a window.
- Text changes are easier to inspect and reproduce than undocumented editor clicks.

These are reasons to test the thesis, not proof that Bevy produces better games.

## Important limitations

- A successful build does not prove that the camera, graphics, UI, or controls work.
  Screenshots and hands-on playtesting remain necessary.
- Rust and ECS both have learning curves. Scheduling and conflicting data access
  can produce bugs that ordinary type checking does not eliminate.
- Initial engine compilation can be expensive. Some dependency/profile/toolchain
  changes cause substantial rebuilds; later builds are not guaranteed to be fast.
- Bevy is pre-1.0 and publishes breaking API changes. Agents may generate code
  from outdated tutorials. Use examples and API docs matching our chosen version.
- Unity also supports scripted editor automation and automated testing. A fair
  comparison must include those capabilities rather than treating it as GUI-only.

## Version policy

At research time, GitHub's latest stable release endpoint returned **Bevy 0.20.1**.
Its tagged manifest declares Rust **1.97.1** as the minimum supported version.
The general setup guide still shows a 0.19 dependency example, demonstrating why
we should check release-specific sources rather than copy version numbers blindly.

Recommendation: start with 0.20.1, commit the generated `Cargo.lock`, record the
working Rust toolchain, and avoid engine upgrades during the first experiment.
Recheck availability when initializing the project; do not silently change versions.

## Primary sources

- [Official introduction and stability warning](https://bevy.org/learn/quick-start/introduction/)
- [Official setup guide](https://bevy.org/learn/quick-start/getting-started/setup/)
- [Official ECS introduction](https://bevy.org/learn/quick-start/getting-started/ecs/)
- [Bevy 0.20.1 release](https://github.com/bevyengine/bevy/releases/tag/v0.20.1)
- [Tagged manifest, including minimum Rust version](https://github.com/bevyengine/bevy/blob/v0.20.1/Cargo.toml)
- [Version-specific API documentation](https://docs.rs/bevy/0.20.1/bevy/)
- [Version-specific examples](https://github.com/bevyengine/bevy/tree/v0.20.1/examples)
