use pathfinding::prelude::{astar, dijkstra};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::building::Building;
use super::tile::{Reservation, Tile, YieldType};
use super::{BuildingHashMap, Direction, Position};
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

        let size = grid.size();

        // fixup buildings
        // Fixup spawners
        for (y, row) in grid.tiles.iter_mut().enumerate() {
            for (x, tile) in row.iter_mut().enumerate() {
                if let Tile::Road(road) = &mut tile.ground {
                    if let Some(station) = road.station {
                        // oofy woofy
                        let pos: Position = (x as i16, y as i16).into();
                        road.connect(pos.default_connections()[0]);
                        road.connect(pos.default_connections()[1]);

                        if !grid.buildings.hash_map.contains_key(&station) {
                            let dir = if x < size.0 as usize / 4 {
                                Direction::RIGHT
                            } else if y < size.1 as usize / 4 {
                                Direction::DOWN
                            } else if x > (size.0 as usize * 3) / 4 {
                                Direction::LEFT
                            } else {
                                Direction::UP
                            };
                            grid.buildings.hash_map.insert(
                                station,
                                Building::new_spawner(
                                    pos,
                                    dir,
                                    SpawnerColors::from_number(station),
                                    1,
                                ),
                            );
                        }
                    }
                }
            }
        }

        grid
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

    pub fn find_path(&self, start: &Position, end: &Position) -> Path {
        let start_path = self.find_road(start)?;
        let mut end_path = self.find_road(end)?;
        let middle_path = self.find_road_path(
            start_path.0.last().unwrap_or(start),
            end_path.0.last().unwrap_or(end),
        )?;

        // start_path + middle_path + end_path
        let mut full_path = Vec::new();
        full_path.extend(&start_path.0[0..start_path.0.len() - 1]);
        full_path.extend(&middle_path.0);

        end_path.0.pop();
        full_path.extend(end_path.0.iter().rev());

        let full_cost: u32 = start_path.1 + middle_path.1 + end_path.1;

        Some((full_path, full_cost))
    }

    pub fn find_road_path(&self, start: &Position, end: &Position) -> Path {
        astar(
            start,
            |p| self.road_successors(p),
            |p| p.distance(end) / 3,
            |p| p == end,
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
        let grid = Grid::new_from_string(">>>");

        let path = grid.find_path(&grid.pos(0, 0), &grid.pos(2, 0)).unwrap();
        assert_eq!(path.0, vec![grid.pos(0, 0), grid.pos(1, 0), grid.pos(2, 0)]);
        assert_eq!(path.1, 2);
    }

    #[test]
    fn test_path_fail() {
        let grid = Grid::new_from_string("<<<");

        assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(2, 0)).is_none());
    }

    #[test]
    #[ignore = "house end pathing needs to be inverted"]
    fn test_path_building() {
        let grid = Grid::new_from_string("hh<<hh\nhh>>hh");

        assert_eq!(
            grid.find_path(&grid.pos(0, 1), &grid.pos(5, 1)).unwrap(),
            ((0..6).map(|i| grid.pos(i, 1)).collect(), 5)
        );
    }

    #[test]
    fn test_path_building_fail() {
        let grid = Grid::new_from_string("_h>>h_");

        assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(3, 0)).is_none());
        assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(5, 0)).is_none());
    }

    #[test]
    fn test_path_dead_end() {
        let grid = Grid::new_from_string("*<\n>*");

        assert_eq!(
            grid.find_path(&grid.pos(0, 1), &grid.pos(0, 0)).unwrap(),
            (
                vec![
                    grid.pos(0, 1),
                    grid.pos(1, 1),
                    grid.pos(1, 0),
                    grid.pos(0, 0)
                ],
                3
            )
        );
    }
}
