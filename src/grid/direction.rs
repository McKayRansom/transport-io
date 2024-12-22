use std::f32::consts::PI;

use macroquad::prelude::rand;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Direction {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl Direction {
    // use super::{Direction};

    // NOT PUB
    const fn new(x: i8, y: i8, z: i8) -> Self {
        Direction { x, y, z }
    }

    // TODO: RENAME to N, E, S, W, and UP DOWN
    pub const RIGHT: Direction = Direction::new(1, 0, 0);
    pub const LEFT: Direction = Direction::new(-1, 0, 0);
    pub const UP: Direction = Direction::new(0, -1, 0);
    pub const DOWN: Direction = Direction::new(0, 1, 0);

    pub const LAYER_UP: Direction = Direction::new(0, 0, 1);
    pub const LAYER_DOWN: Direction = Direction::new(0, 0, -1);

    pub const ALL: [Direction; 4] = [Direction::RIGHT, Direction::LEFT, Direction::UP, Direction::DOWN];

    // TODO: Rename inverse_flat
    pub fn inverse(&self) -> Self {
        Direction::new(-self.x, -self.y, self.z)
    }

    pub fn add(&self, other: &Direction) -> Direction {
        Direction::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }

    pub fn random() -> Self {
        Direction::ALL[rand::gen_range(0, Direction::ALL.len()) as usize]
    }

    pub fn is_horizontal(self) -> bool {
        self.y == 0
    }

    // TODO: Rename rotate_flat
    pub fn rotate_left(self) -> Self {
        Direction::new(self.y, -self.x, self.z)
    }

    // pub fn _rotate(self) -> Self {
    //     match self {
    //         Direction::UP => Direction::RIGHT,
    //         Direction::RIGHT => Direction::DOWN,
    //         Direction::DOWN => Direction::LEFT,
    //         Direction::LEFT => Direction::UP,
    //     }
    // }

    pub fn to_radians(self) -> f32 {
        match self {
            Direction::UP => 0.,
            Direction::RIGHT => PI / 2.0,
            Direction::DOWN => PI,
            Direction::LEFT => PI * 1.5,
            _ => PI / 4.,
        }
    }

    // pub fn _from_keycode(key: KeyCode) -> Option<Direction> {
    //     match key {
    //         KeyCode::Up => Some(Direction::UP),
    //         KeyCode::Down => Some(Direction::DOWN),
    //         KeyCode::Left => Some(Direction::LEFT),
    //         KeyCode::Right => Some(Direction::RIGHT),
    //         _ => None,
    //     }
    // }
}

#[cfg(test)]
mod direction_tests {
    // use std::mem;

    use super::*;

    #[test]
    fn test_inverse() {
        assert_eq!(Direction::RIGHT.inverse(), Direction::LEFT);
        assert_eq!(Direction::LEFT.inverse(), Direction::RIGHT);
        assert_eq!(Direction::UP.inverse(), Direction::DOWN);
        assert_eq!(Direction::DOWN.inverse(), Direction::UP);
    }

    #[test]
    fn test_rotate() {
        assert_eq!(Direction::RIGHT.rotate_left(), Direction::UP);
        assert_eq!(Direction::DOWN.rotate_left(), Direction::RIGHT);
        assert_eq!(Direction::LEFT.rotate_left(), Direction::DOWN);
        assert_eq!(Direction::UP.rotate_left(), Direction::LEFT);
    }
}