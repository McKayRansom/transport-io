// use pathfinding::num_traits::Integer;
use std::f32::consts::PI;
use std::fmt;

use macroquad::color::{Color, WHITE};
use macroquad::input::KeyCode;
use macroquad::math::Rect;
use pathfinding::num_traits::AsPrimitive;
use pathfinding::prelude::astar;

use crate::tileset::Tileset;

const DEFAULT_COST: u32 = 2;
const OCCUPIED_COST: u32 = 3;

const CONNECTIONS_ALL: u32 = 0b1111;

// const EMPTY_ROAD_COLOR: Color = Color::new(0.3, 0.3, 0.3, 0.5);
// const EMPTY_ROAD_COLOR: Color = WHITE;
const RESERVED_PATH_COLOR: Color = Color::new(1.0, 0.1, 0.0, 0.3);
// const CONNECTION_INDICATOR_COLOR: Color = Color::new(0.7, 0.7, 0.7, 0.7);

const HOUSE_SPRITE: u32 = (16 * 1) + 0;

const ROAD_INTERSECTION_SPRITE: u32 = (16 * 3) + 0;
const ROAD_ARROW_SPRITE: u32 = (16 * 3) + 1;
const ROAD_STRAIGHT_SPRITE: u32 = (16 * 3) + 2;

// Now we define the pixel size of each tile, which we make 32x32 pixels.
pub const GRID_CELL_SIZE: (f32, f32) = (32., 32.);

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Hash)]
pub struct Position {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        let z = 0;
        Position { x, y, z }
    }

    pub fn _new_z(x: i16, y: i16, z: i16) -> Self {
        Position { x, y, z }
    }

    pub fn from_screen(screen_pos: (f32, f32), camera_pos: (f32, f32), zoom: f32) -> Self {
        Position::new(
            ((camera_pos.0 + (screen_pos.0 / zoom)) / GRID_CELL_SIZE.0) as i16,
            ((camera_pos.1 + (screen_pos.1 / zoom)) / GRID_CELL_SIZE.1) as i16,
        )
    }

    pub fn new_from_move(pos: &Position, dir: Direction) -> Self {
        match dir {
            Direction::Up => Position::new(pos.x, pos.y - 1),
            Direction::Down => Position::new(pos.x, pos.y + 1),
            Direction::Left => Position::new(pos.x - 1, pos.y),
            Direction::Right => Position::new(pos.x + 1, pos.y),
        }
    }
}

impl From<Position> for Rect {
    fn from(pos: Position) -> Self {
        Rect::new(
            pos.x as f32 * GRID_CELL_SIZE.0,
            pos.y as f32 * GRID_CELL_SIZE.1,
            GRID_CELL_SIZE.0 as f32,
            GRID_CELL_SIZE.1 as f32,
        )
    }
}

