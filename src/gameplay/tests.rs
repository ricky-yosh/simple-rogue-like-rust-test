//! Headless ECS tests for turns, combat, progression, boss flow, and restart.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui_widgets::Activate;

use crate::balance;
use crate::grid::{BOSS_POSITION, DungeonMap, EXIT_POSITION, GridPosition, PLAYER_START, Tile};
use crate::render::{
    Art, BOSS_SIZE, ENEMY_LAYOUT, EXIT_LOCKED_TEXTURE, EXIT_OPEN_TEXTURE, PLAYER_TEXTURE,
    RenderPlugin, TelegraphMarker, grid_to_world,
};
use crate::ui::{
    EndScreenRoot, HudBar, HudRoot, HudText, RestartButton, UiPlugin, UpgradeChoice,
    UpgradePanelRoot,
};

use super::*;

fn gameplay_app() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.add_plugins(GamePlugin);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app
}

fn full_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin::default(),
        StatesPlugin,
    ));
    app.init_asset::<Image>();
    app.add_plugins((GamePlugin, RenderPlugin, UiPlugin));
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app
}

fn spawn_player(app: &mut App, position: GridPosition) -> Entity {
    app.world_mut()
        .spawn((
            RunEntity,
            Player,
            position,
            Health::new(balance::PLAYER_START_MAX_HP),
            Attack::new(balance::PLAYER_START_ATTACK),
            Recovery::new(balance::PLAYER_START_HEAL_ON_KILL),
            PlayerProgress::default(),
        ))
        .id()
}

fn spawn_player_with(app: &mut App, position: GridPosition, max_hp: i32, attack: i32) -> Entity {
    app.world_mut()
        .spawn((
            RunEntity,
            Player,
            position,
            Health::new(max_hp),
            Attack::new(attack),
            Recovery::new(balance::PLAYER_START_HEAL_ON_KILL),
            PlayerProgress::default(),
        ))
        .id()
}

fn spawn_enemy(
    app: &mut App,
    kind: EnemyKind,
    position: GridPosition,
    order: u32,
    health: Option<Health>,
) -> Entity {
    let mut entity = app.world_mut().spawn((
        RunEntity,
        Enemy { kind, order },
        position,
        health.unwrap_or(Health::new(kind.max_hp())),
        Attack::new(kind.attack()),
    ));
    if kind == EnemyKind::Boss {
        entity.insert((Boss, BossPhase::Ready));
    }
    entity.id()
}

fn spawn_exit(app: &mut App, locked: bool) -> Entity {
    app.world_mut().spawn((RunEntity, Exit { locked })).id()
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    // Match InputPlugin's frame edges without pulling in window/render plugins.
    // Releasing also lets a later tap of the same key produce a fresh press.
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.release(key);
    input.clear_just_pressed(key);
    input.clear_just_released(key);
}

fn settle(app: &mut App) {
    app.update();
}

fn state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

fn set_state(app: &mut App, next: GameState) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(next);
    app.update();
}

fn turn_count(app: &App) -> u32 {
    app.world().resource::<TurnCount>().0
}

fn player_position(app: &mut App) -> GridPosition {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&GridPosition, With<Player>>();
    *query.single(world).expect("exactly one player position")
}

fn set_player_position(app: &mut App, position: GridPosition) {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&mut GridPosition, With<Player>>();
    *query
        .single_mut(world)
        .expect("exactly one player position") = position;
}

fn player_health(app: &mut App) -> Health {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Health, With<Player>>();
    *query.single(world).expect("exactly one player health")
}

fn set_player_health(app: &mut App, health: Health) {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&mut Health, With<Player>>();
    *query.single_mut(world).expect("exactly one player health") = health;
}

fn player_progress(app: &mut App) -> PlayerProgress {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&PlayerProgress, With<Player>>();
    *query.single(world).expect("exactly one player progress")
}

fn set_player_progress(app: &mut App, progress: PlayerProgress) {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&mut PlayerProgress, With<Player>>();
    *query
        .single_mut(world)
        .expect("exactly one player progress") = progress;
}

fn player_attack(app: &mut App) -> i32 {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Attack, With<Player>>();
    query
        .single(world)
        .expect("exactly one player attack")
        .damage
}

fn player_recovery(app: &mut App) -> Recovery {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Recovery, With<Player>>();
    *query.single(world).expect("exactly one player recovery")
}

