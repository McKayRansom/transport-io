
use macroquad::{color::WHITE, math::Rect};

use crate::{grid::Direction, tileset::Tileset};

use super::{ConnectionLayer, Connections, ConnectionsIterator, Reserved};

const ROAD_INTERSECTION_SPRITE: u32 = (16 * 3) + 0;
const ROAD_ARROW_SPRITE: u32 = (16 * 3) + 1;
const ROAD_STRAIGHT_SPRITE: u32 = (16 * 3) + 2;


#[derive(Clone, PartialEq, Eq)]
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
                road.connect(Direction::Right);
            }
            '<' => {
                road.connect(Direction::Left);
            }
            '^' => {
                road.connect(Direction::Up);
            }
            '.' => {
                road.connect(Direction::Down);
            }
            'y' => {
                road.connect(Direction::Right);
                road.should_yield = true;
            }
            // Roundabouts - top left
            'l' => {
                road.connect(Direction::Left);
                road.connect(Direction::Down);
            }
            // Roundabouts - top right
            'r' => {
                road.connect(Direction::Left);
                road.connect(Direction::Up);
            }
            // Roundabouts - bottom Left
            'L' => {
                road.connect(Direction::Right);
                road.connect(Direction::Down);
            }
            // Roundabouts - bottom Right
            'R' => {
                road.connect(Direction::Right);
                road.connect(Direction::Up);
            }
            'u' => {
                road.connect_layer(Direction::Right, ConnectionLayer::Up);
            }
            'd' => {
                road.connect_layer(Direction::Right, ConnectionLayer::Down);
            }
            _ => {
                return None;
            }
        }
        Some(road)
    }

    // pub fn should_yield(&self) -> bool {
    //     return self.connections.count() < 2;
    // }

    pub fn is_connected(&self, dir: Direction) -> bool {
        self.connections.has(dir)
    }

    pub fn connect(&mut self, dir: Direction) {
        self.connections.add(ConnectionLayer::Road, dir);
    }

    pub fn connect_layer(&mut self, dir: Direction, layer: ConnectionLayer) {
        self.connections.add(layer, dir);
    }

    pub fn disconnect(&mut self, dir: Direction) {
        self.connections.remove(dir);
    }

    pub fn connection_count(&self) -> u32 {
        self.connections.count()
    }

    pub fn iter_connections(&self) -> ConnectionsIterator {
        self.connections.iter()
    }

    pub fn iter_connections_inverse(&self, layer: ConnectionLayer) -> ConnectionsIterator {
        self.connections.iter_inverse(layer)
    }

    pub fn draw(&self, rect: &Rect, tileset: &Tileset) {
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
