> **Historical checkpoint.** This page records an earlier stage; its code
> links and test counts describe the source at that time, not the finished game.
> For the current implementation, see [the architecture walkthrough](architecture-walkthrough.md).

# Player art: an ECS example

This checkpoint changes only the player's appearance, not its movement rules.

## Follow the player

The player is one **entity** (an identity in Bevy's world), with several
**components** (pieces of data):

- [`Player`](../src/main.rs#L49-L50): a marker identifying it as the player.
- [`GridPosition`](../src/main.rs#L52-L59): its authoritative tile coordinates.
- [`Sprite`](../src/main.rs#L199-L203): its image, color tint, and display size.
- [`Transform`](../src/main.rs#L204): its position in the rendered world.

In [`setup`](../src/main.rs#L176-L206), the
[`commands.spawn` call](../src/main.rs#L196-L205) creates one player entity
with those components.
[`AssetServer::load`](../src/main.rs#L200) requests the
[`PLAYER_TEXTURE` path](../src/main.rs#L15-L16), relative to `assets/`:
[`tile_0085.png`](../assets/third_party/kenney_tiny_dungeon/Tiles/tile_0085.png).
It returns a `Handle<Image>`: a reference to an asset managed by Bevy, not the
image pixels themselves. Loading happens asynchronously; the sprite can be
created before the image finishes loading.

The `Sprite` now references that image instead of a solid green rectangle.
Its default white tint preserves the artwork's colors. The original image is
16 × 16 pixels; [`custom_size`](../src/main.rs#L201) displays it at 32 × 32
world units, using the [size constants](../src/main.rs#L9-L13).
[`ImagePlugin::default_nearest()`](../src/main.rs#L159-L161)
selects nearest-neighbor filtering, avoiding
blur when enlarging pixel art.

The **systems** still operate on the same data:

1. [`move_player`](../src/main.rs#L231-L246) changes `GridPosition` when a fresh
   key press requests a legal move.
2. [`sync_player_transform`](../src/main.rs#L259-L265) derives `Transform` from
   that grid position using [`grid_to_world`](../src/main.rs#L269-L277).
3. Bevy's rendering systems draw the image.

The [chained update registration](../src/main.rs#L172) ensures movement runs
before transform synchronization.

Neither movement system needs to know which character image is used.
Floors and walls are unchanged; animation and scene templates are out of scope.

## Artwork

Kenney's Tiny Dungeon pack is preserved in
[`assets/third_party/kenney_tiny_dungeon/`](../assets/third_party/kenney_tiny_dungeon/),
including its [CC0 license](../assets/third_party/kenney_tiny_dungeon/License.txt).
Source: https://kenney.nl/assets/tiny-dungeon

## Experiment and verification

- Scope: replace only the player square with one existing character image.
- Tools: repository/file tools and local Cargo commands; no delegated agents.
- Human input: supplied the art pack and approved the player-only art pass.
- Incremental compilation: approximately 10 seconds for tests, 2 seconds for
  Clippy, and 5 seconds for the development build. These are not initial Bevy
  compilation timings.
- `cargo test`: all nine tests passed. The
  [startup test](../src/main.rs#L338-L373) now also checks the
  texture reference, white tint, and display size.
- `cargo clippy --all-targets --all-features -- -D warnings` and `cargo build` passed.
- Headless tests do not prove successful image decoding or visual correctness.
  Runtime asset loading and visual verification remain pending.

Run `cargo run` and check:

1. An adventurer replaces the green square, with no green tint or blurry pixels.
2. WASD/arrow movement and wall blocking still work.
3. There are no asset-loading errors in the terminal.
4. Closing the window exits normally.
