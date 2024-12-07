use std::f32::consts::PI;

use macroquad::color::{Color, WHITE};
use macroquad::input::KeyCode;
use macroquad::math::Rect;
use macroquad::shapes::draw_rectangle;
use pathfinding::prelude::astar;

use crate::tileset::Tileset;

const DEFAULT_COST: u32 = 2;
const OCCUPIED_COST: u32 = 3;

// const EMPTY_ROAD_COLOR: Color = Color::new(0.3, 0.3, 0.3, 0.5);
const EMPTY_ROAD_COLOR: Color = WHITE;
const RESERVED_PATH_COLOR: Color = Color::new(1.0, 0.1, 0.0, 0.3);
// const CONNECTION_INDICATOR_COLOR: Color = Color::new(0.7, 0.7, 0.7, 0.7);

const ROAD_INTERSECTION_SPRITE: u32 = (16 * 3) + 0;
const ROAD_ARROW_SPRITE: u32 = (16 * 3) + 1;
const ROAD_STRAIGHT_SPRITE: u32 = (16 * 3) + 2;

// Here we define the size of our game board in terms of how many grid
// cells it will take up. We choose to make a 30 x 20 game board.
pub const GRID_SIZE: (i16, i16) = (30, 20);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
pub const GRID_CELL_SIZE: (f32, f32) = (32., 32.);

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Hash)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Position { x, y }
    }

    pub fn from_screen(x: f32, y: f32) -> Self {
        Position {
            x: x as i16 / GRID_CELL_SIZE.0 as i16,
            y: y as i16 / GRID_CELL_SIZE.1 as i16,
        }
    }

    pub fn _valid(&self) -> bool {
        self.x > 0 && self.y > 0 && self.x < GRID_SIZE.0 && self.y < GRID_SIZE.1
    }

    /// We'll make another helper function that takes one grid position and returns a new one after
    /// making one move in the direction of `dir`.
    /// We use the [`rem_euclid()`](https://doc.rust-lang.org/std/primitive.i16.html#method.rem_euclid)
    /// API when crossing the top/left limits, as the standard remainder function (`%`) returns a
    /// negative value when the left operand is negative.
    /// Only the Up/Left cases require rem_euclid(); for consistency, it's used for all of them.
    pub fn _new_from_move(pos: Position, dir: Direction) -> Self {
        match dir {
            Direction::Up => Position::new(pos.x, (pos.y - 1).rem_euclid(GRID_SIZE.1)),
            Direction::Down => Position::new(pos.x, (pos.y + 1).rem_euclid(GRID_SIZE.1)),
            Direction::Left => Position::new((pos.x - 1).rem_euclid(GRID_SIZE.0), pos.y),
            Direction::Right => Position::new((pos.x + 1).rem_euclid(GRID_SIZE.0), pos.y),
        }
    }
}

// TODO: Merge with macroquad::math::Rect
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rectangle { x, y, w, h }
    }

    pub fn from(&self) -> Rect {
        Rect {
            x: self.x, y: self.y, w: self.w, h: self.h,
        }
    }

    pub fn from_pos(pos: Position) -> Self {
        Rectangle::new(
            (pos.x as f32 * GRID_CELL_SIZE.0) + (GRID_CELL_SIZE.0) / 2.0,
            (pos.y as f32 * GRID_CELL_SIZE.1) + (GRID_CELL_SIZE.1) / 2.0,
            GRID_CELL_SIZE.0 as f32,
            GRID_CELL_SIZE.1 as f32,
        )
    }

    pub fn draw(&self, color: Color) {
        draw_rectangle(self.x, self.y, self.w, self.h, color);
    }
}

