use serde::{Deserialize, Serialize};

use crate::{
    hash_map_id::Id, map::{Direction, Position}
};

use super::Reserved;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Road {
    pub should_yield: bool,
    pub station: Option<Id>,

    #[serde(skip_serializing, skip_deserializing)]
    pub reserved: Reserved,
    connections: Vec<Direction>,
}

impl Road {

    pub fn new_connected(dir: Direction, station: Option<Id>) -> Self {
        Road {
            station,
            should_yield: false,
            reserved: Reserved::new(),
            connections: if dir != Direction::NONE {
                vec![dir]
            } else {
                Vec::new()
            },
        }
    }

    pub fn new() -> Self {
        Road {
            station: None,
            should_yield: false,
            reserved: Reserved::new(),
            connections: Vec::new(),
        }
    }

    pub fn new_from_char(c: char) -> Option<Self> {
        let mut road = Road::new();
        match c {
            '*' => {
                // unconnected road
            }
            '>' => {
                road.connect(Direction::RIGHT);
            }
            '<' => {
                road.connect(Direction::LEFT);
            }
            '^' => {
                road.connect(Direction::UP);
            }
            '.' => {
                road.connect(Direction::DOWN);
            }
            'y' => {
                road.connect(Direction::UP);
                road.should_yield = true;
            }
            // Roundabouts - top left
            'l' => {
                road.connect(Direction::LEFT);
                road.connect(Direction::DOWN);
            }
            // Roundabouts - top right
            'r' => {
                road.connect(Direction::LEFT);
                road.connect(Direction::UP);
            }
            // Roundabouts - bottom Left
            'L' => {
                road.connect(Direction::RIGHT);
                road.connect(Direction::DOWN);
            }
            // Roundabouts - bottom Right
            'R' => {
                road.connect(Direction::RIGHT);
                road.connect(Direction::UP);
            }
            '}' => {
                road.connect(Direction::RIGHT + Direction::LAYER_UP);
            }
            ']' => {
                road.connect(Direction::RIGHT + Direction::LAYER_DOWN);
            }
            '{' => {
                road.connect(Direction::LEFT + Direction::LAYER_UP);
            }
            '[' => {
                road.connect(Direction::LEFT + Direction::LAYER_DOWN);
            }

            _ => {
                return None;
            }
        }
        Some(road)
    }

    pub fn is_connected(&self, dir: Direction) -> bool {
        self.connections.contains(&dir)
    }

    pub fn connect(&mut self, dir: Direction) {
        if dir != Direction::NONE && !self.connections.contains(&dir) {
            self.connections.push(dir);
        }
    }

    pub fn disconnect(&mut self, dir: Direction) {
        if let Some(index) = self.connections.iter_mut().position(|x| x == &dir) {
            self.connections.swap_remove(index);
        }
    }

    pub fn connection_count(&self) -> u32 {
        self.connections.len() as u32
    }

    pub fn get_connections(&self, pos: &Position) -> &[Direction] {
        if !self.connections.is_empty() {
            self.connections.as_slice()
        } else {
            // Dead ends need this maybe?
            &pos.default_connections()[0..1]
        }
    }
}

impl std::fmt::Debug for Road {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.reserved.is_reserved() {
            write!(f, "o")
        } else if self.is_connected(Direction::UP) && self.is_connected(Direction::LEFT) {
            write!(f, "r")
        } else if self.is_connected(Direction::DOWN) && self.is_connected(Direction::LEFT) {
            write!(f, "l")
        } else if self.is_connected(Direction::DOWN) && self.is_connected(Direction::RIGHT) {
            write!(f, "L")
        } else if self.is_connected(Direction::UP) && self.is_connected(Direction::RIGHT) {
            write!(f, "R")
        } else if self.is_connected(Direction::LEFT) {
            write!(f, "<")
        } else if self.is_connected(Direction::RIGHT) {
            write!(f, ">")
        } else if self.is_connected(Direction::UP) {
            write!(f, "^")
        } else if self.is_connected(Direction::DOWN) {
            write!(f, ".")
        } else if self.is_connected(Direction::RIGHT + Direction::LAYER_UP) {
            write!(f, "}}")
        } else if self.is_connected(Direction::RIGHT + Direction::LAYER_DOWN) {
            write!(f, "]")
        } 
        else if self.is_connected(Direction::LEFT + Direction::LAYER_UP) {
            write!(f, "{{")
        } else if self.is_connected(Direction::LEFT + Direction::LAYER_DOWN) {
            write!(f, "[")
        }
        else {
            write!(f, "*")
        }
    }
}

#[cfg(test)]
mod road_tests {
    // use std::mem;

    use super::*;

    #[test]
    fn test_new() {
        let road = Road::new();
        assert_eq!(road.connection_count(), 0, "Connections: {:?}", road);
    }

    #[test]
    fn test_connect() {
        let mut road = Road::new();
        assert!(!road.is_connected(Direction::RIGHT));
        road.connect(Direction::RIGHT);
        assert!(road.is_connected(Direction::RIGHT));
        assert_eq!(road.connection_count(), 1);

        road.connect(Direction::RIGHT);
        road.connect(Direction::RIGHT);
        assert_eq!(road.connection_count(), 1);

        road.disconnect(Direction::RIGHT);
        assert_eq!(road.connection_count(), 0);
        assert!(!road.is_connected(Direction::RIGHT));
    }

    #[test]
    fn test_iter() {
        let mut road = Road::new();
        road.connect(Direction::RIGHT);
        road.connect(Direction::LEFT);
        assert_eq!(
            road.get_connections(&Position::new(0, 0)),
            vec![Direction::RIGHT, Direction::LEFT].as_slice()
        );
    }
}
