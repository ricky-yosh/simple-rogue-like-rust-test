# Goal: Complete a small turn-based dungeon roguelike

## Outcome

Extend the existing Bevy project into a complete, short, replayable dungeon run.

The player explores one handcrafted dungeon, fights enemies, earns XP,
chooses one of three upgrades on level-up, defeats a boss, and reaches an
exit to win. The player can lose and restart without relaunching the app.

This is also a learning project. Keep the implementation small, readable,
and explainable to a developer learning Rust and Bevy ECS.

## Start here

Before editing:

- Read repository instructions, the current source, and the docs.
- Inspect Cargo.toml and Cargo.lock; use the installed Bevy version.
- Preserve working movement and wall-blocking behavior.
- Review existing tests and run them to establish a baseline.
- Use the existing Kenney Tiny Dungeon pack in
  assets/third_party/kenney_tiny_dungeon/.
- Do not assume previous documentation accurately describes current code.

## Scope

### 1. Turn handling and combat

- Keep integer grid coordinates as authoritative gameplay state.
- Preserve WASD and arrow-key movement, one cell per fresh key press.
- Add a wait action using Space.
- Moving into an enemy performs a melee attack instead of moving.
- A successful move, an attack, or waiting consumes exactly one player turn.
- Moving into a wall or outside the map consumes no turn.
- After each player turn, each living regular enemy acts at most once.
- Enemies killed by the player do not act afterward.
- Stop processing further enemy actions if the player dies.
- Resolve enemy actions in a stable, explicit order.
- Prevent living actors from overlapping.
- Do not add diagonal movement.

Include two regular enemy types:

- A pursuer that approaches the player and attacks when adjacent.
- A slower, tougher enemy that acts every other player turn.

Use simple deterministic movement. Sophisticated pathfinding is not required,
but the handcrafted map must allow a playable run.

### 2. Health and XP

- Give the player and enemies health and attack damage.
- Display player HP numerically and with a health bar.
- Award XP exactly once when an enemy is defeated.
- Display level and XP progress.
- Define XP thresholds and combat values in a clearly named place.
- Prevent dead enemies from blocking tiles or awarding XP again.

### 3. Three-choice upgrades

- When XP reaches a level threshold, enter an upgrade-selection state.
- Display three distinct eligible upgrade choices.
- Support mouse selection and keyboard keys 1, 2, and 3.
- Explain each choice's actual effect in the UI.
- Apply exactly one upgrade per earned level.
- Do not allow dungeon actions or enemy turns while choosing.
- Choosing an upgrade does not itself consume a turn.
- If multiple levels are earned, resolve one selection per level.
- If combat ends the run, show the end screen instead of an upgrade panel.

Start with these three repeatable upgrades:

- Vitality: increase maximum HP and heal by the same amount.
- Strength: increase melee attack damage.
- Recovery: increase healing received after defeating an enemy.

Keep values modest and show the current numerical benefit on each card.
A larger randomized upgrade pool is outside this version's scope.

### 4. Boss and ending

- Add one clearly identifiable boss guarding an exit.
- Give the boss a visible health bar and one understandable special behavior,
  such as a telegraphed heavy attack with a recovery turn.
- Explain the boss's warning through visible text or a clear tile indicator.
- The warning must be visible before the player commits the affected action.
- Keep the boss's gameplay footprint one tile.
- Lock the exit until the boss is defeated.
- Entering the unlocked exit wins the run.
- Reaching zero player HP loses the run.
- Win and loss screens display the result and a restart action.
- Support restarting with a visible button and the R key.
- Restart resets the entire run, including upgrades, XP, enemies, boss state,
  pending actions, and UI.
- Restart must not duplicate cameras or persistent UI.

### 5. Presentation

- Keep the existing textured player.
- Use existing pack assets for floors, walls, enemies, boss, and exit.
- Inspect the artwork before selecting tiles; do not guess from filenames.
- Use nearest-neighbor filtering and consistent sizing for pixel art.
- Build health bars and upgrade panels with Bevy UI.
- Use readable text, adequate contrast, and a concise controls display.
- Show useful action feedback, such as damage, blocked movement, and XP gains.
- Static sprites are sufficient. Animation and sound are out of scope.

## Architecture constraints

- Use Bevy ECS intentionally: components for entity data, resources for shared
  state, systems for behavior, and explicit states for gameplay modes.
