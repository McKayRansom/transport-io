use pathfinding::prelude::{astar, dijkstra};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::building::{Building, BuildingType};
use super::tile::{Reservation, Tile, YieldType};
use super::{BuildingHashMap, Direction, Position, DEFAULT_CITY_ID};
use crate::consts::SpawnerColors;
use crate::hash_map_id::{HashMapId, Id};

// const EMPTY_ROAD_COLOR: Color = Color::new(0.3, 0.3, 0.3, 0.5);
// const EMPTY_ROAD_COLOR: Color = WHITE;
// const RESERVED_PATH_COLOR: Color = Color::new(1.0, 0.1, 0.0, 0.3);
// const CONNECTION_INDICATOR_COLOR: Color = Color::new(0.7, 0.7, 0.7, 0.7);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
pub const GRID_CELL_SIZE: (f32, f32) = (32., 32.);

pub const GRID_Z_OFFSET: f32 = 10.;

type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GridTile {
    pub ground: Tile,
    pub bridge: Tile,
}

impl GridTile {
    fn new() -> Self {
        GridTile {
            ground: Tile::new(),
            bridge: Tile::new(),
        }
    }

    fn new_from_char(chr: char) -> Self {
        GridTile {
            ground: Tile::new_from_char(chr),
            bridge: Tile::Empty,
        }
    }

    fn new_from_char_layers(args: (char, char)) -> Self {
        GridTile {
            ground: Tile::new_from_char(args.0),
            bridge: Tile::new_from_char(args.1),
        }
    }

    fn get(&self, pos_z: i16) -> Option<&Tile> {
        if pos_z == 0 {
            Some(&self.ground)
        } else if pos_z == 1 {
            Some(&self.bridge)
        } else {
            None
        }
    }

