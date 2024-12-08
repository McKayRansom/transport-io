use std::f32::consts::PI;
use std::fmt;

use macroquad::color::{Color, WHITE};
use macroquad::input::KeyCode;
use macroquad::math::Rect;
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

// Here we define the size of our game board in terms of how many grid
// cells it will take up. We choose to make a 30 x 20 game board.
pub const GRID_SIZE: (i16, i16) = (30, 30);
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

    pub fn from_screen(screen_pos: (f32, f32), camera_pos: (f32, f32), zoom: f32) -> Self {
        Position {
            x: ((camera_pos.0 + (screen_pos.0 / zoom)) / GRID_CELL_SIZE.0 ) as i16,
            y: ((camera_pos.1 + (screen_pos.1 / zoom)) / GRID_CELL_SIZE.1 ) as i16,
        }
    }

    pub fn _valid(&self) -> bool {
        self.x > 0 && self.y > 0 && self.x < GRID_SIZE.0 && self.y < GRID_SIZE.1
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

// LATER: Merge with macroquad::math::Rect

#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn _from(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        }
    }

    pub fn from_pos(pos: Position) -> Self {
        Rectangle::new(
            pos.x as f32 * GRID_CELL_SIZE.0,
            pos.y as f32 * GRID_CELL_SIZE.1,
            GRID_CELL_SIZE.0 as f32,
            GRID_CELL_SIZE.1 as f32,
        )
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
    pub fn inverse(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
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
}

pub struct ConnectionsIterator {
    connection_bitfield: u32,
}

impl ConnectionsIterator {
    pub fn all_directions() -> Self {
        ConnectionsIterator{connection_bitfield: CONNECTIONS_ALL}
    }

    pub fn no_directions() -> Self {
        ConnectionsIterator{connection_bitfield: 0}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn add(&mut self, layer:ConnectionLayer, dir: Direction) {
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
    }

    pub fn iter_layer(&self, layer: ConnectionLayer) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: (self.connection_bitfield >> (layer as u32 * LAYER_SIZE)) & LAYER_MASK,
        }
    }

    pub fn iter(&self) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: self.connection_bitfield,
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
        assert!(connection.iter().collect::<Vec<Direction>>() == vec![Direction::Right, Direction::Left]);

        assert!(connection.safe_to_block() == false);
    }

    #[test]
    fn test_layer() {

        let mut connection = Connections::new(ConnectionLayer::Driveway, Direction::Right);
        connection.add(ConnectionLayer::Road, Direction::Left);
        assert!(connection.iter().collect::<Vec<Direction>>() == vec![Direction::Left, Direction::Right]);
        assert!(connection.iter_layer(ConnectionLayer::Driveway).collect::<Vec<Direction>>() == vec![Direction::Right]);
        
        assert!(connection.count() == 1);
        assert!(connection.safe_to_block() == true);
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reservation {
    pub position: Position,
    pub start_tick: u32,
    pub end_tick: u32,
}

impl Reservation {
    pub fn new(position: Position, start_tick: u32, end_tick: u32) -> Self {
        Reservation {
            position,
            start_tick,
            end_tick,
        }
    }
}

pub struct ReservationInfo {
    pub later_reservation: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Reservations {
    reservations_bitfield: u32,
}

impl Reservations {
    pub fn new() -> Self {
        Reservations {
            reservations_bitfield: 0,
        }
    }

    fn ticks_to_bitfield(start_tick: u32, end_tick: u32) -> Option<u32> {
        let start_ticks_shl = 1_u32.checked_shl(start_tick).unwrap_or(0);
        let end_ticks_shl = 1_u32.checked_shl(end_tick).unwrap_or(0);

        if start_ticks_shl > 0 && end_ticks_shl > start_ticks_shl {
            Some((end_ticks_shl - 1) - (start_ticks_shl - 1))
        } else {
            None
        }
    }

    pub fn is_reserved(&self, start_tick: u32, end_tick: u32) -> bool {
        if let Some(ticks) = Self::ticks_to_bitfield(start_tick, end_tick) {
            self.reservations_bitfield & ticks != 0
        } else {
            true
        }
    }

    pub fn reserve(&mut self, start_tick: u32, end_tick: u32) -> Option<ReservationInfo> {
        if let Some(ticks) = Self::ticks_to_bitfield(start_tick, end_tick) {
            if ticks & self.reservations_bitfield != 0 {
                None
            } else if ticks < self.reservations_bitfield {
                self.reservations_bitfield |= ticks;
                Some(ReservationInfo {
                    later_reservation: true,
                })
            } else {
                self.reservations_bitfield |= ticks;
                Some(ReservationInfo {
                    later_reservation: false,
                })
            }
        } else {
            None
        }
    }

    pub fn unreserve(&mut self, start_tick: u32, end_tick: u32) {
        if let Some(ticks) = Self::ticks_to_bitfield(start_tick, end_tick) {
            self.reservations_bitfield &= !ticks;
        }
    }
}

impl fmt::Debug for Reservations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reservations: {:b}", self.reservations_bitfield)
    }
}

#[cfg(test)]
mod reservation_tests {
    use super::*;