- Keep grid gameplay rules independent of rendering and UI.
- Make system ordering explicit wherever behavior depends on it.
- Extract small pure functions for rules that benefit from isolated tests.
- Use headless Bevy tests to verify systems and state transitions.
- Split code into a few cohesive modules when useful.
- Avoid speculative frameworks, generic ability engines, or excessive plugins.
- BSN is optional, not a requirement.
- Do not change the Rust toolchain or upgrade Bevy unless blocked; explain any
  necessary dependency change before proceeding.
- Do not download additional assets or dependencies unless necessary and approved.
- Preserve asset licenses.

## Explicit non-goals

Do not add:

- Real-time gameplay.
- Procedural generation or multiple floors.
- Inventory, equipment, shops, or crafting.
- Ranged combat or a general status-effect framework.
- Save/load, multiplayer, or physics.
- Sprite-frame animation, sound, or elaborate visual effects.
- A Unity implementation or a conclusion about engine superiority.

## Implementation checkpoints

Complete these in order:

1. Turn handling, waiting, regular enemies, combat, and player death.
2. Health/XP UI and three-choice level-up selection.
3. Boss, locked exit, win/loss screens, and complete restart.
4. Dungeon artwork, usability, balance pass, and learning documentation.

At each checkpoint:

- Add or update tests for observable behavior.
- Run formatting, tests, strict Clippy, and a build.
- Fix failures before proceeding.
- Record what changed, what was verified, and what remains unverified.
- Continue automatically when checks pass and no product decision is blocking.
- Do not describe a checkpoint as visually verified without actual evidence.

Stop and ask a specific question if a missing requirement or environmental
problem prevents a coherent implementation. Do not repeatedly try equivalent
failed approaches.

## Required automated coverage

At minimum, verify:

- Movement and wall/boundary blocking still work.
- A valid action advances one turn; blocked movement does not.
- Waiting advances a turn without moving the player.
- Bump attacks damage enemies without overlapping them.
- Enemies act at most once per applicable turn.
- The slow enemy follows its every-other-turn rule.
- Dead enemies do not act or award XP twice.
- Player death stops further gameplay.
- XP produces the correct number of level-up choices.
- Upgrade selection blocks gameplay and applies only one choice per level.
- Each upgrade changes its intended gameplay behavior.
- The boss telegraph and recovery follow the defined turn sequence.
- The exit remains locked until the boss dies.
- The unlocked exit produces a win.
- Restart restores initial state without duplicate entities or UI.
- A headless integration smoke test initializes the assembled gameplay systems
  and advances multiple updates without missing-resource or query-conflict errors.

Avoid tests that merely repeat implementation constants.
Headless tests do not establish that assets load or that UI looks correct.

## Final automated checks

Run these exact commands:

    cargo fmt --check
    cargo test
    cargo clippy --all-targets --all-features -- -D warnings
    cargo build

Report actual results. Never claim a command passed if it was not run.

## Manual acceptance checklist

Provide this checklist for the human playtest:

- The dungeon, player, enemies, boss, and exit render correctly.
- Movement, waiting, attacks, and enemy turns are understandable.
- HP, XP, and action feedback are readable.
- Level-up presents three choices and freezes dungeon actions.
- Mouse and keyboard upgrade selection work.
- The boss warning is visible and its behavior is understandable.
- A complete run can be won.
- A run can be lost.
- Restart works after either result and can be repeated.
- No missing-asset errors or panics appear in the terminal.
- Closing the window exits normally.

If direct interactive testing is unavailable, mark these items pending.
Do not block completion of the implementation report on unavailable human input.

## Learning documentation

Create a linked architecture walkthrough explaining:

1. The project/module structure.
2. Which data belongs to components, resources, and game states.
3. How a key press becomes a player action and then an enemy turn.
4. How damage becomes death, XP, and an upgrade choice.
5. How UI reads game state and sends a selection back into gameplay.
6. How the boss and restart flow work.
7. Why rendering positions are separate from grid coordinates.

Use concrete examples from the finished code.
Include repository-relative code links with line anchors.
Explain relevant Rust/Bevy concepts without assuming prior engine knowledge.

Keep a short experiment record covering task scope, model/tool access where
known, incremental build/test timings, failures, human interventions, and
visual results. Mark unknown measurements unknown.

## Definition of done

- All scoped features are implemented.
- Required automated checks pass.
- Documentation matches the implementation.
- The final report lists checks, limitations, and pending manual verification.
- No placeholder gameplay, abandoned experiments, or unrelated cleanup remains.
- No commits, pushes, or merges are performed without explicit authorization.

Stop when this definition is met. Do not expand the game indefinitely.

