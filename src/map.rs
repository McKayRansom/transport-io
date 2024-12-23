use std::{collections::HashMap, fs::{self, File}, io::{Read, Write}, path::Path};

use macroquad::rand::{self, srand};
use serde::{Deserialize, Serialize};

use crate::{
    grid::{self, Direction, Grid, Id, Position},
    tile::{House, Tile},
    tileset::Tileset,
    vehicle::{Status, Vehicle},
};

const CITY_BLOCK_SIZE: i16 = 8;
const CITY_BLOCK_COUNT: i16 = 1;

pub const GRID_SIZE: (i16, i16) = (64, 64);
pub const GRID_CENTER: (i16, i16) = (33, 33);

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub grid: Grid,
    pub vehicle_id: grid::Id,
    pub vehicles: HashMap<grid::Id, Vehicle>,
    pub rating: f32,
    pub grow_ticks: u32,
    pub houses: Vec<Position>,
}

impl Map {
    pub fn new() -> Self {
        srand(1234);
        Map {
            grid: Grid::new(GRID_SIZE.0 as usize, GRID_SIZE.1 as usize),
            vehicle_id: 1,
            vehicles: HashMap::new(),
            rating: 1.0,
            grow_ticks: 0,
            houses: Vec::new(),
        }
    }

    pub fn load_from_file(path: &Path) -> std::io::Result<Map> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut map: Map = serde_json::from_str(&contents)?;

        map.fixup();

