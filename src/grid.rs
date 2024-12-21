use std::fmt;

mod position;
pub use position::*;
mod direction;
pub use direction::*;
mod build;

use macroquad::math::Rect;
use pathfinding::prelude::{astar, dijkstra};
use serde::{Deserialize, Serialize};

use crate::tile::{Tile, YieldType};
use crate::tileset::Tileset;

pub type Id = u64;

// const EMPTY_ROAD_COLOR: Color = Color::new(0.3, 0.3, 0.3, 0.5);
// const EMPTY_ROAD_COLOR: Color = WHITE;
// const RESERVED_PATH_COLOR: Color = Color::new(1.0, 0.1, 0.0, 0.3);
// const CONNECTION_INDICATOR_COLOR: Color = Color::new(0.7, 0.7, 0.7, 0.7);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
pub const GRID_CELL_SIZE: (f32, f32) = (32., 32.);

type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GridTile {
    ground: Tile,
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
}

#[derive(Serialize, Deserialize)]
pub struct Grid {
    tiles: Vec<Vec<GridTile>>,
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
                match &self.tiles[y][x].ground {
                    Tile::Empty => write!(f, "e")?,
                    Tile::Road(road) => road.fmt(f)?,
                    Tile::House(_) => write!(f, "h")?,
                    // => write!(f, "b")?,
                }
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
        Position {
            x: x.clamp(0, self.size.0 - 1),
            y: y.clamp(0, self.size.1 - 1),
            z: 0,
        }
    }

    pub fn try_pos(&self, x: i16, y: i16) -> Option<Position> {
        Position::new(x, y, self.size)
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

    pub fn get_tile(&self, pos: &Position) -> &Tile {
        if pos.z == 1 {
            &self.tiles[pos.y as usize][pos.x as usize].bridge
        } else {
            &self.tiles[pos.y as usize][pos.x as usize].ground
        }
    }

    pub fn get_tile_mut(&mut self, pos: &Position) -> &mut Tile {
        if pos.z == 1 {
            &mut self.tiles[pos.y as usize][pos.x as usize].bridge
        } else {
            &mut self.tiles[pos.y as usize][pos.x as usize].ground
        }
    }

    pub fn find_path(&self, start: &Position, end: &Position) -> Path {
        if let Some(start_path) = self.find_road(start) {
            if let Some(mut end_path) = self.find_road(end) {
                if let Some(middle_path) = self.find_road_path(
                    start_path.0.last().unwrap_or(start),
                    end_path.0.last().unwrap_or(end),
                ) {
                    // start_path + middle_path + end_path
                    let mut full_path = Vec::new();
                    full_path.extend(&start_path.0[0..start_path.0.len() - 1]);
                    full_path.extend(&middle_path.0);

                    end_path.0.pop();
                    full_path.extend(end_path.0.iter().rev());

                    let full_cost: u32 = (start_path.1 + middle_path.1 + end_path.1) - 2;

                    Some((full_path, full_cost))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
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

    pub fn road_successors(&self, pos: &Position) -> Vec<(Position, u32)> {
        let tile = self.get_tile(pos);
        tile.iter_connections()
            .filter_map(|dir| {
                Position::new_from_move(pos, dir, self.size)
                    .map(|new_pos| (new_pos, self.get_tile(&new_pos).cost()))
            })
            .collect()
    }

    pub fn find_road(&self, start: &Position) -> Path {
        if self.get_tile(start).is_road() {
            Some((vec![*start], 1))
        } else {
            dijkstra(
                start,
                |p| self.road_successors(p),
                |p| self.get_tile(p).is_road(),
            )
        }
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

        if let Tile::Road(road) = self.get_tile(position) {
            // For each direction that feeds into this tile in question
            for dir in road.iter_connections_inverse() {
                if let Some(yield_to_pos) = Position::new_from_move(position, dir, self.size) {
                    if self
                        .get_tile(&yield_to_pos)
                        .should_be_yielded_to(should_yield, dir)
                    {
                        return Some(yield_to_pos);
                    }
                }
            }
        }

        None
    }

    pub fn draw_tiles(&self, tileset: &Tileset) {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                tile.ground.draw(self.pos(x as i16, y as i16), tileset);
                tile.bridge
                    .draw_bridge(self.pos(x as i16, y as i16), tileset);
            }
        }
    }

    pub fn draw_houses(&self, tileset: &Tileset) {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if let Tile::House(house) = tile.ground {
                    house.draw(&Rect::from(self.pos(x as i16, y as i16)), tileset);
                }
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
        assert_eq!(*grid.get_tile(&grid.pos(0, 0)), Tile::new_from_char('>'));
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
    fn test_path_house() {
        let grid = Grid::new_from_string("hh>>hh");

        assert_eq!(
            grid.find_path(&grid.pos(0, 0), &grid.pos(5, 0)).unwrap(),
            ((0..6).map(|i| grid.pos(i, 0)).collect(), 5)
        );
    }

    #[test]
    fn test_path_house_fail() {
        let grid = Grid::new_from_string("_h>>h_");

        assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(3, 0)).is_none());
        assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(5, 0)).is_none());
    }
}
