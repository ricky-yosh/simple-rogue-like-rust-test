# A small game and a measurable experiment

## Recommended game: one-floor dungeon escape

Build a **2D, top-down, turn-based roguelike**. Move through a small grid dungeon,
fight a few monsters by walking into them, and reach an exit before health runs out.
A run should take only a few minutes.

This matches the repository's intent while exercising rendering, input, ECS,
UI, state transitions, and eventually procedural generation. Turn-based rules
make failures easier to reproduce than real-time physics or combat.

### First playable scope

- One hand-authored map with walls, floor, a player, monsters, and an exit.
- Arrow keys or WASD move one tile; a wait action lets the player skip a turn.
- Successful movement, attacking, or waiting consumes a turn; walking into a wall does not.
- Monsters act once after each consumed player turn.
- Bump-to-attack combat with simple health and damage.
- Health display, win/loss message, and restart.
- Colored shapes first; no external art required.

Leave inventory, equipment, multiple floors, multiplayer, complex pathfinding,
save files, and an editor out of the first playable. Procedural generation comes
after the fixed-map game works, so generation bugs do not hide gameplay bugs.

## Milestones

| Step | Deliverable | Evidence it works |
| --- | --- | --- |
| 0. Launch | Cargo project, Bevy window, camera, colored player | Launch and close on the target machine |
| 1. Explore | Fixed grid map, keyboard movement, wall blocking | Playtest; automated movement/bounds tests |
| 2. Take turns | Wait action, one enemy type, bump combat | Test that each valid action causes exactly one enemy phase |
| 3. Finish a run | Health UI, exit, death, restart | Win and lose manually; test terminal states and clean restart |
| 4. Vary runs | Seeded dungeon layout and spawn placement | Same seed reproduces layout; test reachable exit and valid spawns |
| 5. Add polish | Simple animation, sound, clearer UI | Screenshots and playtest, not just compilation |

Stop at step 3 for the first complete game. Steps 4–5 are follow-up experiments,
not prerequisites for declaring the first playable successful.

**Status:** step 1 is implemented and covered by headless tests; human visual
verification is pending. See
[the grid movement checkpoint](checkpoint-grid-movement.md). Steps 2–5 have not
started.

## Implementation approach

- Use integer grid coordinates as gameplay truth; derive rendered positions from
  them. Do not use sprite transforms to decide whether a tile is walkable.
- Keep movement, damage, and map-validation rules testable without rendering.
  Use small Bevy ECS tests where system integration is what matters.
- Explicitly order input, player action resolution, enemy action resolution, and
  presentation. Avoid relying on incidental system execution order.
- Start with a small application; split into gameplay-focused modules/plugins
  when responsibilities become distinct. Do not build an engine framework first.
- Keep this documentation current with controls, run commands, and decisions.

## How to test the AI-development thesis

Treat this game as a pilot study, not proof of superiority over other engines.
Measure both successful results and the effort needed to get them.

For each milestone, record:

| Field | What to record |
| --- | --- |
| Task | Prompt, scope, and acceptance criteria decided before implementation |
| Environment | Engine/toolchain version, machine, model, and available agent tools |
| Time | Separate initial build time, incremental compile time, and total task time |
| Iterations | Agent attempts, compiler failures, runtime bugs, and human corrections |
| Human intervention | Manual code edits, editor actions, setup work, and playtesting |
| Outcome | Tests passed, visual evidence, remaining defects, and whether it is playable |

Use the same short loop: define one change → let the agent implement it → inspect
the diff → run checks → playtest → record results. Keep experiments small enough
that a failure has a clear cause.

Once code exists, use `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
and `cargo test` as the routine automated checks, plus `cargo run` for the graphical
workflow. Record what was actually run; a build or test pass is not a playtest pass.

For a later Unity comparison, implement the same small specification with the
same model, comparable tool access and time budget, and identical acceptance
criteria. Include Unity's scripting/editor automation and tests. Separate
one-time setup from feature work, account for Rust/C# experience, and repeat tasks.
One engine's pilot alone cannot establish comparative productivity.

## Suggested next session

First complete the human visual checklist for step 1: launch the game, verify the
grid, movement, blocking, and normal window close, and record the result. After
that, add the wait action and the first monster turn so that each successful
player action consumes exactly one turn. Keep the grid and movement rules intact
and add tests for the turn boundary before adding combat.
