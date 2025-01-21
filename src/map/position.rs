use std::ops::{Add, AddAssign, Div, Sub};

use macroquad::math::Rect;
use serde::*;

use super::Direction;

// pub const Z_TUNNEL = 0
pub const Z_GROUND: i16 = 0;
// pub const Z_BRIDGE: i16 = 1;

// Now we define the pixel size of each tile, which we make 32x32 pixels.
pub const GRID_CELL_SIZE: (f32, f32) = (32., 32.);

const TOP_LEFT_DIRS: [Direction; 2] = [Direction::DOWN, Direction::LEFT];
const TOP_RIGHT_DIRS: [Direction; 2] = [Direction::LEFT, Direction::UP];
const BOT_RIGHT_DIRS: [Direction; 2] = [Direction::UP, Direction::RIGHT];
const BOT_LEFT_DIRS: [Direction; 2] = [Direction::RIGHT, Direction::DOWN];

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
        } else if new_pos.x < self.x {
            Direction::LEFT
        } else {
            Direction::NONE
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

    pub fn iter_area(&self, area: Direction) -> PositionAreaIterator {
        PositionAreaIterator {
            current: *self,
            start: *self,
            size: area,
            finished: false,
        }
    }

    pub fn corner_pos(&self, dir: Direction) -> Self {
        match dir {
            Direction::LEFT => *self + Direction::DOWN_RIGHT,
            Direction::RIGHT => *self,
            Direction::DOWN => *self + Direction::RIGHT,
            Direction::UP => *self + Direction::DOWN,
            // Direction::NONE => pos,
            _ => *self,
        }
    }

    // .<
    // >^
    pub fn default_connections(&self) -> &'static[Direction] {
        let x_rem = self.x % 2;
        let y_rem = self.y % 2;
        match (x_rem, y_rem) {
            (0, 0) => &TOP_LEFT_DIRS,
            (1, 0) => &TOP_RIGHT_DIRS,
            (1, 1) => &BOT_RIGHT_DIRS,
            (0, 1) => &BOT_LEFT_DIRS,
            _ => &[],
        }
    }

}

impl From<(i16, i16)> for Position {
    fn from(pos: (i16, i16)) -> Self {
        Position::new(
            pos.0,
            pos.1,
        )
    }
}

impl From<(i16, i16, i16)> for Position {
    fn from(pos: (i16, i16, i16)) -> Self {
        Position::_new_z(
            pos.0,
            pos.1,
            pos.2,
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


impl Div<i16> for Position {
    type Output = Position;

    fn div(self, other: i16) -> Position {
        Self {
            x: self.x /other, 
            y: self.y  / other,
            z: self.z / other,
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
    pub count: usize,
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

pub struct PositionAreaIterator {
    start: Position,
    current: Position,
    size: Direction,
    finished: bool,
}

impl Iterator for PositionAreaIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let current = self.current;
        self.current.x += 1;
        if self.current.x >= self.start.x + self.size.x as i16 {
            self.current.y += 1;
            self.current.x = self.start.x;
        }
        if self.current.y >= self.start.y + self.size.y as i16 {
            self.finished = true;
        }
        Some(current)
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

    #[test]
    fn test_iter_area() {
        let start_pos: Position = pos(2, 2);
        assert_eq!(
            start_pos.iter_area((1, 1).into()).collect::<Vec<Position>>(),
            vec![pos(2, 2)]
        );

        assert_eq!(
            start_pos.iter_area((2, 2).into()).collect::<Vec<Position>>(),
            vec![pos(2, 2), pos(3, 2), pos(2, 3), pos(3, 3)]
        );

        assert_eq!(
            Position::new(0, 0).iter_area((2, 2).into()).collect::<Vec<Position>>(),
            vec![pos(0, 0), pos(1, 0), pos(0, 1), pos(1, 1)]
        );
    }

    #[test]
    fn test_round_dir() {
        assert_eq!(
            pos(0, 0).default_connections()[0], 
            Direction::DOWN
        )
    }
}