fn enemy_health_of(app: &mut App, kind: EnemyKind) -> Option<i32> {
    let world = app.world_mut();
    let mut query = world.query::<(&Enemy, &Health)>();
    query
        .iter(world)
        .find(|(enemy, _)| enemy.kind == kind)
        .map(|(_, health)| health.current)
}

fn enemy_position_of(app: &mut App, kind: EnemyKind) -> Option<GridPosition> {
    let world = app.world_mut();
    let mut query = world.query::<(&Enemy, &GridPosition)>();
    query
        .iter(world)
        .find(|(enemy, _)| enemy.kind == kind)
        .map(|(_, position)| *position)
}

fn enemy_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<Enemy>>();
    query.iter(world).count()
}

fn regular_enemy_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, (With<Enemy>, Without<Boss>)>();
    query.iter(world).count()
}

fn exit_locked(app: &mut App) -> Option<bool> {
    let world = app.world_mut();
    let mut query = world.query::<&Exit>();
    query.iter(world).next().map(|exit| exit.locked)
}

fn boss_phase(app: &mut App) -> Option<BossPhase> {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&BossPhase, With<Boss>>();
    query.single(world).ok().copied()
}

fn hud_text(app: &mut App, kind: HudText) -> String {
    let world = app.world_mut();
    let mut query = world.query::<(&HudText, &Text)>();
    query
        .iter(world)
        .find(|(text_kind, _)| **text_kind == kind)
        .map(|(_, text)| text.0.clone())
        .expect("matching HUD text")
}

fn hud_bar_width(app: &mut App, kind: HudBar) -> Val {
    let world = app.world_mut();
    let mut query = world.query::<(&HudBar, &Node)>();
    query
        .iter(world)
        .find(|(bar_kind, _)| **bar_kind == kind)
        .map(|(_, node)| node.width)
        .expect("matching HUD bar")
}

fn count<T: Component>(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<T>>();
    query.iter(world).count()
}

#[test]
fn valid_move_advances_exactly_one_turn() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(2, 5));
    app.update();

    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), GridPosition::new(3, 5));
    assert_eq!(turn_count(&app), 1);

    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), GridPosition::new(4, 5));
    assert_eq!(turn_count(&app), 2);
}

#[test]
fn wall_blocks_movement_without_consuming_a_turn() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(1, 5));
    app.update();

    tap(&mut app, KeyCode::KeyA);

    assert_eq!(player_position(&mut app), GridPosition::new(1, 5));
    assert_eq!(turn_count(&app), 0);
    assert!(
        app.world()
            .resource::<Feedback>()
            .lines
            .iter()
            .any(|line| line.contains("Blocked"))
    );
}

#[test]
fn wait_advances_a_turn_without_moving() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(3, 3));
    app.update();

    tap(&mut app, KeyCode::Space);

    assert_eq!(player_position(&mut app), GridPosition::new(3, 3));
    assert_eq!(turn_count(&app), 1);
}

#[test]
fn pursuer_moves_then_attacks_once_per_turn() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(15, 8));
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(17, 8),
        1,
        None,
    );
    app.update();

    tap(&mut app, KeyCode::Space);
    assert_eq!(
        enemy_position_of(&mut app, EnemyKind::Pursuer),
        Some(GridPosition::new(16, 8))
    );
    assert_eq!(player_health(&mut app).current, 24);

    tap(&mut app, KeyCode::Space);
    assert_eq!(
        enemy_position_of(&mut app, EnemyKind::Pursuer),
        Some(GridPosition::new(16, 8))
    );
    assert_eq!(
        player_health(&mut app).current,
        22,
        "one adjacent pursuer should attack exactly once"
    );
}

#[test]
fn brute_acts_every_other_player_turn() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(15, 8));
    spawn_enemy(
        &mut app,
        EnemyKind::Brute,
        GridPosition::new(18, 8),
        1,
        None,
    );
    app.update();

    tap(&mut app, KeyCode::Space);
    assert_eq!(
        enemy_position_of(&mut app, EnemyKind::Brute),
        Some(GridPosition::new(18, 8)),
        "the brute skips the first player turn"
    );

    tap(&mut app, KeyCode::Space);
    assert_eq!(
        enemy_position_of(&mut app, EnemyKind::Brute),
        Some(GridPosition::new(17, 8)),
        "the brute acts on the second player turn"
    );
}

