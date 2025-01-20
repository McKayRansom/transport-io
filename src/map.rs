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

pub mod draw;
pub mod levels;
pub mod vehicle;

mod position;
pub use position::Position;
mod direction;
pub use direction::Direction;
use vehicle::{Status, Vehicle};

use crate::{
    consts::SpawnerColors,
    hash_map_id::{HashMapId, Id},
};

const _CITY_BLOCK_SIZE: i16 = 8;
const _CITY_BLOCK_COUNT: i16 = 1;

pub const DEFAULT_CITY_ID: Id = 1;

type VehicleHashMap = HashMapId<Vehicle>;
type BuildingHashMap = HashMapId<Building>;
type CityHashMap = HashMapId<City>;

#[derive(Serialize, Deserialize, Default)]
pub struct MapMetadata {
    pub is_level: bool,
    pub grow_cities: bool,
    pub level_complete: bool,
    pub level_number: usize,
    // pub level_name: &'static str,
    // pub level_hint: &'static str,
}

impl MapMetadata {
    pub fn new() -> Self {
        Self {
            grow_cities: true,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub metadata: MapMetadata,
    pub grid: Grid,
    pub vehicles: VehicleHashMap,
    cities: CityHashMap,
}

impl Map {
    pub fn new_blank(size: (i16, i16)) -> Self {
        srand(1234);
        Map {
            metadata: MapMetadata::new(),
            grid: Grid::new(size),
            vehicles: VehicleHashMap::new(),
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

    pub fn new_from_string(string: &str) -> Self {
        let mut map = Map {
            metadata: MapMetadata::new(),
            grid: Grid::new_from_string(string),
            vehicles: VehicleHashMap::new(),
            cities: CityHashMap::new(),
        };

        let city_id = map.new_city(
            (map.grid.size().0 / 2, map.grid.size().1 / 2).into(),
            "default city".into(),
        );

        // fixup cities
        for (id, building) in map.grid.buildings.hash_map.iter_mut() {
            building.city_id = city_id;
            map.cities
                .hash_map
                .get_mut(&city_id)
                .unwrap()
                .houses
                .push(*id);
        }

        map
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
            BuildActionBuilding::new(self, Building::new_house(pos + dir.into(), city_id))
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
        start: Option<(Position, Direction)>,
        end: Id,
        color: SpawnerColors,
    ) -> Option<Id> {
        let id = self.vehicles.id;
        if let Ok(mut vehicle) = Vehicle::new(id, start?, end, &mut self.grid) {
            vehicle.color = color;
            Some(self.vehicles.insert(vehicle))
        } else {
            None
        }
    }

    pub fn reserve_building_id(&mut self) -> Id {
        self.grid.buildings.reserve_id()
    }

    pub fn remove_building(&mut self, id: &Id) -> Option<Building> {
        if let Some(building) = self.grid.buildings.hash_map.remove(id) {
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
        self.grid.buildings.hash_map.get(id)
    }

    pub fn insert_building(&mut self, id: Id, building: Building) {
        if let Some(city) = self.cities.hash_map.get_mut(&building.city_id) {
            city.houses.push(id);
        }

        self.grid.buildings.hash_map.insert(id, building);
    }

    fn update_buildings(&mut self) -> bool {
        let mut vehicles_to_add: Vec<Id> = Vec::new();
        let mut all_goals_met = true;

        for (id, building) in &mut self.grid.buildings.hash_map {
            if building.update() {
                vehicles_to_add.push(*id);
            }

            if building.arrived_count < 10 {
                all_goals_met = false;
            }
        }

        for building_id in vehicles_to_add {
            // generate a random destination
            let start_building = self.grid.buildings.hash_map.get(&building_id).unwrap();

            let end_building_id = self.cities.hash_map[&start_building.city_id].random_house();

            if end_building_id == building_id {
                continue;
            }

            if let Some(destination_building) = self.grid.buildings.hash_map.get(&end_building_id) {
                if self
                    .add_vehicle(
                        start_building.spawn_pos(&self.grid),
                        end_building_id,
                        destination_building.color,
                    )
                    .is_none()
                {
                    self.grid
                        .buildings
                        .hash_map
                        .get_mut(&end_building_id)
                        .unwrap()
                        .update_arrived(false);
                };

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
            // let building_id = tile.get_building_id()
            if let Some(building) = self.grid.buildings.hash_map.get_mut(&vehicle.destination) {
                building.update_arrived(status == Status::ReachedDestination);
            }
            self.vehicles.hash_map.remove(&id);

            // self.update_rating(id.1 == Status::ReachedDestination);
        }

        let all_goals_met = self.update_buildings();

        let mut building_to_add: Vec<Building> = Vec::new();
        for city in self.cities.values_mut() {
            if let Some(building) = city.update(&mut self.grid) {
                building_to_add.push(building);
            }
        }

        if self.metadata.grow_cities {
            for building in building_to_add {
                let _ = BuildActionBuilding::new(self, building).execute(self);
            }
        }

        all_goals_met && !self.metadata.level_complete
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
