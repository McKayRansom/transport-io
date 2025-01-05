use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use build::BuildResult;
use building::Building;
use city::City;
use grid::{Grid, ReservationError};
use macroquad::rand::srand;
use serde::{Deserialize, Serialize};

pub mod build;
mod building;
mod city;
pub mod grid;
pub mod tile;

pub mod vehicle;
pub mod levels;

mod position;
pub use position::Position;
mod direction;
pub use direction::Direction;
use tile::Tile;
use vehicle::{Status, Vehicle};

use crate::{
    tileset::Tileset,
    hash_map_id::{HashMapId, Id},
};

const _CITY_BLOCK_SIZE: i16 = 8;
const _CITY_BLOCK_COUNT: i16 = 1;

pub const GRID_SIZE: (i16, i16) = (64, 64);
pub const GRID_CENTER: (i16, i16) = (32, 32);

type VehicleHashMap = HashMapId<Vehicle>;
type BuildingHashMap = HashMapId<Building>;
type CityHashMap = HashMapId<City>;

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub grid: Grid,
    pub vehicles: VehicleHashMap,
    pub buildings: BuildingHashMap,
    pub cities: CityHashMap,
}

impl Map {
    pub fn new() -> Self {
        srand(1234);
        Map {
            grid: Grid::new(GRID_SIZE.0 as usize, GRID_SIZE.1 as usize),
            vehicles: VehicleHashMap::new(),
            buildings: BuildingHashMap::new(),
            cities: CityHashMap::new(),
        }
    }

    pub fn load_from_file(path: String) -> std::io::Result<Map> {

        let mut file = File::open(Path::new(&path))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut map: Map = serde_json::from_str(&contents)?;

        if map.fixup().is_err() {
            println!("Failed to fixup map!")
            // Err()
        }

        Ok(map)
    }

    pub fn save_to_file(&self, path_str: &String) -> std::io::Result<()> {
        let path = Path::new(path_str);
        let _ = fs::create_dir_all(path.parent().unwrap());

        let mut file = File::create(path)?;

        let buf = serde_json::to_string(self).unwrap();

        file.write_all(buf.as_bytes())?;

        Ok(())
    }

    fn fixup(&mut self) -> Result<(), ReservationError> {
        // Any way to not allow this to be called twice?
        for vehicle in &mut self.vehicles.values_mut() {
            vehicle.fixup(&mut self.grid)?
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn new_from_string(string: &str) -> Self {
        Map {
            grid: Grid::new_from_string(string),
            vehicles: VehicleHashMap::new(),
            buildings: BuildingHashMap::new(),
            cities: CityHashMap::new(),
        }
    }

    pub fn new_city(&mut self, pos: Position, name: String) -> BuildResult {
        let mut city = City::new(self.cities.id, pos, name);
        city.generate(&mut self.buildings, &mut self.grid)?;
        self.cities.insert(city);
        Ok(())
    }

    pub fn generate(&mut self) -> BuildResult {
        self.new_city(GRID_CENTER.into(), "CityVille".to_string())
    }

    pub fn add_vehicle(&mut self, start_pos: Position, end_pos: Position) -> Option<Id> {
        let id = self.vehicles.id;
        if let Ok(vehicle) = Vehicle::new(start_pos, id, end_pos, &mut self.grid) {
            Some(self.vehicles.insert(vehicle))
        } else {
            None
        }
    }

    fn update_buildings(&mut self) {
        let mut vehicles_to_add: Vec<(Id, Position)> = Vec::new();
        for building in self.buildings.values_mut() {
            if building.update() {
                vehicles_to_add.push((building.city_id, building.pos));
            }
        }

        for (city_id, start_pos) in vehicles_to_add {
            // generate a random destination
            if let Some(destination_building) = self
                .buildings.hash_map
                .get_mut(&self.cities.hash_map[&city_id].random_house())
            {
                let destination_pos = destination_building.pos;
                let _vehicle = self.add_vehicle(start_pos, destination_pos);

                // destination_building.vehicle_on_the_way = _vehicle;
            }
        }
    }

    pub fn update(&mut self) {
        let mut to_remove: Vec<(Id, Status)> = Vec::new();
        for s in self.vehicles.hash_map.iter_mut() {
            let status = s.1.update(&mut self.grid);
            if status != Status::EnRoute {
                to_remove.push((*s.0, status));
            }
        }
        for id in to_remove {
            let vehicle = self.vehicles.hash_map.get_mut(&id.0).unwrap();
            if let Some(Tile::Building(building_id)) = self.grid.get_tile(&vehicle.destination) {
                if let Some(building) = self.buildings.hash_map.get_mut(building_id) {
                    building.vehicle_on_the_way = None;
                }
            }
            self.vehicles.hash_map.remove(&id.0);

            // self.update_rating(id.1 == Status::ReachedDestination);
        }

        self.update_buildings();

        for city in self.cities.values_mut() {
            city.update(&mut self.buildings, &mut self.grid);
        }

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

    #[test]
    fn test_map_serialize() {
        let mut map = Map::new();

        map.add_vehicle(map.grid.pos(0, 0), map.grid.pos(1, 0));

        let test_path ="saves/test_map.json".to_string();

        map.save_to_file(&test_path).unwrap();

        let mut deserialized: Map = Map::load_from_file(test_path).unwrap();

        assert_eq!(
            deserialized.grid.get_tile(&deserialized.grid.pos(0, 0)),
            map.grid.get_tile(&deserialized.grid.pos(0, 0)),
        );

        let pos = deserialized.grid.pos(0, 0);

        assert!(deserialized
            .grid
            .get_tile_mut(&pos)
            .unwrap()
            .reserve(1234, pos)
            .is_err())
    }
}
