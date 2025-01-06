use macroquad::{
    color::{Color, WHITE},
    prelude::rand,
};
use serde::{Deserialize, Serialize};

use super::Position;

use crate::{
    tileset::{Sprite, Tileset}, hash_map_id::Id,
};

pub const BUILDING_SIZE: (i8, i8) = (2, 2);

const HOUSE_SPRITE: Sprite = Sprite::new_size(6, 0, BUILDING_SIZE);
// const HOUSE_UPDATE_TICKS: i32 = 10 * 16;
const HOUSE_UPDATE_TICKS: i32 = 16;

pub enum SpawnerRates {
    Slow = 16,
    Medium = 32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Building {
    pub pos: Position,
    // pub id: Id,
    pub city_id: Id,
    pub vehicle_on_the_way: Option<Id>,
    production_tics: i32,
    production_rate: i32,
}

impl Building {
    pub fn new_house(pos: Position, city_id: Id) -> Self {
        Building {
            pos,
            // 0,
            city_id,
            vehicle_on_the_way: None,
            production_tics: rand::gen_range(0, HOUSE_UPDATE_TICKS),
            production_rate: HOUSE_UPDATE_TICKS,
        }
    }

    pub fn new_spawner(pos: Position, city_id: Id, spawn_rate: SpawnerRates) -> Self {
        let update_ticks:i32 =  spawn_rate as i32;
        Building {
            pos,
            // 0,
            city_id,
            vehicle_on_the_way: None,
            production_tics: rand::gen_range(0, update_ticks),
            production_rate: update_ticks,
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
