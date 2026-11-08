//! Rendering: loads the Kenney tiles, spawns a run's visuals, and derives all
//! sprite transforms from grid positions. Gameplay never depends on this file.

use bevy::prelude::*;

use crate::balance;
use crate::gameplay::{
    Attack, Boss, BossPhase, Enemy, EnemyKind, Exit, Feedback, GameState, Health, PendingAction,
    Player, PlayerProgress, Recovery, RunEntity, TurnCount, TurnFlow, UpgradeSelection,
};
use crate::grid::{
    BOSS_POSITION, DungeonMap, EXIT_POSITION, GridPosition, PLAYER_START, TILE_SIZE,
};

/// Preserve the previously selected adventurer art.
pub const PLAYER_TEXTURE: &str = "third_party/kenney_tiny_dungeon/Tiles/tile_0085.png";

pub const SKELETON_TEXTURE: &str = "third_party/kenney_tiny_dungeon/Tiles/tile_0109.png";
pub const BRUTE_TEXTURE: &str = "third_party/kenney_tiny_dungeon/Tiles/tile_0096.png";
pub const BOSS_TEXTURE: &str = "third_party/kenney_tiny_dungeon/Tiles/tile_0112.png";
pub const EXIT_LOCKED_TEXTURE: &str = "third_party/kenney_tiny_dungeon/Tiles/tile_0045.png";
pub const EXIT_OPEN_TEXTURE: &str = "third_party/kenney_tiny_dungeon/Tiles/tile_0047.png";
pub const FLOOR_TEXTURES: [&str; 3] = [
    "third_party/kenney_tiny_dungeon/Tiles/tile_0048.png",
    "third_party/kenney_tiny_dungeon/Tiles/tile_0049.png",
    "third_party/kenney_tiny_dungeon/Tiles/tile_0051.png",
];
pub const WALL_TEXTURE: &str = "third_party/kenney_tiny_dungeon/Tiles/tile_0040.png";

const ACTOR_SIZE: f32 = TILE_SIZE;
pub const BOSS_SIZE: f32 = TILE_SIZE * 1.5;

const FLOOR_Z: f32 = -3.0;
const WALL_Z: f32 = -2.0;
const TELEGRAPH_Z: f32 = -1.0;
const EXIT_Z: f32 = 0.0;
const ENEMY_Z: f32 = 1.0;
const BOSS_Z: f32 = 1.5;
const PLAYER_Z: f32 = 2.0;

/// Regular enemies in stable turn order. Positions are validated by tests.
pub const ENEMY_LAYOUT: [(EnemyKind, GridPosition); 4] = [
    (EnemyKind::Pursuer, GridPosition::new(7, 5)),
    (EnemyKind::Brute, GridPosition::new(6, 13)),
    (EnemyKind::Pursuer, GridPosition::new(17, 6)),
    (EnemyKind::Brute, GridPosition::new(19, 13)),
];

#[derive(Resource, Clone)]
pub struct Art {
    pub player: Handle<Image>,
    pub skeleton: Handle<Image>,
    pub brute: Handle<Image>,
    pub boss: Handle<Image>,
    pub exit_locked: Handle<Image>,
    pub exit_open: Handle<Image>,
    pub floors: [Handle<Image>; 3],
    pub wall: Handle<Image>,
}

impl Art {
    /// Handles used by tests that do not include `AssetPlugin`.
    pub fn default_handles() -> Self {
        let empty = Handle::default();
        Self {
            player: empty.clone(),
            skeleton: empty.clone(),
            brute: empty.clone(),
            boss: empty.clone(),
            exit_locked: empty.clone(),
            exit_open: empty.clone(),
            floors: [empty.clone(), empty.clone(), empty],
            wall: Handle::default(),
        }
    }

    fn load(asset_server: &AssetServer) -> Self {
        Self {
            player: asset_server.load(PLAYER_TEXTURE),
            skeleton: asset_server.load(SKELETON_TEXTURE),
            brute: asset_server.load(BRUTE_TEXTURE),
            boss: asset_server.load(BOSS_TEXTURE),
            exit_locked: asset_server.load(EXIT_LOCKED_TEXTURE),
            exit_open: asset_server.load(EXIT_OPEN_TEXTURE),
            floors: FLOOR_TEXTURES.map(|path| asset_server.load(path)),
            wall: asset_server.load(WALL_TEXTURE),
        }
    }
}

impl FromWorld for Art {
    fn from_world(world: &mut World) -> Self {
        world
            .get_resource::<AssetServer>()
            .map(Self::load)
            .unwrap_or_else(Self::default_handles)
    }
}

#[derive(Component)]
pub struct TelegraphMarker;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Art>()
            .add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameState::Playing), spawn_run_if_missing)
            .add_systems(
                Update,
                (
                    sync_grid_transforms,
                    sync_exit_visual,
                    sync_telegraph,
                    sync_boss_warning,
                )
                    .in_set(crate::gameplay::GameSet::Presentation),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    // Exactly one camera for the lifetime of the app. Restart does not touch it.
    commands.spawn(Camera2d);
}

