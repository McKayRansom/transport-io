
use macroquad::{color::WHITE, math::Rect};

use crate::{grid::{ConnectionLayer, Connections, Direction, Id}, tileset::Tileset};



const ROAD_INTERSECTION_SPRITE: u32 = (16 * 3) + 0;
const ROAD_ARROW_SPRITE: u32 = (16 * 3) + 1;
const ROAD_STRAIGHT_SPRITE: u32 = (16 * 3) + 2;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Road {
    pub should_yield: bool,
    pub reserved: Option<Id>,
    pub connections: Connections,
}

impl Road {
    pub fn new(dir: Direction) -> Road {
        Road {
            should_yield: false,
            reserved: None,
            connections: Connections::new(ConnectionLayer::Road, dir),
        }
    }

    // pub fn should_yield(&self) -> bool {
    //     return self.connections.count() < 2;
    // }

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
        if self.reserved.is_some() {
            write!(f, "o")
        }
        else {
            self.connections.fmt(f)
        }
    }
}
