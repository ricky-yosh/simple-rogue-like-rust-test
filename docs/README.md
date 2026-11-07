# Bevy learning experiment

## Goal

Learn Bevy by building a small game, while testing whether its code-first workflow
makes AI-assisted development easier to reproduce, review, and validate.

## Read in this order

1. [Bevy overview](bevy-overview.md): what the engine is and what the experiment can tell us.
2. [Setup guide](setup.md): prerequisites, current machine observations, and first-run steps.
3. [Game and experiment plan](plan.md): a deliberately small game, milestones, and evidence to collect.

## Current status

- Official Bevy introduction, setup, ECS documentation, and latest release were researched.
- The current machine's Rust and Apple development tools were inspected.
- A Rust application now defines a Bevy window, 2D camera, and colored player sprite.
- A headless startup test checks that the camera and visible player spawn only once.
- `cargo fmt --check`, `cargo test` (one test), `cargo clippy --all-targets --all-features -- -D warnings`,
  and `cargo build` passed on the Apple Silicon development machine.
- `cargo run` selected the Apple M1 Metal renderer and logged window creation without
  reporting a panic. The launch was stopped by a two-minute terminal timeout.
- Visual confirmation of the square and normal window-close behavior are still pending;
  the headless test and launch logs do not prove either.
- The full dungeon game design remains a recommendation, not a finalized requirement.

Current checkpoint: confirm the first scene visually and close the window normally.
Next gameplay step: keyboard movement, followed by a grid and walls.
