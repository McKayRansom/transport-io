use macroquad::{
    color::{Color, WHITE},
    prelude::rand,
};
use pathfinding::matrix::directions::N;
use serde::{Deserialize, Serialize};

use super::{Direction, Position};

use crate::{
    tileset::{Sprite, Tileset}, hash_map_id::Id,
};

pub const BUILDING_SIZE: (i8, i8) = (2, 2);

const HOUSE_SPRITE: Sprite = Sprite::new_size(6, 0, BUILDING_SIZE);
const STATION_SPRITE: Sprite = Sprite::new_size(6, 2, BUILDING_SIZE);
// const HOUSE_UPDATE_TICKS: i32 = 10 * 16;
const HOUSE_UPDATE_TICKS: i32 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum BuildingType {
    House,
    Station,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Building {
    pub pos: Position,
    // pub id: Id,
    pub city_id: Id,
    pub vehicle_on_the_way: Option<Id>,
    production_tics: i32,
    production_rate: i32,
    building_type: BuildingType,
}

impl Building {
    pub fn new_house(pos: Position, city_id: Id) -> Self {
        let pos = pos.round_to(2);
        Building {
            pos,
            // 0,
            city_id,
            vehicle_on_the_way: None,
            production_tics: rand::gen_range(0, HOUSE_UPDATE_TICKS),
            production_rate: HOUSE_UPDATE_TICKS,
            building_type: BuildingType::House,
        }
    }

    pub fn new_station(pos: Position) -> Self {
        Building {
            pos,
            city_id: 0,
            vehicle_on_the_way: None,
            production_tics: 0,
            production_rate: 0,
            building_type: BuildingType::Station,
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        let color = if self.vehicle_on_the_way.is_some() {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            WHITE
        };
        let sprite: &Sprite = match self.building_type {
            BuildingType::House => &HOUSE_SPRITE,
            BuildingType::Station => &STATION_SPRITE,
        };
        tileset.draw_tile(*sprite, color, &self.pos.into(), 0.0);
    }

    // pub fn iter_connections(&self, pos: &Position) -> &[Direction] {
        // pos.default_connections()
    // }

    pub fn update(&mut self) -> bool {
        if self.building_type == BuildingType::House {
            self.production_tics += 1;
            if self.production_tics >= HOUSE_UPDATE_TICKS {
                self.production_tics = 0;
                true
            } else {
                false
            }
        } else {
            false
        }
   }
}