#[test]
fn bump_attack_damages_without_overlapping() {
    let mut app = gameplay_app();
    let player = spawn_player(&mut app, GridPosition::new(16, 8));
    let skeleton = spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(17, 8),
        1,
        None,
    );
    app.update();

    tap(&mut app, KeyCode::KeyD);

    assert_eq!(player_position(&mut app), GridPosition::new(16, 8));
    assert_eq!(enemy_health_of(&mut app, EnemyKind::Pursuer), Some(4));
    assert_eq!(turn_count(&app), 1);
    assert_ne!(
        player_position(&mut app),
        enemy_position_of(&mut app, EnemyKind::Pursuer).unwrap()
    );
    assert_eq!(player_health(&mut app).current, 22);

    let world = app.world();
    assert!(world.get::<Player>(player).is_some());
    assert!(world.get::<Enemy>(skeleton).is_some());
}

#[test]
fn kill_awards_xp_heals_once_and_removes_the_blocker() {
    let mut app = gameplay_app();
    spawn_player_with(&mut app, GridPosition::new(16, 8), 24, 4);
    let mut player_health_value = Health::new(24);
    player_health_value.current = 20;
    set_player_health(&mut app, player_health_value);
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(17, 8),
        1,
        Some(Health::new(4)),
    );
    app.update();

    tap(&mut app, KeyCode::KeyD);

    assert_eq!(enemy_health_of(&mut app, EnemyKind::Pursuer), None);
    assert_eq!(enemy_count(&mut app), 0);
    assert_eq!(player_progress(&mut app).xp, balance::SKELETON_XP);
    assert_eq!(player_health(&mut app).current, 22);
    assert_eq!(turn_count(&app), 1);

    // The dead enemy no longer blocks the tile.
    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), GridPosition::new(17, 8));

    let xp_after_kill = player_progress(&mut app).xp;
    tap(&mut app, KeyCode::Space);
    tap(&mut app, KeyCode::Space);
    assert_eq!(
        player_progress(&mut app).xp,
        xp_after_kill,
        "XP must be awarded exactly once"
    );
    assert_eq!(player_health(&mut app).current, 22);
}

#[test]
fn each_living_enemy_acts_at_most_once_per_turn() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(16, 8));
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(17, 8),
        1,
        None,
    );
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(15, 8),
        2,
        None,
    );
    app.update();

    tap(&mut app, KeyCode::Space);

    assert_eq!(
        player_health(&mut app).current,
        20,
        "two adjacent pursuers should each attack once, not repeatedly"
    );
}

#[test]
fn player_death_stops_later_enemy_actions_and_the_run() {
    let mut app = gameplay_app();
    spawn_player_with(&mut app, GridPosition::new(16, 8), 2, 4);
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(17, 8),
        1,
        None,
    );
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(16, 10),
        2,
        None,
    );
    app.update();

    tap(&mut app, KeyCode::Space);
    settle(&mut app);

    assert_eq!(player_health(&mut app).current, 0);
    assert_eq!(
        enemy_position_of(&mut app, EnemyKind::Pursuer),
        Some(GridPosition::new(17, 8)),
        "the killer is still at its tile"
    );
    let world = app.world_mut();
    let mut enemies = world.query::<(&Enemy, &GridPosition)>();
    let positions: Vec<_> = enemies.iter(world).map(|(_, position)| *position).collect();
    assert!(
        positions.contains(&GridPosition::new(16, 10)),
        "the second enemy must not act after the player dies"
    );
    assert_eq!(turn_count(&app), 1);
    assert_eq!(state(&app), GameState::Lost);

    tap(&mut app, KeyCode::Space);
    assert_eq!(turn_count(&app), 1, "lost runs ignore dungeon actions");
}

#[test]
fn xp_triggers_upgrade_state_and_selection_does_not_consume_a_turn() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(16, 8));
    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 1,
            xp: 4,
            pending_upgrades: 0,
        },
    );
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(17, 8),
        1,
        Some(Health::new(4)),
    );
    app.update();

    tap(&mut app, KeyCode::KeyD);
    settle(&mut app);

    let progress = player_progress(&mut app);
    assert_eq!(progress.xp, 8);
    assert_eq!(progress.level, 2);
    assert_eq!(progress.pending_upgrades, 1);
    assert_eq!(state(&app), GameState::ChoosingUpgrade);

    let turn_before_choice = turn_count(&app);
    let position_before_choice = player_position(&mut app);
    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), position_before_choice);
    assert_eq!(turn_count(&app), turn_before_choice);

    tap(&mut app, KeyCode::Digit1);
    settle(&mut app);

    assert_eq!(player_health(&mut app).max, 28);
    assert_eq!(player_health(&mut app).current, 28);
    assert_eq!(player_progress(&mut app).pending_upgrades, 0);
    assert_eq!(state(&app), GameState::Playing);
    assert_eq!(turn_count(&app), turn_before_choice);

    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), GridPosition::new(17, 8));
    assert_eq!(turn_count(&app), turn_before_choice + 1);
}

