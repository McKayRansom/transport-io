use macroquad::{color::WHITE, prelude::rand};
use serde::{Deserialize, Serialize};

use super::{
    build::{BuildError, BuildResult},
    building::Building,
    grid::Grid,
    Direction, Position,
};

use crate::{
    hash_map_id::{HashMapId, Id},
    tileset::Tileset,
};

const CITY_GROW_TICKS: u32 = 16 * 10;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct City {
    pos: Position,
    name: String,
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

    pub fn generate(
        &mut self,
        buildings: &mut HashMapId<Building>,
        grid: &mut Grid,
    ) -> BuildResult {
        self.generate_center_roads(grid)?;

        self.generate_building(self.pos + (2, 2).into(), buildings, grid)?;
        self.generate_building(self.pos + (2, -2).into(), buildings, grid)?;
        self.generate_building(self.pos + (-2, 2).into(), buildings, grid)?;
        self.generate_building(self.pos + (-2, -2).into(), buildings, grid)?;

        for _ in 0..10 {
            self.grow_building(buildings, grid);
        }

        Ok(())
    }

    fn generate_center_roads(&mut self, grid: &mut Grid) -> BuildResult {
        for i in -10..10 {
            grid.build_two_way_road(self.pos + (i, 0).into(), Direction::LEFT)?;
            grid.build_two_way_road(self.pos + (0, i).into(), Direction::DOWN)?;
        }

        Ok(())
    }

    pub fn generate_building(
        &mut self,
        pos: Position,
        buildings: &mut HashMapId<Building>,
        grid: &mut Grid,
    ) -> BuildResult {
        let pos = pos.round_to(2);
        self.houses
            .push(grid.build_building(buildings, Building::new_house(pos, self.id))?);

        Ok(())
    }

    fn grow_building(&mut self, buildings: &mut HashMapId<Building>, grid: &mut Grid) {
        let start_house_id = self.random_house();
        if let Some(building) = buildings.hash_map.get(&start_house_id) {
            let mut building_pos = building.pos;
            let dir = Direction::random();
            loop {
                let pos = building_pos + dir;
                building_pos = pos;
                match self.generate_building(pos, buildings, grid) {
                    Ok(_) => break,
                    Err(BuildError::OccupiedTile) => continue,
                    Err(BuildError::InvalidTile) => break,
                }
            }
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        tileset.draw_text(self.name.as_str(), 32., WHITE, &self.pos.into());
    }

    pub fn update(&mut self, buildings: &mut HashMapId<Building>, grid: &mut Grid) {
        self.grow_ticks += 1;
        if self.grow_ticks > self.grow_rate {
            self.grow_ticks = 0;
            self.grow_building(buildings, grid);
        }
    }

    pub fn random_house(&self) -> Id {
        let index = rand::gen_range(0, self.houses.len());
        self.houses[index]
    }
}
