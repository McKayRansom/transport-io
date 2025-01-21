use macroquad::{color::WHITE, prelude::rand};
use serde::{Deserialize, Serialize};

use super::{
    build::BuildError,
    building::{Building, BUILDING_SIZE},
    grid::Grid,
    Direction, Position,
};

use crate::{
    hash_map_id::Id,
    tileset::Tileset,
};

const CITY_GROW_TICKS: u32 = 16 * 10;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct City {
    pos: Position,
    pub name: String,
    // pub rating: f32,
    grow_ticks: u32,
    pub grow_rate: u32,
    id: Id,
    pub houses: Vec<Id>,
    // vehicle_on_the_way: Option<Id>,
}

impl City {
    pub fn new(id: Id, pos: Position, name: String) -> Self {
        City {
            pos,
            name,
            grow_ticks: rand::gen_range(0, CITY_GROW_TICKS),
            grow_rate: CITY_GROW_TICKS,
            id,
            houses: Vec::new(),
        }
    }


    pub fn grow_building(&self, grid: &Grid) -> Option<Building> {
        let start_house_id = self.random_house();
        if let Some(building) = grid.buildings.hash_map.get(&start_house_id) {
            let mut building_pos = building.pos;
            let dir = Direction::random();
            loop {
                let pos = building_pos + dir;
                building_pos = pos;
                match grid.is_area_clear(&pos, BUILDING_SIZE) {
                    Ok(_) => return Some(Building::new_house(pos, self.id)),
                    Err(BuildError::OccupiedTile) => continue,
                    Err(BuildError::InvalidTile) => break,
                }
            }
        }

        None
    }

    pub fn draw(&self, tileset: &Tileset) {
        tileset.draw_text(self.name.as_str(), 32., WHITE, &self.pos.into());
    }

    pub fn update(&mut self, grid: &mut Grid) -> Option<Building> {
        self.grow_ticks += 1;
        if self.grow_ticks > self.grow_rate {
            self.grow_ticks = 0;
            self.grow_building(grid)
        } else {
            None
        }
    }

    pub fn random_house(&self) -> Id {
        let index = rand::gen_range(0, self.houses.len());
        self.houses[index]
    }
}