#[test]
fn multiple_levels_require_one_selection_each() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(16, 8));
    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 3,
            xp: 18,
            pending_upgrades: 2,
        },
    );
    set_state(&mut app, GameState::ChoosingUpgrade);

    tap(&mut app, KeyCode::Digit1);
    assert_eq!(player_health(&mut app).max, 28);
    assert_eq!(player_progress(&mut app).pending_upgrades, 1);
    assert_eq!(state(&app), GameState::ChoosingUpgrade);
    let turn_before = turn_count(&app);

    tap(&mut app, KeyCode::Digit2);
    settle(&mut app);

    assert_eq!(player_attack(&mut app), 5);
    assert_eq!(player_progress(&mut app).pending_upgrades, 0);
    assert_eq!(state(&app), GameState::Playing);
    assert_eq!(turn_count(&app), turn_before);
}

#[test]
fn mouse_upgrade_selection_uses_the_same_apply_path() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(16, 8));
    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 2,
            xp: 8,
            pending_upgrades: 1,
        },
    );
    set_state(&mut app, GameState::ChoosingUpgrade);
    let turn_before = turn_count(&app);

    app.world_mut()
        .resource_mut::<UpgradeSelection>()
        .0
        .replace(UpgradeKind::Recovery);
    app.update();
    settle(&mut app);

    assert_eq!(player_recovery(&mut app).heal_on_kill, 3);
    assert_eq!(player_progress(&mut app).pending_upgrades, 0);
    assert_eq!(state(&app), GameState::Playing);
    assert_eq!(turn_count(&app), turn_before);
}

#[test]
fn death_beats_a_pending_upgrade_choice() {
    let mut app = gameplay_app();
    spawn_player_with(&mut app, GridPosition::new(16, 8), 2, 4);
    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 1,
            xp: 4,
            pending_upgrades: 0,
        },
    );
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(17, 8),
        1,
        Some(Health::new(4)),
    );
    spawn_enemy(
        &mut app,
        EnemyKind::Pursuer,
        GridPosition::new(15, 8),
        2,
        None,
    );
    app.update();

    tap(&mut app, KeyCode::KeyD);
    settle(&mut app);

    assert_eq!(player_health(&mut app).current, 0);
    assert_eq!(player_progress(&mut app).pending_upgrades, 1);
    assert_eq!(
        state(&app),
        GameState::Lost,
        "the death screen must win over the level-up panel"
    );
}

#[test]
fn boss_telegraphs_then_hits_then_recovers() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(28, 8));
    spawn_enemy(&mut app, EnemyKind::Boss, BOSS_POSITION, 1, None);
    spawn_exit(&mut app, true);
    app.update();

    tap(&mut app, KeyCode::Space);
    assert_eq!(
        boss_phase(&mut app),
        Some(BossPhase::WindingUp {
            target: GridPosition::new(28, 8)
        })
    );
    assert_eq!(player_health(&mut app).current, 24);
    assert_eq!(state(&app), GameState::Playing);

    tap(&mut app, KeyCode::Space);
    assert_eq!(
        player_health(&mut app).current,
        24 - balance::BOSS_HEAVY_DAMAGE
    );
    assert_eq!(boss_phase(&mut app), Some(BossPhase::Recovering));

    tap(&mut app, KeyCode::Space);
    assert_eq!(boss_phase(&mut app), Some(BossPhase::Ready));
    assert_eq!(
        player_health(&mut app).current,
        24 - balance::BOSS_HEAVY_DAMAGE
    );
}