/// Builds the world for a new run once the gameplay state is ready. Returning
/// early when a player already exists lets `ChoosingUpgrade -> Playing` resume
/// without duplicating the dungeon.
#[allow(clippy::too_many_arguments)]
fn spawn_run_if_missing(
    mut commands: Commands,
    existing_players: Query<(), With<Player>>,
    map: Res<DungeonMap>,
    art: Res<Art>,
    mut feedback: ResMut<Feedback>,
    mut turn: ResMut<TurnCount>,
    mut flow: ResMut<TurnFlow>,
    mut pending: ResMut<PendingAction>,
    mut selection: ResMut<UpgradeSelection>,
) {
    if !existing_players.is_empty() {
        return;
    }

    *feedback = Feedback::default();
    *turn = TurnCount::default();
    *flow = TurnFlow::default();
    *pending = PendingAction::default();
    *selection = UpgradeSelection::default();

    spawn_run(&mut commands, &map, &art);
    feedback.say("Explore the dungeon. Defeat the Warden, then take the exit.");
}

fn spawn_run(commands: &mut Commands, map: &DungeonMap, art: &Art) {
    spawn_tiles(commands, map, art);
    spawn_exit(commands, map, art);
    spawn_player(commands, map, art);
    spawn_enemies(commands, map, art);
    spawn_boss(commands, map, art);
    spawn_telegraph(commands, map);
}

