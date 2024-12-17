use macroquad::math::Rect;
use pathfinding::num_traits::AsPrimitive;

use super::{Direction, GRID_CELL_SIZE};

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

    pub fn direction_to(self: Position, new_pos: Position) -> Direction {
        let x_diff = (self.x - new_pos.x).abs();
        let y_diff = (self.y - new_pos.y).abs();
        if new_pos.x > self.x && x_diff >= y_diff {
            Direction::Right
        } else if new_pos.y > self.y && y_diff > x_diff {
            Direction::Down
        } else if new_pos.y < self.y && y_diff > x_diff {
            Direction::Up
        } else {
            Direction::Left
        }
    }

    pub fn new_from_move(pos: &Position, dir: Direction) -> Self {
        match dir {
            Direction::Up => Position::new(pos.x, pos.y - 1),
            Direction::Down => Position::new(pos.x, pos.y + 1),
            Direction::Left => Position::new(pos.x - 1, pos.y),
            Direction::Right => Position::new(pos.x + 1, pos.y),
        }
    }

    pub fn iter_line_to(&self, destination: Position) -> PositionIterator {
        let direction = self.direction_to(destination);
        let count: usize = match direction {
            Direction::Down | Direction::Up => (destination.y - self.y).abs() as usize,
            Direction::Left | Direction::Right => (destination.x - self.x).abs() as usize,
        };
        PositionIterator {
            position: *self,
            direction: direction,
            count: count + 1, // include the destination position
        }
    }
}

pub struct PositionIterator {
    position: Position,
    pub direction: Direction,
    count: usize,
}

impl Iterator for PositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            let position = self.position;
            self.position = Position::new_from_move(&self.position, self.direction);
            Some(position)
        } else {
            None
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

impl<T> From<(T, T)> for Position
where
    T: AsPrimitive<i16>,
{
    fn from(pos: (T, T)) -> Self {
        Position::new(pos.0.as_(), pos.1.as_())
    }
}

#[cfg(test)]
mod position_tests {

    use super::*;

    #[test]
    fn test_init() {
        let pos = Position::new(0, 0);
        assert_eq!(pos, Position { x: 0, y: 0, z: 0 });
    }

    #[test]
    fn test_from_position() {
        assert_eq!(
            Position::new(0, 0).direction_to((3, 0).into()),
            Direction::Right
        );
        assert_eq!(
            Position::new(0, 3).direction_to((0, 0).into()),
            Direction::Up
        );
        assert_eq!(
            Position::new(3, 0).direction_to((0, 0).into()),
            Direction::Left
        );
        assert_eq!(
            Position::new(0, 0).direction_to((0, 3).into()),
            Direction::Down
        );
    }

    #[test]
    fn test_from_position_diagonal() {
        assert_eq!(
            Position::new(0, 0).direction_to((3, 1).into()),
            Direction::Right
        );
        assert_eq!(
            Position::new(1, 3).direction_to((0, 0).into()),
            Direction::Up
        );
        assert_eq!(
            Position::new(3, 1).direction_to((0, 0).into()),
            Direction::Left
        );
        assert_eq!(
            Position::new(0, 0).direction_to((1, 3).into()),
            Direction::Down
        );

        // for even let's just pick Left/Right??
        assert_eq!(
            Position::new(3, 3).direction_to((0, 0).into()),
            Direction::Left
        );
        assert_eq!(
            Position::new(0, 0).direction_to((3, 3).into()),
            Direction::Right
        );
    }

    #[test]
    fn test_iter_line_to() {
        let start_pos: Position = (0, 0).into();
        let end_pos: Position = (3, 0).into();
        let iter = start_pos.iter_line_to(end_pos);
        assert_eq!(iter.position, start_pos);
        assert_eq!(iter.direction, Direction::Right);
        assert_eq!(iter.count, 4);

        let line: Vec<Position> = iter.collect();
        assert_eq!(
            line,
            vec![(0, 0).into(), (1, 0).into(), (2, 0).into(), (3, 0).into()]
        );
    }
}
