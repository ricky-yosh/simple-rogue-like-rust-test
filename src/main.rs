//! Second visual checkpoint: a fixed grid map and turn-based player movement.
//!
//! Gameplay truth is the integer [`GridPosition`] component. The sprite's
//! [`Transform`] is derived from it by [`sync_player_transform`], never the other
//! way around.

use bevy::prelude::*;

/// The width and height of one grid cell in world units. Also the player's size.
const TILE_SIZE: f32 = 32.0;

/// The player's width and height in world units; one unit is one pixel at default zoom.
const PLAYER_SIZE: f32 = TILE_SIZE;

const PLAYER_COLOR: Color = Color::srgb(0.30, 0.90, 0.50);
const FLOOR_COLOR: Color = Color::srgb(0.10, 0.12, 0.16);
const WALL_COLOR: Color = Color::srgb(0.32, 0.35, 0.42);

/// Rendering layers. Larger values draw on top.
const FLOOR_Z: f32 = -2.0;
const WALL_Z: f32 = -1.0;
const PLAYER_Z: f32 = 0.0;

/// A hand-authored map. The first row is the north edge (highest grid y), so
/// reading the strings top to bottom matches what appears on screen.
const FIXED_MAP: [&str; 15] = [
    "####################",
    "#..................#",
    "#..................#",
    "#.........#........#",
    "#.........#........#",
    "#.........#........#",
    "#.........#........#",
    "#..................#",
    "#..................#",
    "#.........#........#",
    "#.........#........#",
    "#.........#........#",
    "#.........#........#",
    "#..................#",
    "####################",
];

/// The tile the player starts on. Its y counts from the south edge, so y = 7 is
/// one of the open rows (the ninth row in [`FIXED_MAP`], counting from the top).
const PLAYER_START: GridPosition = GridPosition::new(2, 7);

#[derive(Component, Debug)]
struct Player;

/// Integer grid coordinates, the authoritative position for gameplay.
///
/// `x` grows east, `y` grows north, and both are measured in tiles.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct GridPosition {
    x: i32,
    y: i32,
}

impl GridPosition {
    const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the position one step away in the given cardinal direction.
    const fn offset(self, delta: (i32, i32)) -> Self {
        Self::new(self.x + delta.0, self.y + delta.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Floor,
    Wall,
}

/// The fixed dungeon layout. It is a resource because every system may need to
/// query it, and there is exactly one map.
#[derive(Resource, Debug)]
struct DungeonMap {
    width: i32,
    height: i32,
    /// Row-major tiles, north row first. Use [`DungeonMap::tile_at`] instead of
    /// indexing directly so the y-axis flip stays in one place.
    tiles: Vec<Tile>,
}

impl DungeonMap {
    fn from_rows(rows: &[&str]) -> Self {
        let height = rows.len();
        assert!(height > 0, "a dungeon map needs at least one row");
        let width = rows[0].len();
        assert!(width > 0, "a dungeon map needs at least one column");
        assert!(
            rows.iter().all(|row| row.len() == width),
            "every dungeon map row must have the same width"
        );

        let tiles = rows
            .iter()
            .flat_map(|row| row.chars())
            .map(|character| match character {
                '#' => Tile::Wall,
                '.' => Tile::Floor,
                other => panic!("unsupported dungeon map character: {other:?}"),
            })
            .collect();

        Self {
            width: width as i32,
            height: height as i32,
            tiles,
        }
    }

    fn fixed() -> Self {
        Self::from_rows(&FIXED_MAP)
    }

    /// Returns the tile at `position`, or `None` outside the map bounds.
    fn tile_at(&self, position: GridPosition) -> Option<Tile> {
        let in_bounds = position.x >= 0
            && position.y >= 0
            && position.x < self.width
            && position.y < self.height;
        if !in_bounds {
            return None;
        }

        let row_from_north = self.height - 1 - position.y;
        let index = row_from_north * self.width + position.x;
        self.tiles.get(index as usize).copied()
    }

    /// Walls and positions outside the map are not walkable.
    fn is_walkable(&self, position: GridPosition) -> bool {
        self.tile_at(position) == Some(Tile::Floor)
    }
}

/// Keys that request a one-cell step, paired with their grid delta. The order
/// decides which direction wins if several keys are pressed in the same frame.
const MOVEMENT_KEYS: [(KeyCode, (i32, i32)); 8] = [
    (KeyCode::KeyW, (0, 1)),
    (KeyCode::ArrowUp, (0, 1)),
    (KeyCode::KeyS, (0, -1)),
    (KeyCode::ArrowDown, (0, -1)),
    (KeyCode::KeyA, (-1, 0)),
    (KeyCode::ArrowLeft, (-1, 0)),
    (KeyCode::KeyD, (1, 0)),
    (KeyCode::ArrowRight, (1, 0)),
];

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.08)))
        .insert_resource(DungeonMap::fixed())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dungeon escape — grid movement".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, sync_player_transform).chain())
        .run();
}

