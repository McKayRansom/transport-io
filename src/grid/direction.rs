use std::f32::consts::PI;

use macroquad::{input::KeyCode, prelude::rand};
use serde::{Deserialize, Serialize};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}


impl From<Direction> for usize {
    fn from(dir: Direction) -> Self {
        dir as usize
    }
}
use std::convert::TryFrom;

impl TryFrom<usize> for Direction {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            x if x == Direction::Up as usize => Ok(Direction::Up),
            x if x == Direction::Down as usize => Ok(Direction::Down),
            x if x == Direction::Left as usize => Ok(Direction::Left),
            x if x == Direction::Right as usize => Ok(Direction::Right),
            _ => Err(()),
        }
    }
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

    pub fn random() -> Self {
        (rand::gen_range(0, 4) as usize).try_into().unwrap()
    }

    pub fn is_horizontal(self) -> bool {
        match self {
            Direction::Up => false,
            Direction::Down => false,
            Direction::Left => true,
            Direction::Right => true,
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


