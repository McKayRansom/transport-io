use std::{f32::consts::PI, ops::{Add, Mul}};

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

    pub const NONE: Direction = Direction::new(0, 0, 0);

    pub const RIGHT: Direction = Direction::new(1, 0, 0);
    pub const LEFT: Direction = Direction::new(-1, 0, 0);
    pub const UP: Direction = Direction::new(0, -1, 0);
    pub const DOWN: Direction = Direction::new(0, 1, 0);

    pub const DOWN_RIGHT: Direction = Direction::new(1, 1, 0);

    pub const LAYER_UP: Direction = Direction::new(0, 0, 1);
    pub const LAYER_DOWN: Direction = Direction::new(0, 0, -1);
    pub const LAYER_DOWN_2: Direction = Direction::new(0, 0, -2);

    pub const ALL: [Direction; 4] = [Direction::RIGHT, Direction::LEFT, Direction::UP, Direction::DOWN];

    pub fn inverse(&self) -> Self {
        Direction::new(-self.x, -self.y, self.z)
    }

    pub fn random() -> Self {
        Direction::ALL[rand::gen_range(0, Direction::ALL.len()) as usize]
    }

    pub fn is_horizontal(self) -> bool {
        self.y == 0
    }

    pub fn rotate_left(self) -> Self {
        Direction::new(self.y, -self.x, self.z)
    }

    pub fn to_radians(self) -> f32 {
        let mut dir: f32 = 0.;
        if self.x > 0 {
            dir += PI / 2.;
        } else if self.x  < 0 {
            dir += PI * 1.5;
        } else if self.y > 0 {
            dir += PI;
        }

        // if self.z > 0 {
        //     dir -= PI / 8.;
        // } else if self.z < 0 {
        //     dir += PI / 8.;
        // }

        dir
    }
}

impl From<(i8, i8)> for Direction {
    fn from(pos: (i8, i8)) -> Self {
        Direction::new(
            pos.0,
            pos.1,
            0,
        )
    }
}

impl From<(i8, i8, i8)> for Direction {
    fn from(pos: (i8, i8, i8)) -> Self {
        Direction::new(
            pos.0,
            pos.1,
            pos.2,
        )
    }
}

impl Add for Direction {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Mul<i8> for Direction {
    type Output = Self;

    fn mul(self, other: i8) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
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

    #[test]
    fn test_to_radians() {
        assert_eq!(Direction::UP.to_radians(), 0.);
        assert_eq!(Direction::RIGHT.to_radians(), PI / 2.0);
        assert_eq!(Direction::DOWN.to_radians(), PI);
        assert_eq!(Direction::LEFT.to_radians(), PI * 1.5);
    }
}