> **Historical checkpoint.** This page records an earlier stage; its code
> links and test counts describe the source at that time, not the finished game.
> For the current implementation, see [the architecture walkthrough](architecture-walkthrough.md).

# Checkpoint 2: fixed grid map and movement

## What was built

- A hand-authored 20 × 15 tile dungeon stored in `FIXED_MAP` (`src/main.rs`).
  `#` is a wall and `.` is floor. The layout includes a closed border and two
  internal wall segments with openings, so both wall types are easy to see and
  test.
- A `GridPosition { x, y }` component on the player. It is the only gameplay
  source of truth. `x` grows east, `y` grows north, and values are whole tiles.
- Keyboard movement: one fresh press of `W`, `A`, `S`, `D`, or an arrow key moves
  the player exactly one cardinal cell. If several movement keys are newly
  pressed in the same frame, the first match in `MOVEMENT_KEYS` wins and only one
  step happens.
- Blocking: `DungeonMap::is_walkable` returns `true` only for an in-bounds floor
  tile. Walls and out-of-bounds positions reject the move, leaving the player's
  grid position unchanged.
- Rendering: the floor and wall tiles are spawned once at startup. The player's
  `Transform` is derived from `GridPosition` by `sync_player_transform`, which
  runs after movement in the same frame. Nothing reads the transform to decide
  whether a move is legal.
- Nine headless tests, including the preserved startup test.

Explicitly **not** added in this checkpoint: monsters, combat, a wait action,
turn counters, procedural generation, and any commit or push.

## Controls

| Action | Keys |
| --- | --- |
| Move north | `W` or `↑` |
| Move south | `S` or `↓` |
| Move west | `A` or `←` |
| Move east | `D` or `→` |

One press is one action. Holding a key does not produce a new move until the key
is released and pressed again, because the game reads `ButtonInput::just_pressed`
rather than `pressed`. (Bevy only records `just_pressed` when a key transitions
from up to down; OS auto-repeat while the key is held does not retrigger it.)

## New Rust concepts

### `const` data and `const fn`

`PLAYER_START`, `FIXED_MAP`, and the tile colors are compile-time constants. Map
dimensions and the player start stay in one place and cannot be changed by
accident at runtime.

`GridPosition::new` and `GridPosition::offset` are `const fn`, which means the
compiler can evaluate them in a `const` item such as `PLAYER_START`.

### Small data types and `Copy`

`GridPosition` is a small struct with two `i32` fields and derives `Clone, Copy`.
`Copy` means passing it or writing `*position` copies the bytes instead of moving
ownership, so it is cheap to use as a value in tests and movement rules.

`Tile` is an enum. Exhaustive `match` on `Tile` makes the renderer handle every
tile kind; adding a new kind later will cause a compile error rather than a
silently missing sprite.

### `Option`, `let-else`, and `then_some`

`DungeonMap::tile_at` returns `Option<Tile>`: `None` means "outside the map" and
is distinct from a wall. `step_if_walkable` uses `bool::then_some` to turn the
walkability check into `Option<GridPosition>`, so "blocked" and "allowed" are
represented by the type instead of a pair of booleans.

`move_player` and `sync_player_transform` use `let ... else` to bail out cleanly
when there is no movement key or the expected single player. This avoids nesting
the rest of the system in `if let` blocks.

### Type aliases for complex types

Bevy's `Query` type includes borrow lifetimes, data, and filter parameters, which
is long enough that Clippy rejects it inline (`clippy::type_complexity`).
`ChangedPlayerTransforms` names that type once. The alias is documentation as
well as a lint fix.

## New Bevy / ECS concepts

### Components versus resources

- `Player` and `GridPosition` are **components**: data attached to one entity.
  There can be many of them, and each entity gets its own value.
- `DungeonMap` is a **resource**: one value shared by the whole world. Both input
  resolution and rendering need the map, so a resource avoids duplicating it or
  storing it on every wall entity.

### Queries and filters

`Query<&mut GridPosition, With<Player>>` iterates the players and borrows each
position mutably. `With<Player>` filters entities to those carrying the marker
component. `single_mut()` returns `Result`, not an optional tuple, because Bevy
distinguishes "no match" from "more than one match"; both are errors for a
single-player game and are handled by returning early.

`sync_player_transform` adds `Changed<GridPosition>`. Bevy tracks change ticks
per component, so the render sync only touches transforms for players whose grid
position changed this frame. A component added during spawning also counts as
changed, which is why the initial sync works without a special startup path.

### Input as a resource and frame edges

`ButtonInput<KeyCode>` is a resource maintained by Bevy's input plugin. Your
system reads it with `Res<ButtonInput<KeyCode>>`. The distinction between
`pressed` and `just_pressed` is the difference between "currently held" and "went
down this frame"; using `just_pressed` gives the discrete, turn-based step.

### Schedule ordering

The update systems are registered as:

```rust
.add_systems(Update, (move_player, sync_player_transform).chain())
```

`.chain()` makes the order explicit: resolve the input-driven move, then update
the sprite transform. Without the chain, Bevy may run them in either order, and
the rendered square could lag one frame behind the authoritative grid position.
`Startup` runs once for camera, tiles, and player creation; `Update` runs every
frame for input and presentation.

### Integer gameplay truth, derived presentation