#[test]
fn boss_heavy_attack_can_be_dodged_after_the_warning() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(28, 8));
    spawn_enemy(&mut app, EnemyKind::Boss, BOSS_POSITION, 1, None);
    spawn_exit(&mut app, true);
    app.update();

    tap(&mut app, KeyCode::Space);
    assert_eq!(
        boss_phase(&mut app),
        Some(BossPhase::WindingUp {
            target: GridPosition::new(28, 8)
        })
    );

    tap(&mut app, KeyCode::KeyA);
    assert_eq!(player_position(&mut app), GridPosition::new(27, 8));
    assert_eq!(player_health(&mut app).current, 24);
    assert_eq!(boss_phase(&mut app), Some(BossPhase::Recovering));
}

#[test]
fn boss_death_unlocks_exit_and_stepping_into_it_wins() {
    let mut app = gameplay_app();
    spawn_player_with(&mut app, GridPosition::new(28, 8), 24, 32);
    spawn_enemy(&mut app, EnemyKind::Boss, BOSS_POSITION, 1, None);
    spawn_exit(&mut app, true);
    app.update();

    tap(&mut app, KeyCode::KeyD);
    assert_eq!(enemy_health_of(&mut app, EnemyKind::Boss), None);
    assert_eq!(exit_locked(&mut app), Some(false));

    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), GridPosition::new(29, 8));

    tap(&mut app, KeyCode::KeyD);
    settle(&mut app);
    assert_eq!(state(&app), GameState::Won);
}

#[test]
fn locked_exit_blocks_without_a_turn_and_open_exit_wins() {
    let mut app = gameplay_app();
    spawn_player(&mut app, GridPosition::new(29, 8));
    let exit = spawn_exit(&mut app, true);
    app.update();

    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), GridPosition::new(29, 8));
    assert_eq!(turn_count(&app), 0);

    app.world_mut()
        .entity_mut(exit)
        .get_mut::<Exit>()
        .unwrap()
        .locked = false;
    tap(&mut app, KeyCode::KeyD);
    settle(&mut app);
    assert_eq!(state(&app), GameState::Won);
}

#[test]
fn full_game_startup_spawns_one_camera_player_boss_exit_and_hud() {
    let mut app = full_app();
    app.update();
    app.update();

    assert_eq!(count::<Camera2d>(&mut app), 1);
    assert_eq!(count::<Player>(&mut app), 1);
    assert_eq!(regular_enemy_count(&mut app), ENEMY_LAYOUT.len());
    assert_eq!(count::<Boss>(&mut app), 1);
    assert_eq!(count::<Exit>(&mut app), 1);
    assert_eq!(count::<HudRoot>(&mut app), 5);
    assert_eq!(count::<UpgradePanelRoot>(&mut app), 0);
    assert_eq!(count::<EndScreenRoot>(&mut app), 0);

    assert_eq!(player_position(&mut app), PLAYER_START);
    let health = player_health(&mut app);
    assert_eq!(health.current, balance::PLAYER_START_MAX_HP);
    assert_eq!(exit_locked(&mut app), Some(true));

    let world = app.world_mut();
    let mut player_sprite = world.query_filtered::<&Sprite, With<Player>>();
    let sprite = player_sprite.single(world).expect("one renderable player");
    assert_eq!(
        sprite.custom_size,
        Some(Vec2::splat(crate::grid::TILE_SIZE))
    );
    assert_eq!(
        world
            .resource::<AssetServer>()
            .get_path(sprite.image.id())
            .map(|path| path.to_string()),
        Some(PLAYER_TEXTURE.to_string())
    );

    let mut boss_sprite = world.query_filtered::<&Sprite, With<Boss>>();
    assert_eq!(
        boss_sprite.single(world).unwrap().custom_size,
        Some(Vec2::splat(BOSS_SIZE))
    );

    let map = world.resource::<DungeonMap>();
    assert_eq!(map.tile_at(PLAYER_START), Some(Tile::Floor));
    assert_eq!(map.tile_at(BOSS_POSITION), Some(Tile::Floor));
    assert_eq!(map.tile_at(EXIT_POSITION), Some(Tile::Floor));
}