    fn get_mut(&mut self, pos_z: i16) -> Option<&mut Tile> {
        if pos_z == 0 {
            Some(&mut self.ground)
        } else if pos_z == 1 {
            Some(&mut self.bridge)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct Grid {
    pub tiles: Vec<Vec<GridTile>>,
    pub buildings: BuildingHashMap,
}

impl Position {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReservationError {
    TileInvalid,
    TileReserved,
    // TileDoNotBlock,
}

impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "\ng")?;
        // for x in 0..self.tiles[0].len() {
        //     write!(f, "{}", x % 10)?;
        // }
        writeln!(f)?;
        for y in 0..self.tiles.len() {
            // write!(f, "{}", y)?;
            let mut has_bridges = false;
            for x in 0..self.tiles[y].len() {
                self.tiles[y][x].ground.fmt(f)?;
                has_bridges |= self.tiles[y][x].bridge != Tile::Empty;
            }
            if has_bridges {
                write!(f, "  ")?;
                for x in 0..self.tiles[y].len() {
                    self.tiles[y][x].bridge.fmt(f)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Grid {
    pub fn new(size: (i16, i16)) -> Self {
        Grid {
            tiles: vec![vec![GridTile::new(); size.0 as usize]; size.1 as usize],
            buildings: HashMapId::new(),
        }
    }

    pub fn size(&self) -> (i16, i16) {
        (self.tiles[0].len() as i16, self.tiles.len() as i16)
    }

    pub fn size_px(&self) -> (f32, f32) {
        (
            self.tiles[0].len() as f32 * GRID_CELL_SIZE.0,
            self.tiles.len() as f32 * GRID_CELL_SIZE.1,
        )
    }

    #[allow(dead_code)]
    pub fn pos(&self, x: i16, y: i16) -> Position {
        Position::new(x, y)
    }

    pub fn new_from_string(string: &str) -> Grid {
        let mut grid = Grid {
            buildings: HashMapId::new(),
            tiles: string
                .split_ascii_whitespace()
                .map(|line| line.chars().map(GridTile::new_from_char).collect())
                .collect(),
        };

        grid.fixup_from_string();

        grid
    }

    fn fixup_from_string(&mut self) {
        let size = self.size();

        for (y, row) in self.tiles.iter_mut().enumerate() {
            for (x, tile) in row.iter_mut().enumerate() {
                // if let Tile::Building(building) = &mut tile.ground {
                let pos: Position = (x as i16, y as i16).into();
                match &mut tile.ground {
                    Tile::Building(building) => {
                        *building = self
                            .buildings
                            .insert(Building::new_house(pos, DEFAULT_CITY_ID));
                    }
                    Tile::Road(road) => {
                        if let Some(station) = road.station {
                            if !self.buildings.hash_map.contains_key(&station) {
                                let dir = if x < size.0 as usize / 4 {
                                    Direction::RIGHT
                                } else if y < size.1 as usize / 4 {
                                    Direction::DOWN
                                } else if x > (size.0 as usize * 3) / 4 {
                                    Direction::LEFT
                                } else {
                                    Direction::UP
                                };
                                self.buildings.hash_map.insert(
                                    station,
                                    Building::new_spawner(
                                        pos,
                                        dir,
                                        SpawnerColors::from_number(station),
                                        DEFAULT_CITY_ID,
                                    ),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn new_from_string_layers(string: &str, bridge_layer: &str) -> Grid {
        let tiles: Vec<Vec<GridTile>> = string
            .split_ascii_whitespace()
            .zip(bridge_layer.split_ascii_whitespace())
            .map(|(line, bridge_line)| {
                line.chars()
                    .zip(bridge_line.chars())
                    .map(GridTile::new_from_char_layers)
                    .collect()
            })
            .collect();

        Grid {
            tiles,
            buildings: HashMapId::new(),
        }
    }

    pub fn get_tile(&self, pos: &Position) -> Option<&Tile> {
        self.tiles
            .get(pos.y as usize)?
            .get(pos.x as usize)?
            .get(pos.z)
    }

    pub fn get_tile_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        self.tiles
            .get_mut(pos.y as usize)?
            .get_mut(pos.x as usize)?
            .get_mut(pos.z)
    }

    pub fn reserve(
        &mut self,
        pos: &Position,
        vehicle_id: Id,
    ) -> Result<Reservation, ReservationError> {
        self.get_tile_mut(pos)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(vehicle_id, *pos)
    }

    pub fn find_path(&self, start: (Position, Direction), end: &Id) -> Path {
        let path_start = start.0 + start.1;

        if !self.get_tile(&path_start)?.is_road() {
            return None;
        }

        let end_building = self.buildings.hash_map.get(end)?;
        if matches!(end_building.building_type, BuildingType::House) {
            // append destination pos (this is already checked to be a road)
            let end_pos_dir = end_building.destination_pos(&self)?;
            let end_pos = end_pos_dir.0 + end_pos_dir.1;
            if let Some(mut path) = self.find_road_path(&path_start, &end_pos) {
                path.0.push(end_pos_dir.0);
                path.1 += 1;
                Some(path)
            } else {
                None
            }
        } else {
            self.find_path_to_building(&path_start, end, &end_building.pos)
        }
    }

    pub fn find_road_path(&self, start: &Position, end: &Position) -> Path {
        astar(
            start,
            |p| self.road_successors(p),
            |p| p.distance(end) / 3,
            |p| p == end,
        )
    }

    pub fn find_path_to_building(&self, start: &Position, end: &Id, end_approx: &Position) -> Path {
        astar(
            start,
            |p| self.road_successors(p),
            |p| p.distance(end_approx) / 3,
            |p| {
                if let Some(tile) = self.get_tile(p) {
                    tile.get_building_id() == Some(*end)
                } else {
                    false
                }
            },
        )
    }

    pub fn road_successors(&self, pos: &Position) -> Vec<(Position, u32)> {
        let tile = self.get_tile(pos);
        if tile.is_none() {
            return Vec::new();
        }
        let tile = tile.unwrap();
        tile.iter_connections(pos)
            .iter()
            .filter_map(|dir| {
                let new_pos = *pos + *dir;
                self.get_tile(&new_pos).map(|tile| (new_pos, tile.cost()))
            })
            .collect()
    }

    pub fn building_successors(&self, pos: &Position) -> Vec<(Position, u32)> {
        let tile = self.get_tile(pos);
        if tile.is_none() {
            return Vec::new();
        }
        let tile = tile.unwrap();
        tile.iter_connections(pos)
            .iter()
            .map(|dir| (*pos + *dir, 1))
            .collect()
    }

    pub fn building_successors_inverse(&self, pos: &Position) -> Vec<(Position, u32)> {
        let tile = self.get_tile(pos);
        if tile.is_none() {
            return Vec::new();
        }
        let tile = tile.unwrap();
        tile.iter_connections(pos)
            .iter()
            .map(|dir| (*pos + dir.inverse(), 1))
            .collect()
    }

    pub fn find_road(&self, start: &Position) -> Path {
        let tile = self.get_tile(start)?;
        if tile.is_road() {
            Some((vec![*start], 0))
        } else {
            dijkstra(
                start,
                |p| self.building_successors(p),
                |p| self.get_tile(p).is_some_and(Tile::is_road),
            )
        }
    }

    pub fn find_road_inverse(&self, start: &Position) -> Path {
        let tile = self.get_tile(start)?;
        if tile.is_road() {
            Some((vec![*start], 0))
        } else {
            dijkstra(
                start,
                |p| self.building_successors_inverse(p),
                |p| self.get_tile(p).is_some_and(Tile::is_road),
            )
        }
    }

    pub fn should_be_yielded_to(
        &self,
        should_yield: YieldType,
        pos: &Position,
        dir_from: Direction,
    ) -> bool {
        self.get_tile(pos)
            .is_some_and(|tile| tile.should_be_yielded_to(should_yield, dir_from))
    }

    pub fn should_we_yield_when_entering(
        &self,
        should_yield: YieldType,
        position: &Position,
    ) -> Option<Position> {
        // never yield from an intersection
        if should_yield == YieldType::Never {
            return None;
        }

        if let Some(Tile::Road(road)) = self.get_tile(position) {
            // For each direction that feeds into this tile in question
            for dir in Direction::ALL
                .iter()
                .filter(|&dir| !road.is_connected(*dir))
            {
                let yield_to_pos = *position + *dir;
                if self.should_be_yielded_to(should_yield, &yield_to_pos, *dir) {
                    return Some(yield_to_pos);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn test_new() {
        let grid = Grid::new_from_string(">>>");
        assert_eq!(
            *grid.get_tile(&grid.pos(0, 0)).unwrap(),
            Tile::new_from_char('>')
        );
    }

    #[test]
    fn test_successors() {
        let grid = Grid::new_from_string(
            "___
             >>>
             ___",
        );
        let suc = grid.road_successors(&grid.pos(1, 1));
        assert_eq!(suc, vec![(grid.pos(2, 1), 1)]);
    }

    #[test]
    fn test_path() {
        let grid = Grid::new_from_string(">>1");

        let path = grid
            .find_path((grid.pos(0, 0), Direction::RIGHT), &1)
            .unwrap();
        assert_eq!(path.0, vec![grid.pos(1, 0), grid.pos(2, 0)]);
        assert_eq!(path.1, 1);
    }

    #[test]
    fn test_path_fail() {
        let grid = Grid::new_from_string("<<1");

        assert!(grid
            .find_path((grid.pos(0, 0), Direction::RIGHT), &1)
            .is_none());
    }

    #[test]
    fn test_path_house() {
        let grid = Grid::new_from_string(
            "__<<h_
             _h>>__",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &1)
                .unwrap(),
            ((2..5).map(|i| grid.pos(i, 1)).collect(), 2)
        );
    }

    #[test]
    fn test_path_building_fail() {
        // let grid = Grid::new_from_string("_h>>h_");

        // assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(3, 0)).is_none());
        // assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(5, 0)).is_none());
    }

    #[test]
    fn test_path_dead_end() {
        let grid = Grid::new_from_string(
            "h_*<
             __>*",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &1)
                .unwrap(),
            (
                vec![
                    grid.pos(2, 1),
                    grid.pos(3, 1),
                    grid.pos(3, 0),
                    grid.pos(2, 0),
                    grid.pos(1, 0),
                ],
                4
            )
        );
    }

    #[test]
    fn test_path_station() {
        let grid = Grid::new_from_string(
            "11<<22
             11>>22",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &2)
                .unwrap(),
            ((2..5).map(|i| grid.pos(i, 1)).collect(), 2)
        );
    }
}