/// And here we implement `From` again to allow us to easily convert between
/// `(i16, i16)` and a `GridPosition`.
impl<T> From<(T, T)> for Position
    where 
        T: AsPrimitive<i16>
{
    fn from(pos: (T, T)) -> Self {
        Position::new(pos.0.as_(), pos.1.as_())
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
    pub fn inverse(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn rotate_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Right => Direction::Up,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
        }
    }

    pub fn _rotate(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn from_position(prev_pos: Position, new_pos: Position) -> Self {
        if new_pos.x > prev_pos.x {
            Direction::Right
        } else if new_pos.y > prev_pos.y {
            Direction::Down
        } else if new_pos.y < prev_pos.y {
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

pub enum ConnectionLayer {
    Road = 0,
    Driveway = 1,
    // Bridge = 2,
}

pub struct ConnectionsIterator {
    connection_bitfield: u32,
}

impl ConnectionsIterator {
    pub fn all_directions() -> Self {
        ConnectionsIterator {
            connection_bitfield: CONNECTIONS_ALL,
        }
    }

    pub fn no_directions() -> Self {
        ConnectionsIterator {
            connection_bitfield: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Connections {
    connection_bitfield: u32,
}

const LAYER_SIZE: u32 = 4;
const LAYER_MASK: u32 = 0b1111;

impl Connections {
    pub fn new(layer: ConnectionLayer, dir: Direction) -> Connections {
        Connections {
            connection_bitfield: (dir as u32) << ((layer as u32) * LAYER_SIZE),
        }
    }

    pub fn add(&mut self, layer: ConnectionLayer, dir: Direction) {
        self.connection_bitfield |= (dir as u32) << layer as u32 * LAYER_SIZE;
    }

    pub fn remove(&mut self, dir: Direction) {
        self.connection_bitfield &= !(dir as u32);
    }

    pub fn count(&self) -> u32 {
        (self.connection_bitfield & LAYER_MASK).count_ones()
    }

    pub fn safe_to_block(&self) -> bool {
        // Don't block intersections!
        // but only for real road intersections
        self.count() < 2
        // true
    }

    pub fn iter_layer(&self, layer: ConnectionLayer) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: (self.connection_bitfield >> (layer as u32 * LAYER_SIZE))
                & LAYER_MASK,
        }
    }

    pub fn iter(&self) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: self.connection_bitfield,
        }
    }

    pub fn iter_inverse(&self, layer: ConnectionLayer) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: (!self.connection_bitfield & LAYER_MASK << (LAYER_SIZE & layer as u32)),
        }
    }

    pub fn has(&self, dir: Direction) -> bool {
        (self.connection_bitfield & dir as u32) != 0
    }
}

impl fmt::Debug for Connections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.connection_bitfield == Direction::Left as u32 {
            write!(f, "<")
        }
        else if self.connection_bitfield == Direction::Right as u32 {
            write!(f, ">")
        }
        else if self.connection_bitfield == Direction::Up as u32 {
            write!(f, "^")
        }
        else if self.connection_bitfield == Direction::Down as u32 {
            write!(f, ".")
        }
        else if self.connection_bitfield == (Direction::Up as u32 | Direction::Left as u32) {
            write!(f, "r")
        }
        else if self.connection_bitfield == (Direction::Down as u32 | Direction::Left as u32) {
            write!(f, "l")
        }
        else if self.connection_bitfield == (Direction::Down as u32 | Direction::Right as u32) {
            write!(f, "L")
        }
        else if self.connection_bitfield == (Direction::Up as u32 | Direction::Right as u32) {
            write!(f, "R")
        }
        else {
            write!(f, "?")
        }
    }
}

impl Iterator for ConnectionsIterator {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.connection_bitfield & Direction::Up as u32 != 0 {
            self.connection_bitfield -= Direction::Up as u32;
            Some(Direction::Up)
        } else if self.connection_bitfield & Direction::Down as u32 != 0 {
            self.connection_bitfield -= Direction::Down as u32;
            Some(Direction::Down)
        } else if self.connection_bitfield & Direction::Right as u32 != 0 {
            self.connection_bitfield -= Direction::Right as u32;
            Some(Direction::Right)
        } else if self.connection_bitfield & Direction::Left as u32 != 0 {
            self.connection_bitfield -= Direction::Left as u32;
            Some(Direction::Left)
        } else if self.connection_bitfield != 0 {
            self.connection_bitfield = self.connection_bitfield >> LAYER_SIZE;
            self.next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod connections_tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(Connections::new(ConnectionLayer::Road, Direction::Right).count() == 1);
    }

    #[test]
    fn test_iter() {
        let mut connection = Connections::new(ConnectionLayer::Road, Direction::Right);
        connection.add(ConnectionLayer::Road, Direction::Left);
        assert!(
            connection.iter().collect::<Vec<Direction>>()
                == vec![Direction::Right, Direction::Left]
        );

        assert!(connection.safe_to_block() == false);
    }

