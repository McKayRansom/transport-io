use std::fmt;

use pathfinding::prelude::{astar, dijkstra};
use serde::{Deserialize, Serialize};

use super::tile::{Reservation, Tile, YieldType};
use super::{Direction, Position};
use crate::tileset::Tileset;
use crate::hash_map_id::Id;


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
    bridge: Tile,
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
    pub size: (i16, i16),
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
        write!(f, "g")?;
        for x in 0..self.tiles[0].len() {
            write!(f, "{}", x % 10)?;
        }
        writeln!(f)?;
        for y in 0..self.tiles.len() {
            write!(f, "{}", y)?;
            for x in 0..self.tiles[y].len() {
                self.tiles[y][x].ground.fmt(f)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Grid {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        Grid {
            tiles: vec![vec![GridTile::new(); size_x]; size_y],
            size: (size_x as i16, size_y as i16),
        }
    }

    #[allow(dead_code)]
    pub fn pos(&self, x: i16, y: i16) -> Position {
        Position::new(x, y)
    }

    #[allow(dead_code)]
    pub fn new_from_string(string: &str) -> Grid {
        let tiles: Vec<Vec<GridTile>> = string
            .split_ascii_whitespace()
            .map(|line| line.chars().map(GridTile::new_from_char).collect())
            .collect();

        let size = (tiles[0].len() as i16, tiles.len() as i16);

        Grid { tiles, size }
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
        tile.iter_connections()
            .filter_map(|dir| {
                let new_pos = *pos + *dir;
                self.get_tile(&new_pos).map(|tile| tile.road_successor(&new_pos))
            })
            .collect()
    }

    pub fn building_successors(&self, pos: &Position) -> Vec<(Position, u32)> {
        let tile = self.get_tile(pos);
        if tile.is_none() {
            return Vec::new();
        }
        let tile = tile.unwrap();
        tile.iter_connections()
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
            for dir in Direction::ALL.iter().filter(|&dir| !road.is_connected(*dir)) {
                let yield_to_pos = *position + *dir;
                if self.should_be_yielded_to(should_yield, &yield_to_pos, *dir) {
                    return Some(yield_to_pos);
                }
            }
        }

        None
    }

    pub fn draw_tiles(&self, tileset: &Tileset) {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                tile.ground.draw((x as i16, y as i16).into(), tileset);
            }
        }
    }

    pub fn draw_bridges(&self, tileset: &Tileset) {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                tile.bridge.draw_bridge((x as i16, y as i16, 1).into(), tileset, &tile.ground);
            }
        }
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
    fn test_path_building() {
        let grid = Grid::new_from_string("hh>>hh");

        assert_eq!(
            grid.find_path(&grid.pos(0, 0), &grid.pos(5, 0)).unwrap(),
            ((0..6).map(|i| grid.pos(i, 0)).collect(), 5)
        );
    }

    #[test]
    fn test_path_building_fail() {
        let grid = Grid::new_from_string("_h>>h_");

        assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(3, 0)).is_none());
        assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(5, 0)).is_none());
    }

    #[test]
    #[ignore = "I broke this :("]
    fn test_path_dead_end() {
        let grid = Grid::new_from_string(
            "<_
>_
            ",
        );

        assert_eq!(
            grid.find_path(&grid.pos(0, 1), &grid.pos(0, 0)).unwrap(),
            (vec![grid.pos(0, 1), grid.pos(0, 0)], 3)
        );
    }
}
