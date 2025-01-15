use build::{BuildAction, BuildActionBuilding, BuildResult};
use building::Building;
use city::City;
use grid::{Grid, ReservationError};
use macroquad::rand::srand;
use serde::{Deserialize, Serialize};

pub mod build;
pub mod building;
mod city;
pub mod grid;
pub mod tile;

pub mod levels;
pub mod vehicle;

mod position;
pub use position::Position;
mod direction;
pub use direction::Direction;
use tile::Tile;
use vehicle::{Status, Vehicle};

use crate::{
    consts::SpawnerColors,
    hash_map_id::{HashMapId, Id},
    tileset::Tileset,
};

const _CITY_BLOCK_SIZE: i16 = 8;
const _CITY_BLOCK_COUNT: i16 = 1;

type VehicleHashMap = HashMapId<Vehicle>;
type BuildingHashMap = HashMapId<Building>;
type CityHashMap = HashMapId<City>;

#[derive(Serialize, Deserialize, Default)]
pub struct MapMetadata {
    pub is_level: bool,
    pub level_complete: bool,
    pub level_number: usize,
}

impl MapMetadata {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub metadata: MapMetadata,
    pub grid: Grid,
    pub vehicles: VehicleHashMap,
    buildings: BuildingHashMap,
    cities: CityHashMap,
}

impl Map {
    pub fn new_blank(size: (i16, i16)) -> Self {
        srand(1234);
        Map {
            metadata: MapMetadata::new(),
            grid: Grid::new(size),
            vehicles: VehicleHashMap::new(),
            buildings: BuildingHashMap::new(),
            cities: CityHashMap::new(),
        }
    }

    pub fn new_generate(size: (i16, i16)) -> Self {
        let mut map = Self::new_blank(size);

        map.generate().expect("Error generating map");

        map
    }

