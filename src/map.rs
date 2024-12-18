use std::collections::HashMap;

use macroquad::rand::{self, srand};

use crate::{
    grid::{self, Direction, Grid, Id, Position},
    tile::{ConnectionLayer, ConnectionsIterator, House, Tile},
    tileset::Tileset,
    vehicle::{Status, Vehicle},
};

const CITY_BLOCK_SIZE: i16 = 8;
const CITY_BLOCK_COUNT: i16 = 4;

pub const GRID_SIZE: (i16, i16) = (30, 30);

pub struct Map {
    pub path_grid: Grid,
    pub vehicle_id: grid::Id,
    pub vehicles: HashMap<grid::Id, Vehicle>,
    pub rating: f32,
}

impl Map {
    pub fn new() -> Self {
        srand(1234);
        Map {
            path_grid: Grid::new(GRID_SIZE.0 as usize, GRID_SIZE.1 as usize),
            vehicle_id: 1,
            vehicles: HashMap::new(),
            rating: 1.0,
        }
    }

    #[allow(unused)]
    pub fn new_from_string(string: &str) -> Self {
        Map {
            path_grid: Grid::new_from_string(string),
            vehicle_id: 1,
            vehicles: HashMap::new(),
            rating: 1.0,
        }
    }

    pub fn generate_road(&mut self, x: i16, y: i16, dir: Direction) {
        if let Some(pos) = self.path_grid.try_pos(x, y) {
            self.path_grid
                .get_tile_mut(&pos)
                .edit_road(|road| road.connect(dir));
        }
    }

    pub fn generate_driveways(&mut self, x: i16, y: i16) {
        // add driveways
        for dir in ConnectionsIterator::all_directions() {
            if let Some(road_pos) =
                Position::new_from_move(&self.path_grid.pos(x, y), dir, self.path_grid.size)
            {
                if let Tile::Road(road) = self.path_grid.get_tile_mut(&road_pos) {
                    road.connect_layer(dir.inverse(), ConnectionLayer::Driveway);
                }
            }
        }
    }

    pub fn generate_house(&mut self, x: i16, y: i16) -> bool {
        let mut success = false;
        if let Some(pos) = self.path_grid.try_pos(x, y) {
            let tile = self.path_grid.get_tile_mut(&pos);
            success = true;
            tile.build(|| {
                Tile::House(House::new())
            })
        }
        success
    }

    pub fn generate_block(&mut self, x: i16, y: i16) {
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
                if self.generate_house(x + i, y + j) {
                    self.generate_driveways(x + i, y + j);
                }
            }
        }
    }

    pub fn generate(&mut self) {
        for i in 0..CITY_BLOCK_COUNT {
            for j in 0..CITY_BLOCK_COUNT {
                self.generate_block(i * CITY_BLOCK_SIZE, j * CITY_BLOCK_SIZE);
            }
        }
    }

    pub fn random_house(&self) -> Option<Position> {
        // find a random block
        let block_x = rand::gen_range(0, CITY_BLOCK_COUNT);
        let block_y = rand::gen_range(0, CITY_BLOCK_COUNT);
        // find a random house
        let house_x = rand::gen_range(1, CITY_BLOCK_SIZE - 1);
        let house_y = rand::gen_range(1, CITY_BLOCK_SIZE - 1);

        self.path_grid.try_pos(
            (block_x * CITY_BLOCK_SIZE) + house_x,
            (block_y * CITY_BLOCK_SIZE) + house_y,
        )
    }

    pub fn add_vehicle(&mut self, start_pos: Position, end_pos: Position) -> Option<Id> {
        let id = self.vehicle_id;
        if let Some(vehicle) =
            Vehicle::new(start_pos, self.vehicle_id, end_pos, &mut self.path_grid)
        {
            self.vehicles.insert(id, vehicle);
            self.vehicle_id += 1;

            Some(id)
        } else {
            None
        }
    }

    fn generate_cars(&mut self) {
        let start_pos = self.random_house();
        let end_pos = self.random_house();
        if start_pos.is_none() || end_pos.is_none() {
            return;
        }

        let start_pos = start_pos.unwrap();
        let end_pos = end_pos.unwrap();

        if let Tile::House(_) = self.path_grid.get_tile(&start_pos) {
            if let Tile::House(start_house) = self.path_grid.get_tile(&end_pos) {
                if start_house.vehicle_on_the_way.is_some() {
                    return;
                }

                let vehicle = self.add_vehicle(start_pos, end_pos);

                if let Tile::House(start_house_again) = self.path_grid.get_tile_mut(&end_pos) {
                    start_house_again.vehicle_on_the_way = vehicle;
                }
            }
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
            let status = s.1.update(&mut self.path_grid);
            if status != Status::EnRoute {
                to_remove.push((*s.0, status));
            }
        }
        for id in to_remove {
            let vehicle = self.vehicles.get_mut(&id.0).unwrap();
            if let Tile::House(house) = self.path_grid.get_tile_mut(&vehicle.destination) {
                house.vehicle_on_the_way = None;
            }
            vehicle.delete(&mut self.path_grid);
            self.vehicles.remove(&id.0);

            self.update_rating(id.1 == Status::ReachedDestination);
        }

        self.generate_cars();
    }

    pub fn draw(&self, tileset: &Tileset) {
        self.path_grid.draw_tiles(tileset);

        for s in self.vehicles.iter() {
            s.1.draw(tileset);
        }

        self.path_grid.draw_houses(tileset);
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
}
