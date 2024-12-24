use std::ops::{Add, AddAssign, Sub};

use macroquad::math::Rect;
use serde::*;

use super::{Direction, GRID_CELL_SIZE, GRID_Z_OFFSET};

// pub const Z_TUNNEL = 0
pub const Z_GROUND: i16 = 0;
// pub const Z_BRIDGE: i16 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Position {
    pub const fn new(x: i16, y: i16) -> Self {
        let z = Z_GROUND;
        Position { x, y, z }
    }

    pub fn _new_z(x: i16, y: i16, z: i16) -> Self {
        Position { x, y, z }
    }

    pub fn round_to(&self, amount: i16) -> Self {
        Position {
            x: self.x - self.x % amount,
            y: self.y - self.y % amount,
            z: self.z,
        }
    }

    pub fn from_screen(screen_pos: (f32, f32), camera_pos: (f32, f32), zoom: f32) -> Self {
        Position::new(
            ((camera_pos.0 + (screen_pos.0 / zoom)) / GRID_CELL_SIZE.0) as i16,
            ((camera_pos.1 + (screen_pos.1 / zoom)) / GRID_CELL_SIZE.1) as i16,
        )
    }

    pub fn distance(&self, other: &Position) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }

    pub fn direction_to(self: Position, new_pos: Position) -> Direction {
        let x_diff = (self.x - new_pos.x).abs();
        let y_diff = (self.y - new_pos.y).abs();
        if new_pos.x > self.x && x_diff >= y_diff {
            Direction::RIGHT
        } else if new_pos.y > self.y && y_diff > x_diff {
            Direction::DOWN
        } else if new_pos.y < self.y && y_diff > x_diff {
            Direction::UP
        } else {
            Direction::LEFT
        }
    }

    pub fn iter_line_to(&self, destination: Position) -> (PositionIterator, Direction) {
        let direction = self.direction_to(destination);
        let count: usize = if direction.y != 0 {
            (destination.y - self.y).unsigned_abs() as usize
        } else if direction.x != 0 {
            (destination.x - self.x).unsigned_abs() as usize
        } else {
            0
        };
        (
            PositionIterator {
                position: *self,
                direction,
                count: count + 1, // include the destination position
            },
            direction,
        )
    }
}

impl From<(i16, i16)> for Position {
    fn from(pos: (i16, i16)) -> Self {
        Position::new(
            pos.0 as i16, 
            pos.1 as i16,
        )
    }
}

impl From<(i16, i16, i16)> for Position {
    fn from(pos: (i16, i16, i16)) -> Self {
        Position::_new_z(
            pos.0 as i16, 
            pos.1 as i16,
            pos.2 as i16,
        )
    }
}

impl Add<Direction> for Position {
    type Output = Self;

    fn add(self, dir: Direction) -> Self {
        Position {
            x: self.x + dir.x as i16,
            y: self.y + dir.y as i16,
            z: self.z + dir.z as i16,
        }
    }
}

impl Sub<Position> for Position {
    type Output = Direction;

    fn sub(self, other: Position) -> Direction {
        Direction {
            x: (self.x - other.x) as i8,
            y: (self.y - other.y) as i8,
            z: (self.z - other.z) as i8,
        }
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, dir: Direction) {
        self.x += dir.x as i16;
        self.y += dir.y as i16;
        self.z += dir.z as i16;
    }
}

pub struct PositionIterator {
    position: Position,
    direction: Direction,
    count: usize,
}

impl Iterator for PositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            None
        } else {
            self.count -= 1;
            let old_pos = self.position;
            self.position += self.direction;
            Some(old_pos)
        }
    }
}

impl From<Position> for Rect {
    fn from(pos: Position) -> Self {
        Rect::new(
            pos.x as f32 * GRID_CELL_SIZE.0,
            pos.y as f32 * GRID_CELL_SIZE.1/* - (pos.z as f32 * GRID_Z_OFFSET) */,
            GRID_CELL_SIZE.0,
            GRID_CELL_SIZE.1,
        )
    }
}

#[cfg(test)]
mod position_tests {

    use super::*;

    fn pos(x: i16, y: i16) -> Position {
        Position::new(x, y)
    }

    #[test]
    fn test_init() {
        assert_eq!(pos(0, 0), Position { x: 0, y: 0, z: 0 });

        // assert!(directions::EAST.x == 1);
    }

    #[test]
    fn test_round() {
        assert_eq!(pos(3, 3).round_to(2), Position { x: 2, y: 2, z: 0 });
        assert_eq!(pos(2, 2).round_to(2), Position { x: 2, y: 2, z: 0 });
    }

    #[test]
    fn test_from_position() {
        assert_eq!(pos(0, 0).direction_to(pos(3, 0)), Direction::RIGHT);
        assert_eq!(pos(0, 3).direction_to(pos(0, 0)), Direction::UP);
        assert_eq!(pos(3, 0).direction_to(pos(0, 0)), Direction::LEFT);
        assert_eq!(pos(0, 0).direction_to(pos(0, 3)), Direction::DOWN);
    }

    #[test]
    fn test_from_position_diagonal() {
        assert_eq!(pos(0, 0).direction_to(pos(3, 1)), Direction::RIGHT);
        assert_eq!(pos(1, 3).direction_to(pos(0, 0)), Direction::UP);
        assert_eq!(pos(3, 1).direction_to(pos(0, 0)), Direction::LEFT);
        assert_eq!(pos(0, 0).direction_to(pos(1, 3)), Direction::DOWN);

        // for even let's just pick Left/Right??
        assert_eq!(pos(3, 3).direction_to(pos(0, 0)), Direction::LEFT);
        assert_eq!(pos(0, 0).direction_to(pos(3, 3)), Direction::RIGHT);
    }

    #[test]
    fn test_iter_line_to() {
        let start_pos: Position = pos(0, 0);
        let end_pos: Position = pos(3, 0);
        let (iter, direction) = start_pos.iter_line_to(end_pos);
        assert_eq!(direction, Direction::RIGHT);

        let line: Vec<Position> = iter.collect();
        assert_eq!(line, vec![pos(0, 0), pos(1, 0), pos(2, 0), pos(3, 0)]);
    }
}
