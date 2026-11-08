# Architecture walkthrough

This is a complete, short, turn-based dungeon run built with Bevy 0.20.1. The
walkthrough follows one key press through movement, combat, XP, upgrades, the
boss, and restart. It also explains the Rust and ECS concepts used along the way.

The automated evidence is in [the experiment record](experiment-record.md); the
human checklist is in [the playtest checklist](playtest-checklist.md).

## 1. Project and module structure

| Path | Responsibility |
| --- | --- |
| [`src/main.rs`](../src/main.rs#L1-L31) | Creates the Bevy app, window, nearest-neighbor image filtering, and the three game plugins. |
| [`src/balance.rs`](../src/balance.rs#L1-L35) | Every combat, XP, and upgrade number in one named place. |
| [`src/grid.rs`](../src/grid.rs#L1-L172) | Integer coordinates, tile enum, the hand-authored map, and walkability. Contains no sprite or UI code. |
| [`src/gameplay/model.rs`](../src/gameplay/model.rs#L1-L254) | Components, resources, states, and small value types shared by gameplay. |
| [`src/gameplay/turn.rs`](../src/gameplay/turn.rs#L1-L402) | Input collection, player action resolution, enemy phase, boss turn, and post-turn checks. |
| [`src/gameplay/progress.rs`](../src/gameplay/progress.rs#L1-L280) | Damage rewards, XP thresholds, upgrade effects, and keyboard/mouse-independent upgrade application. |
| [`src/gameplay/restart.rs`](../src/gameplay/restart.rs#L1-L44) | The shared reset path for R and the restart button. |
| [`src/render.rs`](../src/render.rs#L1-L430) | Loads the Kenney tiles, spawns a run's visible entities, and derives transforms from grid positions. |
| [`src/ui.rs`](../src/ui.rs#L1-L727) | Persistent HUD, three-card upgrade panel, win/loss screens, and button observers. |
| [`src/gameplay/tests.rs`](../src/gameplay/tests.rs#L1-L1070) | Headless ECS tests for the assembled game. |
| [`assets/third_party/kenney_tiny_dungeon/`](../assets/third_party/kenney_tiny_dungeon/) | Unmodified CC0 Tiny Dungeon pack and its license. |

Dependencies point one way: `grid` and `balance` do not know about rendering;
`gameplay` knows grid rules but not UI; `render` and `ui` read gameplay state;
`main` assembles them. That keeps the rules testable without a window or GPU.

## 2. Components, resources, and states

### Components: data attached to one entity

| Component | Where | Why it is a component |
| --- | --- | --- |
| [`Player`](../src/gameplay/model.rs#L10-L11) | Player | Marks the one controllable actor. |
| [`GridPosition`](../src/grid.rs#L26-L40) | Player and enemies | Authoritative integer tile position. One value per actor. |
| [`Enemy`](../src/gameplay/model.rs#L64-L69) | Skeleton, brute, boss | Enemy kind plus a stable turn-order number. |
| [`Health`](../src/gameplay/model.rs#L72-L107) | Player and enemies | Current/max HP and shared damage/heal rules. |
| [`Attack`](../src/gameplay/model.rs#L109-L118) | Player and enemies | Melee damage value. |
| [`Recovery`](../src/gameplay/model.rs#L120-L130) | Player | Healing after a kill; upgraded independently. |
| [`PlayerProgress`](../src/gameplay/model.rs#L132-L149) | Player | Level, XP, and queued level-up choices. |
| [`BossPhase`](../src/gameplay/model.rs#L156-L164) | Boss | Ready, winding-up target, or recovering. |
| [`Exit`](../src/gameplay/model.rs#L150-L153) | Exit door | Locked/unlocked gameplay state. |
| [`RunEntity`](../src/gameplay/model.rs#L13-L15) | Every run entity | Restart sweeps these without touching camera or HUD. |

### Resources: one shared value

| Resource | Purpose |
| --- | --- |
| [`DungeonMap`](../src/grid.rs#L103-L127) | Immutable tile layout shared by movement, AI, rendering, and tests. |
| [`TurnCount`](../src/gameplay/model.rs#L187-L188) | Number of consumed player turns; also drives the brute's every-other-turn rule. |
| [`TurnFlow`](../src/gameplay/model.rs#L177-L180) | Transient flags for "enemy phase is due" and "turn just finished." |
| [`PendingAction`](../src/gameplay/model.rs#L173-L174) | The one action collected this frame. |
| [`Feedback`](../src/gameplay/model.rs#L191-L208) | Latest three HUD messages (damage, blocked movement, XP, boss warning). |
| [`UpgradeSelection`](../src/gameplay/model.rs#L240-L241) | Mouse-card intent consumed by the upgrade system. |
| [`RestartRequested`](../src/gameplay/model.rs#L244-L245) | Restart-button intent consumed by the restart system. |
| [`Art`](../src/render.rs#L50-L62) | Image handles loaded once from the asset pack. |

### States: mutually exclusive modes

[`GameState`](../src/gameplay/model.rs#L248-L254) has four variants:

- `Playing`: normal input, player actions, and enemy phases.
- `ChoosingUpgrade`: upgrade panel only; dungeon input and enemy phases are gated off.
- `Won` and `Lost`: end screens only; R and the restart button are available.

`GamePlugin` initializes the state and gates systems with `run_if` conditions. UI
panels use `OnEnter`/`OnExit`, so a panel cannot be duplicated by repeated state
checks. This is the ECS state-machine equivalent of explicit game modes.

## 3. From a key press to an enemy turn

### Input edge

Bevy exposes keyboard state through the [`ButtonInput<KeyCode>` resource]. The
game uses `just_pressed`, not `pressed`:

```rust
// src/gameplay/turn.rs
if keys.just_pressed(KeyCode::Space) {
    pending.0 = Some(PlayerAction::Wait);
    return;
}
```

[`collect_player_action`](../src/gameplay/turn.rs#L79-L97) walks a fixed table of
WASD/arrow keys and produces exactly one [`PendingAction`](../src/gameplay/model.rs#L166-L170)
per frame. `just_pressed` is true only when the key transitions from up to down,
so holding a key does not continuously move the player. If several keys are
pressed in the same frame, the table order deterministically picks one cardinal
direction; there is no diagonal movement.

### Player action

[`resolve_player_action`](../src/gameplay/turn.rs#L110-L205) is the only place a
player action changes the dungeon:

1. **Wait** clears feedback, adds "You wait and listen," and consumes a turn.
2. **Move into a wall or out of bounds** reports "Blocked by a wall" and returns
   without consuming a turn.
3. **Move into the locked exit** reports that the seal is intact and returns
   without a turn. The unlocked exit sets `GameState::Won`.
4. **Move into a living enemy** calls `enemy_health.take_damage(attack.damage)`
   instead of changing the player's position. The player and enemy never overlap.
5. **Move into a free floor tile** changes `GridPosition` and consumes a turn.

[`consume_turn`](../src/gameplay/turn.rs#L100-L104) increments `TurnCount` and
sets `TurnFlow::enemy_phase_due`. A blocked action does neither; an upgrade choice
also does neither.

### Enemy phase

[`resolve_enemy_phase`](../src/gameplay/turn.rs#L217-L303) only runs when
`enemy_phase_due` is set. It builds a `HashSet` of occupied tiles (player, exit,
and every living enemy), then collects `(Enemy::order, Entity)` pairs and sorts
them. That gives every enemy a stable, explicit order.

- A pursuer moves one deterministic cardinal step toward the player, or attacks
  when Manhattan distance is 1.
- A brute follows the same logic but skips odd `TurnCount`s, so it acts on every
  second player turn.
- The boss runs [`boss_turn`](../src/gameplay/turn.rs#L339-L373).
- After each enemy action the loop checks the player's health and breaks if the
  player is dead. Later enemies do not act after a killing blow.
- Dead enemies are skipped even if their despawn command has not flushed yet.

[`next_step_toward`](../src/gameplay/turn.rs#L318-L336) is a small pure function:
it prefers the axis with the larger distance, falls back to the other axis, and
rejects walls or occupied cells. It is tested without ECS.

### Finishing the turn

[`finish_turn`](../src/gameplay/turn.rs#L376-L397) runs after the enemy phase.
Death is checked before level-up: a dead player goes to `Lost`, even if the same
turn earned enough XP for a level. Otherwise a pending upgrade sends the game to
`ChoosingUpgrade`. Winning sets `Won` during action resolution and skips the
enemy phase.

The ordering is explicit in [`GamePlugin`](../src/gameplay/mod.rs#L35-L104):

```text
Input -> PlayerAction -> EnemyPhase -> PostTurn -> Upgrade -> Presentation -> Ui
```

`GameSet` is a Bevy `SystemSet`. The first four sets only run in `Playing`;
`Upgrade` only runs in `ChoosingUpgrade`; presentation and UI run in every state.
Within the gameplay sets the systems are also chained, so a reader does not have
to guess whether movement happens before enemy AI.

## 4. From damage to death, XP, and an upgrade

[`Health::take_damage`](../src/gameplay/model.rs#L87-L92) returns `true` only on
the transition from alive to zero. It cannot report a second kill for the same
entity. When a bump attack kills an enemy, the action system:

1. reports the kill,
2. calls [`grant_kill_rewards`](../src/gameplay/progress.rs#L48-L60),
3. queues `despawn` for the enemy,
4. unlocks the exit if the enemy was the boss,
5. consumes the turn.

`grant_kill_rewards` adds the enemy's XP through
[`award_xp`](../src/gameplay/progress.rs#L22-L43) and heals the player by
`Recovery::heal_on_kill`. The damage helper returning true only once means the XP
and heal path is entered exactly once per killed enemy; dead enemies also no
longer block the tile (they are despawned and filtered by positive health).

XP thresholds are cumulative and named in [`balance.rs`](../src/balance.rs#L28-L29):
level 2 at 8 XP, level 3 at 18, level 4 at 30. A single award can cross multiple
thresholds; `award_xp` increments `pending_upgrades` once per threshold. The level
cap is 4. This is why killing several enemies can queue two upgrade panels, one
selection per level.

The three upgrades are deliberately redundant and repeatable:

- [`Vitality`](../src/gameplay/progress.rs#L62-L80): max HP +4 and heal 4 now.
- `Strength`: melee damage +1.
- `Recovery`: heal-on-kill +1.

The numbers live in [`balance.rs`](../src/balance.rs#L31-L36). The UI's numeric
effect text is generated from the current stats by
[`upgrade_effect_text`](../src/gameplay/progress.rs#L83-L106), so cards show the
real next values rather than hard-coded prose.

## 5. How UI reads state and sends a selection back

### Persistent HUD

[`spawn_hud`](../src/ui.rs#L150-L305) runs once at startup and creates five
`HudRoot` panels: player stats, boss stats, feedback, turn count, and controls.
They are **not** run entities and are not respawned by restart.

Every frame, [`update_hud`](../src/ui.rs#L307-L421) queries gameplay components and
resources and writes UI data:

- player `Health` → numeric HP text and a `Node` width percentage;
- `PlayerProgress` → level and cumulative XP threshold;
- `Attack` and `Recovery` → stat line;
- boss `Health` and `BossPhase` → boss bar, HP text, and warning text;
- `Feedback` → the last three messages;
- `TurnCount` → turn counter.

The boss warning is the `BossPhase` text plus the red world tile. While
`BossPhase::WindingUp { .. }` is active, the HUD says "HEAVY ATTACK MARKED".
This state is set at the end of one player turn and remains visible throughout
the next player turn, before the heavy attack resolves.

### Upgrade panel

[`spawn_upgrade_panel`](../src/ui.rs#L430-L492) runs on `OnEnter(ChoosingUpgrade)`.
It builds one card for every `UpgradeKind::ALL` entry with
[`spawn_upgrade_card`](../src/ui.rs#L494-L543). Each card:

- is a Bevy `Button` widget with `Hovered`,
- displays a key number, title, exact effect text, and flavor text,
- has an `Activate` observer that writes `UpgradeSelection.0`.

Clicking a card therefore does not mutate gameplay directly; it records intent.
[`handle_upgrade_selection`](../src/gameplay/progress.rs#L124-L174) is the only
gameplay system that consumes that intent, and it also maps keys 1/2/3 to the
same three `UpgradeKind` values. It applies one upgrade, decrements the pending
counter, and only returns to `Playing` when the counter reaches zero. It never
increments `TurnCount` and never triggers an enemy phase. On `OnExit`, the panel
root is despawned recursively by [`despawn_marked`](../src/ui.rs#L723-L727).

The headless test
[`clicking_an_upgrade_card_sets_the_same_selection_resource`](../src/gameplay/tests.rs#L968-L1002)
triggers the card's `Activate` observer directly; actual pointer hit-testing is
part of the manual checklist.

### End screens and restart intent

[`spawn_victory_screen`](../src/ui.rs#L624-L630) and
[`spawn_defeat_screen`](../src/ui.rs#L632-L638) share
[`spawn_end_screen`](../src/ui.rs#L640-L721). They display the result, stats, a
restart button, and the R hint. The button observer writes `RestartRequested`;
[`restart_run`](../src/gameplay/restart.rs#L17-L49) also checks the R key, so both
paths perform the same reset exactly once.

## 6. Boss and restart flow

### Boss cycle

The boss is one entity at `BOSS_POSITION` (29, 8), directly west of the exit at
(30, 8). The exit alcove is walled on the north, south, and east sides, so the
boss tile is its only entrance. The boss's footprint is one tile even though its
sprite is drawn larger for readability.

[`BossPhase`](../src/gameplay/model.rs#L156-L164) is an explicit three-state
cycle:

| Player turn | Boss phase before enemy action | Result |
| --- | --- | --- |
| Any turn while `Ready` and the player is within Manhattan range 3 | `Ready` | Boss sets `WindingUp { target: player_tile }`; warning text and red tile appear; no damage. |
| Next player turn | `WindingUp` | If the player is still on the target, take 7 damage and the warning clears; if the player moved, the attack misses. Either way the boss enters `Recovering`. |
| Next player turn | `Recovering` | Boss returns to `Ready`; no damage. |

The warning is visible before the player commits the next action because the
phase is set after the previous player action and the HUD/world marker update in
the same frame. Moving off the marked tile is the intended counterplay. The boss
does not move, so it remains a one-tile obstacle guarding the exit.

The warning has two visible forms:

- [`update_hud`](../src/ui.rs#L307-L421) writes the red warning text.
- [`sync_telegraph`](../src/render.rs#L320-L339) positions and shows the red
  translucent marker at `BossPhase::WindingUp::target`.
- [`sync_boss_warning`](../src/render.rs#L342-L349) tints the boss sprite during
  the wind-up.

### Exit and win

[`Exit`](../src/gameplay/model.rs#L150-L153) starts locked. The action system
refuses a move into it and consumes no turn. When the boss dies, the action system
sets `locked = false`; [`sync_exit_visual`](../src/render.rs#L310-L318) swaps the
closed-door texture for the open-door texture. Moving into the unlocked exit sets
`GameState::Won`. The tests
[`boss_death_unlocks_exit_and_stepping_into_it_wins`](../src/gameplay/tests.rs#L697-L715)
and
[`locked_exit_blocks_without_a_turn_and_open_exit_wins`](../src/gameplay/tests.rs#L717-L739)
cover both sides.

### Restart

[`restart_run`](../src/gameplay/restart.rs#L17-L49) runs only in `Won` or `Lost`.
It:

1. consumes R or the button intent,
2. despawns every entity with `RunEntity` (tiles, exit, player, enemies, boss,
   telegraph),
3. resets feedback, turn count, turn flow, pending action, and upgrade selection,
4. sets `GameState::Playing`.

`OnExit(Won/Lost)` removes the end screen. `OnEnter(Playing)` calls
[`spawn_run_if_missing`](../src/render.rs#L132-L155), which does nothing if a
player already exists (resuming after an upgrade) and otherwise rebuilds the
whole run. The camera and HUD are not `RunEntity`, so restart cannot duplicate
them. The test
[`restart_resets_everything_without_duplicating_camera_or_hud`](../src/gameplay/tests.rs#L855-L948)
performs two restarts, one through the button observer and one through R.

## 7. Why rendering positions are separate from grid coordinates

Gameplay stores [`GridPosition`](../src/grid.rs#L26-L58) with integer `x`/`y`.
A `Transform` is only a presentation detail. The one conversion is
[`grid_to_world`](../src/render.rs#L287-L297):

```text
world_x = (grid_x - (map_width  - 1) / 2) * 32
world_y = (grid_y - (map_height - 1) / 2) * 32
```

The map is centered on the window. [`sync_grid_transforms`](../src/render.rs#L299-L308)
copies an updated grid position into the actor's transform when the component
changes; no gameplay system ever reads a transform to determine walkability or
occupancy. That keeps the rules deterministic and lets the headless tests inspect
positions without a renderer.

Map strings are written north-to-south for readability, while grid `y` grows
north. [`DungeonMap::tile_at`](../src/grid.rs#L155-L168) performs the single
`height - 1 - y` row flip. Pixel art uses `ImagePlugin::default_nearest()` in
[`main`](../src/main.rs#L15-L30); tiles are 32 world units and actor art is 32
units (boss 48), preserving integer pixel scaling.

## Rust concepts used

- **Derive macros** such as `#[derive(Component)]`, `#[derive(Resource)]`, and
  `#[derive(States)]` register plain Rust data with Bevy's ECS. The struct itself
  remains ordinary data; the macro supplies the ECS plumbing.
- **Enums with data** model small closed sets: `EnemyKind`, `BossPhase`,
  `PlayerAction`, `UpgradeKind`, and `GameState`. `match` must handle every
  variant, so adding a boss phase or upgrade produces compiler errors at every
  site that needs updating instead of a silent fallback.
- **`Copy` value types** such as `GridPosition`, `Health`, and `BossPhase` are
  cheap to copy. They are passed by value into pure rules and read through query
  references without ownership ceremony.
- **`Option` and `let ... else`** represent "there is no action / no target / no
  boss" without sentinel values. Examples are `PendingAction(Option<PlayerAction>)`
  and the `let Some(delta) = ... else { return; }` in action resolution.
- **`Result` from `Query::single_mut`** distinguishes "no match" from "more than
  one match." The game treats both as an invalid run state and returns early.
- **Iterators and `HashSet`** keep rules compact: enemy lookup uses `.find`,
  ordered turn collection uses `.map`, `.filter`, and `sort_by_key`, and
  occupancy checks use a `HashSet<GridPosition>`.
- **Borrowing and disjoint filters**: Bevy needs to prove two queries do not
  mutate the same component. Player and enemy queries use `With<Player>`,
  `With<Enemy>`, and `Without` filters so the borrow checker and Bevy's runtime
  query validation see them as disjoint. This is why `update_hud` explicitly
  separates player/boss queries and HUD bar/boss-panel queries.
- **Type aliases** such as `PlayerActionQuery` and `ChangedPlayerTransforms` keep
  Bevy's long `Query<data, filter>` types readable and satisfy Clippy's
  `type_complexity` lint.

## Where to look in the tests

| Requirement | Representative test |
| --- | --- |
| Movement and wall blocking | [`valid_move_advances_exactly_one_turn`](../src/gameplay/tests.rs#L249-L262), [`wall_blocks_movement_without_consuming_a_turn`](../src/gameplay/tests.rs#L264-L281) |
| Wait and turn boundary | [`wait_advances_a_turn_without_moving`](../src/gameplay/tests.rs#L283-L293) |
| Bump attack / no overlap | [`bump_attack_damages_without_overlapping`](../src/gameplay/tests.rs#L355-L394) |
| Enemy once per turn | [`each_living_enemy_acts_at_most_once_per_turn`](../src/gameplay/tests.rs#L423-L448) |
| Slow brute parity | [`brute_acts_every_other_player_turn`](../src/gameplay/tests.rs#L327-L353) |
| Death stops gameplay | [`player_death_stops_later_enemy_actions_and_the_run`](../src/gameplay/tests.rs#L452-L493) |
| XP queues choices | [`xp_thresholds_queue_one_choice_per_level`](../src/gameplay/progress.rs#L169-L198), [`xp_triggers_upgrade_state_and_selection_does_not_consume_a_turn`](../src/gameplay/tests.rs#L495-L559) |
| Upgrade state freezes play / one per level | [`multiple_levels_require_one_selection_each`](../src/gameplay/tests.rs#L545-L579), [`upgrade_panel_has_three_cards_and_freezes_the_dungeon`](../src/gameplay/tests.rs#L936-L966) |
| Upgrade behavior | [`vitality_raises_max_and_heals`](../src/gameplay/progress.rs#L222-L240), [`strength_raises_melee_damage`](../src/gameplay/progress.rs#L242-L258), [`recovery_raises_heal_on_kill`](../src/gameplay/progress.rs#L260-L276) |
| Boss telegraph/heavy/recovery | [`boss_telegraphs_then_hits_then_recovers`](../src/gameplay/tests.rs#L642-L674), [`boss_heavy_attack_can_be_dodged_after_the_warning`](../src/gameplay/tests.rs#L675-L695) |
| Locked/unlocked exit | [`boss_death_unlocks_exit_and_stepping_into_it_wins`](../src/gameplay/tests.rs#L697-L715), [`locked_exit_blocks_without_a_turn_and_open_exit_wins`](../src/gameplay/tests.rs#L717-L739) |
| Restart / no duplicates | [`restart_resets_everything_without_duplicating_camera_or_hud`](../src/gameplay/tests.rs#L855-L948) |
| HUD reads gameplay state | [`hud_reads_health_level_xp_and_boss_warning_from_game_state`](../src/gameplay/tests.rs#L1004-L1050) |
| Assembled smoke test | [`headless_smoke_advances_assembled_game_systems`](../src/gameplay/tests.rs#L1058-L1073) |
| Reachable playable map | [`all_enemies_boss_and_exit_are_reachable_from_the_player`](../src/render.rs#L404-L429) |
| All art files exist | [`all_selected_art_files_exist`](../src/render.rs#L359-L383) |

## Known limitations

- The headless tests prove rules and entity wiring, not pixel output, text layout,
  pointer hit-testing, or font rendering. The manual checklist is still required.
- The game is intentionally one fixed floor with three upgrade types, one boss,
  static sprites, and no sound/animation. Those are scoped omissions, not missing
  placeholders.
- The window launch was checked for startup and missing assets, but no screenshot
  or interactive playtest was available in this environment.