    pub fn fixup(&mut self) -> Result<(), ReservationError> {
        // Any way to not allow this to be called twice?
        for vehicle in &mut self.vehicles.values_mut() {
            vehicle.fixup(&mut self.grid)?
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn new_from_string(string: &str) -> Self {
        Map {
            metadata: MapMetadata::new(),
            grid: Grid::new_from_string(string),
            vehicles: VehicleHashMap::new(),
            buildings: BuildingHashMap::new(),
            cities: CityHashMap::new(),
        }
    }

    pub fn new_city(&mut self, pos: Position, name: String) -> Id {
        self.cities.insert(City::new(self.cities.id, pos, name))
    }

    #[allow(unused)]
    pub fn get_city(&mut self, id: Id) -> Option<&City> {
        self.cities.hash_map.get(&id)
    }

    pub fn get_city_mut(&mut self, id: Id) -> Option<&mut City> {
        self.cities.hash_map.get_mut(&id)
    }

    fn generate_center_roads(&mut self) -> BuildResult {
        // We need a mut Map for this :(
        // build::action_two_way_road(self.pos, self.pos + Direction::RIGHT * 5)
        //     .execute(map);
        // grid.build_two_way_road(self.pos, self.pos + Direction::LEFT * 5)?;
        // grid.build_two_way_road(self.pos, self.pos + Direction::UP * 5)?;
        // grid.build_two_way_road(self.pos, self.pos + Direction::DOWN * 5)?;

        Ok(())
    }

    fn new_city_generate(&mut self, pos: Position, name: String) -> BuildResult {
        let city_id = self.new_city(pos, name);

        self.generate_center_roads()?;

        for dir in [(2, 2), (2, -2), (-2, 2), (-2, -2)] {
            BuildActionBuilding::new(Building::new_house(pos + dir.into(), city_id))
                .execute(self)?;
        }

        // let city = self.get_city(city_id).unwrap();
        // for _ in 0..10 {
        //     if let Some(building)  = city.grow_building(&self.buildings, &self.grid) {
        //         self.add_building(building);
        //     }
        // }

        Ok(())
    }

    fn generate(&mut self) -> BuildResult {
        let size: Position = self.grid.size().into();
        let grid_center: Position = size / 2;
        self.new_city_generate(grid_center, "CityVille".to_string())
    }

    pub fn add_vehicle(
        &mut self,
        start_pos: Position,
        end_pos: Position,
        color: SpawnerColors,
    ) -> Option<Id> {
        let id = self.vehicles.id;
        if let Ok(mut vehicle) = Vehicle::new(start_pos, id, end_pos, &mut self.grid) {
            vehicle.color = color;
            Some(self.vehicles.insert(vehicle))
        } else {
            None
        }
    }

    pub fn reserve_building_id(&mut self) -> Id {
        self.buildings.reserve_id()
    }

    pub fn add_building(&mut self, building: Building) -> Id {
        let id = self.buildings.insert(building);

        if let Some(city) = self.cities.hash_map.get_mut(&building.city_id) {
            city.houses.push(id);
        }

        id
    }

    pub fn remove_building(&mut self, id: &Id) -> Option<Building> {
        if let Some(building) = self.buildings.hash_map.remove(id) {
            if let Some(city) = self.cities.hash_map.get_mut(&building.city_id) {
                if let Some(pos) = city.houses.iter().position(|x| x == id) {
                    city.houses.swap_remove(pos);
                }
            }
            Some(building)
        } else {
            None
        }
    }

    pub fn get_building(&self, id: &Id) -> Option<&Building> {
        self.buildings.hash_map.get(id)
    }

    pub fn insert_building(&mut self, id: Id, building: Building) {
        if let Some(city) = self.cities.hash_map.get_mut(&building.city_id) {
            city.houses.push(id);
        }

        self.buildings.hash_map.insert(id, building);
    }

    fn update_buildings(&mut self) -> bool {
        let mut vehicles_to_add: Vec<(Id, Position)> = Vec::new();
        let mut all_goals_met = true;

        for building in self.buildings.values_mut() {
            if building.update() {
                vehicles_to_add.push((building.city_id, building.pos));
            }

            if building.arrived_count < 10 {
                all_goals_met = false;
            }
        }

        for (city_id, start_pos) in vehicles_to_add {
            // generate a random destination
            if let Some(destination_building) = self
                .buildings
                .hash_map
                .get(&self.cities.hash_map[&city_id].random_house())
            {
                if destination_building.pos == start_pos {
                    // TODO: Some way to not pick the same house
                    continue;
                }
                // TODO: PASS THIS MESS
                let _vehicle = self.add_vehicle(
                    start_pos,
                    destination_building.pos,
                    destination_building.color,
                );

                // destination_building.vehicle_on_the_way = _vehicle;
            }
        }

        all_goals_met
    }

    pub fn update(&mut self) -> bool {
        let mut to_remove: Vec<(Id, Status)> = Vec::new();
        for s in self.vehicles.hash_map.iter_mut() {
            let status = s.1.update(&mut self.grid);
            if status != Status::EnRoute {
                to_remove.push((*s.0, status));
            }
        }
        for (id, status) in to_remove {
            let vehicle = self.vehicles.hash_map.get_mut(&id).unwrap();
            if let Some(Some(building_id)) = self
                .grid
                .get_tile(&vehicle.destination)
                .map(Tile::get_building_id)
            {
                // let building_id = tile.get_building_id()
                if let Some(building) = self.buildings.hash_map.get_mut(&building_id) {
                    if status == Status::ReachedDestination {
                        building.arrived_count += 1;
                    } else if building.arrived_count > 0 {
                        building.arrived_count -= 1;
                    }
                }
            }
            self.vehicles.hash_map.remove(&id);

            // self.update_rating(id.1 == Status::ReachedDestination);
        }

        let all_goals_met = self.update_buildings();

        let mut building_to_add: Vec<Building> = Vec::new();
        for city in self.cities.values_mut() {
            if let Some(building) = city.update(&mut self.buildings, &mut self.grid) {
                building_to_add.push(building);
            }
        }

        all_goals_met && !self.metadata.level_complete
    }

    pub fn draw(&self, tileset: &Tileset) {
        self.grid.draw_tiles(tileset);

        for s in self.vehicles.hash_map.iter() {
            if s.1.pos.z == 0 {
                s.1.draw(tileset);
            }
        }

        self.grid.draw_bridges(tileset);

        for s in self.vehicles.hash_map.iter() {
            if s.1.pos.z == 1 {
                s.1.draw(tileset);
            }
        }

        for b in self.buildings.hash_map.values() {
            b.draw(tileset);
        }

        for c in self.cities.hash_map.values() {
            c.draw(tileset);
        }
    }
}

#[cfg(test)]
mod map_tests {

    use super::*;

    #[test]
    fn test_map_rating() {
        Map::new_from_string(">>>>");
        // assert_eq!(map.rating, 1.0);
        // map.update_rating(true);
        // assert_eq!(map.rating, 1.0);
        // map.update_rating(false);
        // assert_eq!(map.rating, 0.9);
    }

    #[test]
    fn test_map_generate() {
        // let mut map = Map::new_from_string("__\n__");

        // map.new_city((0, 0).into(), "test_city".to_string()).unwrap();

        // assert_eq!(map.buildings.len(), 1);

        // assert_eq!(map.vehicles.len(), 0);
        // assert_eq!(map.vehicle_id, 1);

        // for _ in 0..10 * 16 {
        //     map.update_buildings();
        // }

        // assert_eq!(map.vehicles.len(), 1);
        // assert_eq!(map.vehicle_id, 2);
    }
}
