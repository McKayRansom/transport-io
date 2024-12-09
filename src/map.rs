use std::collections::HashMap;

use macroquad::rand::{self, srand};

use crate::{
    grid::{ConnectionsIterator, Direction, Grid, House, Position, Tile},
    station::Station,
    tileset::Tileset,
    vehicle::Vehicle,
};

const CITY_BLOCK_SIZE: i16 = 10;
const CITY_BLOCK_COUNT: i16 = 2;

pub struct Map {
    pub path_grid: Grid,
    pub vehicle_id: u16, // TODO: Change to u32 just in case
    pub vehicles: HashMap<u16, Vehicle>,
    pub stations: Vec<Station>,
}

impl Map {
    pub fn new() -> Self {
        srand(1234);
        Map {
            path_grid: Grid::new(),
            vehicle_id: 0,
            // vehicles: Vec::new(),
            vehicles: HashMap::new(),
            stations: Vec::new(),
        }
    }

    pub fn generate_road(&mut self, pos: Position, dir: Direction) {
        self.path_grid.add_tile_connection(&pos, dir);
    }

    pub fn generate_house(&mut self, pos: Position) {
        if let Some(tile) = self.path_grid.get_tile_mut(&pos) {
            if *tile == Tile::Empty {
                *tile = Tile::House(House { people_heading_to: false });
                // add driveways
                for dir in ConnectionsIterator::all_directions() {
                    let road_pos = Position::new_from_move(&pos, dir);
                    if let Some(Tile::Road(road)) = self.path_grid.get_tile_mut(&road_pos) {
                        road.connections.add(crate::grid::ConnectionLayer::Driveway, dir.inverse());
                    }
                }
            }
        }
    }

    pub fn generate_block(&mut self, x: i16, y: i16) {
        // top
        for i in 0..CITY_BLOCK_SIZE {
            self.generate_road(Position::new(x + i, y), Direction::Right);
            self.generate_road(
                Position::new(x + (CITY_BLOCK_SIZE - 1), y + i),
                Direction::Down,
            );
            self.generate_road(
                Position::new(x + i, y + (CITY_BLOCK_SIZE - 1)),
                Direction::Left,
            );
            self.generate_road(Position::new(x, y + i), Direction::Up);
        }

        // houses (all for now)
        for i in 0..CITY_BLOCK_SIZE {
            for j in 0..CITY_BLOCK_SIZE {
                self.generate_house(Position::new(x + i, y + j));
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

    pub fn random_house(&self) -> Position {
        // find a random block
        let block_x = rand::gen_range(0, CITY_BLOCK_COUNT);
        let block_y = rand::gen_range(0, CITY_BLOCK_COUNT);
        // find a random house
        let house_x = rand::gen_range(1, CITY_BLOCK_SIZE - 1);
        let house_y = rand::gen_range(1, CITY_BLOCK_SIZE - 1);

        Position {
            x: (block_x * CITY_BLOCK_SIZE) + house_x,
            y: (block_y * CITY_BLOCK_SIZE) + house_y,
        }
    }

    fn generate_cars(&mut self) {
        let start_house = self.random_house();
        let end_house = self.random_house();

        if let Some(Tile::House(house)) = self.path_grid.get_tile_mut(&end_house) {
            if house.people_heading_to {
                return;
            }
            house.people_heading_to = true;
        } else {
            return;
        }

        if let Some(Tile::House(_)) = self.path_grid.get_tile(&start_house) {
            
        } else {
            return;
        }

        println!("Start: {start_house:?} End: {end_house:?}");
        if let Some(vehicle ) = Vehicle::new(start_house, end_house, &mut self.path_grid) {
            self.vehicles.insert(
                self.vehicle_id,
                vehicle,
            );

            self.vehicle_id += 1;
        }
    }

    pub fn update(&mut self) -> u32 {
        let mut delivered = 0;
        let mut to_remove: Vec<u16> = Vec::new();
        for s in self.vehicles.iter_mut() {
            let finished = s.1.update(&mut self.path_grid);
            if let Some(destination) = finished {
                delivered += 1;
                s.1.clear_reserved(&mut self.path_grid);
                if let Some(Tile::House(house)) = self.path_grid.get_tile_mut(&destination) {
                    house.people_heading_to = false;
                }
                to_remove.push(*s.0);
            }
        }
        for id in to_remove {
            self.vehicles.remove(&id);
        }

        self.generate_cars();

        delivered
    }

    pub fn draw(&self, tileset: &Tileset) {
        self.path_grid.draw_tiles(tileset);

        for s in self.stations.iter() {
            s.draw(tileset);
        }

        for s in self.vehicles.iter() {
            s.1.draw(tileset);
        }

        self.path_grid.draw_houses(tileset);
    }
}
