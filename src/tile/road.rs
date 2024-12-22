
use macroquad::{color::WHITE, math::Rect};
use serde::{Deserialize, Serialize};

use crate::{grid::Direction, tileset::Tileset};

use super::{Connections, Reserved};

const ROAD_INTERSECTION_SPRITE: u32 = 16 * 3;
const ROAD_ARROW_SPRITE: u32 = (16 * 3) + 1;
const ROAD_STRAIGHT_SPRITE: u32 = (16 * 3) + 2;


#[derive(Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub struct Road {
    pub should_yield: bool,
    pub reserved: Reserved,
    connections: Connections,
}

impl Road {
    pub fn new() -> Self {
        Road {
            should_yield: false,
            reserved: Reserved::new(),
            connections: Connections::new(),
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
        self.connections.has(dir)
    }

    pub fn connect(&mut self, dir: Direction) {
        self.connections.add(dir);
    }

    pub fn disconnect(&mut self, dir: Direction) {
        self.connections.remove(dir);
    }

    pub fn connection_count(&self) -> u32 {
        self.connections.count()
    }

    pub fn iter_connections(&self) -> std::slice::Iter<'_, Direction> {
        self.connections.iter()
    }

    pub fn draw(&self, rect: &Rect, tileset: &Tileset) {
        let connection_count = self.connections.count();

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
            write!(f, "o")
        }
        else {
            self.connections.fmt(f)
        }
    }
}
