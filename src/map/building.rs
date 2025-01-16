use macroquad::{
    color::{Color, WHITE},
    prelude::rand,
};
use serde::{Deserialize, Serialize};

use super::{Direction, Position};

use crate::{
    consts::SpawnerColors, hash_map_id::Id, tileset::{Sprite, Tileset}
};

pub const BUILDING_SIZE: (i8, i8) = (2, 2);

const HOUSE_SPRITE: Sprite = Sprite::new_size(6, 0, BUILDING_SIZE);
const STATION_SPRITE: Sprite = Sprite::new_size(6, 2, BUILDING_SIZE);
const SPAWNER_SPRITE: Sprite = Sprite::new_size(6, 4, BUILDING_SIZE);
const HOUSE_UPDATE_TICKS: i32 = 10 * 16;
const SPAWNER_UPDATE_TICKS: i32 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum BuildingType {
    House,
    Station,
    Spawner,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Building {
    pub pos: Position,
    pub dir: Option<Direction>,
    pub color: SpawnerColors,
    pub city_id: Id,
    pub vehicle_on_the_way: Option<Id>,
    pub arrived_count: i32,
    production_tics: i32,
    production_rate: i32,
    building_type: BuildingType,
}

impl Building {
    pub fn new_house(pos: Position, city_id: Id) -> Self {
        Building {
            pos,
            dir: None,
            color: SpawnerColors::Blue,
            city_id,
            vehicle_on_the_way: None,
            arrived_count: 0,
            production_tics: rand::gen_range(0, HOUSE_UPDATE_TICKS),
            production_rate: HOUSE_UPDATE_TICKS,
            building_type: BuildingType::House,
        }
    }

    pub fn new_station(pos: Position, city_id: Id) -> Self {
        Building {
            pos,
            dir: None,
            color: SpawnerColors::Blue,
            city_id,
            vehicle_on_the_way: None,
            arrived_count: 0,
            production_tics: rand::gen_range(0, HOUSE_UPDATE_TICKS),
            production_rate: HOUSE_UPDATE_TICKS,
            building_type: BuildingType::House,
        }
    }

    pub fn new_spawner(pos: Position, dir: Direction, color: SpawnerColors, city_id: Id) -> Self {
        Building {
            pos,
            dir: Some(dir),
            color,
            city_id,
            vehicle_on_the_way: None,
            arrived_count: 0,
            production_tics: rand::gen_range(0, SPAWNER_UPDATE_TICKS),
            production_rate: SPAWNER_UPDATE_TICKS,
            building_type: BuildingType::Spawner,
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        let (sprite, color): (&Sprite, Color) = match self.building_type {
            BuildingType::House => (&HOUSE_SPRITE, WHITE),
            BuildingType::Station => (&STATION_SPRITE, WHITE),
            BuildingType::Spawner => (&SPAWNER_SPRITE, self.color.color()),
        };
        tileset.draw_tile(*sprite, color, &self.pos.into(), 0.0);

        tileset.draw_text(
            format!("{}", self.arrived_count).as_str(),
            16.,
            WHITE,
            &(self.pos + Direction::DOWN_RIGHT).into(),
        );
    }

    pub fn spawn_pos(&self) -> Position {
        if let Some(dir) = self.dir {
            self.pos.corner_pos(dir.inverse())
        } else {
            self.pos
        }
    }

    pub fn destination_pos(&self) -> Position {
        if let Some(dir) = self.dir {
            self.pos.corner_pos(dir)
        } else {
            self.pos
        }
    }

    pub fn update_arrived(&mut self, success: bool) {
        if success {
            self.arrived_count += 1;
        } else if self.arrived_count > 0 {
            self.arrived_count -= 1;
        }
    }

    pub fn update(&mut self) -> bool {
        if self.building_type != BuildingType::Station {
            self.production_tics += 1;
            if self.production_tics >= self.production_rate {
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
