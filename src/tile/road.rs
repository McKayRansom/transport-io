
use macroquad::{color::WHITE, math::Rect};
use serde::{Deserialize, Serialize};

use crate::{grid::Direction, tileset::Tileset};

use super::{Reserved};

const ROAD_INTERSECTION_SPRITE: u32 = 16 * 3;
const ROAD_ARROW_SPRITE: u32 = (16 * 3) + 1;
const ROAD_STRAIGHT_SPRITE: u32 = (16 * 3) + 2;


#[derive(Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub struct Road {
    pub should_yield: bool,
    pub reserved: Reserved,
    connections: Vec<Direction>,
}

impl Road {
    pub fn new() -> Self {
        Road {
            should_yield: false,
            reserved: Reserved::new(),
            connections: Vec::new(),
        }
    }

    pub fn new_from_char(c: char) -> Option<Self> {
        let mut road = Road::new();
        match c {
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
                road.connect(Direction::RIGHT);
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
            'u' => {
                road.connect(Direction::RIGHT + Direction::LAYER_UP);
            }
            'd' => {
                road.connect(Direction::RIGHT + Direction::LAYER_DOWN);
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
        if !self.connections.contains(&dir) {
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

    pub fn iter_connections(&self) -> std::slice::Iter<'_, Direction> {
        self.connections.iter()
    }

    pub fn draw(&self, rect: &Rect, tileset: &Tileset) {
        let connection_count = self.connection_count();

        if connection_count != 1 {
            tileset.draw_tile(ROAD_INTERSECTION_SPRITE, WHITE, rect, 0.0);
        }

        for dir in self.connections.iter() {
            if connection_count == 1 {
                let sprite = if self.should_yield {
                    ROAD_STRAIGHT_SPRITE + 2
                } else {
                    ROAD_STRAIGHT_SPRITE
                };
                tileset.draw_tile(sprite, WHITE, rect, dir.to_radians());
            } else {
                tileset.draw_tile(ROAD_ARROW_SPRITE, WHITE, rect, dir.to_radians());
            }
        }

        // if self.reserved {
        //     tileset.draw_rect(&rect, RESERVED_PATH_COLOR);
        // }
    }
}

impl std::fmt::Debug for Road {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.reserved.is_reserved() {
            write!(f, "o")?;
        }
        if self.is_connected(Direction::UP) && self.is_connected(Direction::LEFT) {
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
        // } else if self.has_layer(Direction::RIGHT, ConnectionType::Up) {
        //     write!(f, "u")
        // } else if self.has_layer(Direction::RIGHT, ConnectionType::Down) {
        //     write!(f, "d")
        } else {
            write!(f, "?")
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
            road.iter_connections().collect::<Vec<&Direction>>(),
            vec![&Direction::RIGHT, &Direction::LEFT]
        );
    }

}