        Ok(map)
    }

    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {

        let _ = fs::create_dir_all(path.parent().unwrap());

        let mut file = File::create(path)?;

        let buf = serde_json::to_string(self).unwrap();

        file.write_all(buf.as_bytes())?;

        Ok(())
    }

    fn fixup(&mut self) {
        // Any way to not allow this to be called twice?
        for (_id, vehicle) in &mut self.vehicles {
            vehicle.fixup(&mut self.grid);
        }
    }

    #[allow(unused)]
    pub fn new_from_string(string: &str) -> Self {
        Map {
            grid: Grid::new_from_string(string),
            vehicle_id: 1,
            vehicles: HashMap::new(),
            rating: 1.0,
            grow_ticks: 0,
            houses: Vec::new(),
        }
    }

    pub fn generate_road(&mut self, x: i16, y: i16, dir: Direction) {
        if let Some(pos) = self.grid.try_pos(x, y) {
            self.grid
                .get_tile_mut(&pos)
                .edit_road(|road| road.connect(dir));
        }
    }

    pub fn generate_house(&mut self, x: i16, y: i16) -> bool {
        let mut success = false;
        if let Some(pos) = self.grid.try_pos(x, y) {
            let tile = self.grid.get_tile_mut(&pos);
            tile.build(|| {
                success = true;
                Tile::House(House::new())
            });
            if success {
                self.houses.push(pos);
            }
        }
       success
    }

    pub fn _generate_block(&mut self, x: i16, y: i16) {
        // top
        for i in 0..CITY_BLOCK_SIZE {
            self.generate_road(x + i, y, Direction::Right);
            self.generate_road(x + (CITY_BLOCK_SIZE - 1), y + i, Direction::Down);
            self.generate_road(x + i, y + (CITY_BLOCK_SIZE - 1), Direction::Left);
            self.generate_road(x, y + i, Direction::Up);
        }

        // houses (all for now)
        for i in 0..CITY_BLOCK_SIZE {
            for j in 0..CITY_BLOCK_SIZE {
                self.generate_house(x + i, y + j);
            }
        }
    }

    fn generate_center_roads(&mut self) {
        for i in -5..5 {
            self.grid.build_two_way_road(self.grid.pos(GRID_CENTER.0 + i, GRID_CENTER.1), Direction::Left);
            self.grid.build_two_way_road(self.grid.pos(GRID_CENTER.0 + 0, GRID_CENTER.1 + i), Direction::Down);
        }
    }

    fn grow_house(&mut self) {
        let i = rand::gen_range(0, self.houses.len());
        if let Some(mut house_pos) = self.houses.get(i).cloned() {
            let dir = Direction::random();
            while let Some(pos) = Position::new_from_move(&house_pos, dir, self.grid.size) {
                house_pos = pos;
                if self.generate_house(pos.x, pos.y) {
                    return;
                }
            }
        }
    }

    fn generate_first_houses(&mut self) {
        self.generate_house(GRID_CENTER.0 + 1, GRID_CENTER.1 + 1);
        self.generate_house(GRID_CENTER.0 + 1, GRID_CENTER.1 - 2);
        self.generate_house(GRID_CENTER.0 - 2, GRID_CENTER.1 + 1);
        self.generate_house(GRID_CENTER.0 - 2, GRID_CENTER.1 - 2);

        for _ in 0..50 {
            self.grow_house();
        }
    }

    pub fn generate(&mut self) {
        self.generate_center_roads();
        self.generate_first_houses();
        // the oofs
        // for i in 0..CITY_BLOCK_COUNT {
            // for j in 0..CITY_BLOCK_COUNT {
                // self.generate_block(i * CITY_BLOCK_SIZE, j * CITY_BLOCK_SIZE);
            // }
        // }
    }

    pub fn random_house(&self) -> Option<Position> {
        // find a random block
        let block_x = rand::gen_range(0, CITY_BLOCK_COUNT);
        let block_y = rand::gen_range(0, CITY_BLOCK_COUNT);
        // find a random house
        let house_x = rand::gen_range(1, CITY_BLOCK_SIZE - 1);
        let house_y = rand::gen_range(1, CITY_BLOCK_SIZE - 1);

        self.grid.try_pos(
            (block_x * CITY_BLOCK_SIZE) + house_x,
            (block_y * CITY_BLOCK_SIZE) + house_y,
        )
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

    fn generate_cars(&mut self) {
        let start_index = rand::gen_range(0, self.houses.len());
        let end_index = rand::gen_range(0, self.houses.len());

        let start_pos = self.houses.get(start_index).cloned();
        let end_pos = self.houses.get(end_index).cloned();
        if start_pos.is_none() || end_pos.is_none() {
            return;
        }

        let start_pos = start_pos.unwrap();
        let end_pos = end_pos.unwrap();

        if let Tile::House(_) = self.grid.get_tile(&start_pos) {
            if let Tile::House(start_house) = self.grid.get_tile(&end_pos) {
                if start_house.vehicle_on_the_way.is_some() {
                    return;
                }

                let vehicle = self.add_vehicle(start_pos, end_pos);

                if let Tile::House(start_house_again) = self.grid.get_tile_mut(&end_pos) {
                    start_house_again.vehicle_on_the_way = vehicle;
                }
            } else {
                self.houses.swap_remove(end_index);
            }
        } else {
            self.houses.swap_remove(start_index);
        }
    }

    pub fn update_rating(&mut self, success: bool) {
        if success {
            self.rating = (1. * 0.1) + (self.rating * 0.9);
        } else {
            self.rating = (0. * 0.1) + (self.rating * 0.9);
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
            if let Tile::House(house) = self.grid.get_tile_mut(&vehicle.destination) {
                house.vehicle_on_the_way = None;
            }
            self.vehicles.remove(&id.0);

            self.update_rating(id.1 == Status::ReachedDestination);
        }

        self.generate_cars();

        if self.rating > 0.9 {
            self.grow_ticks += 1;
            if self.grow_ticks > 60 {
                self.grow_house();
                self.grow_ticks = 0;
            }
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        self.grid.draw_tiles(tileset);

        for s in self.vehicles.iter() {
            s.1.draw(tileset);
        }

        self.grid.draw_houses(tileset);
    }
}

#[cfg(test)]
mod map_tests {

    use super::*;

    #[test]
    fn test_map_rating() {
        let mut map = Map::new_from_string(">>>>");
        assert_eq!(map.rating, 1.0);
        map.update_rating(true);
        assert_eq!(map.rating, 1.0);
        map.update_rating(false);
        assert_eq!(map.rating, 0.9);
    }

    #[test]
    fn test_map_serialize() {
        let mut map = Map::new();

        map.generate_road(0, 0, Direction::Right);

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
            .reserve(1234, pos)
            .is_err())
    }

}