#[test]
fn rendering_derives_player_position_and_exit_state() {
    let mut app = full_app();
    app.update();
    app.update();

    set_player_position(&mut app, GridPosition::new(28, 8));
    app.update();
    let map = DungeonMap::fixed();
    let world = app.world_mut();
    let mut player_transform = world.query_filtered::<&Transform, With<Player>>();
    let transform = player_transform.single(world).unwrap();
    assert_eq!(
        transform.translation,
        grid_to_world(&map, GridPosition::new(28, 8)).extend(2.0)
    );

    let exit_entity = {
        let mut exits = world.query_filtered::<Entity, With<Exit>>();
        exits.single(world).unwrap()
    };
    world
        .entity_mut(exit_entity)
        .get_mut::<Exit>()
        .unwrap()
        .locked = false;
    app.update();
    let world = app.world();
    let exit = world.get::<Exit>(exit_entity).unwrap();
    assert!(!exit.locked);
    let sprite = world.get::<Sprite>(exit_entity).unwrap();
    assert_eq!(
        world
            .resource::<AssetServer>()
            .get_path(sprite.image.id())
            .map(|path| path.to_string()),
        Some(EXIT_OPEN_TEXTURE.to_string())
    );

    let locked = world.resource::<Art>().exit_locked.id();
    assert_eq!(
        world
            .resource::<AssetServer>()
            .get_path(locked)
            .map(|p| p.to_string()),
        Some(EXIT_LOCKED_TEXTURE.to_string())
    );
}

#[test]
fn telegraph_marker_tracks_the_boss_warning_tile() {
    let mut app = full_app();
    app.update();
    app.update();

    set_player_position(&mut app, GridPosition::new(28, 8));
    app.update();
    tap(&mut app, KeyCode::Space);

    let map = DungeonMap::fixed();
    let world = app.world_mut();
    let mut marker = world.query_filtered::<(&Transform, &Visibility), With<TelegraphMarker>>();
    let (transform, visibility) = marker.single(world).expect("one telegraph marker");
    assert_eq!(*visibility, Visibility::Visible);
    assert_eq!(
        transform.translation,
        grid_to_world(&map, GridPosition::new(28, 8)).extend(-1.0)
    );
}

#[test]
fn restart_resets_everything_without_duplicating_camera_or_hud() {
    let mut app = full_app();
    app.update();
    app.update();

    let camera_count = count::<Camera2d>(&mut app);
    let hud_count = count::<HudRoot>(&mut app);
    let art_player = app.world().resource::<Art>().player.id();

    // Dirty the run to prove reset behavior.
    set_player_health(
        &mut app,
        Health {
            current: 1,
            max: 40,
        },
    );
    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 4,
            xp: 99,
            pending_upgrades: 2,
        },
    );
    app.world_mut().resource_mut::<TurnCount>().0 = 77;
    set_state(&mut app, GameState::Lost);
    assert_eq!(count::<EndScreenRoot>(&mut app), 1);

    // First restart through the actual button observer.
    let restart_button = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<RestartButton>>();
        query.single(world).expect("one restart button")
    };
    app.world_mut()
        .entity_mut(restart_button)
        .trigger(|entity| Activate { entity });
    app.update();
    app.update();
    assert!(!app.world().resource::<RestartRequested>().0);

    assert_eq!(state(&app), GameState::Playing);
    assert_eq!(count::<Camera2d>(&mut app), camera_count);
    assert_eq!(count::<HudRoot>(&mut app), hud_count);
    assert_eq!(count::<Player>(&mut app), 1);
    assert_eq!(count::<EndScreenRoot>(&mut app), 0);
    assert_eq!(count::<UpgradePanelRoot>(&mut app), 0);
    assert_eq!(count::<Exit>(&mut app), 1);
    assert_eq!(count::<Boss>(&mut app), 1);
    assert_eq!(regular_enemy_count(&mut app), ENEMY_LAYOUT.len());
    assert_eq!(player_position(&mut app), PLAYER_START);
    assert_eq!(
        player_health(&mut app),
        Health::new(balance::PLAYER_START_MAX_HP)
    );
    assert_eq!(player_progress(&mut app), PlayerProgress::default());
    assert_eq!(turn_count(&app), 0);
    assert_eq!(exit_locked(&mut app), Some(true));
    assert_eq!(boss_phase(&mut app), Some(BossPhase::Ready));
    assert_eq!(app.world().resource::<Art>().player.id(), art_player);

    // A victory/restart, this time using R, proves the flow works after either
    // result and can be repeated without duplicating UI.
    set_state(&mut app, GameState::Won);
    assert_eq!(count::<EndScreenRoot>(&mut app), 1);
    tap(&mut app, KeyCode::KeyR);
    settle(&mut app);
    assert_eq!(state(&app), GameState::Playing);
    assert_eq!(count::<Camera2d>(&mut app), 1);
    assert_eq!(count::<HudRoot>(&mut app), 5);
    assert_eq!(count::<Player>(&mut app), 1);
    assert_eq!(count::<EndScreenRoot>(&mut app), 0);
    assert_eq!(
        player_health(&mut app),
        Health::new(balance::PLAYER_START_MAX_HP)
    );
    assert_eq!(turn_count(&app), 0);
}

