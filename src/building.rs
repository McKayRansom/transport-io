use macroquad::{color::{Color, WHITE}, prelude::rand};
use serde::{Deserialize, Serialize};

use crate::{grid::{Id, Position}, tileset::{Sprite, Tileset}};


const HOUSE_SPRITE: Sprite = Sprite::new_size(1, 0, (1, 1));

const HOUSE_UPDATE_TICKS: i32 = 10 * 16;


#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Building {
    pub pos: Position,
    id: Id,
    pub city_id: Id,
    pub vehicle_on_the_way: Option<Id>,
    production_tics: i32,
}

impl Building {
    pub fn new(pos: Position, id: Id, city_id: Id) -> Self {
        Building {
            pos,
            id,
            city_id,
            vehicle_on_the_way: None,
            production_tics: rand::gen_range(0, HOUSE_UPDATE_TICKS),
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        let color = if self.vehicle_on_the_way.is_some() {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            WHITE
        };
        tileset.draw_tile(HOUSE_SPRITE, color, &self.pos.into(), 0.0);
    }

    pub fn update(&mut self) -> bool {
        self.production_tics += 1;
        if self.production_tics >= HOUSE_UPDATE_TICKS {
            self.production_tics = 0;
            true
        } else {
            false
        }
    }
}