fn setup(mut commands: Commands, map: Res<DungeonMap>) {
    commands.spawn(Camera2d);

    for y in 0..map.height {
        for x in 0..map.width {
            let position = GridPosition::new(x, y);
            let Some(tile) = map.tile_at(position) else {
                continue;
            };
            let (color, z) = match tile {
                Tile::Floor => (FLOOR_COLOR, FLOOR_Z),
                Tile::Wall => (WALL_COLOR, WALL_Z),
            };
            commands.spawn((
                Sprite::from_color(color, Vec2::splat(TILE_SIZE)),
                Transform::from_translation(grid_to_world(&map, position).extend(z)),
            ));
        }
    }

    commands.spawn((
        Player,
        PLAYER_START,
        Sprite::from_color(PLAYER_COLOR, Vec2::splat(PLAYER_SIZE)),
        Transform::from_translation(grid_to_world(&map, PLAYER_START).extend(PLAYER_Z)),
    ));
}

/// Returns the movement delta requested by a *fresh* key press this frame.
///
/// `just_pressed` (rather than `pressed`) keeps the game turn-based: holding a
/// key does not slide the player, and one press moves exactly one cell.
fn requested_move(keys: &ButtonInput<KeyCode>) -> Option<(i32, i32)> {
    MOVEMENT_KEYS
        .iter()
        .find(|(key, _)| keys.just_pressed(*key))
        .map(|(_, delta)| *delta)
}

/// Pure movement rule: propose a target and accept it only if the map allows it.
fn step_if_walkable(
    map: &DungeonMap,
    from: GridPosition,
    delta: (i32, i32),
) -> Option<GridPosition> {
    let target = from.offset(delta);
    map.is_walkable(target).then_some(target)
}

/// Turns a fresh key press into at most one grid step. Blocked moves leave the
/// player where they were and do not need a special case.
fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    map: Res<DungeonMap>,
    mut players: Query<&mut GridPosition, With<Player>>,
) {
    let Some(delta) = requested_move(&keys) else {
        return;
    };
    let Ok(mut position) = players.single_mut() else {
        return;
    };

    if let Some(target) = step_if_walkable(&map, *position, delta) {
        *position = target;
    }
}

/// The query [`sync_player_transform`] needs: only players whose grid position
/// changed this frame. The alias keeps Bevy's filter types readable.
type ChangedPlayerTransforms<'w, 's> = Query<
    'w,
    's,
    (&'static GridPosition, &'static mut Transform),
    (With<Player>, Changed<GridPosition>),
>;

/// Derives the rendered position from the integer grid position. Runs only when
/// the grid position changes, and after [`move_player`] in the same frame.
fn sync_player_transform(map: Res<DungeonMap>, mut players: ChangedPlayerTransforms) {
    let Ok((position, mut transform)) = players.single_mut() else {
        return;
    };

    transform.translation = grid_to_world(&map, *position).extend(PLAYER_Z);
}