    #[test]
    fn test_reserve() {
        let mut r = Reservations::new();

        assert!(r.reserve(0, 31).unwrap().later_reservation == false);
        assert!(r.is_reserved(0, 31) == true);
        assert!(r.is_reserved(0, 1) == true);

        let new_info = r.reserve(0, 31);
        assert!(new_info.is_none());
    }

    #[test]
    fn test_reserve_tick_limit() {
        let mut r = Reservations::new();

        assert!(r.reserve(32, 31).is_none());
        assert!(r.reserve(31, 32).is_none());
    }

    #[test]
    fn test_reserve_end_before_start() {
        let mut r = Reservations::new();
        assert!(r.reserve(1, 0).is_none());
        assert!(r.reserve(18, 3).is_none());
    }

    #[test]
    fn test_reserve_later() {
        let mut r = Reservations::new();
        assert!(r.reserve(1, 2).unwrap().later_reservation == false);
        assert!(r.reserve(0, 1).unwrap().later_reservation == true);
        assert!(r.reserve(2, 3).unwrap().later_reservation == false);
    }

    #[test]
    fn test_unreserve() {
        let mut r = Reservations::new();
        assert!(r.reserve(1, 2).unwrap().later_reservation == false);
        assert!(r.reserve(0, 1).unwrap().later_reservation == true);
        r.unreserve(0, 1);
        assert!(r.reserve(0, 1).unwrap().later_reservation == true);
        r.unreserve(0, 1);
        r.unreserve(1, 2);
        assert!(r.is_reserved(0, 1) == false);
        assert!(r.reserve(0, 1).unwrap().later_reservation == false);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Road {
    pub should_yield: bool,
    pub blocked: bool,
    pub reservations: Reservations,
    pub connections: Connections,
}

impl Road {
    pub fn new(dir: Direction) -> Road {
        Road {
            should_yield: false,
            blocked: false,
            reservations: Reservations::new(),
            connections: Connections::new(ConnectionLayer::Road, dir),
        }
    }

    fn draw(&self, rect: &Rectangle, tileset: &Tileset) {
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

        if self.reservations.is_reserved(0, 31) {
            tileset.draw_rect(&rect, RESERVED_PATH_COLOR);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct House {
    pub people_heading_to: bool,
}

impl House {

    fn draw(&self, rect: &Rectangle, tileset: &Tileset) {
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
        let rect = Rectangle::from_pos(pos);

        match self {
            
            Tile::Road(road) => road.draw(&rect, tileset),
            _ => {}
        }
    }
    
    fn should_yield(&self) -> bool {
        match self {
            Tile::Road(road) => road.should_yield,
            Tile::House(_) => true,
            _ => true,
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

    pub fn pos_is_valid(&self, pos: &Position) -> bool {
        pos.x >= 0 && pos.x < GRID_SIZE.0 && pos.y >= 0 && pos.y < GRID_SIZE.1
    }

    pub fn get_tile(&self, pos: &Position) -> Option<&Tile> {
        if self.pos_is_valid(pos) {
            Some(&self.tiles[pos.x as usize][pos.y as usize])
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        if self.pos_is_valid(pos) {
            Some(&mut self.tiles[pos.x as usize][pos.y as usize])
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
            !road.blocked
        } else {
            false
        }
    }

    pub fn reserve_position(
        &mut self,
        reservation: &Reservation,
    ) -> Option<(Connections, ReservationInfo)> {
        match self.get_tile_mut(&reservation.position) {
            Some(Tile::Road(road)) => {
                if let Some(info) = road
                    .reservations
                    .reserve(reservation.start_tick, reservation.end_tick)
                {
                    // road.occupied = true;
                    Some((road.connections, info))
                } else {
                    None
                }
            }
            Some(Tile::House(_)) => Some((
                Connections::new(ConnectionLayer::Driveway, Direction::Right),
                ReservationInfo {
                    later_reservation: false,
                },
            )),
            Some(Tile::Empty) => None,
            None => None,
        }
    }

    pub fn unreserve_position(&mut self, reservation: &Reservation) {
        if let Some(Tile::Road(road)) = self.get_tile_mut(&reservation.position) {
            road.reservations
                .unreserve(reservation.start_tick, reservation.end_tick);
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
                                    if road.reservations.is_reserved(0, 1) {
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
        for i in 0..GRID_SIZE.0 {
            for j in 0..GRID_SIZE.1 {
                let pos = Position::new(i, j);
                self.tiles[i as usize][j as usize].draw(pos, tileset);
            }
        }
    }

    pub fn draw_houses(&self, tileset: &Tileset) {
        for i in 0..GRID_SIZE.0 {
            for j in 0..GRID_SIZE.1 {
                let pos =Position::new(i, j);
                if let Some(Tile::House(house)) = self.get_tile(&pos) {
                    house.draw(&Rectangle::from_pos(pos), tileset);
                }
            }
        }
    }
    
    pub fn should_yield(&self, pos: &Position) -> bool {
        if let Some(tile) = self.get_tile(pos) {
            tile.should_yield()
        } else {
            false
        }
    }
}