/// And here we implement `From` again to allow us to easily convert between
/// `(i16, i16)` and a `GridPosition`.
impl From<(i16, i16)> for Position {
    fn from(pos: (i16, i16)) -> Self {
        Position { x: pos.0, y: pos.1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up = 1,
    Down = 2,
    Left = 4,
    Right = 8,
}

impl Direction {
    pub fn _inverse(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn rotate(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn from_position(pos1: Position, pos2: Position) -> Self {
        if pos2.x > pos1.x {
            Direction::Right
        } else if pos2.y > pos1.y {
            Direction::Down
        } else if pos2.y < pos1.y {
            Direction::Up
        } else {
            Direction::Left
        }
    }

    pub fn to_radians(self) -> f32 {
        match self {
            Direction::Up => 0.,
            Direction::Right => PI / 2.0,
            Direction::Down => PI,
            Direction::Left => PI * 1.5,
        } 
    }

    pub fn _from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;


pub struct TileDirIter {
    connections: u32,
}

impl Iterator for TileDirIter {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.connections & Direction::Up as u32 != 0 {
            self.connections -= Direction::Up as u32;
            Some(Direction::Up)
        } else if self.connections & Direction::Down as u32 != 0 {
            self.connections -= Direction::Down as u32;
            Some(Direction::Down)
        } else if self.connections & Direction::Right as u32 != 0 {
            self.connections -= Direction::Right as u32;
            Some(Direction::Right)
        } else if self.connections & Direction::Left as u32 != 0 {
            self.connections -= Direction::Left as u32;
            Some(Direction::Left)
        } else {
            None
        }
    }
}

pub struct PathTileIter {
    start_pos: Position,
    connections: u32,
}

impl Iterator for PathTileIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.connections & Direction::Up as u32 != 0 {
            self.connections -= Direction::Up as u32;
            Some(Position {
                x: self.start_pos.x,
                y: self.start_pos.y - 1,
            })
        } else if self.connections & Direction::Down as u32 != 0 {
            self.connections -= Direction::Down as u32;
            Some(Position {
                x: self.start_pos.x,
                y: self.start_pos.y + 1,
            })
        } else if self.connections & Direction::Right as u32 != 0 {
            self.connections -= Direction::Right as u32;
            Some(Position {
                x: self.start_pos.x + 1,
                y: self.start_pos.y,
            })
        } else if self.connections & Direction::Left as u32 != 0 {
            self.connections -= Direction::Left as u32;
            Some(Position {
                x: self.start_pos.x - 1,
                y: self.start_pos.y,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    allowed: bool,
    occupied: bool,
    connections: u32,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            allowed: false,
            occupied: false,
            connections: 0,
        }
    }

    fn connect(&mut self, dir: Direction) {
        self.connections |= dir as u32;
    }

    fn connections_count(&self) -> u32 {
        self.connections.count_ones()
    }

    fn connections_as_iter(&self, start_pos: Position) -> PathTileIter {
        PathTileIter {
            connections: self.connections,
            start_pos,
        }
    }

    fn directions_as_iter(&self) -> TileDirIter {
        TileDirIter {
            connections: self.connections
        }
    }

    fn draw(&self, pos: Position, tileset: &Tileset) {
        if !self.allowed {
            return;
        }

        let rect = Rectangle::from_pos(pos);

        // let color = if self.occupied {
        //     RESERVED_PATH_COLOR
        // } else {
        //     EMPTY_ROAD_COLOR
        // };

        let connection_count = self.connections_count();

        if connection_count != 1 {
            tileset.draw_tile(ROAD_INTERSECTION_SPRITE, WHITE, &rect, 0.0);
        }

        for dir in self.directions_as_iter() {
            if connection_count == 1 {
                tileset.draw_tile(ROAD_STRAIGHT_SPRITE, WHITE, &rect, dir.to_radians());
            }
            else {
                tileset.draw_tile(ROAD_ARROW_SPRITE, WHITE, &rect, dir.to_radians());
            }
        }
    }
}

pub struct Grid {
    tiles: Vec<Vec<Tile>>,
}

impl Position {
    fn distance(&self, other: &Position) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            tiles: vec![vec![Tile::new(); GRID_SIZE.1 as usize]; GRID_SIZE.0 as usize],
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
        self.tiles[pos.x as usize][pos.y as usize]
            .connections_as_iter(*pos)
            // .filter(|x| self.allowed.contains(x) && x.valid())
            .map(|p| {
                (
                    p,
                    if self.is_occupied(&p) {
                        OCCUPIED_COST
                    } else {
                        DEFAULT_COST
                    },
                )
            })
            .collect()
    }

    pub fn connection_count(&self, pos: &Position) -> u32 {
        self.tiles[pos.x as usize][pos.y as usize].connections_count()
    }

    pub fn is_allowed(&self, pos: &Position) -> bool {
        self.tiles[pos.x as usize][pos.y as usize].allowed
    }

    pub fn get_dirs(&self, pos: &Position) -> PathTileIter {
        self.tiles[pos.x as usize][pos.y as usize].connections_as_iter(*pos)
    }

    pub fn add_allowed(&mut self, pos: &Position, direction: Direction) {
        self.tiles[pos.x as usize][pos.y as usize].allowed = true;
        self.tiles[pos.x as usize][pos.y as usize].connect(direction);
    }

    pub fn remove_allowed(&mut self, pos: &Position) {
        self.tiles[pos.x as usize][pos.y as usize].allowed = false;
        self.tiles[pos.x as usize][pos.y as usize].connections = 0;
    }

    pub fn is_occupied(&self, pos: &Position) -> bool {
        self.tiles[pos.x as usize][pos.y as usize].occupied
    }

    pub fn add_occupied(&mut self, pos: &Position) {
        self.tiles[pos.x as usize][pos.y as usize].occupied = true
    }

    pub fn remove_occupied(&mut self, pos: &Position) {
        self.tiles[pos.x as usize][pos.y as usize].occupied = false
    }

    pub fn draw_tiles(&self, tileset: &Tileset) {
        for i in 0..GRID_SIZE.0 {
            for j in 0..GRID_SIZE.1 {
                let pos = Position::new(i, j);
                self.tiles[i as usize][j as usize].draw(pos, tileset);
            }
        }
    }
}
