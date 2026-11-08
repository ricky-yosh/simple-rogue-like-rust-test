# Bevy learning experiment

## Goal

Learn Bevy by building a small game, while testing whether its code-first workflow
makes AI-assisted development easier to reproduce, review, and validate.

## Read in this order

1. [Architecture walkthrough](architecture-walkthrough.md): finished module
   structure, components/resources/states, key-press-to-enemy-turn flow,
   XP/upgrades, UI wiring, boss/restart, and grid-vs-render separation.
2. [Experiment record](experiment-record.md): task scope, environment, timings,
   failures, automated results, and what remains visually unverified.
3. [Manual playtest checklist](playtest-checklist.md): the remaining human checks.
4. [Setup guide](setup.md): prerequisites and machine observations.
5. [Game and experiment plan](plan.md): the original scope and milestone history.
6. [Bevy overview](bevy-overview.md): engine and ECS background.

## Current status

- Bevy 0.20.1 runs a complete short run: fixed 32 × 18 dungeon, textured player,
  two regular enemy types, a telegraphed boss, a locked exit, XP, three repeatable
  upgrades, win/loss screens, and full restart.
- Integer `GridPosition` values are gameplay truth; every sprite transform is
  derived from them.
- `WASD`/arrows move, Space waits, bumping attacks, `1`/`2`/`3` or mouse select
  upgrades, and `R`/the button restarts after a result.
- **47 headless tests** cover movement, blocking, turns, combat, enemy order,
  brute parity, death, XP, upgrade freezing/effects, boss phases, exit locks,
  HUD state reading, restart, map reachability, asset paths, and an assembled
  smoke test.
- Final `cargo fmt --check`, `cargo test`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo build`
  passed on the Apple M1 development machine.
- `cargo run` created the window with the Metal renderer and reported no
  asset-loading errors. It was stopped by the agent after about 12 seconds.
- Visual layout, pointer hit-testing, font rendering, and a complete manual run
  are **still pending**. See the playtest checklist.

Current checkpoint: human playtest using
[the manual checklist](playtest-checklist.md). No product decision is blocking
the automated implementation.

## Historical checkpoints

These explain earlier stages and their line links refer to the source as it was
then; use the architecture walkthrough for current code:

- [Grid map and movement](checkpoint-grid-movement.md)
- [Player art](checkpoint-player-art.md)
