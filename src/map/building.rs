use macroquad::prelude::rand;
use serde::{Deserialize, Serialize};

use super::{grid::Grid, tile::Tile, Direction, Position};

use crate::{
    consts::SpawnerColors,
    hash_map_id::Id,
};

pub const BUILDING_SIZE: (i8, i8) = (2, 2);

const HOUSE_UPDATE_TICKS: i32 = 10 * 16;
const SPAWNER_UPDATE_TICKS: i32 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BuildingType {
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
    pub building_type: BuildingType,
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

    pub fn spawn_pos(&self, grid: &Grid) -> Option<(Position, Direction)> {
        for pos in self.pos.iter_area((BUILDING_SIZE.0 as i16, BUILDING_SIZE.1 as i16)) {
            let dir = pos.default_connections()[1];
            let pos_adj = pos + dir;
            if let Some(Tile::Road(_)) = grid.get_tile(&pos_adj) {
                return Some((pos, dir))
            }
        }
        // should there be a default?
        None
    }

    pub fn destination_pos(&self, grid: &Grid) -> Option<(Position, Direction)> {
        for pos in self.pos.iter_area((BUILDING_SIZE.0 as i16, BUILDING_SIZE.1 as i16)) {
            let dir = pos.default_connections()[0].inverse();
            let pos_adj = pos + dir;
            if let Some(Tile::Road(_)) = grid.get_tile(&pos_adj) {
                return Some((pos, dir))
            }
        }
        None
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
