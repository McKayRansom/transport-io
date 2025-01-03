use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use macroquad::rand::srand;
use serde::{Deserialize, Serialize};

use crate::{
    building::Building,
    city::City,
    grid::{self, BuildError, BuildResult, Direction, Grid, Id, Position, ReservationError},
    tile::Tile,
    tileset::Tileset,
    vehicle::{Status, Vehicle},
};

const _CITY_BLOCK_SIZE: i16 = 8;
const _CITY_BLOCK_COUNT: i16 = 1;

pub const GRID_SIZE: (i16, i16) = (64, 64);
pub const GRID_CENTER: (i16, i16) = (33, 33);

type VehicleHashMap = HashMap<grid::Id, Vehicle>;
type BuildingHashMap = HashMap<grid::Id, Building>;

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub grid: Grid,
    pub vehicle_id: grid::Id,
    pub vehicles: VehicleHashMap,
    pub building_id: grid::Id,
    pub buildings: BuildingHashMap,
    pub cities_id: grid::Id,
    pub cities: HashMap<grid::Id, City>,
}

impl Map {
    pub fn new() -> Self {
        srand(1234);
        Map {
            grid: Grid::new(GRID_SIZE.0 as usize, GRID_SIZE.1 as usize),
            vehicle_id: 1,
            vehicles: HashMap::new(),
            building_id: 1,
            buildings: HashMap::new(),
            cities_id: 1,
            cities: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &Path) -> std::io::Result<Map> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut map: Map = serde_json::from_str(&contents)?;

        if map.fixup().is_err() {
            println!("Failed to fixup map!")
            // Err()
        }

        Ok(map)
    }

    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
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
            vehicle_id: 1,
            vehicles: HashMap::new(),
            building_id: 1,
            buildings: HashMap::new(),
            cities_id: 1,
            cities: HashMap::new(),
        }
    }

    pub fn _generate_road(&mut self, x: i16, y: i16, dir: Direction) -> BuildResult {
        let pos = self.grid.pos(x, y);

        self.grid.build_road(&pos, dir)
    }

    pub fn new_city(&mut self, pos: Position, name: String) -> Id {
        let id = self.building_id;
        self.cities
            .insert(id, City::new(self.building_id, pos, name));

        self.building_id += 1;

        id
    }

    pub fn generate_building(&mut self, x: i16, y: i16, city_id: Id) -> BuildResult {
        let pos = self.grid.pos(x, y);

        let building = Building::new(pos, self.building_id, city_id);

        self.grid.build_building(&pos, self.building_id)?;

        self.buildings.insert(self.building_id, building);
        if let Some(city) = self.cities.get_mut(&city_id) {
            city.houses.push(self.building_id);
        }
        self.building_id += 1;

        Ok(())
    }

    pub fn _generate_block(&mut self, x: i16, y: i16, city_id: Id) -> BuildResult {
        // top

        for i in 0.._CITY_BLOCK_SIZE {
            self._generate_road(x + i, y, Direction::RIGHT)?;
            self._generate_road(x + (_CITY_BLOCK_SIZE - 1), y + i, Direction::DOWN)?;
            self._generate_road(x + i, y + (_CITY_BLOCK_SIZE - 1), Direction::LEFT)?;
            self._generate_road(x, y + i, Direction::UP)?;
        }

        // buildings (all for now)
        for i in 0.._CITY_BLOCK_SIZE {
            for j in 0.._CITY_BLOCK_SIZE {
                self.generate_building(x + i, y + j, city_id)?;
            }
        }

        Ok(())
    }

    fn generate_center_roads(&mut self) -> BuildResult {
        for i in -10..10 {
            self.grid.build_two_way_road(
                self.grid.pos(GRID_CENTER.0 + i, GRID_CENTER.1),
                Direction::LEFT,
            )?;
            self.grid.build_two_way_road(
                self.grid.pos(GRID_CENTER.0, GRID_CENTER.1 + i),
                Direction::DOWN,
            )?;
        }

        Ok(())
    }

    fn grow_building(&mut self, city_id: Id) {
        let start_house_id = self.cities[&city_id].random_house();
        if let Some(building) = self.buildings.get(&start_house_id) {
            let mut building_pos = building.pos;
            let dir = Direction::random();
            loop {
                let pos = building_pos + dir;
                building_pos = pos;
                match self.generate_building(pos.x, pos.y, city_id) {
                    Ok(_) => break,
                    Err(BuildError::OccupiedTile) => continue,
                    Err(BuildError::InvalidTile) => break,
                }
            }
        }
    }

    fn generate_first_buildings(&mut self) -> BuildResult {
        let city_id = self.new_city(GRID_CENTER.into(), "C".to_string());

        self.generate_building(GRID_CENTER.0 + 1, GRID_CENTER.1 + 1, city_id)?;
        self.generate_building(GRID_CENTER.0 + 1, GRID_CENTER.1 - 2, city_id)?;
        self.generate_building(GRID_CENTER.0 - 2, GRID_CENTER.1 + 1, city_id)?;
        self.generate_building(GRID_CENTER.0 - 2, GRID_CENTER.1 - 2, city_id)?;

        for _ in 0..50 {
            self.grow_building(city_id);
        }

        Ok(())
    }

    pub fn generate(&mut self) -> BuildResult {
        self.generate_center_roads()?;
        self.generate_first_buildings()?;
        // the oofs
        // for i in 0..CITY_BLOCK_COUNT {
        // for j in 0..CITY_BLOCK_COUNT {
        // self.generate_block(i * CITY_BLOCK_SIZE, j * CITY_BLOCK_SIZE);
        // }
        // }
        Ok(())
    }

    pub fn add_vehicle(&mut self, start_pos: Position, end_pos: Position) -> Option<Id> {
        let id = self.vehicle_id;
        if let Ok(vehicle) = Vehicle::new(start_pos, self.vehicle_id, end_pos, &mut self.grid) {
            self.vehicles.insert(id, vehicle);
            self.vehicle_id += 1;

            Some(id)
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
            if let Some(destination_building) =
                self.buildings.get_mut(&self.cities[&city_id].random_house())
            {
                let destination_pos = destination_building.pos;
                let _vehicle = self.add_vehicle(start_pos, destination_pos);

                // destination_building.vehicle_on_the_way = vehicle;
            }
        }
    }

    pub fn update(&mut self) {
        let mut to_remove: Vec<(grid::Id, Status)> = Vec::new();
        for s in self.vehicles.iter_mut() {
            let status = s.1.update(&mut self.grid);
            if status != Status::EnRoute {
                to_remove.push((*s.0, status));
            }
        }
        for id in to_remove {
            let vehicle = self.vehicles.get_mut(&id.0).unwrap();
            if let Some(Tile::Building(building_id)) = self.grid.get_tile(&vehicle.destination) {
                if let Some(building) = self.buildings.get_mut(building_id) {
                    building.vehicle_on_the_way = None;
                }
            }
            self.vehicles.remove(&id.0);

            // self.update_rating(id.1 == Status::ReachedDestination);
        }

        self.update_buildings();

        // if self.rating > 0.9 {
        //     self.grow_ticks += 1;
        //     if self.grow_ticks > 60 {
        //         self.grow_building();
        //         self.grow_ticks = 0;
        //     }
        // }
    }

    pub fn draw(&self, tileset: &Tileset) {
        self.grid.draw_tiles(tileset);

        for s in self.vehicles.iter() {
            if s.1.pos.z == 0 {
                s.1.draw(tileset);
            }
        }

        self.grid.draw_bridges(tileset);

        for s in self.vehicles.iter() {
            if s.1.pos.z == 1 {
                s.1.draw(tileset);
            }
        }

        for b in self.buildings.values() {
            b.draw(tileset);
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
        let mut map = Map::new_from_string("__");

        let city = map.new_city((0, 0).into(), "test_city".to_string());

        map.generate_building(0, 0, city).unwrap();
        map.generate_building(1, 0, city).unwrap();

        assert_eq!(map.buildings.len(), 2);

        assert_eq!(map.vehicles.len(), 0);
        assert_eq!(map.vehicle_id, 1);

        for _ in 0 .. 10 * 16 {
            map.update_buildings();
        }

        assert_eq!(map.vehicles.len(), 2);
        assert_eq!(map.vehicle_id, 3);
    }

    #[test]
    fn test_map_serialize() {
        let mut map = Map::new();

        map._generate_road(0, 0, Direction::RIGHT).unwrap();

        map.add_vehicle(map.grid.pos(0, 0), map.grid.pos(1, 0));

        let test_path = Path::new("saves/test_map.json");

        map.save_to_file(test_path).unwrap();

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
