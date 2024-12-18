// use pathfinding::num_traits::Integer;
use std::fmt;

mod position;
pub use position::*;
mod direction;
pub use direction::*;
mod connections;
pub use connections::*;

use macroquad::math::Rect;
use pathfinding::prelude::astar;

use crate::tile::{House, Road, Tile};
use crate::tileset::Tileset;

pub type Id = u64;

const DEFAULT_COST: u32 = 1;
const OCCUPIED_COST: u32 = 2;

// const EMPTY_ROAD_COLOR: Color = Color::new(0.3, 0.3, 0.3, 0.5);
// const EMPTY_ROAD_COLOR: Color = WHITE;
// const RESERVED_PATH_COLOR: Color = Color::new(1.0, 0.1, 0.0, 0.3);
// const CONNECTION_INDICATOR_COLOR: Color = Color::new(0.7, 0.7, 0.7, 0.7);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
pub const GRID_CELL_SIZE: (f32, f32) = (32., 32.);


type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;



#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GridTile {
    pub ground: Option<Tile>,
    bridge: Option<Tile>,
}

impl GridTile {
    fn new() -> Self {
        GridTile {
            ground: Some(Tile::new()),
            bridge: None,
        }
    }
}
pub struct Grid {
    pub tiles: Vec<Vec<GridTile>>,
}