    #[test]
    fn test_layer() {
        let mut connection = Connections::new(ConnectionLayer::Driveway, Direction::Right);
        connection.add(ConnectionLayer::Road, Direction::Left);
        assert!(
            connection.iter().collect::<Vec<Direction>>()
                == vec![Direction::Left, Direction::Right]
        );
        assert!(
            connection
                .iter_layer(ConnectionLayer::Driveway)
                .collect::<Vec<Direction>>()
                == vec![Direction::Right]
        );

        assert!(connection.count() == 1);
        assert!(connection.safe_to_block() == true);
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Road {
    pub should_yield: bool,
    pub reserved: bool,
    pub connections: Connections,
}

impl Road {
    pub fn new(dir: Direction) -> Road {
        Road {
            should_yield: false,
            reserved: false,
            connections: Connections::new(ConnectionLayer::Road, dir),
        }
    }

    // pub fn should_yield(&self) -> bool {
    //     return self.connections.count() < 2;
    // }

    fn draw(&self, rect: &Rect, tileset: &Tileset) {
        let connection_count = self.connections.count();

        if connection_count != 1 {
            tileset.draw_tile(ROAD_INTERSECTION_SPRITE, WHITE, &rect, 0.0);
        }

        for dir in self.connections.iter_layer(ConnectionLayer::Road) {
            if connection_count == 1 {
                let sprite = if self.should_yield {
                    ROAD_STRAIGHT_SPRITE + 2
                } else {
                    ROAD_STRAIGHT_SPRITE
                };
                tileset.draw_tile(sprite, WHITE, &rect, dir.to_radians());
            } else {
                tileset.draw_tile(ROAD_ARROW_SPRITE, WHITE, &rect, dir.to_radians());
            }
        }

        // if self.reserved {
        //     tileset.draw_rect(&rect, RESERVED_PATH_COLOR);
        // }
    }
}

impl fmt::Debug for Road {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.reserved {
            write!(f, "o")
        }
        else {
            self.connections.fmt(f)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct House {
    pub people_heading_to: bool,
}

impl House {
    fn draw(&self, rect: &Rect, tileset: &Tileset) {
        let color = if self.people_heading_to {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            WHITE
        };
        tileset.draw_tile(HOUSE_SPRITE, color, &rect, 0.0);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    House(House),
    Road(Road),
}

impl Tile {
    fn new() -> Tile {
        Tile::Empty
    }

    fn iter_connections(&self) -> ConnectionsIterator {
        match self {
            Tile::Road(road) => road.connections.iter(),
            Tile::House(_) => ConnectionsIterator::all_directions(),
            Tile::Empty => ConnectionsIterator::no_directions(),
        }
    }
    fn draw(&self, pos: Position, tileset: &Tileset) {
        let rect = Rect::from(pos);

        match self {
            Tile::Road(road) => road.draw(&rect, tileset),
            _ => {}
        }
    }

    // pub fn should_yield(&self) -> bool {
    //     match self {
    //         Tile::Road(road) => road.should_yield(),
    //         Tile::House(_) => true,
    //         _ => true,
    //     }
    // }
}

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
    TileBlockable,
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
    pub fn new_grid_from_ascii(ascii: &str) -> Grid {
        let mut pos = Position::new(0, 0);
        let size_x = ascii.find('\n').unwrap_or(ascii.len());
        let size_y = ascii.lines().count();
        println!("size_x: {}, size_y: {}", size_x, size_y);
        let mut grid = Grid::new(size_x, size_y);
        for chr in ascii.chars() {
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
                        people_heading_to: true,
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
            !road.reserved
        } else {
            false
        }
    }

    pub fn reserve_position(&mut self, pos: &Position) -> ReservationStatus {
        match self.get_tile_mut(pos) {
            Some(Tile::Road(road)) => {
                if road.reserved /* TODO: Add check for intersection full */ {
                    ReservationStatus::TileReserved
                // } else if road.connections.safe_to_block() {
                    // road.reserved = true;
                    // ReservationStatus::TileBlockable
                } else {
                    road.reserved = true;
                    ReservationStatus::TileBlockable
                    // ReservationStatus::TileDoNotBlock
                }
            }
            Some(Tile::House(_)) => ReservationStatus::TileBlockable,
            Some(Tile::Empty) => ReservationStatus::TileInvalid,
            None => ReservationStatus::TileInvalid,
        }
    }

    pub fn unreserve_position(&mut self, pos: &Position) {
        if let Some(Tile::Road(road)) = self.get_tile_mut(&pos) {
            road.reserved = false;
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
                                    if road.reserved {
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
