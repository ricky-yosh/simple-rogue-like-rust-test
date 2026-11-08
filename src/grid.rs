//! Grid coordinates, tile data, and the hand-authored dungeon.
//!
//! Nothing in this module knows about sprites, cameras, or UI. Rendering
//! converts these integer coordinates to world positions; gameplay never reads
//! a sprite's `Transform` to decide what is legal.

use bevy::prelude::*;

/// The width and height of one grid cell in world units.
pub const TILE_SIZE: f32 = 32.0;

pub const MAP_WIDTH: i32 = 32;
pub const MAP_HEIGHT: i32 = 18;

/// The player's authoritative starting tile.
pub const PLAYER_START: GridPosition = GridPosition::new(3, 3);

/// The boss guards a one-tile alcove; the exit is directly behind it.
pub const BOSS_POSITION: GridPosition = GridPosition::new(29, 8);
pub const EXIT_POSITION: GridPosition = GridPosition::new(30, 8);

/// Integer grid coordinates: the authoritative position for gameplay.
///
/// `x` grows east, `y` grows north, and both are measured in tiles.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the position one step away in the given cardinal direction.
    pub const fn offset(self, delta: (i32, i32)) -> Self {
        Self::new(self.x + delta.0, self.y + delta.1)
    }

    /// Cardinal-tile distance; diagonal neighbours are distance 2.
    pub fn manhattan_distance(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (0, 1),
            Self::South => (0, -1),
            Self::East => (1, 0),
            Self::West => (-1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Floor,
    Wall,
}

/// The fixed dungeon layout.
///
/// The first row is the north edge (highest grid `y`), so the strings read in
/// the same top-to-bottom order as the screen. `tile_at` performs the single
/// north-row-to-grid-y conversion.
///
/// Layout: a split left room, a tall central hall, and a boss room whose exit
/// alcove has exactly one entrance. The boss spawns in that entrance.
const FIXED_MAP: [&str; MAP_HEIGHT as usize] = [
    "################################", // y = 17
    "#..........####.......##.......#", // y = 16
    "#..........####.......##.......#", // y = 15
    "#..........####.......##.......#", // y = 14
    "#..........####.......##.......#", // y = 13
    "#..........####.......##.......#", // y = 12
    "#..........####.......##.......#", // y = 11
    "#..........####.......##.......#", // y = 10
    "#####.#########.......##.....###", // y = 9: left-room doorway at x=5
    "#..........####................#", // y = 8: central-to-boss corridor
    "#..........####.......##.....###", // y = 7
    "#..........####.......##.......#", // y = 6
    "#..........####.......##.......#", // y = 5
    "#.....................##.......#", // y = 4: corridor from left to central
    "#..........####.......##.......#", // y = 3
    "#..........####.......##.......#", // y = 2
    "#..........####.......##.......#", // y = 1
    "################################", // y = 0
];

/// One shared map resource. It is immutable during a run.
#[derive(Resource, Debug)]
pub struct DungeonMap {
    width: i32,
    height: i32,
    /// Row-major tiles, north row first.
    tiles: Vec<Tile>,
}

impl DungeonMap {
    pub fn fixed() -> Self {
        let map = Self::from_rows(&FIXED_MAP);
        debug_assert_eq!(map.width, MAP_WIDTH);
        debug_assert_eq!(map.height, MAP_HEIGHT);
        map
    }

    /// Builds a map from strings so tests can use tiny rooms.
    pub fn from_rows(rows: &[&str]) -> Self {
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

    pub const fn width(&self) -> i32 {
        self.width
    }

    pub const fn height(&self) -> i32 {
        self.height
    }

    /// Returns the tile at `position`, or `None` outside the map bounds.
    pub fn tile_at(&self, position: GridPosition) -> Option<Tile> {
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
    pub fn is_walkable(&self, position: GridPosition) -> bool {
        self.tile_at(position) == Some(Tile::Floor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_map_has_a_closed_wall_border() {
        let map = DungeonMap::fixed();

        for x in 0..map.width() {
            assert_eq!(map.tile_at(GridPosition::new(x, 0)), Some(Tile::Wall));
            assert_eq!(
                map.tile_at(GridPosition::new(x, map.height() - 1)),
                Some(Tile::Wall)
            );
        }
        for y in 0..map.height() {
            assert_eq!(map.tile_at(GridPosition::new(0, y)), Some(Tile::Wall));
            assert_eq!(
                map.tile_at(GridPosition::new(map.width() - 1, y)),
                Some(Tile::Wall)
            );
        }
    }

    #[test]
    fn spawn_and_objective_tiles_are_floor() {
        let map = DungeonMap::fixed();

        assert_eq!(map.tile_at(PLAYER_START), Some(Tile::Floor));
        assert_eq!(map.tile_at(BOSS_POSITION), Some(Tile::Floor));
        assert_eq!(map.tile_at(EXIT_POSITION), Some(Tile::Floor));
        // The exit is enclosed except through the boss tile.
        assert_eq!(
            map.tile_at(GridPosition::new(29, 7)),
            Some(Tile::Wall),
            "north side of the exit alcove should be wall"
        );
        assert_eq!(
            map.tile_at(GridPosition::new(29, 9)),
            Some(Tile::Wall),
            "south side of the exit alcove should be wall"
        );
    }

    #[test]
    fn row_reading_order_matches_the_drawn_map() {
        let map = DungeonMap::fixed();

        // The doorway in the y = 9 dividing wall is at x = 5.
        assert_eq!(map.tile_at(GridPosition::new(5, 9)), Some(Tile::Floor));
        assert_eq!(map.tile_at(GridPosition::new(6, 9)), Some(Tile::Wall));
        // The left-to-central corridor is at y = 4.
        assert_eq!(map.tile_at(GridPosition::new(12, 4)), Some(Tile::Floor));
        assert_eq!(map.tile_at(GridPosition::new(12, 5)), Some(Tile::Wall));
    }

    #[test]
    fn out_of_bounds_tiles_are_not_walkable() {
        let map = DungeonMap::fixed();

        assert_eq!(map.tile_at(GridPosition::new(-1, 3)), None);
        assert_eq!(map.tile_at(GridPosition::new(0, -1)), None);
        assert_eq!(map.tile_at(GridPosition::new(MAP_WIDTH, 3)), None);
        assert_eq!(map.tile_at(GridPosition::new(3, MAP_HEIGHT)), None);
        assert!(!map.is_walkable(GridPosition::new(-1, 3)));
    }

    #[test]
    fn manhattan_distance_counts_cardinal_steps_only() {
        let origin = GridPosition::new(3, 4);

        assert_eq!(origin.manhattan_distance(origin), 0);
        assert_eq!(origin.manhattan_distance(GridPosition::new(4, 4)), 1);
        assert_eq!(origin.manhattan_distance(GridPosition::new(5, 6)), 4);
    }
}
