# Experiment record: complete dungeon run

## Task

Extend the existing Bevy movement checkpoint into a short, replayable,
turn-based dungeon game:

- fixed 32 × 18 hand-authored dungeon;
- WASD/arrow movement and Space wait;
- bump combat, two regular enemy types, stable enemy order;
- HP, damage, XP, three repeatable upgrades, mouse/keyboard selection;
- one boss with a telegraphed heavy attack and recovery turn;
- locked exit, win/loss screens, mouse/R restart;
- Kenney Tiny Dungeon artwork throughout;
- headless tests, strict checks, and learning documentation.

Explicitly out of scope and not added: procedural generation, inventory,
ranged combat, animation, sound, save/load, multiplayer, and commits/pushes.

## Environment

| Item | Value |
| --- | --- |
| Date | 2026-11-07 |
| Machine | Apple M1, 16 GiB, macOS 26.6.2 Tahoe |
| Rust | `rustc 1.99.1`, `cargo 1.99.1` |
| Engine | Bevy 0.20.1, unchanged from the baseline |
| Agent/harness | OpenCode, DeepSeek V4.1 Flash |
| Assets | Existing Kenney Tiny Dungeon pack, CC0, unmodified |
| Human input | User supplied the goal and the asset pack; no manual code edits during this pass |

## Baseline

Before editing, the existing project had movement and wall blocking plus the
player-art checkpoint. `cargo test` passed **9 tests**. `Cargo.toml` already
declared Bevy 0.20.1 and Rust 1.99.1; no dependency or toolchain change was made.

## What was built

- [`src/balance.rs`](../src/balance.rs): all combat, XP, and upgrade values.
- [`src/grid.rs`](../src/grid.rs): integer coordinates, map parsing, walkability,
  the reachable hand-authored map, and objective tiles.
- [`src/gameplay/`](../src/gameplay): components/resources/states, turn
  resolution, enemy phase, boss cycle, XP/upgrades, restart, and tests.
- [`src/render.rs`](../src/render.rs): asset handles, tile/actor spawning,
  grid-to-world conversion, transform sync, exit/telegraph/boss visuals.
- [`src/ui.rs`](../src/ui.rs): persistent HUD, upgrade cards, end screens, and
  pointer observers for upgrades/restart.
- [`src/main.rs`](../src/main.rs): 1280 × 800 window, nearest filtering, plugins.
- Documentation: [architecture walkthrough](architecture-walkthrough.md),
  [playtest checklist](playtest-checklist.md), updated README and docs index.

## Art selection

The pack was inspected rather than guessed from filenames. A labeled contact
sheet was generated from the 132 individual 16 × 16 tiles and reviewed visually.
The finished mapping is:

| Purpose | Tile |
| --- | --- |
| Player (preserves the earlier choice) | `tile_0085.png` adventurer |
| Pursuing enemy | `tile_0109.png` skeleton |
| Slow brute | `tile_0096.png` armored brute |
| Boss | `tile_0112.png` green Warden, drawn at 48 px |
| Floor variants | `tile_0048`, `tile_0049`, `tile_0051` |
| Wall | `tile_0040` brick |
| Locked exit | `tile_0045` closed door |
| Open exit | `tile_0047` open door |

No extra downloads or dependencies were added.

## Automated results

Final commands were run from the repository root after the last code change:

| Command | Result | Observed time |
| --- | --- | --- |
| `cargo fmt --check` | Passed, no output | 0.10 s |
| `cargo test` | **47 passed, 0 failed, 0 ignored** | 0.56 s cached |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed | 1.66 s |
| `cargo build` | Passed | 7.91 s |

Important earlier measurements:

- First test-profile compile after adding the modules: about 3.55 s.
- Later test compiles after adding the integration suite: about 5.96–10.22 s.
- The full Bevy dependency build was already cached from the earlier checkpoints;
  a clean initial engine build was not measured again.
- The exact total wall-clock task time was not captured.

The final test list includes movement, blocking, waiting, bump combat, enemy
ordering, brute parity, player death, XP, upgrade selection, every upgrade effect,
boss telegraph/heavy/recovery, exit lock/win, HUD state reading, restart, and an
assembled smoke test. See the test table in the
[architecture walkthrough](architecture-walkthrough.md#where-to-look-in-the-tests).

## Iterations and failures

1. **Private systems / API mismatches on first compile.** Child modules' turn
   systems needed `pub(super)`; `Art` could not implement both `Default` and
   `FromWorld`; Bevy 0.20 has no `in_state(...).or(...)`; `is_adjacent` could not
   stay `const`. Fixed by visibility changes, `Art::default_handles`, two state
   run conditions, and a normal function.
2. **Clippy `-D warnings` pass.** Initial failures were `type_complexity`,
   `too_many_arguments`, and `manual_is_multiple_of`. Fixed with query type
   aliases, a documented allow for Bevy system parameter counts, and
   `is_multiple_of`.
3. **Test harness repeated-key bug.** The first headless input helper cleared
   only `just_pressed`; the key stayed in `ButtonInput::pressed`, so a second tap
   of the same key produced no new edge. Fixed by also releasing the key after
   each update, matching real input.
4. **Bevy B0001 query conflict.** `update_hud` had two `&mut Node` queries and
   overlapping `Health` queries. Fixed with disjoint filters
   (`Without<BossHudPanel>` and `Without<Player>`). The assembled startup test
   then passed.
5. **Stale binary / asset root.** Running `./target/debug/...` directly without
   `CARGO_MANIFEST_DIR`/`BEVY_ASSET_ROOT` makes Bevy look under
   `target/debug/assets`; the first runtime check used a stale binary. The actual
   `cargo run` path was rebuilt and used for the final launch. This is documented
   so future manual runs use `cargo run` as the README says.
6. **Apple Events not authorized.** An attempt to query the window bounds or send
   keystrokes through System Events failed with `-1743`. No screenshot or
   automated input was possible, so visual verification remains pending rather
   than being claimed.

## Runtime check

`cargo run` was launched and stopped after about 12 seconds by the agent. The
log showed:

```text
SystemInfo { os: "macOS 26.6.2 Tahoe", ... cpu: "Apple M1", ... }
AdapterInfo { name: "Apple M1", ... backend: Metal, ... }
Creating new window Dungeon escape
```

No panic and no asset-loading errors appeared. A separate direct binary launch
with `BEVY_ASSET_ROOT="$PWD"` also produced no asset errors. This proves startup
and asset paths, not visual correctness.

## Human intervention and visual results

- No manual code edits were required during implementation.
- No interactive playtest or screenshot was available.
- Visual checks are **pending** and listed in the
  [playtest checklist](playtest-checklist.md).
- Headless tests cannot prove font rendering, UI layout, pointer hit-testing, or
  that the chosen sprites look correct in the window.

## Repository state

The work remains in the Jujutsu working copy. No commit, push, merge, toolchain
change, or dependency change was made.
