# Bevy learning experiment

## Goal

Learn Bevy by building a small game, while testing whether its code-first workflow
makes AI-assisted development easier to reproduce, review, and validate.

## Read in this order

1. [Bevy overview](bevy-overview.md): what the engine is and what the experiment can tell us.
2. [Setup guide](setup.md): prerequisites, current machine observations, and first-run steps.
3. [Game and experiment plan](plan.md): a deliberately small game, milestones, and evidence to collect.
4. [Checkpoint 2: grid map and movement](checkpoint-grid-movement.md): current code,
   new Rust/ECS concepts, test results, experiment record, and visual checklist.

## Current status

- Bevy 0.20.1 runs with a 2D camera, a fixed 20 × 15 tile map, and a colored
  player sprite.
- `GridPosition` stores integer tile coordinates as gameplay truth; the sprite
  transform is derived from it each time the grid position changes.
- `W`/`A`/`S`/`D` and the arrow keys move one cardinal cell per fresh key press.
  Walls and out-of-bounds positions block movement.
- Nine headless tests cover startup, WASD and arrow movement, wall blocking,
  boundary blocking, one-step-per-press behavior, and grid-to-render derivation.
- `cargo fmt --check`, `cargo test` (9 tests),
  `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo build`
  passed on the Apple Silicon development machine.
- `cargo run` selected the Apple M1 Metal renderer and created the window without
  a panic. The launch was stopped by the agent after eight seconds.
- Visual confirmation of the grid, movement, blocking, and normal window-close
  behavior is still pending.

Current checkpoint: human visual verification using the checklist in
[the grid movement checkpoint](checkpoint-grid-movement.md#human-visual-verification-checklist).

Next gameplay step after visual sign-off: add a wait action and one monster turn,
so each successful player action consumes exactly one turn (milestone 2).
