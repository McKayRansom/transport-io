use macroquad::prelude::rand;
use serde::{Deserialize, Serialize};

use super::{
    grid::Grid,
    tile::Tile,
    Direction, Position,
};

use crate::{consts::SpawnerColors, hash_map_id::Id};

pub const BUILDING_SIZE: Direction = Direction::new(2, 2, 0);

const HOUSE_UPDATE_TICKS: i32 = 10 * 16;
const SPAWNER_UPDATE_TICKS: i32 = 16;

const CARGO_MAX: u32 = 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BuildingType {
    House,
    Station,
    Spawner,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Building {
    pub pos: Position,
    pub dir: Option<Direction>,
    pub color: SpawnerColors,
    pub city_id: Id,
    pub my_vehicle: Option<Id>,
    pub arrived_count: i32,
    production_tics: i32,
    production_rate: i32,
    pub building_type: BuildingType,
    pub routes: Vec<Id>,
    pub cargo_count: u32,
}


impl Building {
    pub fn new_house(pos: Position, city_id: Id) -> Self {
        Building {
            pos,
            dir: None,
            color: SpawnerColors::Blue,
            city_id,
            my_vehicle: None,
            arrived_count: 1, // Spawn with conditions met for now...
            production_tics: rand::gen_range(0, HOUSE_UPDATE_TICKS),
            production_rate: HOUSE_UPDATE_TICKS,
            building_type: BuildingType::House,
            routes: Vec::new(),
            cargo_count: 0,
        }
    }

    pub fn new_station(pos: Position) -> Self {
        Building {
            pos,
            dir: None,
            color: SpawnerColors::Red,
            city_id: 0,
            my_vehicle: None,
            arrived_count: 0,
            production_tics: rand::gen_range(0, SPAWNER_UPDATE_TICKS),
            production_rate: SPAWNER_UPDATE_TICKS,
            building_type: BuildingType::Station,
            routes: Vec::new(),
            cargo_count: 0,
        }
    }

    pub fn new_spawner(pos: Position, dir: Direction, color: SpawnerColors, city_id: Id) -> Self {
        Building {
            pos,
            dir: Some(dir),
            color,
            city_id,
            my_vehicle: None,
            arrived_count: 0,
            production_tics: rand::gen_range(0, SPAWNER_UPDATE_TICKS),
            production_rate: SPAWNER_UPDATE_TICKS,
            building_type: BuildingType::Spawner,
            routes: Vec::new(),
            cargo_count: 0,
        }
    }

    // ONLY FOR HOUSES!!!
    pub fn driveway_pos(pos: Position, grid: &Grid) -> Option<(Position, Direction)> {
        // if let Some(dir) = self.dir {
            // let dir = dir.inverse();
            // return Some((RoadPosition::corner_pos(self.pos, dir), dir.inverse()));
        // }
        for dir in Direction::ALL {
            // let dir = pos.default_connections()[1];
            let pos_adj = pos + dir;
            if let Some(Tile::Road(_)) = grid.get_tile(&pos_adj) {
                return Some((pos_adj, dir));
            }
        }
        // should there be a default?
        None
    }

    pub fn destination_pos(&self, grid: &Grid) -> Option<(Position, Direction)> {
        if let Some(dir) = self.dir {
            return Some((self.pos, dir.inverse()));
        }
        for dir in Direction::ALL {
            // let dir = pos.default_connections()[0].inverse();
            let pos_adj = self.pos + dir;
            if let Some(Tile::Road(_)) = grid.get_tile(&pos_adj) {
                return Some((self.pos, dir));
            }
        }
        None
    }

    fn arrived_max(&self) -> i32 {
        match self.building_type {
            BuildingType::House => 1,
            BuildingType::Station => 10, // This makes no sense...
            BuildingType::Spawner => 10,
        }
    }

    pub fn update_arrived(&mut self, success: bool) {
        if success {
            if self.arrived_count < self.arrived_max() {
                self.arrived_count += 1;
            } else {
                // Temp: to fix save games in progress
                self.arrived_count = self.arrived_max();
            }
        } else if self.arrived_count > 0 {
            self.arrived_count -= 1;
        }
    }

    pub fn arrived_goal_met(&self) -> bool {
        self.arrived_count >= self.arrived_max()
    }

    pub fn update(&mut self) -> bool {
        if self.building_type != BuildingType::Station && self.my_vehicle.is_none() {
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

    // TODO: for stations only!
    // TODO: return false if over max?
    pub fn add_cargo(&mut self) {
        self.cargo_count += 1;
        if self.cargo_count > CARGO_MAX {
            self.cargo_count = CARGO_MAX;
        }
    }

    pub fn pickup_cargo(&mut self) -> u32 {
        let amount = self.cargo_count.min(10);
        self.cargo_count -= amount;
        amount
    }
}