impl Position {
    fn distance(&self, other: &Position) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReservationStatus {
    TileInvalid,
    TileReserved,
    TileSuccess,
    // TileDoNotBlock,
}



impl fmt::Debug for Grid  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "g")?;
        for x in 0..self.tiles[0].len() {
            write!(f, "{}", x % 10)?;
        }
        write!(f, "\n")?;
        for y in 0..self.tiles.len() {
            write!(f, "{}", y)?;
            for x in 0..self.tiles[y].len() {
                match self.tiles[y][x].ground {
                    Some(Tile::Empty) => write!(f, "e")?,
                    Some(Tile::Road(road)) => {
                        road.fmt(f)?
                    }
                    Some(Tile::House(_)) => write!(f, "h")?,
                    None => write!(f, "b")?,
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Grid {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        Grid {
            tiles: vec![vec![GridTile::new(); size_x as usize]; size_y as usize],
        }
    }

    #[allow(dead_code)]
    pub fn new_from_string(string: &str) -> Grid {
        let mut pos = Position::new(0, 0);
        let size_x = string.find('\n').unwrap_or(string.len());
        let size_y = string.lines().count();
        println!("size_x: {}, size_y: {}", size_x, size_y);
        let mut grid = Grid::new(size_x, size_y);
        for chr in string.chars() {
            match chr {
                '>' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                }
                '<' => {
                    grid.add_tile_connection(&pos, Direction::Left);
                }
                '^' => {
                    grid.add_tile_connection(&pos, Direction::Up);
                }
                '.' => {
                    grid.add_tile_connection(&pos, Direction::Down);
                }
                'y' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                    if let Tile::Road(road) = grid.get_tile_mut(&pos).unwrap() {
                        road.should_yield = true;
                    }
                }
                'h' => {
                    *grid.get_tile_mut(&pos).unwrap() = Tile::House(House {
                        vehicle_on_the_way: None,
                    });
                }
                '_' => {
                    *grid.get_tile_mut(&pos).unwrap() = Tile::Empty;
                }
                // Roundabouts - top left
                'l' => {
                    grid.add_tile_connection(&pos, Direction::Left);
                    grid.add_tile_connection(&pos, Direction::Down);
                }
                // Roundabouts - top right
                'r' => {
                    grid.add_tile_connection(&pos, Direction::Left);
                    grid.add_tile_connection(&pos, Direction::Up);
                }
                // Roundabouts - bottom Left
                'L' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                    grid.add_tile_connection(&pos, Direction::Down);
                }
                // Roundabouts - bottom Right
                'R' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                    grid.add_tile_connection(&pos, Direction::Up);
                }

                '\n' => {
                    pos.y += 1;
                    pos.x = -1;
                }
                ' ' => {
                    pos.x -= 1;
                }
                _ => {}
            }
            pos.x += 1;
        }
        grid
    }

    pub fn get_tile(&self, pos: &Position) -> Option<&Tile> {
        if let Some(grid_row) = self.tiles.get(pos.y as usize) {
            if let Some(tile) = grid_row.get(pos.x as usize) {
                if pos.z == 0 {
                    tile.ground.as_ref()
                } else {
                    tile.bridge.as_ref()
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        if let Some(grid_row) = self.tiles.get_mut(pos.y as usize) {
            if let Some(tile) = grid_row.get_mut(pos.x as usize) {
                if pos.z == 0 {
                    tile.ground.as_mut()
                } else {
                    tile.bridge.as_mut()
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn add_tile_connection(&mut self, pos: &Position, dir: Direction) {
        if let Some(tile) = self.get_tile_mut(pos) {
            if let Tile::Road(road) = tile {
                road.connections.add(ConnectionLayer::Road, dir);
            } else {
                *tile = Tile::Road(Road::new(dir));
            }
        }
    }

    pub fn remove_tile_connection(&mut self, pos: &Position, dir: Direction) {
        if let Some(Tile::Road(road)) = self.get_tile_mut(pos) {
            road.connections.remove(dir);
            if road.connections.count() == 0 {
                self.clear_tile(pos);
            }
        }
    }

    pub fn clear_tile(&mut self, pos: &Position) {
        if let Some(tile) = self.get_tile_mut(pos) {
            *tile = Tile::Empty;
        }
    }

    pub fn _is_driveable(&self, pos: &Position) -> bool {
        if let Some(Tile::Road(road)) = self.get_tile(pos) {
            !road.reserved.is_some()
        } else {
            false
        }
    }

    pub fn reserve_position(&mut self, pos: &Position, id: Id) -> ReservationStatus {
        match self.get_tile_mut(pos) {
            Some(Tile::Road(road)) => {
                if road.reserved.is_some() /* TODO: Add check for intersection full */ {
                    ReservationStatus::TileReserved
                // } else if road.connections.safe_to_block() {
                    // road.reserved = true;
                    // ReservationStatus::TileBlockable
                } else {
                    road.reserved = Some(id);
                    ReservationStatus::TileSuccess
                    // ReservationStatus::TileDoNotBlock
                }
            }
            Some(Tile::House(_)) => ReservationStatus::TileSuccess,
            Some(Tile::Empty) => ReservationStatus::TileInvalid,
            None => ReservationStatus::TileInvalid,
        }
    }

    pub fn unreserve_position(&mut self, pos: &Position) {
        if let Some(Tile::Road(road)) = self.get_tile_mut(&pos) {
            road.reserved = None;
        }
    }

    pub fn find_path(&self, start: &Position, end: &Position) -> Path {
        let result = astar(
            start,
            |p| self.successors(p),
            |p| p.distance(end) / 3,
            |p| p == end,
        );

        result
    }

    fn successors(&self, pos: &Position) -> Vec<(Position, u32)> {
        if let Some(tile) = self.get_tile(pos) {
            tile.iter_connections()
                .map(|dir| {
                    let new_pos = Position::new_from_move(pos, dir);
                    (
                        new_pos,
                        if let Some(tile) = self.get_tile(&new_pos) {
                            match tile {
                                Tile::Road(road) => {
                                    if road.reserved.is_some() {
                                        OCCUPIED_COST
                                    } else {
                                        DEFAULT_COST
                                    }
                                }
                                Tile::House(_) => DEFAULT_COST * 2,
                                Tile::Empty => DEFAULT_COST * 3,
                            }
                        } else {
                            DEFAULT_COST * 3
                        },
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn draw_tiles(&self, tileset: &Tileset) {
        for y in 0..self.tiles.len() {
            for x in 0..self.tiles[y].len() {
                let pos = Position::new(x as i16, y as i16);
                if let Some(tile) = self.get_tile(&pos) {
                    tile.draw(pos, tileset);
                }
            }
        }
    }

    pub fn draw_houses(&self, tileset: &Tileset) {
        for y in 0..self.tiles.len() {
            for x in 0..self.tiles[y].len() {
                let pos = Position::new(x as i16, y as i16);
                if let Some(Tile::House(house)) = self.get_tile(&pos) {
                    house.draw(&Rect::from(pos), tileset);
                }
            }
        }
    }

    // pub fn should_yield(&self, pos: &Position) -> bool {
    //     if let Some(tile) = self.get_tile(pos) {
    //         tile.should_yield()
    //     } else {
    //         false
    //     }
    // }
}