`grid_to_world` is the only coordinate conversion:

```text
world_x = (grid_x - (width  - 1) / 2) * TILE_SIZE
world_y = (grid_y - (height - 1) / 2) * TILE_SIZE
```

The map is centered on the window origin. The inverse conversion (world to grid)
does not exist, because no gameplay decision may depend on the sprite transform.
Map rows are written north-to-south for readability; `tile_at` converts the
north-first row index to grid `y` with `height - 1 - y`. Keeping that flip in one
function prevents off-by-one errors in every other system.

### Headless ECS tests

The tests build a bare `App`, insert `DungeonMap` and a `ButtonInput`, add the
movement systems, and spawn one player. They do not create a window or GPU
context. Input is injected with `ButtonInput::press` followed by `App::update`;
the helper then clears `just_pressed`, mirroring what Bevy's input plugin does
each frame. This tests the same system code that runs in the real game while
keeping the tests fast and deterministic.

## Automated verification

All commands were run from the repository root with Rust 1.99.1 and Bevy 0.20.1:

```sh
cargo fmt
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

Results:

- `cargo fmt --check`: passed (no output).
- `cargo test`: **9 passed, 0 failed**:
  - `startup_creates_one_camera_and_visible_player_without_duplicates`
  - `wasd_moves_one_cell_per_press`
  - `arrow_keys_move_one_cell_per_press`
  - `walls_block_movement_and_keep_the_rendered_position_in_sync`
  - `map_boundaries_block_movement`
  - `border_walls_block_movement_at_the_map_edge`
  - `moving_again_requires_a_new_key_press`
  - `rendered_transform_is_derived_from_the_grid_position`
  - `fixed_map_has_a_closed_wall_border`
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build`: passed.

Test output excerpt:

```text
running 9 tests
test tests::fixed_map_has_a_closed_wall_border ... ok
test tests::map_boundaries_block_movement ... ok
test tests::walls_block_movement_and_keep_the_rendered_position_in_sync ... ok
test tests::rendered_transform_is_derived_from_the_grid_position ... ok
test tests::moving_again_requires_a_new_key_press ... ok
test tests::arrow_keys_move_one_cell_per_press ... ok
test tests::border_walls_block_movement_at_the_map_edge ... ok
test tests::wasd_moves_one_cell_per_press ... ok
test tests::startup_creates_one_camera_and_visible_player_without_duplicates ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The startup test was preserved and extended. It still checks exactly one `Camera2d`
and exactly one visible, non-transparent player sprite, and now also checks that
the player starts on `PLAYER_START` and that the sprite transform equals
`grid_to_world(PLAYER_START)`.

## Runtime launch

The built binary was launched for eight seconds and then stopped by the agent:

```sh
./target/debug/simple_rogue_like_rust_test
```

Log evidence:

```text
SystemInfo { os: "macOS 26.6.2 Tahoe", kernel: "25.6.0", cpu: "Apple M1", ... }
AdapterInfo { name: "Apple M1", ... backend: Metal, ... }
Creating new window Dungeon escape — grid movement (69v0)
```

There was no panic in the log. This proves the window and Metal renderer start;
it does **not** prove the grid looks correct or that keys move the square.

## Experiment record

| Field | Record |
| --- | --- |
| Task | Add a fixed grid map, integer grid-truth positions, WASD/arrow one-cell movement, wall/boundary blocking, derived rendering, and headless movement/blocking tests. Preserve the startup test. No monsters, combat, procedural generation, or commit/push. |
| Environment | macOS 26.6.2 (Tahoe), Apple M1, 16 GiB; rustc/cargo 1.99.1; Bevy 0.20.1; OpenCode with DeepSeek V4.1 Flash. |
| Time | Full task wall-clock time was not captured. Incremental timings observed: first test-profile compile after the change 5.33 s; later test compile 3.57 s; Clippy check 1.25 s after dependencies were checked; `cargo build` 2.71 s; window creation about 0.39 s after process start. Dependency builds were already cached from the previous checkpoint. |
| Iterations | Two automated check cycles. First `cargo test` passed but warned about an unused test helper; the helper was removed. First `cargo clippy ... -D warnings` failed with `clippy::type_complexity` on the filtered `Query`; fixed with the `ChangedPlayerTransforms` type alias. The second full check cycle passed. One runtime launch, no panic. |
| Human intervention | None during implementation. Human visual verification is pending at this checkpoint. |
| Outcome | 9/9 tests pass; fmt, Clippy with warnings denied, and build pass; window launches on Metal. Not yet proven: visual layout, key handling in the real window, and window-close behavior. No commit or push was made. |

## Human visual verification checklist

Run `cargo run` and confirm:

1. The window title is "Dungeon escape — grid movement" and the background is
   dark.
2. A 20 × 15 grid is visible: a gray closed border, two gray interior wall
   segments split by an open middle row, and dark floor tiles.
3. A green square starts on a floor tile in the left-middle area (grid `(2, 7)`).
4. Each single press of `W`/`A`/`S`/`D` or an arrow key moves the square exactly
   one tile in that direction.
5. Walking into the border or an interior wall leaves the square in place; it
   never overlaps a wall or leaves the map.
6. Holding a key does not make the square slide continuously.
7. Closing the window exits normally.

If any item fails, record which key and which tile, then stop before adding the
next gameplay step.