/// Converts a grid cell to the world-space center of that cell. The map is
/// centered on the origin, so this is the only place that does coordinate math.
fn grid_to_world(map: &DungeonMap, position: GridPosition) -> Vec2 {
    let center_x = (map.width as f32 - 1.0) / 2.0;
    let center_y = (map.height as f32 - 1.0) / 2.0;

    Vec2::new(
        (position.x as f32 - center_x) * TILE_SIZE,
        (position.y as f32 - center_y) * TILE_SIZE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn startup_test_app() -> App {
        let mut app = App::new();
        app.insert_resource(DungeonMap::fixed());
        app.add_systems(Startup, setup);
        app
    }

    /// A headless app with the movement systems and one player, but no rendering
    /// plugins. Input is injected directly into the `ButtonInput` resource.
    fn movement_test_app(start: GridPosition) -> App {
        let mut app = App::new();
        app.insert_resource(DungeonMap::fixed());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Update, (move_player, sync_player_transform).chain());
        app.world_mut().spawn((
            Player,
            start,
            Sprite::from_color(PLAYER_COLOR, Vec2::splat(PLAYER_SIZE)),
            Transform::default(),
        ));
        app
    }

    fn press(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
        // Mirrors what InputPlugin does after each frame, so the next update sees
        // the key as held rather than newly pressed.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(key);
    }

    fn player_position(app: &mut App) -> GridPosition {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&GridPosition, With<Player>>();
        *query
            .single(world)
            .expect("test app should contain exactly one player position")
    }

    fn player_translation(app: &mut App) -> Vec3 {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&Transform, With<Player>>();
        query
            .single(world)
            .expect("test app should contain exactly one player transform")
            .translation
    }

    #[test]
    fn startup_creates_one_camera_and_visible_player_without_duplicates() {
        let mut app = startup_test_app();

        app.update();
        app.update();

        let world = app.world_mut();
        assert_eq!(
            world.query::<&Camera2d>().iter(world).count(),
            1,
            "startup should create exactly one 2D camera"
        );

        let mut players = world
            .query_filtered::<(&Sprite, &Transform, &Visibility, &GridPosition), With<Player>>();
        let (sprite, transform, visibility, position) = players
            .single(world)
            .expect("startup should create exactly one renderable player");
        assert_eq!(*position, PLAYER_START);
        assert_eq!(
            transform.translation,
            grid_to_world(&DungeonMap::fixed(), PLAYER_START).extend(PLAYER_Z),
            "the sprite should be rendered at the spawn tile"
        );
        assert_eq!(sprite.custom_size, Some(Vec2::splat(PLAYER_SIZE)));
        assert!(
            sprite.color.alpha() > 0.0,
            "the player must not be transparent"
        );
        assert_ne!(*visibility, Visibility::Hidden);
    }

    #[test]
    fn wasd_moves_one_cell_per_press() {
        let mut app = movement_test_app(PLAYER_START);
        app.update();

        press(&mut app, KeyCode::KeyD);
        assert_eq!(player_position(&mut app), GridPosition::new(3, 7));

        press(&mut app, KeyCode::KeyW);
        assert_eq!(player_position(&mut app), GridPosition::new(3, 8));

        press(&mut app, KeyCode::KeyA);
        assert_eq!(player_position(&mut app), GridPosition::new(2, 8));

        press(&mut app, KeyCode::KeyS);
        assert_eq!(player_position(&mut app), PLAYER_START);
    }

    #[test]
    fn arrow_keys_move_one_cell_per_press() {
        let mut app = movement_test_app(PLAYER_START);
        app.update();

        press(&mut app, KeyCode::ArrowUp);
        assert_eq!(player_position(&mut app), GridPosition::new(2, 8));

        press(&mut app, KeyCode::ArrowRight);
        assert_eq!(player_position(&mut app), GridPosition::new(3, 8));

        press(&mut app, KeyCode::ArrowDown);
        assert_eq!(player_position(&mut app), GridPosition::new(3, 7));

        press(&mut app, KeyCode::ArrowLeft);
        assert_eq!(player_position(&mut app), PLAYER_START);
    }

    #[test]
    fn walls_block_movement_and_keep_the_rendered_position_in_sync() {
        // (10, 5) is an interior wall; approach it from the west.
        let start = GridPosition::new(9, 5);
        let mut app = movement_test_app(start);
        app.update();

        press(&mut app, KeyCode::KeyD);

        assert_eq!(player_position(&mut app), start, "the wall should block");
        assert_eq!(
            player_translation(&mut app),
            grid_to_world(&DungeonMap::fixed(), start).extend(PLAYER_Z),
            "a blocked move must not move the sprite either"
        );
    }

    #[test]
    fn map_boundaries_block_movement() {
        let map = DungeonMap::from_rows(&[".....", ".....", "....."]);
        let west_edge = GridPosition::new(0, 1);
        let east_edge = GridPosition::new(4, 1);

        assert_eq!(step_if_walkable(&map, west_edge, (-1, 0)), None);
        assert_eq!(step_if_walkable(&map, east_edge, (1, 0)), None);
        assert_eq!(
            step_if_walkable(&map, west_edge, (0, -1)),
            Some(GridPosition::new(0, 0))
        );
        assert!(
            !map.is_walkable(GridPosition::new(-1, 1)),
            "positions outside the map are never walkable"
        );
        assert_eq!(map.tile_at(GridPosition::new(5, 1)), None);
    }

    #[test]
    fn border_walls_block_movement_at_the_map_edge() {
        let start = GridPosition::new(1, 5);
        let mut app = movement_test_app(start);
        app.update();

        press(&mut app, KeyCode::KeyA);

        assert_eq!(
            player_position(&mut app),
            start,
            "the fixed map's west border wall should block"
        );
    }

    #[test]
    fn moving_again_requires_a_new_key_press() {
        let mut app = movement_test_app(PLAYER_START);
        app.update();

        press(&mut app, KeyCode::KeyD);
        assert_eq!(player_position(&mut app), GridPosition::new(3, 7));

        // The key is still "pressed" but no longer "just pressed".
        app.update();
        assert_eq!(
            player_position(&mut app),
            GridPosition::new(3, 7),
            "holding a key must not slide the player"
        );
    }

    #[test]
    fn rendered_transform_is_derived_from_the_grid_position() {
        let mut app = movement_test_app(PLAYER_START);
        app.update();

        press(&mut app, KeyCode::KeyD);

        let position = player_position(&mut app);
        assert_eq!(position, GridPosition::new(3, 7));
        assert_eq!(
            player_translation(&mut app),
            grid_to_world(&DungeonMap::fixed(), position).extend(PLAYER_Z)
        );
    }

    #[test]
    fn fixed_map_has_a_closed_wall_border() {
        let map = DungeonMap::fixed();

        for x in 0..map.width {
            assert_eq!(map.tile_at(GridPosition::new(x, 0)), Some(Tile::Wall));
            assert_eq!(
                map.tile_at(GridPosition::new(x, map.height - 1)),
                Some(Tile::Wall)
            );
        }
        for y in 0..map.height {
            assert_eq!(map.tile_at(GridPosition::new(0, y)), Some(Tile::Wall));
            assert_eq!(
                map.tile_at(GridPosition::new(map.width - 1, y)),
                Some(Tile::Wall)
            );
        }
        assert_eq!(map.tile_at(PLAYER_START), Some(Tile::Floor));
    }
}