fn spawn_tiles(commands: &mut Commands, map: &DungeonMap, art: &Art) {
    for y in 0..map.height() {
        for x in 0..map.width() {
            let position = GridPosition::new(x, y);
            let Some(tile) = map.tile_at(position) else {
                continue;
            };
            let (image, z) = match tile {
                crate::grid::Tile::Floor => {
                    let variant = floor_variant(position);
                    (art.floors[variant].clone(), FLOOR_Z)
                }
                crate::grid::Tile::Wall => (art.wall.clone(), WALL_Z),
            };
            commands.spawn((
                RunEntity,
                Sprite {
                    image,
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_translation(grid_to_world(map, position).extend(z)),
            ));
        }
    }
}

fn spawn_exit(commands: &mut Commands, map: &DungeonMap, art: &Art) {
    commands.spawn((
        RunEntity,
        Exit { locked: true },
        Sprite {
            image: art.exit_locked.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(grid_to_world(map, EXIT_POSITION).extend(EXIT_Z)),
    ));
}

fn spawn_player(commands: &mut Commands, map: &DungeonMap, art: &Art) {
    commands.spawn((
        RunEntity,
        Player,
        PLAYER_START,
        Health::new(balance::PLAYER_START_MAX_HP),
        Attack::new(balance::PLAYER_START_ATTACK),
        Recovery::new(balance::PLAYER_START_HEAL_ON_KILL),
        PlayerProgress::default(),
        Sprite {
            image: art.player.clone(),
            custom_size: Some(Vec2::splat(ACTOR_SIZE)),
            ..default()
        },
        Transform::from_translation(grid_to_world(map, PLAYER_START).extend(PLAYER_Z)),
    ));
}

fn spawn_enemies(commands: &mut Commands, map: &DungeonMap, art: &Art) {
    for (index, (kind, position)) in ENEMY_LAYOUT.into_iter().enumerate() {
        let image = match kind {
            EnemyKind::Pursuer => art.skeleton.clone(),
            EnemyKind::Brute => art.brute.clone(),
            EnemyKind::Boss => art.boss.clone(),
        };
        commands.spawn((
            RunEntity,
            Enemy {
                kind,
                order: index as u32 + 1,
            },
            position,
            Health::new(kind.max_hp()),
            Attack::new(kind.attack()),
            Sprite {
                image,
                custom_size: Some(Vec2::splat(ACTOR_SIZE)),
                ..default()
            },
            Transform::from_translation(grid_to_world(map, position).extend(ENEMY_Z)),
        ));
    }
}

fn spawn_boss(commands: &mut Commands, map: &DungeonMap, art: &Art) {
    commands.spawn((
        RunEntity,
        Boss,
        Enemy {
            kind: EnemyKind::Boss,
            order: 100,
        },
        BossPhase::Ready,
        BOSS_POSITION,
        Health::new(balance::BOSS_MAX_HP),
        Attack::new(balance::BOSS_HEAVY_DAMAGE),
        Sprite {
            image: art.boss.clone(),
            custom_size: Some(Vec2::splat(BOSS_SIZE)),
            ..default()
        },
        Transform::from_translation(grid_to_world(map, BOSS_POSITION).extend(BOSS_Z)),
    ));
}

fn spawn_telegraph(commands: &mut Commands, map: &DungeonMap) {
    commands.spawn((
        RunEntity,
        TelegraphMarker,
        Sprite::from_color(Color::srgba(0.95, 0.12, 0.12, 0.55), Vec2::splat(TILE_SIZE)),
        Transform::from_translation(grid_to_world(map, BOSS_POSITION).extend(TELEGRAPH_Z)),
        Visibility::Hidden,
    ));
}

fn floor_variant(position: GridPosition) -> usize {
    (position.x * 7 + position.y * 13).rem_euclid(3) as usize
}

/// Converts a grid cell to the world-space center of that cell. The map is
/// centered on the origin, so this is the only coordinate conversion.
pub fn grid_to_world(map: &DungeonMap, position: GridPosition) -> Vec2 {
    let center_x = (map.width() as f32 - 1.0) / 2.0;
    let center_y = (map.height() as f32 - 1.0) / 2.0;

    Vec2::new(
        (position.x as f32 - center_x) * TILE_SIZE,
        (position.y as f32 - center_y) * TILE_SIZE,
    )
}

/// Grid position is authoritative; this system only copies it into the render
/// transform when it changes. Existing z ordering is preserved.
fn sync_grid_transforms(
    map: Res<DungeonMap>,
    mut actors: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
) {
    for (position, mut transform) in &mut actors {
        let world = grid_to_world(&map, *position);
        transform.translation.x = world.x;
        transform.translation.y = world.y;
    }
}

fn sync_exit_visual(exit: Query<(&Exit, &mut Sprite), Changed<Exit>>, art: Res<Art>) {
    for (exit, mut sprite) in exit {
        sprite.image = if exit.locked {
            art.exit_locked.clone()
        } else {
            art.exit_open.clone()
        };
    }
}

fn sync_telegraph(
    map: Res<DungeonMap>,
    boss: Query<&BossPhase, With<Boss>>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<TelegraphMarker>>,
) {
    let target = boss.iter().next().and_then(|phase| match phase {
        BossPhase::WindingUp { target } => Some(*target),
        _ => None,
    });

    for (mut transform, mut visibility) in &mut marker {
        if let Some(target) = target {
            let world = grid_to_world(&map, target);
            transform.translation.x = world.x;
            transform.translation.y = world.y;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn sync_boss_warning(boss: Query<(&BossPhase, &mut Sprite), With<Boss>>) {
    for (phase, mut sprite) in boss {
        sprite.color = match phase {
            BossPhase::WindingUp { .. } => Color::srgb(1.0, 0.55, 0.55),
            _ => Color::WHITE,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{MAP_HEIGHT, MAP_WIDTH};
    use std::collections::{HashSet, VecDeque};
    use std::path::Path;

    #[test]
    fn all_selected_art_files_exist() {
        let paths = [
            PLAYER_TEXTURE,
            SKELETON_TEXTURE,
            BRUTE_TEXTURE,
            BOSS_TEXTURE,
            EXIT_LOCKED_TEXTURE,
            EXIT_OPEN_TEXTURE,
            WALL_TEXTURE,
            FLOOR_TEXTURES[0],
            FLOOR_TEXTURES[1],
            FLOOR_TEXTURES[2],
        ];

        for path in paths {
            assert!(
                Path::new("assets").join(path).is_file(),
                "missing art asset: {path}"
            );
        }
    }

    #[test]
    fn enemy_layout_is_floor_and_unique() {
        let map = DungeonMap::fixed();
        let mut seen = std::collections::HashSet::new();

        for (kind, position) in ENEMY_LAYOUT {
            assert_eq!(
                map.tile_at(position),
                Some(crate::grid::Tile::Floor),
                "{} spawns on a non-floor tile",
                kind.name()
            );
            assert_ne!(position, PLAYER_START);
            assert_ne!(position, BOSS_POSITION);
            assert_ne!(position, EXIT_POSITION);
            assert!(
                seen.insert(position),
                "duplicate enemy spawn at {position:?}"
            );
        }
    }

    #[test]
    fn all_enemies_boss_and_exit_are_reachable_from_the_player() {
        let map = DungeonMap::fixed();
        let mut reachable = HashSet::from([PLAYER_START]);
        let mut queue = VecDeque::from([PLAYER_START]);

        while let Some(position) = queue.pop_front() {
            for delta in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let next = position.offset(delta);
                if map.is_walkable(next) && reachable.insert(next) {
                    queue.push_back(next);
                }
            }
        }

        for (kind, position) in ENEMY_LAYOUT {
            assert!(
                reachable.contains(&position),
                "{} at {position:?} is unreachable",
                kind.name()
            );
        }
        assert!(reachable.contains(&BOSS_POSITION));
        assert!(reachable.contains(&EXIT_POSITION));
    }

    #[test]
    fn floor_variation_is_deterministic_and_in_range() {
        let first = floor_variant(GridPosition::new(3, 3));
        assert_eq!(first, floor_variant(GridPosition::new(3, 3)));
        assert!(first < FLOOR_TEXTURES.len());

        for x in 0..crate::grid::MAP_WIDTH {
            for y in 0..crate::grid::MAP_HEIGHT {
                assert!(floor_variant(GridPosition::new(x, y)) < FLOOR_TEXTURES.len());
            }
        }
    }

    #[test]
    fn grid_conversion_centers_the_map() {
        let map = DungeonMap::fixed();

        assert_eq!(
            grid_to_world(&map, GridPosition::new(0, 0)),
            Vec2::new(-496.0, -272.0)
        );
        assert_eq!(
            grid_to_world(&map, GridPosition::new(MAP_WIDTH - 1, MAP_HEIGHT - 1)),
            Vec2::new(496.0, 272.0)
        );
    }
}