#[test]
fn upgrade_panel_has_three_cards_and_freezes_the_dungeon() {
    let mut app = full_app();
    app.update();
    app.update();

    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 2,
            xp: 8,
            pending_upgrades: 1,
        },
    );
    set_state(&mut app, GameState::ChoosingUpgrade);

    assert_eq!(count::<UpgradePanelRoot>(&mut app), 1);
    assert_eq!(count::<UpgradeChoice>(&mut app), 3);

    let position = player_position(&mut app);
    let turns = turn_count(&app);
    tap(&mut app, KeyCode::KeyD);
    assert_eq!(player_position(&mut app), position);
    assert_eq!(turn_count(&app), turns);

    tap(&mut app, KeyCode::Digit3);
    settle(&mut app);
    assert_eq!(player_recovery(&mut app).heal_on_kill, 3);
    assert_eq!(count::<UpgradePanelRoot>(&mut app), 0);
    assert_eq!(state(&app), GameState::Playing);
}

#[test]
fn clicking_an_upgrade_card_sets_the_same_selection_resource() {
    let mut app = full_app();
    app.update();
    app.update();

    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 2,
            xp: 8,
            pending_upgrades: 1,
        },
    );
    set_state(&mut app, GameState::ChoosingUpgrade);

    let vitality_card = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &UpgradeChoice)>();
        query
            .iter(world)
            .find(|(_, choice)| choice.0 == UpgradeKind::Vitality)
            .map(|(entity, _)| entity)
            .expect("a Vitality card")
    };
    app.world_mut()
        .entity_mut(vitality_card)
        .trigger(|entity| Activate { entity });
    app.update();
    settle(&mut app);

    assert_eq!(player_health(&mut app).max, 28);
    assert_eq!(player_progress(&mut app).pending_upgrades, 0);
    assert_eq!(state(&app), GameState::Playing);
}

#[test]
fn hud_reads_health_level_xp_and_boss_warning_from_game_state() {
    let mut app = full_app();
    app.update();
    app.update();

    assert_eq!(hud_text(&mut app, HudText::Hp), "HP 24 / 24");
    assert_eq!(
        hud_bar_width(&mut app, HudBar::PlayerHp),
        Val::Percent(100.0)
    );
    assert!(hud_text(&mut app, HudText::LevelXp).contains("LEVEL 1"));
    assert!(hud_text(&mut app, HudText::LevelXp).contains("XP 0 / 8"));

    set_player_health(
        &mut app,
        Health {
            current: 12,
            max: 24,
        },
    );
    set_player_progress(
        &mut app,
        PlayerProgress {
            level: 2,
            xp: 8,
            pending_upgrades: 0,
        },
    );
    app.update();

    assert_eq!(hud_text(&mut app, HudText::Hp), "HP 12 / 24");
    assert_eq!(
        hud_bar_width(&mut app, HudBar::PlayerHp),
        Val::Percent(50.0)
    );
    assert!(hud_text(&mut app, HudText::LevelXp).contains("LEVEL 2"));
    assert!(hud_text(&mut app, HudText::LevelXp).contains("XP 8 / 18"));

    // Put the player in boss threat range and wait for the telegraph.
    set_player_position(&mut app, GridPosition::new(28, 8));
    app.update();
    tap(&mut app, KeyCode::Space);

    assert!(hud_text(&mut app, HudText::BossHp).contains("32 / 32"));
    assert!(
        hud_text(&mut app, HudText::BossPhase).contains("HEAVY ATTACK"),
        "the warning must be visible before the next affected action"
    );

    tap(&mut app, KeyCode::Space);
    assert!(hud_text(&mut app, HudText::BossPhase).contains("Recovering"));
}

#[test]
fn headless_smoke_advances_assembled_game_systems() {
    let mut app = full_app();
    app.update();
    app.update();

    for _ in 0..4 {
        tap(&mut app, KeyCode::Space);
        app.update();
    }

    // If enemies have already ended the run, these updates must still be safe.
    for _ in 0..10 {
        app.update();
    }

    assert!(count::<Camera2d>(&mut app) == 1);
    assert!(count::<Player>(&mut app) <= 1);
}
